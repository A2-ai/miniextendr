//! Values extracted from a sidecar keep their R owner through moves and clones.
mod r_test_utils;

use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::{ExternalPtr, IntoR, SEXP};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(ExternalPtr)]
struct Payload(Arc<AtomicUsize>);

impl Drop for Payload {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Clone, ExternalPtr)]
struct Host {
    #[r_data]
    value: Sidecar<SEXP>,
}

fn host(drops: &Arc<AtomicUsize>) -> ExternalPtr<Host> {
    let payload = ExternalPtr::new(Payload(Arc::clone(drops)));
    let result = ExternalPtr::new(Host {
        value: Sidecar::new(payload.as_sexp()),
    });
    drop(payload);
    result
}

fn collect() {
    unsafe { miniextendr_api::sys::R_gc() };
}

#[test]
fn extracted_values_and_their_clones_keep_the_slot_alive() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let value = ExternalPtr::into_inner(host(&drops));
        collect();
        // Check reachability before reading a potentially collected pointer.
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let copy = value.clone();
        drop(value);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let back = ExternalPtr::new(copy);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(unsafe { ExternalPtr::<Payload>::wrap_sexp(Host::value(&back)) }.is_some());
        drop(back);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn raw_extraction_keeps_the_slot_until_rewrapping() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let raw = ExternalPtr::into_raw(host(&drops));
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let back = unsafe { ExternalPtr::from_raw(raw) };
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(back);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn dropping_or_replacing_the_last_pending_copy_releases_the_slot() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut value = ExternalPtr::into_inner(host(&drops));
        let copy = value.clone();
        value.value = Sidecar::new(SEXP::nil());
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(copy);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        drop(value);
    });
}

#[derive(Clone)]
struct CollectOnFlush(SEXP);

impl miniextendr_api::TryFromSexp for CollectOnFlush {
    type Error = std::convert::Infallible;
    fn try_from_sexp(value: SEXP) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl IntoR for CollectOnFlush {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        collect();
        Ok(self.0)
    }
}

#[derive(ExternalPtr)]
struct CollectingHost {
    #[r_data]
    value: Sidecar<CollectOnFlush>,
}

#[test]
fn a_pending_root_survives_conversion_into_the_new_slot() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let payload = ExternalPtr::new(Payload(Arc::clone(&drops)));
        let owner = ExternalPtr::new(CollectingHost {
            value: Sidecar::new(CollectOnFlush(payload.as_sexp())),
        });
        drop(payload);
        let value = ExternalPtr::into_inner(owner);
        let back = ExternalPtr::new(value);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(back);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

struct RejectReload;

impl miniextendr_api::TryFromSexp for RejectReload {
    type Error = &'static str;
    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        Err("intentional reload failure")
    }
}

impl IntoR for RejectReload {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(SEXP::nil())
    }
}

#[derive(ExternalPtr)]
struct FailingHost {
    #[r_data]
    value: Sidecar<SEXP>,
    #[r_data]
    rejects: Sidecar<RejectReload>,
    drops: Arc<AtomicUsize>,
}

impl Drop for FailingHost {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn failed_extraction_does_not_leave_a_self_rooting_partial_load() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut owner = ExternalPtr::new(FailingHost {
            value: Sidecar::new(SEXP::nil()),
            rejects: Sidecar::new(RejectReload),
            drops: Arc::clone(&drops),
        });
        let sexp = owner.as_sexp();
        FailingHost::set_value(&mut owner, sexp);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ExternalPtr::into_inner(owner)
            }))
            .is_err()
        );
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn failed_extraction_preserves_values_that_were_already_pending() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut owner = ExternalPtr::new(FailingHost {
            value: Sidecar::new(SEXP::nil()),
            rejects: Sidecar::new(RejectReload),
            drops: Arc::new(AtomicUsize::new(0)),
        });
        owner.borrow_mut().value = ExternalPtr::into_inner(host(&drops)).value;
        let alias = unsafe { ExternalPtr::<FailingHost>::wrap_sexp(owner.as_sexp()) }.unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ExternalPtr::into_inner(alias)
            }))
            .is_err()
        );
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(unsafe { ExternalPtr::<Payload>::wrap_sexp(FailingHost::value(&owner)) }.is_some());
        drop(owner);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}
