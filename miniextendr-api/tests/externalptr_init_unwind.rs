//! Constructor unwinding must balance R roots and retain ownership of every box.
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

#[test]
fn every_constructor_unprotects_after_initial_conversion_panics() {
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
                assert_balanced_panic(|| construct(host(&drops, true)));
            }
            unsafe { miniextendr_api::sys::R_gc() };
            assert_eq!(
                drops.load(Ordering::SeqCst),
                8,
                "finalizers must reclaim every failed constructor"
            );
        }
    });
}

#[test]
fn a_partially_built_list_releases_completed_and_pending_elements() {
    r_test_utils::with_r_thread(|| {
        let drops = Arc::new(AtomicUsize::new(0));
        let items = (0..4).map(|i| host(&drops, i == 1)).collect::<Vec<_>>();
        assert_balanced_panic(|| {
            ExternalPtr::collect_into_r_list(items);
        });
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(drops.load(Ordering::SeqCst), 4);
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
            assert_eq!(drops.load(Ordering::SeqCst), 4);
        }
    });
}
