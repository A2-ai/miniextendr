//! Checked vs unchecked FFI wrapper benchmarks.

use miniextendr_api::ffi::{self, SEXPTYPE};

const SIZE_INDICES: &[usize] = &[0, 2, 4];
const ALLOC_SIZES: &[usize] = &[1, 16, 256, 4096, 65536];

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench]
fn scalar_integer_checked() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(1);
        divan::black_box(sexp);
    }
}

#[divan::bench]
fn scalar_integer_unchecked() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger_unchecked(1);
        divan::black_box(sexp);
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn xlength_checked(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    unsafe {
        let len = ffi::Rf_xlength(sexp);
        divan::black_box(len);
    }
}

#[divan::bench(args = SIZE_INDICES)]
fn xlength_unchecked(size_idx: usize) {
    let sexp = miniextendr_bench::fixtures().int_vec(size_idx);
    unsafe {
        let len = ffi::Rf_xlength_unchecked(sexp);
        divan::black_box(len);
    }
}

#[divan::bench(args = ALLOC_SIZES)]
fn alloc_vector_checked(len: usize) {
    unsafe {
        let sexp = ffi::Rf_allocVector(SEXPTYPE::INTSXP, len as ffi::R_xlen_t);
        divan::black_box(sexp);
    }
}

#[divan::bench(args = ALLOC_SIZES)]
fn alloc_vector_unchecked(len: usize) {
    unsafe {
        let sexp = ffi::Rf_allocVector_unchecked(SEXPTYPE::INTSXP, len as ffi::R_xlen_t);
        divan::black_box(sexp);
    }
}
