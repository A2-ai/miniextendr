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

use crate::RNativeType;
use crate::externalptr::{ExternalPtr, IntoExternalPtr};
use crate::into_r::IntoR;
use crate::list::{IntoList, List};
use crate::named_vector::AtomicElement;

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
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> crate::SEXP {
        self.0.into_list().into_sexp()
    }
}

// The historical public `convert::IntoDataFrame` (`-> List`) is retired. Its row→List engine
// now lives in `crate::dataframe::ColumnSource` (an internal `#[doc(hidden)]` trait that the
// enum-flatten codegen and the new public `dataframe::IntoDataFrame` both delegate to). The
// public verb surface is `dataframe::{IntoDataFrame, FromDataFrame}` (returning
// `Result<DataFrame, DataFrameError>`), which mirrors `IntoR` / `TryFromSexp`.
pub use crate::dataframe::ColumnSource;

// region: Struct-field scatter helper

/// Scatter a typed column SEXP from a dense inner data frame into a new
/// SEXP of length `n_rows`, placing `NA`/`NULL` at rows not in `present_idx`.
///
/// This is called by `DataFrameRow`-derived enum code to flatten struct-typed
/// variant fields into prefixed columns of the parent data frame.
///
/// The output type mirrors the input:
/// - REALSXP → REALSXP (NA_real_ fill)
/// - INTSXP  → INTSXP  (NA_integer_ fill)
/// - LGLSXP  → LGLSXP  (NA_logical fill)
/// - STRSXP  → STRSXP  (NA_character_ fill)
/// - VECSXP  → VECSXP  (R_NilValue fill)
/// - anything else → VECSXP (R_NilValue fill)
///
/// # Safety
///
/// Must be called on the R main thread. `src` must be a valid SEXP of length
/// `>= present_idx.len()`. `n_rows` must equal the total row count of the
/// parent data frame.
#[doc(hidden)]
pub unsafe fn scatter_column(
    src: crate::SEXP,
    present_idx: &[usize],
    n_rows: usize,
) -> crate::SEXP {
    // SAFETY: caller guarantees R main thread; src is valid; n_rows is correct.
    #[allow(unused_unsafe)]
    unsafe {
        use crate::{SEXPTYPE, SexpExt as _};

        let stype = src.type_of();
        let n_present = present_idx.len();

        match stype {
            SEXPTYPE::REALSXP => {
                let out = crate::sys::Rf_allocVector(SEXPTYPE::REALSXP, n_rows as isize);
                // Fill with NA_real_ (R's NA for doubles is a specific NaN bit pattern;
                // f64::NAN has the correct bit pattern since R uses a quiet NaN sentinel).
                for i in 0..(n_rows as isize) {
                    out.set_real_elt(i, f64::NAN);
                }
                for (pi, &row_i) in present_idx.iter().enumerate() {
                    if pi < n_present {
                        out.set_real_elt(row_i as isize, src.real_elt(pi as isize));
                    }
                }
                out
            }
            SEXPTYPE::INTSXP => {
                let out = crate::sys::Rf_allocVector(SEXPTYPE::INTSXP, n_rows as isize);
                for i in 0..(n_rows as isize) {
                    out.set_integer_elt(i, i32::MIN); // NA_integer_
                }
                for (pi, &row_i) in present_idx.iter().enumerate() {
                    if pi < n_present {
                        out.set_integer_elt(row_i as isize, src.integer_elt(pi as isize));
                    }
                }
                out
            }
            SEXPTYPE::LGLSXP => {
                let out = crate::sys::Rf_allocVector(SEXPTYPE::LGLSXP, n_rows as isize);
                for i in 0..(n_rows as isize) {
                    out.set_logical_elt(i, i32::MIN); // NA_logical
                }
                for (pi, &row_i) in present_idx.iter().enumerate() {
                    if pi < n_present {
                        out.set_logical_elt(row_i as isize, src.logical_elt(pi as isize));
                    }
                }
                out
            }
            SEXPTYPE::STRSXP => {
                let out = crate::sys::Rf_allocVector(SEXPTYPE::STRSXP, n_rows as isize);
                // Fill with NA_character_
                for i in 0..(n_rows as isize) {
                    out.set_string_elt(i, crate::SEXP::na_string());
                }
                for (pi, &row_i) in present_idx.iter().enumerate() {
                    if pi < n_present {
                        let charsxp = src.string_elt(pi as isize);
                        out.set_string_elt(row_i as isize, charsxp);
                    }
                }
                out
            }
            SEXPTYPE::VECSXP => {
                let out = crate::sys::Rf_allocVector(SEXPTYPE::VECSXP, n_rows as isize);
                // R_NilValue fill is automatic (Rf_allocVector zero-initialises VECSXP slots).
                for (pi, &row_i) in present_idx.iter().enumerate() {
                    if pi < n_present {
                        let elt = src.vector_elt(pi as isize);
                        out.set_vector_elt(row_i as isize, elt);
                    }
                }
                out
            }
            _ => {
                // Unknown/unsupported type — produce a VECSXP list-column.
                // Cells for absent rows remain R_NilValue.
                let out = crate::sys::Rf_allocVector(SEXPTYPE::VECSXP, n_rows as isize);
                for (pi, &row_i) in present_idx.iter().enumerate() {
                    if pi < n_present {
                        let elt = src.vector_elt(pi as isize);
                        out.set_vector_elt(row_i as isize, elt);
                    }
                }
                out
            }
        }
    }
}
// endregion

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
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    #[inline]
    fn into_sexp(self) -> crate::SEXP {
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
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }

    #[inline]
    fn into_sexp(self) -> crate::SEXP {
        // Directly allocate a length-1 R vector and write the scalar value.
        // This avoids the intermediate Rust Vec allocation.
        unsafe {
            let sexp = crate::sys::Rf_allocVector(T::SEXP_TYPE, 1);
            let ptr = T::dataptr_mut(sexp);
            std::ptr::write(ptr, self.0);
            sexp
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::SEXP {
        unsafe {
            let sexp = crate::sys::Rf_allocVector_unchecked(T::SEXP_TYPE, 1);
            let ptr = T::dataptr_mut(sexp);
            std::ptr::write(ptr, self.0);
            sexp
        }
    }
}
// endregion

// region: Named pair wrappers

/// Wrap a tuple pair collection and convert it to a **named R list** (VECSXP).
///
/// Preserves insertion order and allows duplicate names (sequence semantics).
///
/// # Supported input types
///
/// | Input | Bounds |
/// |-------|--------|
/// | `Vec<(K, V)>` | `K: AsRef<str>`, `V: IntoR` |
/// | `[(K, V); N]` | `K: AsRef<str>`, `V: IntoR` |
/// | `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + IntoR` |
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn make_config() -> AsNamedList<Vec<(String, i32)>> {
///     AsNamedList(vec![
///         ("width".into(), 100),
///         ("height".into(), 200),
///     ])
/// }
/// // In R: make_config() returns list(width = 100L, height = 200L)
/// ```
#[derive(Debug, Clone)]
pub struct AsNamedList<T>(pub T);

impl<T> From<T> for AsNamedList<T> {
    fn from(value: T) -> Self {
        AsNamedList(value)
    }
}

impl<K: AsRef<str>, V: IntoR> IntoR for AsNamedList<Vec<(K, V)>> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        let pairs: Vec<(K, crate::SEXP)> = self
            .0
            .into_iter()
            .map(|(k, v)| (k, v.into_sexp()))
            .collect();
        List::from_raw_pairs(pairs).into_sexp()
    }
}

impl<K: AsRef<str>, V: IntoR, const N: usize> IntoR for AsNamedList<[(K, V); N]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        let pairs: Vec<(K, crate::SEXP)> = self
            .0
            .into_iter()
            .map(|(k, v)| (k, v.into_sexp()))
            .collect();
        List::from_raw_pairs(pairs).into_sexp()
    }
}

impl<K: AsRef<str>, V: Clone + IntoR> IntoR for AsNamedList<&[(K, V)]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        let pairs: Vec<(&K, crate::SEXP)> = self
            .0
            .iter()
            .map(|(k, v)| (k, v.clone().into_sexp()))
            .collect();
        List::from_raw_pairs(pairs).into_sexp()
    }
}

/// Wrap a tuple pair collection and convert it to a **named atomic R vector**
/// (INTSXP, REALSXP, LGLSXP, RAWSXP, or STRSXP).
///
/// Preserves insertion order and allows duplicate names (sequence semantics).
/// Values must be homogeneous and implement [`AtomicElement`].
///
/// # Supported input types
///
/// | Input | Bounds |
/// |-------|--------|
/// | `Vec<(K, V)>` | `K: AsRef<str>`, `V: AtomicElement` |
/// | `[(K, V); N]` | `K: AsRef<str>`, `V: AtomicElement` |
/// | `&[(K, V)]` | `K: AsRef<str>`, `V: Clone + AtomicElement` |
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn make_scores() -> AsNamedVector<Vec<(&str, f64)>> {
///     AsNamedVector(vec![("alice", 95.0), ("bob", 87.5)])
/// }
/// // In R: make_scores() returns c(alice = 95.0, bob = 87.5)
/// ```
#[derive(Debug, Clone)]
pub struct AsNamedVector<T>(pub T);

impl<T> From<T> for AsNamedVector<T> {
    fn from(value: T) -> Self {
        AsNamedVector(value)
    }
}

impl<K: AsRef<str>, V: AtomicElement> IntoR for AsNamedVector<Vec<(K, V)>> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        named_vector_from_pairs(self.0)
    }
}

impl<K: AsRef<str>, V: AtomicElement, const N: usize> IntoR for AsNamedVector<[(K, V); N]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        named_vector_from_pairs(self.0)
    }
}

impl<K: AsRef<str>, V: Clone + AtomicElement> IntoR for AsNamedVector<&[(K, V)]> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        self.try_into_sexp()
    }

    fn into_sexp(self) -> crate::SEXP {
        let (keys, values): (Vec<&K>, Vec<V>) = self.0.iter().map(|(k, v)| (k, v.clone())).unzip();
        let sexp = V::vec_to_sexp(values);
        unsafe {
            crate::sys::Rf_protect(sexp);
            crate::named_vector::set_names_on_sexp(sexp, &keys);
            crate::sys::Rf_unprotect(1);
        }
        sexp
    }
}

/// Shared helper: build a named atomic vector from an owning iterator of (key, value) pairs.
fn named_vector_from_pairs<K, V>(pairs: impl IntoIterator<Item = (K, V)>) -> crate::SEXP
where
    K: AsRef<str>,
    V: AtomicElement,
{
    let (keys, values): (Vec<K>, Vec<V>) = pairs.into_iter().unzip();
    let sexp = V::vec_to_sexp(values);
    unsafe {
        crate::sys::Rf_protect(sexp);
        crate::named_vector::set_names_on_sexp(sexp, &keys);
        crate::sys::Rf_unprotect(1);
    }
    sexp
}
// endregion

// region: Extension traits for ergonomic wrapping
//
// These extension traits provide method-style wrapping that works even when
// the destination type isn't constrained (i.e., `value.wrap_list()` instead
// of `value.into()` which requires type inference).
//
// ```ignore
// // These all work without type annotations:
// let wrapped = my_struct.wrap_list();
// let ptr = my_value.wrap_external_ptr();
// let native = my_num.wrap_r_native();
// ```

/// Extension trait for wrapping values as [`AsList`].
///
/// This trait is automatically implemented for all types that implement [`IntoList`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::AsListExt;
///
/// #[derive(IntoList)]
/// struct Point { x: f64, y: f64 }
///
/// let point = Point { x: 1.0, y: 2.0 };
/// let wrapped: AsList<Point> = point.wrap_list();
/// ```
pub trait AsListExt: IntoList + Sized {
    /// Wrap `self` in [`AsList`] for R list conversion.
    fn wrap_list(self) -> AsList<Self> {
        AsList(self)
    }
}

impl<T: IntoList> AsListExt for T {}

/// Extension trait for wrapping values as [`AsExternalPtr`].
///
/// This trait is automatically implemented for all types that implement [`IntoExternalPtr`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::AsExternalPtrExt;
///
/// #[derive(ExternalPtr)]
/// struct Connection { handle: u64 }
///
/// let conn = Connection { handle: 42 };
/// let wrapped: AsExternalPtr<Connection> = conn.wrap_external_ptr();
/// ```
pub trait AsExternalPtrExt: IntoExternalPtr + Sized {
    /// Wrap `self` in [`AsExternalPtr`] for R external pointer conversion.
    fn wrap_external_ptr(self) -> AsExternalPtr<Self> {
        AsExternalPtr(self)
    }
}

impl<T: IntoExternalPtr> AsExternalPtrExt for T {}

/// Extension trait for wrapping values as [`AsRNative`].
///
/// This trait is automatically implemented for all types that implement [`RNativeType`].
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::convert::AsRNativeExt;
///
/// let x: f64 = 42.5;
/// let wrapped: AsRNative<f64> = x.wrap_r_native();
/// ```
pub trait AsRNativeExt: RNativeType + Sized {
    /// Wrap `self` in [`AsRNative`] for native R scalar conversion.
    fn wrap_r_native(self) -> AsRNative<Self> {
        AsRNative(self)
    }
}

impl<T: RNativeType> AsRNativeExt for T {}

/// Extension trait for wrapping tuple pair collections as [`AsNamedList`].
///
/// # Example
///
/// ```ignore
/// let pairs = vec![("x".to_string(), 1i32), ("y".to_string(), 2i32)];
/// let wrapped = pairs.wrap_named_list();
/// ```
pub trait AsNamedListExt: Sized {
    /// Wrap `self` in [`AsNamedList`] for named R list conversion.
    fn wrap_named_list(self) -> AsNamedList<Self> {
        AsNamedList(self)
    }
}

impl<K: AsRef<str>, V: IntoR> AsNamedListExt for Vec<(K, V)> {}
impl<K: AsRef<str>, V: IntoR, const N: usize> AsNamedListExt for [(K, V); N] {}
impl<K: AsRef<str>, V: Clone + IntoR> AsNamedListExt for &[(K, V)] {}

/// Extension trait for wrapping tuple pair collections as [`AsNamedVector`].
///
/// # Example
///
/// ```ignore
/// let pairs = vec![("alice".to_string(), 95.0f64), ("bob".to_string(), 87.5)];
/// let wrapped = pairs.wrap_named_vector();
/// ```
pub trait AsNamedVectorExt: Sized {
    /// Wrap `self` in [`AsNamedVector`] for named atomic R vector conversion.
    fn wrap_named_vector(self) -> AsNamedVector<Self> {
        AsNamedVector(self)
    }
}

impl<K: AsRef<str>, V: AtomicElement> AsNamedVectorExt for Vec<(K, V)> {}
impl<K: AsRef<str>, V: AtomicElement, const N: usize> AsNamedVectorExt for [(K, V); N] {}
impl<K: AsRef<str>, V: Clone + AtomicElement> AsNamedVectorExt for &[(K, V)] {}
// endregion

// region: Display/FromStr trait adapters

/// Wrap a `T: Display` and convert it to an R character scalar.
///
/// Any type implementing `std::fmt::Display` can be returned to R as a string
/// without implementing miniextendr traits.
///
/// # Example
///
/// ```ignore
/// use std::net::IpAddr;
///
/// #[miniextendr]
/// fn format_ip(ip: &str) -> AsDisplay<IpAddr> {
///     AsDisplay(ip.parse().unwrap())
/// }
/// // R gets: "192.168.1.1"
/// ```
#[derive(Debug, Clone, Copy)]
pub struct AsDisplay<T>(pub T);

impl<T: std::fmt::Display> IntoR for AsDisplay<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.0.to_string().into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Ok(unsafe { self.0.to_string().into_sexp_unchecked() })
    }
}

/// Wrap a `Vec<T: Display>` and convert it to an R character vector.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn format_errors(errors: Vec<std::io::Error>) -> AsDisplayVec<std::io::Error> {
///     AsDisplayVec(errors)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AsDisplayVec<T>(pub Vec<T>);

impl<T: std::fmt::Display> IntoR for AsDisplayVec<T> {
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        let strings: Vec<String> = self.0.into_iter().map(|x| x.to_string()).collect();
        Ok(strings.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        let strings: Vec<String> = self.0.into_iter().map(|x| x.to_string()).collect();
        Ok(unsafe { strings.into_sexp_unchecked() })
    }
}

/// Wrap a parsed `T: FromStr` from an R character scalar.
///
/// Pass an R character scalar and it will be parsed into `T` via `str::parse()`.
///
/// # Example
///
/// ```ignore
/// use std::net::IpAddr;
///
/// #[miniextendr]
/// fn check_ip(addr: AsFromStr<IpAddr>) -> bool {
///     addr.0.is_loopback()
/// }
/// // R: check_ip("127.0.0.1") → TRUE
/// ```
#[derive(Debug, Clone)]
pub struct AsFromStr<T>(pub T);

impl<T: std::str::FromStr> crate::from_r::TryFromSexp for AsFromStr<T>
where
    T::Err: std::fmt::Display,
{
    type Error = crate::from_r::SexpError;

    fn try_from_sexp(sexp: crate::SEXP) -> Result<Self, Self::Error> {
        let s: &str = crate::from_r::TryFromSexp::try_from_sexp(sexp)?;
        let value = s
            .parse::<T>()
            .map_err(|e| crate::from_r::SexpError::InvalidValue(format!("{e}")))?;
        Ok(AsFromStr(value))
    }

    unsafe fn try_from_sexp_unchecked(sexp: crate::SEXP) -> Result<Self, Self::Error> {
        let s: &str = unsafe { crate::from_r::TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let value = s
            .parse::<T>()
            .map_err(|e| crate::from_r::SexpError::InvalidValue(format!("{e}")))?;
        Ok(AsFromStr(value))
    }
}

/// Wrap a `Vec<T: FromStr>` parsed from an R character vector.
///
/// Each element of the R character vector is parsed into `T`.
/// All parse errors are collected with their indices.
///
/// # Example
///
/// ```ignore
/// use std::net::IpAddr;
///
/// #[miniextendr]
/// fn parse_ips(addrs: AsFromStrVec<IpAddr>) -> Vec<bool> {
///     addrs.0.into_iter().map(|ip| ip.is_loopback()).collect()
/// }
/// // R: parse_ips(c("127.0.0.1", "8.8.8.8")) → c(TRUE, FALSE)
/// ```
#[derive(Debug, Clone)]
pub struct AsFromStrVec<T>(pub Vec<T>);

impl<T: std::str::FromStr> crate::from_r::TryFromSexp for AsFromStrVec<T>
where
    T::Err: std::fmt::Display,
{
    type Error = crate::from_r::SexpError;

    fn try_from_sexp(sexp: crate::SEXP) -> Result<Self, Self::Error> {
        let strings: Vec<String> = crate::from_r::TryFromSexp::try_from_sexp(sexp)?;
        let mut result = Vec::with_capacity(strings.len());
        let mut errors = Vec::new();
        for (i, s) in strings.iter().enumerate() {
            match s.parse::<T>() {
                Ok(v) => result.push(v),
                Err(e) => errors.push(format!("index {i}: {e}")),
            }
        }
        if errors.is_empty() {
            Ok(AsFromStrVec(result))
        } else {
            Err(crate::from_r::SexpError::InvalidValue(format!(
                "parse errors: {}",
                errors.join("; ")
            )))
        }
    }

    unsafe fn try_from_sexp_unchecked(sexp: crate::SEXP) -> Result<Self, Self::Error> {
        let strings: Vec<String> =
            unsafe { crate::from_r::TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let mut result = Vec::with_capacity(strings.len());
        let mut errors = Vec::new();
        for (i, s) in strings.iter().enumerate() {
            match s.parse::<T>() {
                Ok(v) => result.push(v),
                Err(e) => errors.push(format!("index {i}: {e}")),
            }
        }
        if errors.is_empty() {
            Ok(AsFromStrVec(result))
        } else {
            Err(crate::from_r::SexpError::InvalidValue(format!(
                "parse errors: {}",
                errors.join("; ")
            )))
        }
    }
}
// endregion

// region: Collect — zero-allocation iterator-to-R-vector adapters

/// Write an `ExactSizeIterator` of native R types directly into an R vector.
///
/// Skips the intermediate `Vec` allocation — the R vector is allocated once
/// and the iterator writes directly into it.
///
/// Requires `ExactSizeIterator` because R vectors must know their length
/// at allocation time.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn sines(n: i32) -> Collect<impl ExactSizeIterator<Item = f64>> {
///     Collect((0..n).map(|i| (i as f64).sin()))
/// }
/// ```
pub struct Collect<I>(pub I);

impl<I, T> IntoR for Collect<I>
where
    I: ExactSizeIterator<Item = T>,
    T: crate::RNativeType,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }

    #[inline]
    fn into_sexp(self) -> crate::SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<T>(self.0.len());
            for (slot, val) in dst.iter_mut().zip(self.0) {
                *slot = val;
            }
            sexp
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector_unchecked::<T>(self.0.len());
            for (slot, val) in dst.iter_mut().zip(self.0) {
                *slot = val;
            }
            sexp
        }
    }
}

/// Write an `ExactSizeIterator` of `String` directly into an R character vector.
///
/// Strings require per-element CHARSXP allocation (no bulk `copy_from_slice`),
/// so this is a separate type from [`Collect`].
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn upper(words: Vec<String>) -> CollectStrings<impl ExactSizeIterator<Item = String>> {
///     CollectStrings(words.into_iter().map(|w| w.to_uppercase()))
/// }
/// ```
pub struct CollectStrings<I>(pub I);

impl<I> IntoR for CollectStrings<I>
where
    I: ExactSizeIterator<Item = String>,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        // Collect String refs for str_iter_to_strsxp.
        let strings: Vec<String> = self.0.collect();
        Ok(crate::into_r::str_iter_to_strsxp(
            strings.iter().map(|s| s.as_str()),
        ))
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        let strings: Vec<String> = self.0.collect();
        Ok(unsafe {
            crate::into_r::str_iter_to_strsxp_unchecked(strings.iter().map(|s| s.as_str()))
        })
    }
}

/// Write an `ExactSizeIterator` of `Option<T>` directly into an R vector with NA support.
///
/// `None` values become `NA` in R. Works for `f64` and `i32`.
///
/// # Example
///
/// ```ignore
/// #[miniextendr]
/// fn with_gaps(n: i32) -> CollectNA<impl ExactSizeIterator<Item = Option<f64>>> {
///     CollectNA((0..n).map(|i| if i % 3 == 0 { None } else { Some(i as f64) }))
/// }
/// ```
pub struct CollectNA<I>(pub I);

impl<I> IntoR for CollectNA<I>
where
    I: ExactSizeIterator<Item = Option<f64>>,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }

    #[inline]
    fn into_sexp(self) -> crate::SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<f64>(self.0.len());
            for (slot, val) in dst.iter_mut().zip(self.0) {
                *slot = val.unwrap_or(crate::altrep_traits::NA_REAL);
            }
            sexp
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector_unchecked::<f64>(self.0.len());
            for (slot, val) in dst.iter_mut().zip(self.0) {
                *slot = val.unwrap_or(crate::altrep_traits::NA_REAL);
            }
            sexp
        }
    }
}

/// Write an `ExactSizeIterator` of `Option<i32>` directly into an R integer vector with NA.
pub struct CollectNAInt<I>(pub I);

impl<I> IntoR for CollectNAInt<I>
where
    I: ExactSizeIterator<Item = Option<i32>>,
{
    type Error = std::convert::Infallible;

    #[inline]
    fn try_into_sexp(self) -> Result<crate::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }

    #[inline]
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::SEXP, Self::Error> {
        Ok(unsafe { self.into_sexp_unchecked() })
    }

    #[inline]
    fn into_sexp(self) -> crate::SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector::<i32>(self.0.len());
            for (slot, val) in dst.iter_mut().zip(self.0) {
                *slot = val.unwrap_or(crate::altrep_traits::NA_INTEGER);
            }
            sexp
        }
    }

    #[inline]
    unsafe fn into_sexp_unchecked(self) -> crate::SEXP {
        unsafe {
            let (sexp, dst) = crate::into_r::alloc_r_vector_unchecked::<i32>(self.0.len());
            for (slot, val) in dst.iter_mut().zip(self.0) {
                *slot = val.unwrap_or(crate::altrep_traits::NA_INTEGER);
            }
            sexp
        }
    }
}
// endregion
