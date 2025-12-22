//! Integration tests for rayon bridge functionality.
//!
//! Tests `with_r_vec` and parallel collection with an embedded R runtime.
//!
//! All tests run in a single test function to avoid R state isolation issues
//! between test threads (similar to allocator.rs).

#![cfg(feature = "rayon")]

use miniextendr_api::ffi::{REAL, Rf_xlength, SEXP};
use miniextendr_api::rayon_bridge::{RVec, with_r_vec};
use rayon::prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_r() {
    INIT.call_once(|| unsafe {
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as rpkg/src/entrypoint.c.in
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_worker_init();
        assert!(
            miniextendr_engine::r_initialized_sentinel(),
            "Rf_initialize_R did not set C stack sentinels"
        );
        std::mem::forget(engine);
    });
}

/// Helper to read f64 values from an R REALSXP vector.
unsafe fn read_real_vec(sexp: SEXP) -> Vec<f64> {
    unsafe {
        let len = Rf_xlength(sexp) as usize;
        if len == 0 {
            return Vec::new();
        }
        let ptr = REAL(sexp);
        std::slice::from_raw_parts(ptr, len).to_vec()
    }
}

#[test]
fn rayon_suite() {
    initialize_r();

    test_with_r_vec_basic();
    test_with_r_vec_parallel_write();
    test_with_r_vec_i32();
    test_with_r_vec_empty();
    test_with_r_vec_large();
    test_rvec_parallel_collect();
    test_rvec_into_sexp();
}

fn test_with_r_vec_basic() {
    let sexp = with_r_vec::<f64, _>(10, |slice| {
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as f64;
        }
    });

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(
        result,
        vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
    );
}

fn test_with_r_vec_parallel_write() {
    let sexp = with_r_vec::<f64, _>(1000, |slice| {
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = (i * i) as f64;
        });
    });

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 1000);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, (i * i) as f64, "mismatch at index {}", i);
    }
}

fn test_with_r_vec_i32() {
    let sexp = with_r_vec::<i32, _>(100, |slice| {
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = i as i32 * 2;
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 100);

    let ptr = unsafe { miniextendr_api::ffi::INTEGER(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    for (i, &v) in slice.iter().enumerate() {
        assert_eq!(v, i as i32 * 2, "mismatch at index {}", i);
    }
}

fn test_rvec_parallel_collect() {
    let result: RVec<f64> = (0..1000).into_par_iter().map(|i| i as f64 * 0.5).collect();

    assert_eq!(result.len(), 1000);
    for (i, &v) in result.as_slice().iter().enumerate() {
        assert_eq!(v, i as f64 * 0.5, "mismatch at index {}", i);
    }
}

fn test_rvec_into_sexp() {
    let rvec: RVec<f64> = (0..100).into_par_iter().map(|i| i as f64).collect();
    let sexp = miniextendr_api::into_r::IntoR::into_sexp(rvec);

    let result = unsafe { read_real_vec(sexp) };
    assert_eq!(result.len(), 100);
    for (i, &v) in result.iter().enumerate() {
        assert_eq!(v, i as f64);
    }
}

fn test_with_r_vec_empty() {
    let sexp = with_r_vec::<f64, _>(0, |slice| {
        assert!(slice.is_empty());
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, 0);
}

fn test_with_r_vec_large() {
    const SIZE: usize = 100_000;
    let sexp = with_r_vec::<f64, _>(SIZE, |slice| {
        slice.par_iter_mut().enumerate().for_each(|(i, v)| {
            *v = (i % 1000) as f64;
        });
    });

    let len = unsafe { Rf_xlength(sexp) } as usize;
    assert_eq!(len, SIZE);

    // Spot check some values
    let ptr = unsafe { REAL(sexp) };
    let slice = unsafe { std::slice::from_raw_parts(ptr, SIZE) };
    assert_eq!(slice[0], 0.0);
    assert_eq!(slice[999], 999.0);
    assert_eq!(slice[1000], 0.0);
    assert_eq!(slice[50_000], 0.0);
}
