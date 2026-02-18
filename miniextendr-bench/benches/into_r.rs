//! Benchmarks for Rust -> R conversions (IntoR trait).
//!
//! Measures the cost of converting Rust types to R SEXP values.

use miniextendr_api::IntoR;
use miniextendr_api::ffi::SEXP;
use miniextendr_bench::{LARGE_SIZES, SIZES};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// =============================================================================
// Scalar conversions
// =============================================================================

#[divan::bench]
fn scalar_i32() -> SEXP {
    let val: i32 = 42;
    divan::black_box(val.into_sexp())
}

#[divan::bench]
fn scalar_f64() -> SEXP {
    let val: f64 = std::f64::consts::PI;
    divan::black_box(val.into_sexp())
}

#[divan::bench]
fn scalar_bool() -> SEXP {
    let val: bool = true;
    divan::black_box(val.into_sexp())
}

#[divan::bench]
fn scalar_option_i32_some() -> SEXP {
    let val: Option<i32> = Some(42);
    divan::black_box(val.into_sexp())
}

#[divan::bench]
fn scalar_option_i32_none() -> SEXP {
    let val: Option<i32> = None;
    divan::black_box(val.into_sexp())
}

#[divan::bench]
fn scalar_option_f64_some() -> SEXP {
    let val: Option<f64> = Some(std::f64::consts::PI);
    divan::black_box(val.into_sexp())
}

#[divan::bench]
fn scalar_option_f64_none() -> SEXP {
    let val: Option<f64> = None;
    divan::black_box(val.into_sexp())
}

// =============================================================================
// Vector conversions - integers
// =============================================================================

/// Pre-generate Vec<i32> of given size for benchmarking.
fn make_int_vec(n: usize) -> Vec<i32> {
    (0..n as i32).collect()
}

#[divan::bench(args = SIZES)]
fn vec_i32(n: usize) -> SEXP {
    let vec = make_int_vec(n);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn slice_i32(bencher: divan::Bencher, n: usize) {
    let vec = make_int_vec(n);
    bencher.bench(|| {
        let slice: &[i32] = &vec;
        divan::black_box(slice.into_sexp())
    });
}

// =============================================================================
// Vector conversions - floats
// =============================================================================

fn make_real_vec(n: usize) -> Vec<f64> {
    (0..n).map(|i| i as f64).collect()
}

#[divan::bench(args = SIZES)]
fn vec_f64(n: usize) -> SEXP {
    let vec = make_real_vec(n);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn slice_f64(bencher: divan::Bencher, n: usize) {
    let vec = make_real_vec(n);
    bencher.bench(|| {
        let slice: &[f64] = &vec;
        divan::black_box(slice.into_sexp())
    });
}

// =============================================================================
// Vector conversions - logical (RLogical)
// =============================================================================

use miniextendr_api::ffi::RLogical;

fn make_logical_vec(n: usize) -> Vec<RLogical> {
    (0..n)
        .map(|i| {
            if i % 2 == 0 {
                RLogical::TRUE
            } else {
                RLogical::FALSE
            }
        })
        .collect()
}

#[divan::bench(args = SIZES)]
fn vec_logical(n: usize) -> SEXP {
    let vec = make_logical_vec(n);
    divan::black_box(vec.into_sexp())
}

// =============================================================================
// Vector conversions - raw bytes
// =============================================================================

fn make_u8_vec(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 256) as u8).collect()
}

#[divan::bench(args = SIZES)]
fn vec_u8(n: usize) -> SEXP {
    let vec = make_u8_vec(n);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn slice_u8(bencher: divan::Bencher, n: usize) {
    let vec = make_u8_vec(n);
    bencher.bench(|| {
        let slice: &[u8] = &vec;
        divan::black_box(slice.into_sexp())
    });
}

// =============================================================================
// Vector conversions - Option<T> (NA handling)
// =============================================================================

fn make_option_i32_vec(n: usize, na_density: f64) -> Vec<Option<i32>> {
    (0..n)
        .map(|i| {
            if (i as f64 / n as f64) < na_density {
                None
            } else {
                Some(i as i32)
            }
        })
        .collect()
}

#[divan::bench(args = SIZES)]
fn vec_option_i32_no_na(n: usize) -> SEXP {
    let vec = make_option_i32_vec(n, 0.0);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn vec_option_i32_10pct_na(n: usize) -> SEXP {
    let vec = make_option_i32_vec(n, 0.1);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn vec_option_i32_50pct_na(n: usize) -> SEXP {
    let vec = make_option_i32_vec(n, 0.5);
    divan::black_box(vec.into_sexp())
}

fn make_option_f64_vec(n: usize, na_density: f64) -> Vec<Option<f64>> {
    (0..n)
        .map(|i| {
            if (i as f64 / n as f64) < na_density {
                None
            } else {
                Some(i as f64)
            }
        })
        .collect()
}

#[divan::bench(args = SIZES)]
fn vec_option_f64_no_na(n: usize) -> SEXP {
    let vec = make_option_f64_vec(n, 0.0);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn vec_option_f64_10pct_na(n: usize) -> SEXP {
    let vec = make_option_f64_vec(n, 0.1);
    divan::black_box(vec.into_sexp())
}

// =============================================================================
// String conversions
// =============================================================================

fn make_string_vec(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("item_{}", i)).collect()
}

#[divan::bench(args = SIZES)]
fn vec_string(n: usize) -> SEXP {
    let vec = make_string_vec(n);
    divan::black_box(vec.into_sexp())
}

fn make_str_vec(n: usize) -> Vec<&'static str> {
    // Use a static slice to avoid lifetime issues
    static ITEMS: [&str; 10] = [
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
    ];
    (0..n).map(|i| ITEMS[i % ITEMS.len()]).collect()
}

#[divan::bench(args = SIZES)]
fn vec_str(n: usize) -> SEXP {
    let vec = make_str_vec(n);
    divan::black_box(vec.into_sexp())
}

fn make_option_string_vec(n: usize, na_density: f64) -> Vec<Option<String>> {
    (0..n)
        .map(|i| {
            if (i as f64 / n as f64) < na_density {
                None
            } else {
                Some(format!("item_{}", i))
            }
        })
        .collect()
}

#[divan::bench(args = SIZES)]
fn vec_option_string_no_na(n: usize) -> SEXP {
    let vec = make_option_string_vec(n, 0.0);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = SIZES)]
fn vec_option_string_10pct_na(n: usize) -> SEXP {
    let vec = make_option_string_vec(n, 0.1);
    divan::black_box(vec.into_sexp())
}

// =============================================================================
// Unit and special types
// =============================================================================

#[divan::bench]
fn unit_type() -> SEXP {
    divan::black_box(().into_sexp())
}

// =============================================================================
// Large-scale benchmarks (A10: detect GC pressure / cache effects at 100K-1M)
// =============================================================================

#[divan::bench(args = LARGE_SIZES)]
fn scale_vec_i32(n: usize) -> SEXP {
    let vec = make_int_vec(n);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = LARGE_SIZES)]
fn scale_vec_f64(n: usize) -> SEXP {
    let vec = make_real_vec(n);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = LARGE_SIZES)]
fn scale_vec_string(n: usize) -> SEXP {
    let vec = make_string_vec(n);
    divan::black_box(vec.into_sexp())
}

#[divan::bench(args = LARGE_SIZES)]
fn scale_vec_option_i32_50pct_na(n: usize) -> SEXP {
    let vec = make_option_i32_vec(n, 0.5);
    divan::black_box(vec.into_sexp())
}
