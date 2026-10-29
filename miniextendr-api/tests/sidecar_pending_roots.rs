//! Pending sidecar values keep their R object alive from construction, and
//! values extracted from a sidecar keep it through moves and clones, until a
//! slot holds it (#1961, #2216).
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

#[test]
fn a_fresh_value_is_rooted_from_construction() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let payload = ExternalPtr::new(Payload(Arc::clone(&drops)));
        let pending = Sidecar::<SEXP>::new(payload.as_sexp());
        let second = Sidecar::<SEXP>::new(payload.as_sexp());
        drop(payload);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(pending);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let copy = second.clone();
        drop(second);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(copy);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn a_fresh_value_stays_rooted_while_its_struct_is_wrapped() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let payload = ExternalPtr::new(Payload(Arc::clone(&drops)));
        let value = Host {
            value: Sidecar::new(payload.as_sexp()),
        };
        drop(payload);
        collect();
        let owner = ExternalPtr::new(value);
        collect();
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(unsafe { ExternalPtr::<Payload>::wrap_sexp(Host::value(&owner)) }.is_some());
        drop(owner);
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
struct RejectingHost {
    #[r_data]
    rejects: Sidecar<RejectReload>,
}

/// A slot moves out and back as its R object: only a getter converts it.
#[test]
fn extraction_moves_slots_without_converting_them() {
    r_test_utils::with_r_thread(|| {
        let owner = ExternalPtr::new(RejectingHost {
            rejects: Sidecar::new(RejectReload),
        });
        let back = ExternalPtr::new(ExternalPtr::into_inner(owner));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                RejectingHost::rejects(&back)
            }))
            .is_err()
        );
    });
}
