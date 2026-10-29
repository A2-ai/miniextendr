//! Constructor unwinding must balance R roots and retain ownership of every box.
//!
//! A `Sidecar` converts its value when it is made (#2216), so a conversion
//! panic unwinds out of `Sidecar::new`, before any constructor allocates.
mod r_test_utils;

use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::{ExternalPtr, IntoR, SEXP, TryFromSexp};
use std::convert::Infallible;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// Test-only introspection of R's protection stack, never linked into rpkg.
unsafe extern "C" {
    static R_PPStackTop: std::os::raw::c_int;
}

struct InitialValue(bool);

impl IntoR for InitialValue {
    type Error = Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        assert!(!self.0, "intentional initial sidecar conversion panic");
        Ok(vec![2_i32, 3, 5].into_sexp())
    }
}

impl TryFromSexp for InitialValue {
    type Error = &'static str;

    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> {
        Err("test only exercises construction")
    }
}

#[derive(ExternalPtr)]
struct InitialSidecar {
    #[r_data]
    value: Sidecar<InitialValue>,
    drops: Arc<AtomicUsize>,
}

impl Drop for InitialSidecar {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

fn host(drops: &Arc<AtomicUsize>, panic: bool) -> InitialSidecar {
    InitialSidecar {
        value: Sidecar::new(InitialValue(panic)),
        drops: Arc::clone(drops),
    }
}

fn assert_balanced_panic(f: impl FnOnce()) {
    let baseline = unsafe { R_PPStackTop };
    let error = catch_unwind(AssertUnwindSafe(f)).expect_err("conversion should panic");
    let message = error
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| error.downcast_ref::<String>().map(String::as_str));
    assert_eq!(
        message,
        Some("intentional initial sidecar conversion panic")
    );
    assert_eq!(
        unsafe { R_PPStackTop },
        baseline,
        "constructor leaked PROTECT entries"
    );
}

#[derive(ExternalPtr)]
struct Payload(Arc<AtomicUsize>);

impl Drop for Payload {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Converts to the R object it carries, or panics.
struct CarriedValue(SEXP, bool);

impl IntoR for CarriedValue {
    type Error = Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        assert!(!self.1, "intentional initial sidecar conversion panic");
        Ok(self.0)
    }
}

/// A panic in `IntoR` unwinds out of `Sidecar::new` before any constructor
/// runs: the protect stack is balanced, no value was built, and nothing the
/// failed calls left behind roots the object they carried.
#[test]
fn a_conversion_panic_unwinds_out_of_sidecar_new() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        for _ in 0..8 {
            assert_balanced_panic(|| drop(host(&drops, true)));
        }
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(drops.load(Ordering::SeqCst), 0, "no value was built");

        let payload_drops = Arc::new(AtomicUsize::new(0));
        let payload = ExternalPtr::new(Payload(Arc::clone(&payload_drops)));
        for _ in 0..8 {
            assert_balanced_panic(|| {
                drop(Sidecar::new(CarriedValue(payload.as_sexp(), true)));
            });
        }
        let kept = Sidecar::new(CarriedValue(payload.as_sexp(), false));
        drop(payload);
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(
            payload_drops.load(Ordering::SeqCst),
            0,
            "the kept value roots it"
        );
        drop(kept);
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(
            payload_drops.load(Ordering::SeqCst),
            1,
            "a failed conversion left a root behind"
        );
    });
}

#[test]
fn every_constructor_unprotects_and_hands_its_box_to_the_finalizer() {
    r_test_utils::with_r_thread(|| {
        let constructors: [fn(InitialSidecar); 4] = [
            |value| drop(ExternalPtr::new(value)),
            |value| unsafe { drop(ExternalPtr::new_unchecked(value)) },
            |value| unsafe { drop(ExternalPtr::from_raw(Box::into_raw(Box::new(value)))) },
            |value| unsafe {
                drop(ExternalPtr::from_raw_unchecked(Box::into_raw(Box::new(
                    value,
                ))))
            },
        ];
        for construct in constructors {
            let drops = Arc::new(AtomicUsize::new(0));
            for _ in 0..8 {
                let baseline = unsafe { R_PPStackTop };
                construct(host(&drops, false));
                assert_eq!(
                    unsafe { R_PPStackTop },
                    baseline,
                    "constructor leaked PROTECT entries"
                );
            }
            unsafe { miniextendr_api::sys::R_gc() };
            assert_eq!(
                drops.load(Ordering::SeqCst),
                8,
                "finalizers must reclaim every dropped pointer"
            );
        }
    });
}

#[test]
fn a_conversion_panic_while_collecting_a_list_drops_the_values_built() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let items = (0..4).map(|i| host(&drops, i == 2));
        assert_balanced_panic(|| {
            ExternalPtr::collect_into_r_list(items);
        });
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn an_iterator_panic_reclaims_values_already_collected() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let items = (0..4).map(|i| {
            assert!(i != 2, "intentional iterator panic");
            host(&drops, false)
        });
        assert!(
            catch_unwind(AssertUnwindSafe(|| ExternalPtr::collect_into_r_list(items))).is_err()
        );
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn construction_and_unwinding_keep_roots_under_gc_torture() {
    r_test_utils::with_r_thread(|| {
        use miniextendr_api::expression::r_eval_str_global;
        use miniextendr_api::{OwnedProtect, SexpExt};
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                unsafe { r_eval_str_global("gctorture(FALSE)").unwrap() };
            }
        }
        let _reset = Reset;
        unsafe { r_eval_str_global("gctorture(TRUE)").unwrap() };
        for _ in 0..4 {
            let drops = Arc::new(AtomicUsize::new(0));
            let owner = ExternalPtr::new(host(&drops, false));
            unsafe { miniextendr_api::sys::R_gc() };
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            assert!(ExternalPtr::<InitialSidecar>::try_from_sexp(owner.as_sexp()).is_ok());
            drop(owner);
            unsafe { miniextendr_api::sys::R_gc() };
            assert_eq!(drops.load(Ordering::SeqCst), 1);

            let list = unsafe {
                OwnedProtect::new(ExternalPtr::collect_into_r_list([
                    host(&drops, false),
                    host(&drops, false),
                ]))
            };
            unsafe { miniextendr_api::sys::R_gc() };
            assert!(ExternalPtr::<InitialSidecar>::try_from_sexp(list.vector_elt(1)).is_ok());
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            drop(list);
            unsafe { miniextendr_api::sys::R_gc() };
            assert_eq!(drops.load(Ordering::SeqCst), 3);

            assert_balanced_panic(|| {
                ExternalPtr::new(host(&drops, true));
            });
            unsafe { miniextendr_api::sys::R_gc() };
            assert_eq!(drops.load(Ordering::SeqCst), 3);
        }
    });
}
