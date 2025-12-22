//! Benchmarks for R -> Rust conversions (TryFromSexp trait).
//!
//! Measures the cost of converting R SEXP values to Rust types.

use miniextendr_api::TryFromSexp;
use miniextendr_api::ffi;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

// =============================================================================
// Scalar conversions
// =============================================================================

#[divan::bench]
fn scalar_i32() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(42);
        let val: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }
}

#[divan::bench]
fn scalar_f64() {
    unsafe {
        let sexp = ffi::Rf_ScalarReal(std::f64::consts::PI);
        let val: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }
}

#[divan::bench]
fn scalar_bool() {
    unsafe {
        let sexp = ffi::Rf_ScalarLogical(1);
        let val: bool = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }
}

#[divan::bench]
fn scalar_option_i32_value() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(42);
        let val: Option<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }
}

#[divan::bench]
fn scalar_option_i32_na() {
    unsafe {
        let sexp = ffi::Rf_ScalarInteger(i32::MIN); // NA_INTEGER
        let val: Option<i32> = TryFromSexp::try_from_sexp(sexp).unwrap();
        divan::black_box(val);
    }
}

// =============================================================================
// Slice conversions - zero-copy
// =============================================================================

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn slice_i32(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let val: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn slice_f64(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let val: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn slice_u8(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let val: &[u8] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

// =============================================================================
// Manual pointer access comparison (baseline)
// =============================================================================

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn manual_slice_i32(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    unsafe {
        let ptr = ffi::INTEGER(sexp);
        let len = ffi::Rf_xlength(sexp) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        divan::black_box(slice);
    }
}

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn manual_slice_f64(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    unsafe {
        let ptr = ffi::REAL(sexp);
        let len = ffi::Rf_xlength(sexp) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        divan::black_box(slice);
    }
}

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn manual_slice_u8(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    unsafe {
        let ptr = ffi::RAW(sexp);
        let len = ffi::Rf_xlength(sexp) as usize;
        let slice = std::slice::from_raw_parts(ptr, len);
        divan::black_box(slice);
    }
}

// =============================================================================
// String conversions
// =============================================================================

#[divan::bench]
fn string_utf8() {
    let sexp = fixtures().utf8_strsxp();
    let val: String = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

#[divan::bench]
fn string_latin1() {
    let sexp = fixtures().latin1_strsxp();
    let val: String = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

// =============================================================================
// Element-by-element iteration (using raw pointers)
// =============================================================================

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn iterate_int_elt(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    unsafe {
        let len = ffi::Rf_xlength(sexp);
        let mut sum = 0i64;
        for i in 0..len {
            sum += ffi::INTEGER_ELT(sexp, i) as i64;
        }
        divan::black_box(sum);
    }
}

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn iterate_int_ptr(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    unsafe {
        let ptr = ffi::INTEGER(sexp);
        let len = ffi::Rf_xlength(sexp) as usize;
        let mut sum = 0i64;
        for i in 0..len {
            sum += *ptr.add(i) as i64;
        }
        divan::black_box(sum);
    }
}

#[divan::bench(args = [0, 1, 2, 3, 4])]
fn iterate_int_slice(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let sum: i64 = slice.iter().map(|&x| x as i64).sum();
    divan::black_box(sum);
}
