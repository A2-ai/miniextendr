//! Preserve list benchmarks (DLL doubly-linked list).
//!
//! Pre-allocates SEXPs outside the timed region to measure pure
//! protection overhead. Compares checked vs unchecked variants.

use divan::Bencher;
use miniextendr_api::ffi::{R_PreserveObject, R_ReleaseObject};
use miniextendr_api::preserve;
use miniextendr_bench::raw_ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: single insert+release

#[divan::bench]
fn preserve_insert_release(bencher: Bencher) {
    let sexp = unsafe { raw_ffi::Rf_ScalarInteger(1) };
    unsafe { R_PreserveObject(sexp) };

    bencher.bench_local(|| unsafe {
        let cell = preserve::insert(sexp);
        preserve::release(cell);
    });

    unsafe { R_ReleaseObject(sexp) };
}

#[divan::bench]
fn preserve_insert_release_unchecked(bencher: Bencher) {
    let sexp = unsafe { raw_ffi::Rf_ScalarInteger(1) };
    unsafe { R_PreserveObject(sexp) };

    bencher.bench_local(|| unsafe {
        let cell = preserve::insert_unchecked(sexp);
        preserve::release_unchecked(cell);
    });

    unsafe { R_ReleaseObject(sexp) };
}

#[divan::bench]
fn protect_unprotect(bencher: Bencher) {
    let sexp = unsafe { raw_ffi::Rf_ScalarInteger(1) };
    unsafe { R_PreserveObject(sexp) };

    bencher.bench_local(|| unsafe {
        raw_ffi::Rf_protect(sexp);
        raw_ffi::Rf_unprotect(1);
    });

    unsafe { R_ReleaseObject(sexp) };
}

// endregion
