//! Benchmarks for coerced vs non-coerced conversions.
//!
//! Compares the cost of:
//! - Direct type access (no coercion needed)
//! - R-level coercion via `sexp.coerce()`
//! - Rust-level coercion via Coerce/TryCoerce traits

use miniextendr_api::TryFromSexp;
use miniextendr_api::coerce::{Coerce, TryCoerce};
use miniextendr_api::ffi::{SEXPTYPE, SexpExt};
use miniextendr_bench::SIZES;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

// region: Scalar: Direct vs R-coerced access

/// Direct read: INTSXP -> i32 (no coercion)
#[divan::bench]
fn scalar_int_direct() {
    let sexp = fixtures().int_vec(0); // size 1
    let val: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

/// R-coerced: INTSXP -> REALSXP -> f64
#[divan::bench]
fn scalar_int_to_real_r_coerce() {
    let sexp = fixtures().int_vec(0);
    let coerced = sexp.coerce(SEXPTYPE::REALSXP);
    let val: f64 = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(val);
}

/// Rust-coerced: INTSXP -> i32 -> f64 (via Coerce trait)
#[divan::bench]
fn scalar_int_to_real_rust_coerce() {
    let sexp = fixtures().int_vec(0);
    let val: i32 = TryFromSexp::try_from_sexp(sexp).unwrap();
    let coerced: f64 = val.coerce();
    divan::black_box(coerced);
}

/// Direct read: REALSXP -> f64 (no coercion)
#[divan::bench]
fn scalar_real_direct() {
    let sexp = fixtures().real_vec(0);
    let val: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(val);
}

/// R-coerced: REALSXP -> INTSXP -> i32
#[divan::bench]
fn scalar_real_to_int_r_coerce() {
    let sexp = fixtures().real_vec(0);
    let coerced = sexp.coerce(SEXPTYPE::INTSXP);
    let val: i32 = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(val);
}

/// Rust-coerced: REALSXP -> f64 -> i32 (via TryCoerce, checks for overflow/precision)
#[divan::bench]
fn scalar_real_to_int_rust_coerce() {
    let sexp = fixtures().real_vec(0);
    let val: f64 = TryFromSexp::try_from_sexp(sexp).unwrap();
    let coerced: i32 = val.try_coerce().unwrap();
    divan::black_box(coerced);
}
// endregion

// region: Vector: Direct vs R-coerced access

/// Direct slice: INTSXP -> &[i32] (zero-copy)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_int_slice_direct(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// R-coerced: INTSXP -> REALSXP -> &[f64]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_int_to_real_r_coerce(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let coerced = sexp.coerce(SEXPTYPE::REALSXP);
    let slice: &[f64] = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(slice);
}

/// Rust-coerced: INTSXP -> &[i32] -> Vec<f64> (element-wise)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_int_to_real_rust_coerce(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let coerced: Vec<f64> = slice.coerce();
    divan::black_box(coerced);
}

/// Direct slice: REALSXP -> &[f64] (zero-copy)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_real_slice_direct(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// R-coerced: REALSXP -> INTSXP -> &[i32]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_real_to_int_r_coerce(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let coerced = sexp.coerce(SEXPTYPE::INTSXP);
    let slice: &[i32] = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(slice);
}

/// Rust-coerced: REALSXP -> &[f64] -> Vec<i32> (element-wise with validation)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_real_to_int_rust_coerce(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    // Manual coercion since TryCoerce doesn't have blanket slice impl
    let coerced: Vec<i32> = slice.iter().map(|&x| x.try_coerce().unwrap()).collect();
    divan::black_box(coerced);
}
// endregion

// region: Logical coercion (LGLSXP <-> INTSXP)

/// Direct: LGLSXP -> &[RLogical]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_lgl_direct(size_idx: usize) {
    use miniextendr_api::ffi::RLogical;
    let sexp = fixtures().lgl_vec(size_idx);
    let slice: &[RLogical] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// R-coerced: LGLSXP -> INTSXP -> &[i32]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_lgl_to_int_r_coerce(size_idx: usize) {
    let sexp = fixtures().lgl_vec(size_idx);
    let coerced = sexp.coerce(SEXPTYPE::INTSXP);
    let slice: &[i32] = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(slice);
}

/// R-coerced: INTSXP -> LGLSXP -> &[RLogical]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_int_to_lgl_r_coerce(size_idx: usize) {
    use miniextendr_api::ffi::RLogical;
    let sexp = fixtures().int_vec(size_idx);
    let coerced = sexp.coerce(SEXPTYPE::LGLSXP);
    let slice: &[RLogical] = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(slice);
}
// endregion

// region: Raw coercion (RAWSXP <-> other)

/// Direct: RAWSXP -> &[u8]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_raw_direct(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let slice: &[u8] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// R-coerced: RAWSXP -> INTSXP -> &[i32]
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_raw_to_int_r_coerce(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let coerced = sexp.coerce(SEXPTYPE::INTSXP);
    let slice: &[i32] = TryFromSexp::try_from_sexp(coerced).unwrap();
    divan::black_box(slice);
}

/// Rust-coerced: RAWSXP -> &[u8] -> Vec<i32>
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_raw_to_int_rust_coerce(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let slice: &[u8] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let coerced: Vec<i32> = slice.coerce();
    divan::black_box(coerced);
}
// endregion

// region: Pure Rust coercion baselines (no R involved)

/// Baseline: i32 -> f64 coercion in pure Rust
#[divan::bench(args = SIZES)]
fn rust_only_i32_to_f64(n: usize) {
    let data: Vec<i32> = (0..n as i32).collect();
    let coerced: Vec<f64> = data.coerce();
    divan::black_box(coerced);
}

/// Baseline: f64 -> i32 coercion in pure Rust (with validation)
#[divan::bench(args = SIZES)]
fn rust_only_f64_to_i32(n: usize) {
    let data: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let coerced: Vec<i32> = data.iter().map(|&x| x.try_coerce().unwrap()).collect();
    divan::black_box(coerced);
}

/// Baseline: u8 -> i32 widening coercion
#[divan::bench(args = SIZES)]
fn rust_only_u8_to_i32(n: usize) {
    let data: Vec<u8> = (0..n).map(|i| (i % 256) as u8).collect();
    let coerced: Vec<i32> = data.coerce();
    divan::black_box(coerced);
}
// endregion
