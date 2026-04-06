//! FFI call overhead benchmarks.
//!
//! Measures the cost of calling R C-API functions through checked vs unchecked
//! wrappers.

use miniextendr_api::ffi::{self, SEXP, SEXPTYPE, SexpExt};
use miniextendr_bench::SIZES;
use miniextendr_bench::raw_ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: Vector allocation benchmarks

#[divan::bench(args = SIZES)]
fn alloc_intsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_allocVector(SEXPTYPE::INTSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_realsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_allocVector(SEXPTYPE::REALSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_lglsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_allocVector(SEXPTYPE::LGLSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_rawsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_allocVector(SEXPTYPE::RAWSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_strsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}
// endregion

// region: Scalar creation benchmarks

#[divan::bench]
fn scalar_integer() -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_ScalarInteger(42);
        divan::black_box(sexp)
    }
}

#[divan::bench]
fn scalar_real() -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_ScalarReal(std::f64::consts::PI);
        divan::black_box(sexp)
    }
}

#[divan::bench]
fn scalar_logical() -> SEXP {
    unsafe {
        let sexp = raw_ffi::Rf_ScalarLogical(1);
        divan::black_box(sexp)
    }
}
// endregion

// region: Data pointer access benchmarks (no allocation, just pointer retrieval)

#[divan::bench(args = [0, 2, 4])]
fn integer_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = raw_ffi::INTEGER(fixtures.int_vec(size_idx));
        divan::black_box(ptr);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn real_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = raw_ffi::REAL(fixtures.real_vec(size_idx));
        divan::black_box(ptr);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn logical_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = raw_ffi::LOGICAL(fixtures.lgl_vec(size_idx));
        divan::black_box(ptr);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn raw_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = raw_ffi::RAW(fixtures.raw_vec(size_idx));
        divan::black_box(ptr);
    }
}
// endregion

// region: PROTECT / UNPROTECT benchmarks

#[divan::bench]
fn protect_unprotect_single() {
    unsafe {
        let sexp = raw_ffi::Rf_ScalarInteger(1);
        raw_ffi::Rf_protect(sexp);
        raw_ffi::Rf_unprotect(1);
        divan::black_box(sexp);
    }
}

#[divan::bench(args = [1, 4, 16])]
fn protect_unprotect_n(n: i32) {
    unsafe {
        for _ in 0..n {
            let sexp = raw_ffi::Rf_ScalarInteger(1);
            raw_ffi::Rf_protect(sexp);
        }
        raw_ffi::Rf_unprotect(n);
    }
}
// endregion

// region: Length retrieval benchmarks

#[divan::bench(args = [0, 2, 4])]
fn xlength(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let len = raw_ffi::Rf_xlength(fixtures.int_vec(size_idx));
        divan::black_box(len);
    }
}
// endregion

// region: Element access benchmarks (INTEGER_ELT, REAL_ELT, etc.)

#[divan::bench(args = [0, 2, 4])]
fn integer_elt(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    let val = fixtures.int_vec(size_idx).integer_elt(0);
    divan::black_box(val);
}

#[divan::bench(args = [0, 2, 4])]
fn real_elt(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    let val = fixtures.real_vec(size_idx).real_elt(0);
    divan::black_box(val);
}

#[divan::bench(args = [0, 2, 4])]
fn logical_elt(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    let val = fixtures.lgl_vec(size_idx).logical_elt(0);
    divan::black_box(val);
}
// endregion
