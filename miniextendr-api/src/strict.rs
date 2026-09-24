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
/// `checked_vec_*_into_sexp` raises for the return value of Rust type
/// `container` (e.g. `"Vec<i64>"`).
///
/// Uses [`BatchedErrors::element_message`]'s grammar (#1192/#1097), each
/// reason with the 1-based positions of the elements in the returned vector:
/// `strict conversion failed for Vec<i64>: i64 value 3000000000 is outside R
/// integer range (-2147483647..=2147483647) (element 2); use a non-strict ...`.
/// A return value has no `e$rust_type`, so the message names the type.
fn panic_strict_vec_batched(container: &str, errors: BatchedErrors) -> ! {
    panic!(
        "strict conversion failed for {container}: {}; use a non-strict function to allow \
         lossy f64 widening",
        errors.element_message()
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
            errors.push(i, || {
                format!(
                    "i64 value {x} is outside R integer range ({}..={})",
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
            errors.push(i, || {
                format!("u64 value {x} exceeds R integer max ({})", i32::MAX)
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
                    errors.push(i, || {
                        format!(
                            "i64 value {x} is outside R integer range ({}..={})",
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
                    errors.push(i, || {
                        format!("u64 value {x} exceeds R integer max ({})", i32::MAX)
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
//
// A rejected input is an argument error, not a panic (#1594): each helper
// returns `Result<_, SexpError>`, and the generated wrapper turns an `Err`
// into the same `kind = "conversion"` condition as any other conversion
// failure (`'x' must be a single whole number: got logical`, `e$param`,
// `e$rust_type`). The vector helpers walk the whole input and report every
// failing element at once, with R's 1-based positions.

/// The element reason for an R `NA` in a strict input that cannot hold one.
const NA_NOT_ALLOWED: &str = "NA is not allowed";

/// Convert R SEXP to `i64` in strict mode.
///
/// Only INTSXP and REALSXP are accepted. RAWSXP and LGLSXP are rejected.
/// For REALSXP, uses `TryCoerce` to reject fractional, NaN, and out-of-range
/// values; `NA` is rejected too.
#[inline]
pub fn checked_try_from_sexp_i64(sexp: SEXP) -> Result<i64, SexpError> {
    checked_try_from_sexp_numeric_scalar::<i64>(sexp)
}

/// Convert R SEXP to `u64` in strict mode.
#[inline]
pub fn checked_try_from_sexp_u64(sexp: SEXP) -> Result<u64, SexpError> {
    checked_try_from_sexp_numeric_scalar::<u64>(sexp)
}

/// Convert R SEXP to `isize` in strict mode.
#[inline]
pub fn checked_try_from_sexp_isize(sexp: SEXP) -> Result<isize, SexpError> {
    let val = checked_try_from_sexp_i64(sexp)?;
    narrow::<i64, isize>(val).map_err(SexpError::InvalidValue)
}

/// Convert R SEXP to `usize` in strict mode.
#[inline]
pub fn checked_try_from_sexp_usize(sexp: SEXP) -> Result<usize, SexpError> {
    let val = checked_try_from_sexp_u64(sexp)?;
    narrow::<u64, usize>(val).map_err(SexpError::InvalidValue)
}

/// Convert R SEXP to `Vec<i64>` in strict mode.
pub fn checked_vec_try_from_sexp_i64(sexp: SEXP) -> Result<Vec<i64>, SexpError> {
    checked_vec_try_from_sexp_numeric::<i64>(sexp)
}

/// Convert R SEXP to `Vec<u64>` in strict mode.
pub fn checked_vec_try_from_sexp_u64(sexp: SEXP) -> Result<Vec<u64>, SexpError> {
    checked_vec_try_from_sexp_numeric::<u64>(sexp)
}

/// Convert R SEXP to `Vec<isize>` in strict mode.
pub fn checked_vec_try_from_sexp_isize(sexp: SEXP) -> Result<Vec<isize>, SexpError> {
    let vals = checked_vec_try_from_sexp_i64(sexp)?;
    batch_elements(&vals, narrow::<i64, isize>)
}

/// Convert R SEXP to `Vec<usize>` in strict mode.
pub fn checked_vec_try_from_sexp_usize(sexp: SEXP) -> Result<Vec<usize>, SexpError> {
    let vals = checked_vec_try_from_sexp_u64(sexp)?;
    batch_elements(&vals, narrow::<u64, usize>)
}

/// Convert R SEXP to `Vec<Option<i64>>` in strict mode.
///
/// Applies the same input-SEXP-type gate as [`checked_vec_try_from_sexp_i64`]
/// — only INTSXP and REALSXP are accepted; LGLSXP and RAWSXP are rejected.
/// NA elements become `None`; type strictness and missingness are orthogonal.
pub fn checked_vec_option_try_from_sexp_i64(sexp: SEXP) -> Result<Vec<Option<i64>>, SexpError> {
    checked_vec_option_try_from_sexp_numeric::<i64>(sexp)
}

/// Convert R SEXP to `Vec<Option<u64>>` in strict mode.
pub fn checked_vec_option_try_from_sexp_u64(sexp: SEXP) -> Result<Vec<Option<u64>>, SexpError> {
    checked_vec_option_try_from_sexp_numeric::<u64>(sexp)
}

/// Convert R SEXP to `Vec<Option<isize>>` in strict mode.
pub fn checked_vec_option_try_from_sexp_isize(sexp: SEXP) -> Result<Vec<Option<isize>>, SexpError> {
    let vals = checked_vec_option_try_from_sexp_i64(sexp)?;
    batch_elements(&vals, |opt| opt.map(narrow::<i64, isize>).transpose())
}

/// Convert R SEXP to `Vec<Option<usize>>` in strict mode.
pub fn checked_vec_option_try_from_sexp_usize(sexp: SEXP) -> Result<Vec<Option<usize>>, SexpError> {
    let vals = checked_vec_option_try_from_sexp_u64(sexp)?;
    batch_elements(&vals, |opt| opt.map(narrow::<u64, usize>).transpose())
}

/// Narrow a 64-bit value to the pointer-sized type, with the reason of a
/// failure (only reachable on a 32-bit target).
fn narrow<S, T>(val: S) -> Result<T, String>
where
    S: Copy + std::fmt::Display,
    T: TryFrom<S>,
{
    T::try_from(val).map_err(|_| format!("{val} does not fit in {}", std::any::type_name::<T>()))
}

/// The strict input-type gate: only INTSXP and REALSXP are accepted.
fn strict_type_error(actual: SEXPTYPE) -> SexpError {
    crate::from_r::SexpTypeError {
        expected: SEXPTYPE::INTSXP,
        actual,
    }
    .into()
}

/// Convert every element of `slice`, batching every failure (its reason and
/// 1-based position) into one [`SexpError::InvalidValue`] instead of stopping
/// at the first.
fn batch_elements<S: Copy, U>(
    slice: &[S],
    convert: impl Fn(S) -> Result<U, String>,
) -> Result<Vec<U>, SexpError> {
    let mut out = Vec::with_capacity(slice.len());
    let mut errors = BatchedErrors::default();
    for (i, &v) in slice.iter().enumerate() {
        match convert(v) {
            Ok(x) => out.push(x),
            Err(reason) => errors.push(i, || reason),
        }
    }
    if errors.is_empty() {
        Ok(out)
    } else {
        Err(errors.into_element_error())
    }
}

/// One INTSXP element: `NA_integer_` is `None`; any other value coerces.
fn strict_int_elt<T>(v: i32) -> Result<Option<T>, String>
where
    i32: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Display,
{
    if v == crate::altrep_traits::NA_INTEGER {
        return Ok(None);
    }
    TryCoerce::<T>::try_coerce(v)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// One REALSXP element: `NA_real_` is `None`; any other value coerces, so a
/// fractional, NaN or out-of-range double fails.
fn strict_real_elt<T>(v: f64) -> Result<Option<T>, String>
where
    f64: TryCoerce<T>,
    <f64 as TryCoerce<T>>::Error: std::fmt::Display,
{
    if crate::from_r::is_na_real(v) {
        return Ok(None);
    }
    TryCoerce::<T>::try_coerce(v)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// Generic strict scalar conversion: only INTSXP and REALSXP allowed, of
/// length 1 and not `NA`.
#[inline]
fn checked_try_from_sexp_numeric_scalar<T>(sexp: SEXP) -> Result<T, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Display,
    <f64 as TryCoerce<T>>::Error: std::fmt::Display,
{
    let actual = sexp.type_of();
    let value = match actual {
        SEXPTYPE::INTSXP => {
            let value: i32 = TryFromSexp::try_from_sexp(sexp)?;
            strict_int_elt::<T>(value)
        }
        SEXPTYPE::REALSXP => {
            let value: f64 = TryFromSexp::try_from_sexp(sexp)?;
            strict_real_elt::<T>(value)
        }
        _ => return Err(strict_type_error(actual)),
    };
    match value {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err(crate::from_r::SexpNaError { sexp_type: actual }.into()),
        Err(reason) => Err(SexpError::InvalidValue(reason)),
    }
}

/// Generic strict vector conversion: only INTSXP and REALSXP allowed; an `NA`
/// element is an error, like any other element that does not convert.
fn checked_vec_try_from_sexp_numeric<T>(sexp: SEXP) -> Result<Vec<T>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Display,
    <f64 as TryCoerce<T>>::Error: std::fmt::Display,
{
    let required = |v: Option<T>| v.ok_or_else(|| NA_NOT_ALLOWED.to_string());
    match sexp.type_of() {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            batch_elements(slice, |v| strict_int_elt::<T>(v).and_then(required))
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            batch_elements(slice, |v| strict_real_elt::<T>(v).and_then(required))
        }
        actual => Err(strict_type_error(actual)),
    }
}

/// Generic strict `Vec<Option<T>>` conversion: only INTSXP and REALSXP allowed.
///
/// Mirrors [`checked_vec_try_from_sexp_numeric`] but maps R's NA sentinel
/// (`NA_INTEGER` for INTSXP, `NA_REAL` for REALSXP) to `None` instead of
/// erroring — missingness is orthogonal to the input-type gate.
fn checked_vec_option_try_from_sexp_numeric<T>(sexp: SEXP) -> Result<Vec<Option<T>>, SexpError>
where
    i32: TryCoerce<T>,
    f64: TryCoerce<T>,
    <i32 as TryCoerce<T>>::Error: std::fmt::Display,
    <f64 as TryCoerce<T>>::Error: std::fmt::Display,
{
    match sexp.type_of() {
        SEXPTYPE::INTSXP => {
            let slice: &[i32] = unsafe { sexp.as_slice() };
            batch_elements(slice, strict_int_elt::<T>)
        }
        SEXPTYPE::REALSXP => {
            let slice: &[f64] = unsafe { sexp.as_slice() };
            batch_elements(slice, strict_real_elt::<T>)
        }
        actual => Err(strict_type_error(actual)),
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
            msg.starts_with("strict conversion failed for Vec<i64>: i64 value"),
            "{msg}"
        );
        assert!(msg.contains("(element 2)"), "{msg}");
        assert!(msg.contains("(element 4)"), "{msg}");
        assert!(!msg.contains("index"), "{msg}");
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
            msg.starts_with("strict conversion failed for Vec<u64>: u64 value"),
            "{msg}"
        );
        assert!(msg.contains("(elements 2, 4)"), "{msg}");
        assert!(!msg.contains("index"), "{msg}");
    }

    #[test]
    fn vec_option_i64_batches_multiple_out_of_range_indices() {
        let result = std::panic::catch_unwind(|| {
            checked_vec_option_i64_into_sexp(vec![Some(1), Some(i64::MAX), None, Some(i64::MIN)])
        });
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(
            msg.starts_with("strict conversion failed for Vec<Option<i64>>: i64 value"),
            "{msg}"
        );
        assert!(msg.contains("(element 2)"), "{msg}");
        assert!(msg.contains("(element 4)"), "{msg}");
        assert!(!msg.contains("index"), "{msg}");
    }

    #[test]
    fn vec_option_u64_batches_multiple_out_of_range_indices() {
        let bad = i32::MAX as u64 + 1;
        let result = std::panic::catch_unwind(|| {
            checked_vec_option_u64_into_sexp(vec![Some(0), Some(bad), None, Some(bad)])
        });
        let msg = panic_message(result.expect_err("should panic for out-of-range elements"));
        assert!(
            msg.starts_with("strict conversion failed for Vec<Option<u64>>: u64 value"),
            "{msg}"
        );
        assert!(msg.contains("(elements 2, 4)"), "{msg}");
        assert!(!msg.contains("index"), "{msg}");
    }
}
// endregion
