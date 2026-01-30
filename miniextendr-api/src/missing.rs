//! Support for R's missing arguments.
//!
//! When an R function is called without providing a value for a formal argument,
//! R passes `R_MissingArg` as a placeholder. This is different from `NULL` -
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
//! - `Missing<T>` treats `R_MissingArg` as missing and any other value (including `NULL`) as present.
//!
//! Use `Missing<Option<T>>` if you need to distinguish between:
//! - Missing argument (not passed)
//! - `NULL` (explicitly passed `NULL`)
//! - A value (explicitly passed a non-NULL value)

use crate::ffi::{R_MissingArg, SEXP};
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
    #[inline]
    pub fn is_absent(&self) -> bool {
        matches!(self, Missing::Absent)
    }

    /// Alias for `is_absent()` to match R's `missing()` function.
    #[inline]
    pub fn is_missing(&self) -> bool {
        self.is_absent()
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

// =============================================================================
// TryFromSexp implementation
// =============================================================================

/// Check if a SEXP is the missing argument sentinel.
#[inline]
pub fn is_missing_arg(sexp: SEXP) -> bool {
    std::ptr::addr_eq(sexp.0, unsafe { R_MissingArg.0 })
}

impl<T> TryFromSexp for Missing<T>
where
    T: TryFromSexp,
    <T as TryFromSexp>::Error: Into<SexpError>,
{
    type Error = SexpError;

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
mod tests {
    use super::*;

    #[test]
    fn missing_methods() {
        let present: Missing<i32> = Missing::Present(42);
        let absent: Missing<i32> = Missing::Absent;

        assert!(present.is_present());
        assert!(!present.is_absent());
        assert!(!present.is_missing());

        assert!(!absent.is_present());
        assert!(absent.is_absent());
        assert!(absent.is_missing());

        assert_eq!(present.into_option(), Some(42));
        assert_eq!(absent.into_option(), None);
    }

    #[test]
    fn missing_unwrap_or() {
        let present: Missing<i32> = Missing::Present(42);
        let absent: Missing<i32> = Missing::Absent;

        assert_eq!(present.unwrap_or(0), 42);
        assert_eq!(absent.unwrap_or(0), 0);
    }

    #[test]
    fn missing_map() {
        let present: Missing<i32> = Missing::Present(21);
        let absent: Missing<i32> = Missing::Absent;

        assert_eq!(present.map(|x| x * 2), Missing::Present(42));
        assert_eq!(absent.map(|x| x * 2), Missing::Absent);
    }

    #[test]
    fn missing_default() {
        let m: Missing<i32> = Missing::default();
        assert!(m.is_absent());
    }

    #[test]
    fn missing_from_option() {
        let some: Missing<i32> = Some(42).into();
        let none: Missing<i32> = None.into();

        assert_eq!(some, Missing::Present(42));
        assert_eq!(none, Missing::Absent);
    }
}
