//! A worker function's `ExternalPtr<T>` arguments stay borrowed on R's main
//! thread while the worker runs (#2092).
#![cfg(feature = "worker-thread")]
// Exercises the deprecated `worker` option (#2117).
#![allow(deprecated)]

mod r_test_utils;

use miniextendr_api::condition::catch_condition;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{ExternalPtr, RCondition, SEXP, SEXPREC, TryFromSexp, miniextendr};
use std::sync::atomic::{AtomicPtr, Ordering};

#[derive(ExternalPtr)]
pub struct WorkerProbe {
    value: i32,
}

// A `SEXP` parameter would pin the function to the main thread, so the
// object's pointer travels through a static instead.
static OBJECT: AtomicPtr<SEXPREC> = AtomicPtr::new(std::ptr::null_mut());

/// Whether a main-thread borrow of `OBJECT` conflicts, `exclusive` or not.
fn main_thread_conflicts(exclusive: bool) -> bool {
    miniextendr_api::worker::with_r_thread(move || {
        let mut alias =
            unsafe { ExternalPtr::<WorkerProbe>::wrap_sexp(SEXP(OBJECT.load(Ordering::Relaxed))) }
                .unwrap();
        let attempt = catch_condition(|| {
            if exclusive {
                drop(alias.try_borrow_mut());
            } else {
                drop(alias.try_borrow());
            }
        });
        attempt.is_err()
    })
}

#[miniextendr(worker, noexport)]
pub fn worker_exclusive_arg(mut obj: ExternalPtr<WorkerProbe>) -> i32 {
    assert!(!miniextendr_api::worker::is_r_main_thread());
    assert!(main_thread_conflicts(false));
    obj.value += 1;
    obj.value
}

#[miniextendr(worker, noexport)]
pub fn worker_shared_arg(obj: ExternalPtr<WorkerProbe>) -> i32 {
    assert!(!miniextendr_api::worker::is_r_main_thread());
    assert!(main_thread_conflicts(true));
    assert!(!main_thread_conflicts(false));
    obj.value
}

unsafe fn call(suffix: &str, obj: SEXP) -> SEXP {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(suffix.as_bytes())
        })
        .expect("worker wrapper must be registered");
    assert_eq!(entry.numArgs, 2);
    let wrapper = unsafe {
        std::mem::transmute::<
            unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void,
            unsafe extern "C-unwind" fn(SEXP, SEXP) -> SEXP,
        >(entry.fun.unwrap())
    };
    unsafe { wrapper(SEXP::nil(), obj) }
}

#[test]
fn worker_arguments_hold_their_borrow_on_the_main_thread() {
    r_test_utils::with_r_thread(|| unsafe {
        let obj = ExternalPtr::new(WorkerProbe { value: 4 });
        OBJECT.store(obj.as_sexp().0, Ordering::Relaxed);
        let scope = ProtectScope::new();
        for (suffix, expected) in [("_worker_exclusive_arg", 5), ("_worker_shared_arg", 5)] {
            let result = scope.protect(call(suffix, obj.as_sexp())).get();
            assert!(RCondition::from_tagged_sexp(result).is_none(), "{suffix}");
            assert_eq!(i32::try_from_sexp(result).unwrap(), expected, "{suffix}");
        }
        // The borrows ended with the calls.
        assert!(!main_thread_conflicts(true));
    });
}
