//! `Result<T, E>` conversions to R.
//!
//! Used by `#[miniextendr(unwrap_in_r)]` to pass Results to R as lists
//! with `ok` and `err` fields, instead of unwrapping in Rust.
//! Also provides `NullOnErr` for `Result<T, ()>` → NULL-on-error semantics.

use std::collections::HashMap;

use crate::into_r::IntoR;

/// Convert `Result<T, E>` to R (value-style, for `#[miniextendr(unwrap_in_r)]`).
///
/// # Behavior
///
/// - `Ok(value)` → returns the converted value directly
/// - `Err(msg)` → returns `list(error = "<msg>")` (value-style error)
///
/// # When This Is Used
///
/// This impl is **only used** when `#[miniextendr(unwrap_in_r)]` is specified.
/// Without that attribute, `#[miniextendr]` functions returning `Result<T, E>`
/// will unwrap in Rust and raise an R error on `Err` (error boundary semantics).
///
/// # Error Handling Summary
///
/// | Mode | On `Err(e)` | Bound Required |
/// |------|-------------|----------------|
/// | Default | R error via panic | `E: Debug` |
/// | `unwrap_in_r` | `list(error = ...)` | `E: Display` |
///
/// **Default** (without `unwrap_in_r`): `Result<T, E>` acts as an error boundary:
/// - `Ok(v)` → `v` converted to R
/// - `Err(e)` → R error with Debug-formatted message (requires `E: Debug`)
///
/// **With `unwrap_in_r`**: `Result<T, E>` is passed through to R:
/// - `Ok(v)` → `v` converted to R
/// - `Err(e)` → `list(error = e.to_string())` (requires `E: Display`)
///
/// # Example
///
/// ```ignore
/// // Default: error boundary - Err becomes R stop()
/// #[miniextendr]
/// fn divide(x: f64, y: f64) -> Result<f64, String> {
///     if y == 0.0 { Err("division by zero".into()) }
///     else { Ok(x / y) }
/// }
/// // In R: tryCatch(divide(1, 0), error = ...) catches the error
///
/// // Value-style: Err becomes list(error = ...)
/// #[miniextendr(unwrap_in_r)]
/// fn divide_safe(x: f64, y: f64) -> Result<f64, String> {
///     if y == 0.0 { Err("division by zero".into()) }
///     else { Ok(x / y) }
/// }
/// // In R: result <- divide_safe(1, 0)
/// //       if (!is.null(result$error)) { handle error }
/// ```
impl<T, E> IntoR for Result<T, E>
where
    T: IntoR,
    E: std::fmt::Display,
{
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Ok(value) => value.into_sexp(),
            Err(msg) => {
                // Create list(error = msg) for R-side error handling
                let mut map = HashMap::with_capacity(1);
                map.insert("error".to_string(), msg.to_string());
                map.into_sexp()
            }
        }
    }
}

/// Marker type for `Result<T, ()>` that converts `Err(())` to NULL.
///
/// This type is used internally by the `#[miniextendr]` macro when handling
/// `Result<T, ()>` return types. When the error type is `()`, there's no
/// error message to report, so we return NULL instead of raising an error.
///
/// # Usage
///
/// You typically don't use this directly. When you write:
///
/// ```ignore
/// #[miniextendr]
/// fn maybe_value(x: i32) -> Result<i32, ()> {
///     if x > 0 { Ok(x) } else { Err(()) }
/// }
/// ```
///
/// The macro generates code that converts `Err(())` to `Err(NullOnErr)` and
/// returns `NULL` in R.
///
/// # Note
///
/// `NullOnErr` intentionally does NOT implement `Display` to avoid conflicting
/// with the generic `IntoR for Result<T, E: Display>` impl. It has its own
/// specialized `IntoR` impl that returns NULL on error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullOnErr;

/// Convert `Result<T, NullOnErr>` to R, returning NULL on error.
///
/// This is a special case for `Result<T, ()>` types where the error
/// carries no information. Instead of raising an R error, we return NULL.
impl<T: IntoR> IntoR for Result<T, NullOnErr> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> crate::ffi::SEXP {
        match self {
            Ok(value) => value.into_sexp(),
            Err(NullOnErr) => crate::ffi::SEXP::null(),
        }
    }
}
// endregion
