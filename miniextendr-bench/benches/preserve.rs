//! Preserve list benchmarks.

use miniextendr_api::ffi;
use miniextendr_api::preserve;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn preserve_insert_release() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(1);
        let cell = preserve::insert(sexp);
        preserve::release(cell);
        divan::black_box(cell);
    }
}

#[divan::bench]
fn preserve_insert_release_unchecked() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(1);
        let cell = preserve::insert_unchecked(sexp);
        preserve::release_unchecked(cell);
        divan::black_box(cell);
    }
}

#[divan::bench]
fn protect_unprotect() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(1);
        ffi::Rf_protect(sexp);
        ffi::Rf_unprotect(1);
        divan::black_box(sexp);
    }
}
