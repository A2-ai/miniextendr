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
//!
//! # Paired with
//!
//! This is the **strict outbound** path. Its lax counterpart is the bare
//! [`IntoR`] impls for `i64`/`u64`/`isize`/`usize` (see [`crate::into_r`]).
//! Failure mode of staying on the lax path when you cared about exact
//! representation: a counter / ID just above `i32::MAX` lands in R as
//! `REALSXP`, and anything above `2^53` starts colliding silently.
//!
//! There is **no `TryFromSexpStrict` trait** — inbound is already
//! strict-by-default because [`crate::from_r::TryFromSexp`] returns
//! `Result<T, SexpError>`. The looser inbound path is
//! [`crate::coerce::Coerce`] / [`crate::coerce::TryCoerce`].
//!
//! For storage-directed conversions (force `Vec<i64>` into `INTSXP` and error
//! if any element doesn't fit) see [`crate::into_r_as::IntoRAs`] — it's a
//! third path with value-based runtime checks.

use crate::coerce::TryCoerce;
use crate::from_r::{BatchedErrors, SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::{SEXP, SEXPTYPE, SexpExt};

/// Fold a batched strict-vec [`BatchedErrors`] into the single panic every
/// `checked_vec_*_into_sexp` raises, under `container` (e.g. `"Vec<i64>"`).
///
/// Reuses [`BatchedErrors::into_error`]'s `"<container> conversion failed:
/// invalid value at index <i>: ...; and N more"` grammar (#1192/#1097), but
/// extracts the inner message directly rather than going through
/// `SexpError`'s `Display` impl — that impl wraps every variant as `"invalid
/// value: {msg}"`, which would read as a doubled "invalid value: `Vec<i64>`
/// conversion failed: invalid value at index 0: ..." in a panic message.
fn panic_strict_vec_batched(container: &str, errors: BatchedErrors) -> ! {
    let SexpError::InvalidValue(msg) = errors.into_error(container) else {
        unreachable!("BatchedErrors::into_error always returns SexpError::InvalidValue")
    };
    panic!(
        "strict conversion failed: {msg}; use a non-strict function to allow lossy f64 widening"
    );
}

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
            val,
            i32::MAX
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
///
/// Walks the whole vector, batching every out-of-range element into one
/// panic instead of aborting at the first — see `panic_strict_vec_batched`.
pub fn checked_vec_i64_into_sexp(val: Vec<i64>) -> SEXP {
    let mut coerced: Vec<i32> = Vec::with_capacity(val.len());
    let mut errors = BatchedErrors::default();
    for (i, x) in val.into_iter().enumerate() {
        if x > i32::MIN as i64 && x <= i32::MAX as i64 {
            coerced.push(x as i32);
        } else {
            errors.push(|| {
                format!(
                    "invalid value at index {i}: i64 value {x} is outside R integer range \
                     ({}..={})",
                    i32::MIN as i64 + 1,
                    i32::MAX
                )
            });
        }
    }
    if !errors.is_empty() {
        panic_strict_vec_batched("Vec<i64>", errors);
    }
    coerced.into_sexp()
}

/// Convert `Vec<u64>` to R integer vector, panicking if any element > i32::MAX.
///
/// Walks the whole vector, batching every out-of-range element into one
/// panic instead of aborting at the first — see `panic_strict_vec_batched`.
pub fn checked_vec_u64_into_sexp(val: Vec<u64>) -> SEXP {
    let mut coerced: Vec<i32> = Vec::with_capacity(val.len());
    let mut errors = BatchedErrors::default();
    for (i, x) in val.into_iter().enumerate() {
        if x <= i32::MAX as u64 {
            coerced.push(x as i32);
        } else {
            errors.push(|| {
                format!(
                    "invalid value at index {i}: u64 value {x} exceeds R integer max ({})",
                    i32::MAX
                )
            });
        }
    }
    if !errors.is_empty() {
        panic_strict_vec_batched("Vec<u64>", errors);
    }
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

/// Convert `Vec<Option<i64>>` to R integer vector in strict mode.
///
/// Panics if any `Some(x)` value is outside i32 range. `None` becomes
/// `NA_INTEGER`. Walks the whole vector, batching every out-of-range `Some`
/// into one panic instead of aborting at the first — see
/// `panic_strict_vec_batched`.
pub fn checked_vec_option_i64_into_sexp(val: Vec<Option<i64>>) -> SEXP {
    let mut coerced: Vec<Option<i32>> = Vec::with_capacity(val.len());
    let mut errors = BatchedErrors::default();
    for (i, opt) in val.into_iter().enumerate() {
        match opt {
            Some(x) => {
                if x > i32::MIN as i64 && x <= i32::MAX as i64 {
                    coerced.push(Some(x as i32));
                } else {
                    errors.push(|| {
                        format!(
                            "invalid value at index {i}: i64 value {x} is outside R integer range \
                             ({}..={})",
                            i32::MIN as i64 + 1,
                            i32::MAX
                        )
                    });
                }
            }
            None => coerced.push(None),
        }
    }
    if !errors.is_empty() {
        panic_strict_vec_batched("Vec<Option<i64>>", errors);
    }
    coerced.into_sexp()
}

/// Convert `Vec<Option<u64>>` to R integer vector in strict mode.
///
/// Walks the whole vector, batching every out-of-range `Some` into one
/// panic instead of aborting at the first — see `panic_strict_vec_batched`.
pub fn checked_vec_option_u64_into_sexp(val: Vec<Option<u64>>) -> SEXP {
    let mut coerced: Vec<Option<i32>> = Vec::with_capacity(val.len());
    let mut errors = BatchedErrors::default();
    for (i, opt) in val.into_iter().enumerate() {
        match opt {
            Some(x) => {
                if x <= i32::MAX as u64 {
                    coerced.push(Some(x as i32));
                } else {
                    errors.push(|| {
                        format!(
                            "invalid value at index {i}: u64 value {x} exceeds R integer max ({})",
                            i32::MAX
                        )
                    });
                }
            }
            None => coerced.push(None),
        }
    }
    if !errors.is_empty() {
        panic_strict_vec_batched("Vec<Option<u64>>", errors);
    }
    coerced.into_sexp()
}

/// Convert `Vec<Option<isize>>` to R integer vector in strict mode.
pub fn checked_vec_option_isize_into_sexp(val: Vec<Option<isize>>) -> SEXP {
    checked_vec_option_i64_into_sexp(val.into_iter().map(|opt| opt.map(|x| x as i64)).collect())
}

/// Convert `Vec<Option<usize>>` to R integer vector in strict mode.
pub fn checked_vec_option_usize_into_sexp(val: Vec<Option<usize>>) -> SEXP {
    checked_vec_option_u64_into_sexp(val.into_iter().map(|opt| opt.map(|x| x as u64)).collect())
}

/// Convert `Option<i64>` to R integer in strict mode.
/// Panics if `Some(x)` is outside i32 range. `None` becomes `NA_integer_`.
#[inline]
pub fn checked_option_i64_into_sexp(val: Option<i64>) -> SEXP {
    match val {
        Some(x) => checked_into_sexp_i64(x),
        None => Option::<i32>::None.into_sexp(),
    }
}

/// Convert `Option<u64>` to R integer in strict mode.
/// Panics if `Some(x)` exceeds i32::MAX. `None` becomes `NA_integer_`.
#[inline]
pub fn checked_option_u64_into_sexp(val: Option<u64>) -> SEXP {
    match val {
        Some(x) => checked_into_sexp_u64(x),
        None => Option::<i32>::None.into_sexp(),
    }
}

/// Convert `Option<isize>` to R integer in strict mode.
#[inline]
pub fn checked_option_isize_into_sexp(val: Option<isize>) -> SEXP {
    checked_option_i64_into_sexp(val.map(|x| x as i64))
}

/// Convert `Option<usize>` to R integer in strict mode.
#[inline]
pub fn checked_option_usize_into_sexp(val: Option<usize>) -> SEXP {
    checked_option_u64_into_sexp(val.map(|x| x as u64))
}

// region: Strict INPUT helpers — only accept INTSXP and REALSXP, reject RAWSXP/LGLSXP

/// Convert R SEXP to `i64` in strict mode.
///
/// Only INTSXP and REALSXP are accepted. RAWSXP and LGLSXP are rejected.
/// For REALSXP, uses `TryCoerce` to reject fractional, NaN, and out-of-range values.
#[inline]
pub fn checked_try_from_sexp_i64(sexp: SEXP, param: &str) -> i64 {
    checked_try_from_sexp_numeric_scalar::<i64>(sexp, param)
}

/// Convert R SEXP to `u64` in strict mode.
#[inline]
pub fn checked_try_from_sexp_u64(sexp: SEXP, param: &str) -> u64 {
    checked_try_from_sexp_numeric_scalar::<u64>(sexp, param)
}

/// Convert R SEXP to `isize` in strict mode.
#[inline]
pub fn checked_try_from_sexp_isize(sexp: SEXP, param: &str) -> isize {
    let val = checked_try_from_sexp_i64(sexp, param);
    isize::try_from(val).unwrap_or_else(|_| {
        panic!(
            "strict conversion failed for parameter '{}': i64 value {} does not fit in isize",
            param, val
        )
    })
}

/// Convert R SEXP to `usize` in strict mode.
#[inline]
pub fn checked_try_from_sexp_usize(sexp: SEXP, param: &str) -> usize {
    let val = checked_try_from_sexp_u64(sexp, param);
    usize::try_from(val).unwrap_or_else(|_| {
        panic!(
            "strict conversion failed for parameter '{}': u64 value {} does not fit in usize",
            param, val
        )
    })
}

/// Convert R SEXP to `Vec<i64>` in strict mode.
pub fn checked_vec_try_from_sexp_i64(sexp: SEXP, param: &str) -> Vec<i64> {
    checked_vec_try_from_sexp_numeric::<i64>(sexp, param)
}

/// Convert R SEXP to `Vec<u64>` in strict mode.
pub fn checked_vec_try_from_sexp_u64(sexp: SEXP, param: &str) -> Vec<u64> {
    checked_vec_try_from_sexp_numeric::<u64>(sexp, param)
}

/// Convert R SEXP to `Vec<isize>` in strict mode.
pub fn checked_vec_try_from_sexp_isize(sexp: SEXP, param: &str) -> Vec<isize> {
    checked_vec_try_from_sexp_i64(sexp, param)
        .into_iter()
        .map(|x| {
            isize::try_from(x).unwrap_or_else(|_| {
            panic!(
                "strict conversion failed for parameter '{}': i64 value {} does not fit in isize",
                param, x
            )
        })
        })
        .collect()
}

/// Convert R SEXP to `Vec<usize>` in strict mode.
pub fn checked_vec_try_from_sexp_usize(sexp: SEXP, param: &str) -> Vec<usize> {
    checked_vec_try_from_sexp_u64(sexp, param)
        .into_iter()
        .map(|x| {
            usize::try_from(x).unwrap_or_else(|_| {
            panic!(
                "strict conversion failed for parameter '{}': u64 value {} does not fit in usize",
                param, x
            )
        })
        })
        .collect()
}

/// Convert R SEXP to `Vec<Option<i64>>` in strict mode.
///
/// Applies the same input-SEXP-type gate as [`checked_vec_try_from_sexp_i64`]
/// — only INTSXP and REALSXP are accepted; LGLSXP and RAWSXP are rejected.
/// NA elements become `None`; type strictness and missingness are orthogonal.
pub fn checked_vec_option_try_from_sexp_i64(sexp: SEXP, param: &str) -> Vec<Option<i64>> {
    checked_vec_option_try_from_sexp_numeric::<i64>(sexp, param)
}

/// Convert R SEXP to `Vec<Option<u64>>` in strict mode.
pub fn checked_vec_option_try_from_sexp_u64(sexp: SEXP, param: &str) -> Vec<Option<u64>> {
    checked_vec_option_try_from_sexp_numeric::<u64>(sexp, param)
}

/// Convert R SEXP to `Vec<Option<isize>>` in strict mode.
pub fn checked_vec_option_try_from_sexp_isize(sexp: SEXP, param: &str) -> Vec<Option<isize>> {
    checked_vec_option_try_from_sexp_i64(sexp, param)
        .into_iter()
        .map(|opt| {
            opt.map(|x| {
                isize::try_from(x).unwrap_or_else(|_| {
                    panic!(
                        "strict conversion failed for parameter '{}': i64 value {} does not fit in isize",
                        param, x
                    )
                })
            })
        })
        .collect()
}

/// Convert R SEXP to `Vec<Option<usize>>` in strict mode.
pub fn checked_vec_option_try_from_sexp_usize(sexp: SEXP, param: &str) -> Vec<Option<usize>> {
    checked_vec_option_try_from_sexp_u64(sexp, param)
        .into_iter()
        .map(|opt| {
            opt.map(|x| {
                usize::try_from(x).unwrap_or_else(|_| {
                    panic!(
                        "strict conversion failed for parameter '{}': u64 value {} does not fit in usize",
                        param, x
                    )
                })
            })
        })
        .collect()
}

/// Generic strict scalar conversion: only INTSXP and REALSXP allowed.
#[inline]
fn checked_try_from_sexp_numeric_scalar<T>(sexp: SEXP, param: &str) -> T
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = TryFromSexp::try_from_sexp(sexp).unwrap_or_else(|e| {
                panic!(
                    "strict conversion failed for parameter '{}': {:?}",
                    param, e
                )
            });
            TryCoerce::<T>::try_coerce(value).unwrap_or_else(|e| {
                panic!(
                    "strict conversion failed for parameter '{}': {:?}",
                    param, e
                )
            })
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = TryFromSexp::try_from_sexp(sexp).unwrap_or_else(|e| {
                panic!(
                    "strict conversion failed for parameter '{}': {:?}",
                    param, e
                )
            });
            TryCoerce::<T>::try_coerce(value).unwrap_or_else(|e| {
                panic!(
                    "strict conversion failed for parameter '{}': {:?}",
                    param, e
                )
            })
        }
        _ => panic!(
            "strict conversion failed for parameter '{}': expected integer or double, got {:?}",
            param, actual
        ),
    }
}

/// Generic strict vector conversion: only INTSXP and REALSXP allowed.
fn checked_vec_try_from_sexp_numeric<T>(sexp: SEXP, param: &str) -> Vec<T>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .copied()
                .map(|v| {
                    TryCoerce::<T>::try_coerce(v).unwrap_or_else(|e| {
                        panic!(
                            "strict conversion failed for parameter '{}': {:?}",
                            param, e
                        )
                    })
                })
                .collect()
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .copied()
                .map(|v| {
                    TryCoerce::<T>::try_coerce(v).unwrap_or_else(|e| {
                        panic!(
                            "strict conversion failed for parameter '{}': {:?}",
                            param, e
                        )
                    })
                })
                .collect()
        }
        _ => panic!(
            "strict conversion failed for parameter '{}': expected integer or double vector, got {:?}",
            param, actual
        ),
    }
}

/// Generic strict `Vec<Option<T>>` conversion: only INTSXP and REALSXP allowed.
///
/// Mirrors [`checked_vec_try_from_sexp_numeric`] but maps R's NA sentinel
/// (`NA_INTEGER` for INTSXP, `NA_REAL` for REALSXP) to `None` instead of
/// erroring — missingness is orthogonal to the input-type gate.
fn checked_vec_option_try_from_sexp_numeric<T>(sexp: SEXP, param: &str) -> Vec<Option<T>>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Debug,
    <f64 as TryCoerce<T>>::Error: std::fmt::Debug,
{
    let actual = sexp.type_of();
    match actual {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .copied()
                .map(|v| {
                    if v == crate::altrep_traits::NA_INTEGER {
                        None
                    } else {
                        Some(TryCoerce::<T>::try_coerce(v).unwrap_or_else(|e| {
                            panic!(
                                "strict conversion failed for parameter '{}': {:?}",
                                param, e
                            )
                        }))
                    }
                })
                .collect()
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            slice
                .iter()
                .copied()
                .map(|v| {
                    if crate::from_r::is_na_real(v) {
                        None
                    } else {
                        Some(TryCoerce::<T>::try_coerce(v).unwrap_or_else(|e| {
                            panic!(
                                "strict conversion failed for parameter '{}': {:?}",
                                param, e
                            )
                        }))
                    }
                })
                .collect()
        }
        _ => panic!(
            "strict conversion failed for parameter '{}': expected integer or double vector, got {:?}",
            param, actual
        ),
    }
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

    /// Downcast a `panic!` payload (produced via format args, so always a
    /// `String`) into an owned `String` for message assertions.
    fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
        *payload
            .downcast::<String>()
            .expect("panic payload should be a String")
    }

    #[test]
    fn vec_i64_batches_multiple_out_of_range_indices() {
        let result =
            std::panic::catch_unwind(|| checked_vec_i64_into_sexp(vec![1, i64::MAX, 2, i64::MIN]));
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(
            msg.starts_with("strict conversion failed: Vec<i64> conversion failed:"),
            "{msg}"
        );
        assert!(msg.contains("invalid value at index 1: i64 value"), "{msg}");
        assert!(msg.contains("invalid value at index 3: i64 value"), "{msg}");
        assert!(
            !msg.contains("and "),
            "should not summarize under the cap: {msg}"
        );
        assert!(
            msg.ends_with("use a non-strict function to allow lossy f64 widening"),
            "{msg}"
        );
    }

    #[test]
    fn vec_i64_batches_caps_and_summarizes_remainder() {
        let vals: Vec<i64> = std::iter::repeat_n(i64::MAX, 15).collect();
        let result = std::panic::catch_unwind(|| checked_vec_i64_into_sexp(vals));
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(msg.contains("and 5 more"), "{msg}");
    }

    #[test]
    fn vec_u64_batches_multiple_out_of_range_indices() {
        let bad = i32::MAX as u64 + 1;
        let result = std::panic::catch_unwind(|| checked_vec_u64_into_sexp(vec![0, bad, 1, bad]));
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(
            msg.starts_with("strict conversion failed: Vec<u64> conversion failed:"),
            "{msg}"
        );
        assert!(msg.contains("invalid value at index 1: u64 value"), "{msg}");
        assert!(msg.contains("invalid value at index 3: u64 value"), "{msg}");
    }

    #[test]
    fn vec_option_i64_batches_multiple_out_of_range_indices() {
        let result = std::panic::catch_unwind(|| {
            checked_vec_option_i64_into_sexp(vec![Some(1), Some(i64::MAX), None, Some(i64::MIN)])
        });
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(
            msg.starts_with("strict conversion failed: Vec<Option<i64>> conversion failed:"),
            "{msg}"
        );
        assert!(msg.contains("invalid value at index 1: i64 value"), "{msg}");
        assert!(msg.contains("invalid value at index 3: i64 value"), "{msg}");
    }

    #[test]
    fn vec_option_u64_batches_multiple_out_of_range_indices() {
        let bad = i32::MAX as u64 + 1;
        let result = std::panic::catch_unwind(|| {
            checked_vec_option_u64_into_sexp(vec![Some(0), Some(bad), None, Some(bad)])
        });
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(
            msg.starts_with("strict conversion failed: Vec<Option<u64>> conversion failed:"),
            "{msg}"
        );
        assert!(msg.contains("invalid value at index 1: u64 value"), "{msg}");
        assert!(msg.contains("invalid value at index 3: u64 value"), "{msg}");
    }
}
// endregion
