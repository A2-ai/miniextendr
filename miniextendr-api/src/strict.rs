//! Strict conversion helpers for `#[miniextendr(strict)]`.
//!
//! These functions panic instead of silently widening when a value cannot be
//! exactly represented as an R integer (`INTSXP`). This provides an opt-in
//! alternative to the default `IntoR` behavior which silently falls back to
//! `REALSXP` (f64) for out-of-range values.
//!
//! # Motivation
//!
//! R has no native 64-bit integer type. The default `i64::into_sexp()` picks
//! `INTSXP` when the value fits and `REALSXP` otherwise — silently losing
//! precision for values outside `[-2^53, 2^53]`. With `#[miniextendr(strict)]`,
//! the macro generates calls to these helpers instead, which panic (→ R error)
//! if the value doesn't fit in i32.

use crate::ffi::SEXP;
use crate::into_r::IntoR;

/// Convert `i64` to R integer, panicking if outside i32 range.
///
/// The valid range is `(i32::MIN, i32::MAX]` — `i32::MIN` is excluded because
/// it is `NA_integer_` in R.
#[inline]
pub fn checked_into_sexp_i64(val: i64) -> SEXP {
    if val > i32::MIN as i64 && val <= i32::MAX as i64 {
        (val as i32).into_sexp()
    } else {
        panic!(
            "strict conversion failed: i64 value {} is outside R integer range \
             ({}..={}); use a non-strict function to allow lossy f64 widening",
            val,
            i32::MIN as i64 + 1,
            i32::MAX
        );
    }
}

/// Convert `u64` to R integer, panicking if > i32::MAX.
#[inline]
pub fn checked_into_sexp_u64(val: u64) -> SEXP {
    if val <= i32::MAX as u64 {
        (val as i32).into_sexp()
    } else {
        panic!(
            "strict conversion failed: u64 value {} exceeds R integer max ({}); \
             use a non-strict function to allow lossy f64 widening",
            val, i32::MAX
        );
    }
}

/// Convert `isize` to R integer, panicking if outside i32 range.
#[inline]
pub fn checked_into_sexp_isize(val: isize) -> SEXP {
    checked_into_sexp_i64(val as i64)
}

/// Convert `usize` to R integer, panicking if > i32::MAX.
#[inline]
pub fn checked_into_sexp_usize(val: usize) -> SEXP {
    checked_into_sexp_u64(val as u64)
}

/// Convert `Vec<i64>` to R integer vector, panicking if any element is outside i32 range.
pub fn checked_vec_i64_into_sexp(val: Vec<i64>) -> SEXP {
    let coerced: Vec<i32> = val
        .into_iter()
        .map(|x| {
            if x > i32::MIN as i64 && x <= i32::MAX as i64 {
                x as i32
            } else {
                panic!(
                    "strict conversion failed: i64 value {} is outside R integer range \
                     ({}..={}); use a non-strict function to allow lossy f64 widening",
                    x,
                    i32::MIN as i64 + 1,
                    i32::MAX
                );
            }
        })
        .collect();
    coerced.into_sexp()
}

/// Convert `Vec<u64>` to R integer vector, panicking if any element > i32::MAX.
pub fn checked_vec_u64_into_sexp(val: Vec<u64>) -> SEXP {
    let coerced: Vec<i32> = val
        .into_iter()
        .map(|x| {
            if x <= i32::MAX as u64 {
                x as i32
            } else {
                panic!(
                    "strict conversion failed: u64 value {} exceeds R integer max ({}); \
                     use a non-strict function to allow lossy f64 widening",
                    x, i32::MAX
                );
            }
        })
        .collect();
    coerced.into_sexp()
}

/// Convert `Vec<isize>` to R integer vector, panicking if any element is outside i32 range.
pub fn checked_vec_isize_into_sexp(val: Vec<isize>) -> SEXP {
    checked_vec_i64_into_sexp(val.into_iter().map(|x| x as i64).collect())
}

/// Convert `Vec<usize>` to R integer vector, panicking if any element > i32::MAX.
pub fn checked_vec_usize_into_sexp(val: Vec<usize>) -> SEXP {
    checked_vec_u64_into_sexp(val.into_iter().map(|x| x as u64).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64_in_range_succeeds() {
        // These should not panic (we can't check SEXP in unit tests without R,
        // but we can verify no panic occurs)
        let _ = std::panic::catch_unwind(|| checked_into_sexp_i64(0));
        let _ = std::panic::catch_unwind(|| checked_into_sexp_i64(42));
        let _ = std::panic::catch_unwind(|| checked_into_sexp_i64(-1));
        let _ = std::panic::catch_unwind(|| checked_into_sexp_i64(i32::MAX as i64));
    }

    #[test]
    fn i64_out_of_range_panics() {
        let result = std::panic::catch_unwind(|| checked_into_sexp_i64(i64::MAX));
        assert!(result.is_err(), "should panic for i64::MAX");

        let result = std::panic::catch_unwind(|| checked_into_sexp_i64(i32::MIN as i64));
        assert!(result.is_err(), "should panic for i32::MIN (NA_integer_)");

        let result = std::panic::catch_unwind(|| checked_into_sexp_i64(i32::MAX as i64 + 1));
        assert!(result.is_err(), "should panic for i32::MAX + 1");
    }

    #[test]
    fn u64_in_range_succeeds() {
        let _ = std::panic::catch_unwind(|| checked_into_sexp_u64(0));
        let _ = std::panic::catch_unwind(|| checked_into_sexp_u64(i32::MAX as u64));
    }

    #[test]
    fn u64_out_of_range_panics() {
        let result = std::panic::catch_unwind(|| checked_into_sexp_u64(i32::MAX as u64 + 1));
        assert!(result.is_err());
    }
}
