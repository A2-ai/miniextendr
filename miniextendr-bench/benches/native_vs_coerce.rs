//! Benchmarks comparing RNative path vs Coercion path.
//!
//! R has fixed native types (i32, f64, u8). Getting a `Vec<i32>` from R
//! uses the **RNative path**: get slice, then copy. Getting a `Vec<i64>`
//! requires the **Coercion path**: read each element and widen.
//!
//! This benchmark shows the cost difference between:
//! - `&[i32]` - zero-copy slice (baseline)
//! - `Vec<i32>` - RNative: memcpy from slice
//! - `Vec<i64>` - Coercion: element-wise i32 → i64 widening
//! - `Vec<u32>` - Coercion: element-wise i32 → u32 with bounds check

use miniextendr_api::TryFromSexp;
use miniextendr_api::coerce::TryCoerce;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[inline(always)]
fn fixtures() -> miniextendr_bench::Fixtures {
    miniextendr_bench::fixtures()
}

// =============================================================================
// Integer conversions: RNative vs Coercion
// =============================================================================

/// Baseline: INTSXP → &[i32] (zero-copy, O(1))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn slice_i32_zerocopy(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// RNative path: INTSXP → Vec<i32> (memcpy, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_i32_rnative(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<i32> = slice.to_vec();
    divan::black_box(vec);
}

/// Coercion path: INTSXP → Vec<i64> (element-wise widening, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_i64_coerce(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<i64> = slice.iter().map(|&x| x as i64).collect();
    divan::black_box(vec);
}

/// Coercion path: INTSXP → Vec<u32> (element-wise with TryCoerce check, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_u32_coerce_checked(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<u32> = slice.iter().map(|&x| x.try_coerce().unwrap()).collect();
    divan::black_box(vec);
}

/// Coercion path: INTSXP → Vec<u32> (unchecked cast, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_u32_coerce_unchecked(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<u32> = slice.iter().map(|&x| x as u32).collect();
    divan::black_box(vec);
}

/// Coercion path: INTSXP → Vec<usize> (element-wise, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_usize_coerce(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<usize> = slice.iter().map(|&x| x.try_coerce().unwrap()).collect();
    divan::black_box(vec);
}

// =============================================================================
// Real (f64) conversions: RNative vs Coercion
// =============================================================================

/// Baseline: REALSXP → &[f64] (zero-copy, O(1))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn slice_f64_zerocopy(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// RNative path: REALSXP → Vec<f64> (memcpy, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_f64_rnative(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<f64> = slice.to_vec();
    divan::black_box(vec);
}

/// Coercion path: REALSXP → Vec<f32> (narrowing, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_f32_coerce(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<f32> = slice.iter().map(|&x| x as f32).collect();
    divan::black_box(vec);
}

// =============================================================================
// Raw (u8) conversions: RNative vs Coercion
// =============================================================================

/// Baseline: RAWSXP → &[u8] (zero-copy, O(1))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn slice_u8_zerocopy(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let slice: &[u8] = TryFromSexp::try_from_sexp(sexp).unwrap();
    divan::black_box(slice);
}

/// RNative path: RAWSXP → Vec<u8> (memcpy, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_u8_rnative(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let slice: &[u8] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<u8> = slice.to_vec();
    divan::black_box(vec);
}

/// Coercion path: RAWSXP → Vec<i32> (widening, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn vec_i32_from_raw_coerce(size_idx: usize) {
    let sexp = fixtures().raw_vec(size_idx);
    let slice: &[u8] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<i32> = slice.iter().map(|&x| x as i32).collect();
    divan::black_box(vec);
}

// =============================================================================
// Cross-type: Integer to Real coercion comparison
// =============================================================================

/// RNative→RNative: INTSXP → Vec<i32> (memcpy)
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn int_to_vec_i32_memcpy(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<i32> = slice.to_vec();
    divan::black_box(vec);
}

/// Cross-type: INTSXP → Vec<f64> (coercion i32→f64, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn int_to_vec_f64_coerce(size_idx: usize) {
    let sexp = fixtures().int_vec(size_idx);
    let slice: &[i32] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<f64> = slice.iter().map(|&x| x as f64).collect();
    divan::black_box(vec);
}

/// Cross-type: REALSXP → Vec<i32> (coercion f64→i32, O(n))
#[divan::bench(args = [0, 1, 2, 3, 4])]
fn real_to_vec_i32_coerce(size_idx: usize) {
    let sexp = fixtures().real_vec(size_idx);
    let slice: &[f64] = TryFromSexp::try_from_sexp(sexp).unwrap();
    let vec: Vec<i32> = slice.iter().map(|&x| x as i32).collect();
    divan::black_box(vec);
}
