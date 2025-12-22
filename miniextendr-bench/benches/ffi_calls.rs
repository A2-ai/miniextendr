//! FFI call overhead benchmarks.
//!
//! Measures the cost of calling R C-API functions through checked vs unchecked
//! wrappers.

use miniextendr_api::ffi::{self, SEXP, SEXPTYPE};
use miniextendr_bench::SIZES;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Vector allocation benchmarks
// =============================================================================

#[divan::bench(args = SIZES)]
fn alloc_intsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = ffi::Rf_allocVector(SEXPTYPE::INTSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_realsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = ffi::Rf_allocVector(SEXPTYPE::REALSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_lglsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = ffi::Rf_allocVector(SEXPTYPE::LGLSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_rawsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = ffi::Rf_allocVector(SEXPTYPE::RAWSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

#[divan::bench(args = SIZES)]
fn alloc_strsxp(n: usize) -> SEXP {
    unsafe {
        let sexp = ffi::Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
        divan::black_box(sexp)
    }
}

// =============================================================================
// Scalar creation benchmarks
// =============================================================================

#[divan::bench]
fn scalar_integer() -> SEXP {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(42);
        divan::black_box(sexp)
    }
}

#[divan::bench]
fn scalar_real() -> SEXP {
    unsafe {
        let sexp = ffi::Rf_ScalarReal(std::f64::consts::PI);
        divan::black_box(sexp)
    }
}

#[divan::bench]
fn scalar_logical() -> SEXP {
    unsafe {
        let sexp = ffi::Rf_ScalarLogical(1);
        divan::black_box(sexp)
    }
}

// =============================================================================
// Data pointer access benchmarks (no allocation, just pointer retrieval)
// =============================================================================

#[divan::bench(args = [0, 2, 4])]
fn integer_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = ffi::INTEGER(fixtures.int_vec(size_idx));
        divan::black_box(ptr);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn real_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = ffi::REAL(fixtures.real_vec(size_idx));
        divan::black_box(ptr);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn logical_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = ffi::LOGICAL(fixtures.lgl_vec(size_idx));
        divan::black_box(ptr);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn raw_ptr(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let ptr = ffi::RAW(fixtures.raw_vec(size_idx));
        divan::black_box(ptr);
    }
}

// =============================================================================
// PROTECT / UNPROTECT benchmarks
// =============================================================================

#[divan::bench]
fn protect_unprotect_single() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(1);
        ffi::Rf_protect(sexp);
        ffi::Rf_unprotect(1);
        divan::black_box(sexp);
    }
}

#[divan::bench(args = [1, 4, 16])]
fn protect_unprotect_n(n: i32) {
    unsafe {
        for _ in 0..n {
            let sexp = ffi::Rf_ScalarInteger(1);
            ffi::Rf_protect(sexp);
        }
        ffi::Rf_unprotect(n);
    }
}

// =============================================================================
// Length retrieval benchmarks
// =============================================================================

#[divan::bench(args = [0, 2, 4])]
fn xlength(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let len = ffi::Rf_xlength(fixtures.int_vec(size_idx));
        divan::black_box(len);
    }
}

// =============================================================================
// Element access benchmarks (INTEGER_ELT, REAL_ELT, etc.)
// =============================================================================

#[divan::bench(args = [0, 2, 4])]
fn integer_elt(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let val = ffi::INTEGER_ELT(fixtures.int_vec(size_idx), 0);
        divan::black_box(val);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn real_elt(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let val = ffi::REAL_ELT(fixtures.real_vec(size_idx), 0);
        divan::black_box(val);
    }
}

#[divan::bench(args = [0, 2, 4])]
fn logical_elt(size_idx: usize) {
    let fixtures = miniextendr_bench::fixtures();
    unsafe {
        let val = ffi::LOGICAL_ELT(fixtures.lgl_vec(size_idx), 0);
        divan::black_box(val);
    }
}
