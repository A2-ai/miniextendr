//! Shared worker references must borrow Rust-owned snapshots, not R storage.
#![cfg(feature = "worker-thread")]

mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{IntoR, RCondition, SEXP, SEXPREC, SexpExt, TryFromSexp, miniextendr};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

static R_DATA_ADDRESS: AtomicUsize = AtomicUsize::new(0);
// A `SEXP` parameter would pin the function to the main thread, so the vector
// the worker mutates travels through a static instead.
static R_VALUES: AtomicPtr<SEXPREC> = AtomicPtr::new(std::ptr::null_mut());

#[miniextendr(worker, noexport)]
pub fn worker_native_inputs<'a>(
    scalar: &'a i32,
    optional: Option<&'a i32>,
    values: &'a [i32],
    optional_values: Option<&'a [i32]>,
    __owned_values: i32,
) -> Vec<i32> {
    assert!(!miniextendr_api::worker::is_r_main_thread());
    if !values.is_empty() {
        assert_ne!(
            values.as_ptr().addr(),
            R_DATA_ADDRESS.load(Ordering::Relaxed)
        );
        miniextendr_api::worker::with_r_thread(|| unsafe {
            // This mutation is disjoint from the worker's owned snapshot.
            SEXP(R_VALUES.load(Ordering::Relaxed)).as_mut_slice::<i32>()[0] = 999;
            miniextendr_api::sys::R_gc();
        });
    }
    vec![
        *scalar,
        optional.copied().unwrap_or(-1),
        values.iter().sum(),
        optional_values.map(|v| v.iter().sum()).unwrap_or(-1),
        __owned_values,
    ]
}

type Wrapper = unsafe extern "C-unwind" fn(SEXP, SEXP, SEXP, SEXP, SEXP, SEXP) -> SEXP;

unsafe fn wrapper() -> Wrapper {
    let entry = miniextendr_api::registry::MX_CALL_DEFS
        .iter()
        .find(|entry| {
            !entry.name.is_null()
                && unsafe { std::ffi::CStr::from_ptr(entry.name) }
                    .to_bytes()
                    .ends_with(b"_worker_native_inputs")
        })
        .expect("worker wrapper must be registered");
    assert_eq!(entry.numArgs, 6);
    unsafe {
        std::mem::transmute::<unsafe extern "C-unwind" fn() -> *mut core::ffi::c_void, Wrapper>(
            entry.fun.unwrap(),
        )
    }
}

#[test]
fn native_worker_inputs_are_owned_snapshots_with_exact_borrowed_shape_checks() {
    r_test_utils::with_r_thread(|| unsafe {
        let call = wrapper();
        let scope = ProtectScope::new();
        let scalar = scope.protect(7_i32.into_sexp());
        let values = scope.protect(vec![3_i32, 5].into_sexp());
        let empty = scope.protect(Vec::<i32>::new().into_sexp());
        R_DATA_ADDRESS.store(
            values.get().as_slice::<i32>().as_ptr().addr(),
            Ordering::Relaxed,
        );
        R_VALUES.store(values.get().0, Ordering::Relaxed);
        let result = scope.protect(call(
            SEXP::nil(),
            scalar.get(),
            scalar.get(),
            values.get(),
            values.get(),
            scalar.get(),
        ));
        assert_eq!(
            Vec::<i32>::try_from_sexp(result.get()).unwrap(),
            [7, 7, 8, 8, 7]
        );
        assert_eq!(values.get().as_slice::<i32>(), [999, 5]);

        let result = scope.protect(call(
            SEXP::nil(),
            scalar.get(),
            SEXP::nil(),
            empty.get(),
            SEXP::nil(),
            scalar.get(),
        ));
        assert_eq!(
            Vec::<i32>::try_from_sexp(result.get()).unwrap(),
            [7, -1, 0, -1, 7]
        );

        // Optional native references treat NA as a sentinel value, not None.
        let na = scope.protect(i32::MIN.into_sexp());
        let result = scope.protect(call(
            SEXP::nil(),
            scalar.get(),
            na.get(),
            empty.get(),
            empty.get(),
            scalar.get(),
        ));
        assert_eq!(
            Vec::<i32>::try_from_sexp(result.get()).unwrap(),
            [7, i32::MIN, 0, 0, 7]
        );

        // Copy through the reference converter, not the scalar value converter.
        for bad in [empty.get(), values.get()] {
            let refused = scope.protect(call(
                SEXP::nil(),
                bad,
                SEXP::nil(),
                empty.get(),
                SEXP::nil(),
                scalar.get(),
            ));
            let (_, parts) = RCondition::from_tagged_sexp(refused.get())
                .unwrap()
                .into_parts();
            assert!(parts.message.contains("scalar"), "{}", parts.message);
        }
    });
}
