//! Support for R's missing arguments.
//!
//! When an R function is called without providing a value for a formal argument,
//! R passes `SEXP::missing_arg()` as a placeholder. This is different from `NULL` -
//! a missing argument means "not provided", while `NULL` is an explicit value.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::{miniextendr, Missing};
//!
//! #[miniextendr]
//! fn greet(name: Missing<String>) -> String {
//!     match name.into_option() {
//!         Some(n) => format!("Hello, {}!", n),
//!         None => "Hello, stranger!".to_string(),
//!     }
//! }
//! ```
//!
//! In R:
//! ```r
//! greet("Alice")  # "Hello, Alice!"
//! greet()         # "Hello, stranger!"
//! ```
//!
//! # Difference from `Option<T>`
//!
//! - `Option<T>` treats `NULL` as `None` and any other value as `Some(T)`.
//! - `Missing<T>` treats `SEXP::missing_arg()` as missing and any other value (including `NULL`) as present.
//!
//! Use `Missing<Option<T>>` if you need to distinguish between:
//! - Missing argument (not passed)
//! - `NULL` (explicitly passed `NULL`)
//! - A value (explicitly passed a non-NULL value)
//!
//! # How it works: the `quote(expr=)` sentinel
//!
//! In regular R functions, you detect missing arguments with `missing()`:
//!
//! ```r
//! my_func <- function(x) {
//!   if (missing(x)) "not provided" else x
//! }
//! my_func()   # "not provided"
//! my_func(5)  # 5
//! ```
//!
//! However, `.Call()` (the FFI boundary) forces argument evaluation, so an
//! omitted argument errors before the call even reaches Rust:
//!
//! ```r
//! wrapper <- function(x) .Call(rust_fn, x)
//! wrapper()  # Error: argument "x" is missing, with no default
//! ```
//!
//! The generated R wrapper tests `missing()` at the argument position of the
//! `.Call()`:
//!
//! ```r
//! wrapper <- function(x) .Call(rust_fn, if (missing(x)) quote(expr=) else x)
//! ```
//!
//! `quote(expr=)` is an R idiom for the "missing" state: calling `quote()`
//! without its `expr` argument returns `SEXP::missing_arg()`. The call
//! proceeds, the sentinel passes through `.Call()`, and `Missing<T>` reads it
//! as `Missing::Absent`. Only the argument position may hold the sentinel: a
//! variable bound to it errors on lookup.
//!
//! # A default on `Missing<T>`
//!
//! `#[miniextendr(default = "...")]` on a `Missing<T>` parameter (or
//! `defaults(x = "...")` on a method) only writes the formal, so the usage
//! line shows the value the function stands for when the argument is left
//! out:
//!
//! ```r
//! wrapper <- function(tol = 1e-8) .Call(rust_fn, if (missing(tol)) quote(expr=) else tol)
//! ```
//!
//! `missing(tol)` is `TRUE` for an omitted argument even when the formal has
//! a default, so an omitted argument still reaches Rust as `Missing::Absent`
//! and the default is never evaluated. The Rust code decides what absence
//! means; the default in the formal documents it.
//!
//! A supplied value is always `Missing::Present`, even one `identical()` to
//! the default: R's `missing(x)` is `FALSE` once the caller supplies `x`, and
//! a caller who writes `wrapper(tol = 1e-8)` may mean that value. A caller
//! forwarding its own formal, `function(tol = 1e-8) wrapper(tol)`, hands its
//! default over as a value too. This differs from a single-choice
//! `match_arg` / `choices` parameter under `Missing<..>`, whose formal is the
//! whole choice vector: that vector is never one valid choice, so passing it
//! whole reads as omitted. Such a choice parameter takes no `default`.

use crate::SEXP;
use crate::from_r::{SexpError, TryFromSexp};

/// Wrapper type that detects if an R argument was not passed (missing).
///
/// This corresponds to R's `missing()` function. When a function parameter
/// has type `Missing<T>`, it will be `Missing::Absent` if the caller didn't
/// provide that argument, or `Missing::Present(value)` if they did.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::{miniextendr, Missing};
///
/// #[miniextendr]
/// fn maybe_square(x: Missing<f64>) -> f64 {
///     match x {
///         Missing::Present(val) => val * val,
///         Missing::Absent => 0.0,
///     }
/// }
/// ```
///
/// In R:
/// ```r
/// maybe_square(5)  # 25
/// maybe_square()   # 0
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing<T> {
    /// The argument was provided.
    Present(T),
    /// The argument was not provided (missing in R).
    Absent,
}

impl<T> Missing<T> {
    /// Returns `true` if the argument was provided.
    #[inline]
    pub fn is_present(&self) -> bool {
        matches!(self, Missing::Present(_))
    }

    /// Returns `true` if the argument was not provided.
    ///
    /// Named to match R's `missing()` function.
    #[inline]
    pub fn is_missing(&self) -> bool {
        matches!(self, Missing::Absent)
    }

    /// Convert to `Option<T>`, returning `None` if missing.
    #[inline]
    pub fn into_option(self) -> Option<T> {
        match self {
            Missing::Present(v) => Some(v),
            Missing::Absent => None,
        }
    }

    /// Get a reference to the value if present.
    #[inline]
    pub fn as_ref(&self) -> Missing<&T> {
        match self {
            Missing::Present(v) => Missing::Present(v),
            Missing::Absent => Missing::Absent,
        }
    }

    /// Get a mutable reference to the value if present.
    #[inline]
    pub fn as_mut(&mut self) -> Missing<&mut T> {
        match self {
            Missing::Present(v) => Missing::Present(v),
            Missing::Absent => Missing::Absent,
        }
    }

    /// Returns the contained value or a default.
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Missing::Present(v) => v,
            Missing::Absent => default,
        }
    }

    /// Returns the contained value or computes it from a closure.
    #[inline]
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Missing::Present(v) => v,
            Missing::Absent => f(),
        }
    }

    /// Maps `Missing<T>` to `Missing<U>` by applying a function.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Missing<U> {
        match self {
            Missing::Present(v) => Missing::Present(f(v)),
            Missing::Absent => Missing::Absent,
        }
    }

    /// Returns the contained value, panicking if absent.
    ///
    /// # Panics
    ///
    /// Panics if the value is `Absent`.
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Missing::Present(v) => v,
            Missing::Absent => panic!("called `Missing::unwrap()` on an `Absent` value"),
        }
    }

    /// Returns the contained value, panicking with a custom message if absent.
    ///
    /// # Panics
    ///
    /// Panics with the provided message if the value is `Absent`.
    #[inline]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Missing::Present(v) => v,
            Missing::Absent => panic!("{}", msg),
        }
    }
}

impl<T: Default> Missing<T> {
    /// Returns the contained value or the default for that type.
    #[inline]
    pub fn unwrap_or_default(self) -> T {
        match self {
            Missing::Present(v) => v,
            Missing::Absent => T::default(),
        }
    }
}

impl<T> Default for Missing<T> {
    /// The default is `Absent`.
    #[inline]
    fn default() -> Self {
        Missing::Absent
    }
}

impl<T> From<T> for Missing<T> {
    #[inline]
    fn from(value: T) -> Self {
        Missing::Present(value)
    }
}

impl<T> From<Option<T>> for Missing<T> {
    #[inline]
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => Missing::Present(v),
            None => Missing::Absent,
        }
    }
}

impl<T> From<Missing<T>> for Option<T> {
    #[inline]
    fn from(missing: Missing<T>) -> Self {
        missing.into_option()
    }
}

// region: TryFromSexp implementation

/// Check if a SEXP is the missing argument sentinel.
#[inline]
pub fn is_missing_arg(sexp: SEXP) -> bool {
    std::ptr::addr_eq(sexp.0, SEXP::missing_arg().0)
}

impl<T> TryFromSexp for Missing<T>
where
    T: TryFromSexp,
    <T as TryFromSexp>::Error: Into<SexpError>,
{
    const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> = T::NATIVE_BORROW;
    const CHARACTER_ONLY: bool = T::CHARACTER_ONLY;

    type Error = SexpError;
    // An omitted argument passes `no_na`.
    #[inline]
    fn __mx_has_na(&self) -> bool {
        matches!(self, Missing::Present(v) if v.__mx_has_na())
    }

    // A given argument is read by its value's type (an `Either` asks the arm
    // it converted to).
    #[inline]
    fn __mx_input_has_na(&self, input: SEXP) -> bool {
        matches!(self, Missing::Present(v) if v.__mx_input_has_na(input))
    }

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if is_missing_arg(sexp) {
            Ok(Missing::Absent)
        } else {
            T::try_from_sexp(sexp)
                .map(Missing::Present)
                .map_err(Into::into)
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if is_missing_arg(sexp) {
            Ok(Missing::Absent)
        } else {
            unsafe { T::try_from_sexp_unchecked(sexp) }
                .map(Missing::Present)
                .map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests;
// endregion
