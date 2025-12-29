//! Wrapper helpers to force specific `IntoR` representations.
//!
//! This module provides two approaches for controlling how Rust types are converted to R:
//!
//! ## 1. `As*` Wrappers (Call-site Control)
//!
//! Use these wrappers when you want to override the conversion for a single return value:
//!
//! - [`AsList<T>`]: Convert `T` to an R list via [`IntoList`]
//! - [`AsExternalPtr<T>`]: Convert `T` to an R external pointer
//! - [`AsRNative<T>`]: Convert scalar `T` to a length-1 R vector
//!
//! ```ignore
//! #[miniextendr]
//! fn get_data() -> AsList<MyStruct> {
//!     AsList(MyStruct { x: 1, y: 2 })
//! }
//! ```
//!
//! ## 2. `Prefer*` Derive Macros (Type-level Control)
//!
//! Use these derives when a type should *always* use a specific conversion:
//!
//! - `#[derive(IntoList, PreferList)]`: Type always converts to R list
//! - `#[derive(ExternalPtr, PreferExternalPtr)]`: Type always converts to external pointer
//! - `#[derive(RNativeType, PreferRNativeType)]`: Newtype always converts to native R scalar
//!
//! ```ignore
//! #[derive(IntoList, PreferList)]
//! struct Point { x: f64, y: f64 }
//!
//! #[miniextendr]
//! fn make_point() -> Point {  // Automatically becomes R list
//!     Point { x: 1.0, y: 2.0 }
//! }
//! ```
//!
//! ## 3. `#[miniextendr(return = "...")]` Attribute
//!
//! Use this when you want to control conversion for a specific `#[miniextendr]` function
//! without modifying the type itself:
//!
//! - `return = "list"`: Wrap result in `AsList`
//! - `return = "externalptr"`: Wrap result in `AsExternalPtr`
//! - `return = "native"`: Wrap result in `AsRNative`
//!
//! ```ignore
//! #[miniextendr(return = "list")]
//! fn get_as_list() -> MyStruct {
//!     MyStruct { x: 1 }
//! }
//! ```
//!
//! ## Choosing the Right Approach
//!
//! | Situation | Recommended Approach |
//! |-----------|---------------------|
//! | Type should *always* convert one way | `Prefer*` derive |
//! | Override conversion for one function | `As*` wrapper or `return` attribute |
//! | Type has multiple valid representations | Don't use `Prefer*`; use `As*` or `return` |

use crate::externalptr::{ExternalPtr, IntoExternalPtr};
use crate::ffi::RNativeType;
use crate::into_r::IntoR;
use crate::list::IntoList;

/// Wrap a value and convert it to an R list via [`IntoList`] when returned from Rust.
///
/// Use this wrapper when you want to convert a single value to an R list without
/// making that the default behavior for the type.
///
/// # Example
///
/// ```ignore
/// #[derive(IntoList)]
/// struct Point { x: f64, y: f64 }
///
/// #[miniextendr]
/// fn make_point() -> AsList<Point> {
///     AsList(Point { x: 1.0, y: 2.0 })
/// }
/// // In R: make_point() returns list(x = 1.0, y = 2.0)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AsList<T: IntoList>(pub T);

impl<T: IntoList> From<T> for AsList<T> {
    fn from(value: T) -> Self {
        AsList(value)
    }
}

impl<T: IntoList> IntoR for AsList<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        self.0.into_list().into_sexp()
    }
}

/// Wrap a value and convert it to an R external pointer when returned from Rust.
///
/// Use this wrapper when you want to return a Rust value as an opaque pointer
/// that R code can pass back to Rust functions later.
///
/// # Example
///
/// ```ignore
/// struct Connection { handle: u64 }
///
/// impl IntoExternalPtr for Connection { /* ... */ }
///
/// #[miniextendr]
/// fn open_connection(path: &str) -> AsExternalPtr<Connection> {
///     AsExternalPtr(Connection { handle: 42 })
/// }
/// // In R: open_connection("foo") returns an external pointer
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AsExternalPtr<T: IntoExternalPtr>(pub T);

impl<T: IntoExternalPtr> From<T> for AsExternalPtr<T> {
    fn from(value: T) -> Self {
        AsExternalPtr(value)
    }
}

impl<T: IntoExternalPtr> IntoR for AsExternalPtr<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        ExternalPtr::new(self.0).into_sexp()
    }
}

/// Wrap a scalar [`RNativeType`] and force native R vector conversion.
///
/// This creates a length-1 R vector containing the scalar value. Use this when
/// you want to ensure a value is converted to its native R representation (e.g.,
/// `i32` → integer vector, `f64` → numeric vector) rather than another path
/// like `IntoExternalPtr`.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Copy, RNativeType)]
/// struct Meters(f64);
///
/// #[miniextendr]
/// fn distance() -> AsRNative<Meters> {
///     AsRNative(Meters(42.5))
/// }
/// // In R: distance() returns 42.5 (numeric vector of length 1)
/// ```
///
/// # Performance
///
/// This wrapper directly allocates an R vector and writes the value,
/// avoiding intermediate Rust allocations.
#[derive(Debug, Clone, Copy)]
pub struct AsRNative<T: RNativeType>(pub T);

impl<T: RNativeType> From<T> for AsRNative<T> {
    fn from(value: T) -> Self {
        AsRNative(value)
    }
}

impl<T: RNativeType> IntoR for AsRNative<T> {
    #[inline]
    fn into_sexp(self) -> crate::ffi::SEXP {
        // Directly allocate a length-1 R vector and write the scalar value.
        // This avoids the intermediate Rust Vec allocation.
        unsafe {
            let sexp = crate::ffi::Rf_allocVector(T::SEXP_TYPE, 1);
            let ptr = T::dataptr_mut(sexp);
            std::ptr::write(ptr, self.0);
            sexp
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::ffi::SEXP {
        unsafe {
            let sexp = crate::ffi::Rf_allocVector_unchecked(T::SEXP_TYPE, 1);
            let ptr = T::dataptr_mut(sexp);
            std::ptr::write(ptr, self.0);
            sexp
        }
    }
}
