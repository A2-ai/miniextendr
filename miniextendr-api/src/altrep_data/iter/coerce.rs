//! Iterator-backed ALTREP data adaptors with coercion support.
//!
//! Provides `IterIntCoerceData`, `IterRealCoerceData`, and `IterIntFromBoolData`
//! for iterators that produce values coercible to the target R type, plus the
//! string/complex adaptors (`IterStringData`, `IterComplexData`).
//!
//! See the iterator-adaptor section in the [`altrep_data`](crate::altrep_data)
//! module docs for how to expose
//! these adaptors to R (wrap in a `#[derive(Altrep*)]` + `#[altrep(manual)]`
//! struct).

use super::IterState;
use super::macros::define_iter_data;
use crate::altrep_data::{AltComplexData, AltIntegerData, AltRealData, AltStringData, AltrepLen};

/// Iterator-backed integer vector data adaptor with coercion from any integer-like type.
///
/// Wraps an iterator producing values that coerce to `i32` (e.g., `u16`, `i8`, etc.)
/// and implements the data-level traits ([`AltrepLen`] + [`AltIntegerData`]).
/// To expose it to R, wrap it in a `#[derive(AltrepInteger)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterIntCoerceData;
///
/// // Create from an iterator of u16 values
/// let iter = (0..10u16).map(|x| x * 100);
/// let data = IterIntCoerceData::from_iter(iter, 10);
/// // Values are coerced from u16 to i32 when accessed
/// ```
pub struct IterIntCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<i32> + Copy,
{
    state: IterState<I, T>,
}

impl<I, T> IterIntCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<i32> + Copy,
{
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I, T> IterIntCoerceData<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: crate::coerce::Coerce<i32> + Copy,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I, T> AltrepLen for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<i32> + Copy,
{
    fn altrep_len(&self) -> usize {
        self.state.len()
    }
}

impl<I, T> AltIntegerData for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<i32> + Copy,
{
    fn elt(&self, i: usize) -> i32 {
        self.state
            .get_element(i)
            .map(|val| val.coerce())
            .unwrap_or(crate::altrep_traits::NA_INTEGER)
    }
}

/// Iterator-backed real vector data adaptor with coercion from any float-like type.
///
/// Wraps an iterator producing values that coerce to `f64` (e.g., `f32`, integer types).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterRealCoerceData;
///
/// // Create from an iterator of f32 values
/// let iter = (0..5).map(|x| x as f32 * 1.5);
/// let data = IterRealCoerceData::from_iter(iter, 5);
/// // Values are coerced from f32 to f64 when accessed
/// ```
pub struct IterRealCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<f64> + Copy,
{
    state: IterState<I, T>,
}

impl<I, T> IterRealCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<f64> + Copy,
{
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I, T> IterRealCoerceData<I, T>
where
    I: ExactSizeIterator<Item = T>,
    T: crate::coerce::Coerce<f64> + Copy,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I, T> AltrepLen for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<f64> + Copy,
{
    fn altrep_len(&self) -> usize {
        self.state.len()
    }
}

impl<I, T> AltRealData for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T>,
    T: crate::coerce::Coerce<f64> + Copy,
{
    fn elt(&self, i: usize) -> f64 {
        self.state
            .get_element(i)
            .map(|val| val.coerce())
            .unwrap_or(crate::altrep_traits::NA_REAL)
    }
}

define_iter_data! {
/// Iterator-backed integer vector data adaptor with coercion from bool.
///
/// Wraps an iterator producing `bool` values that coerce to `i32`.
/// Useful for converting boolean iterators to integer vectors.
IterIntFromBoolData, bool
}

impl<I> AltIntegerData for IterIntFromBoolData<I>
where
    I: Iterator<Item = bool>,
{
    fn elt(&self, i: usize) -> i32 {
        use crate::coerce::Coerce;
        self.state
            .get_element(i)
            .map(|val| val.coerce())
            .unwrap_or(crate::altrep_traits::NA_INTEGER)
    }
}

define_iter_data! {
/// Iterator-backed string vector data adaptor.
///
/// Wraps an iterator producing `String` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltStringData`]) for backing an ALTREP character
/// vector. To expose it to R, wrap it in a `#[derive(AltrepString)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
///
/// # Performance Warning
///
/// Unlike other `Iter*Data` types, **accessing ANY element forces full materialization
/// of the entire iterator**. This is because R's `AltStringData::elt()` returns a borrowed
/// `&str`, which requires stable storage. The internal `RefCell` cannot provide the required
/// lifetime, so all strings must be materialized upfront.
///
/// This means:
/// - `elt(0)` on a million-element iterator will generate ALL million strings
/// - There is no lazy evaluation benefit for string iterators
/// - Memory usage equals the full vector regardless of access patterns
///
/// For truly lazy string ALTREP, consider implementing a custom type that stores
/// strings in a way that allows borrowing without full materialization (e.g., arena
/// allocation or caching generated strings incrementally).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterStringData;
///
/// let iter = (0..5).map(|x| format!("item_{}", x));
/// let data = IterStringData::from_iter(iter, 5);
/// // First access to ANY element will materialize all 5 strings
/// ```
IterStringData, String
}

impl<I> AltStringData for IterStringData<I>
where
    I: Iterator<Item = String>,
{
    fn elt(&self, i: usize) -> Option<&str> {
        // Materialize to get stable storage for &str references
        // This is necessary because we can't return &str from RefCell borrows
        let materialized = self.state.materialize_all();
        materialized.get(i).map(|s| s.as_str())
    }
}

define_iter_data! {
/// Iterator-backed complex number vector data adaptor.
///
/// Wraps an iterator producing `Rcomplex` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltComplexData`]) for backing an ALTREP complex
/// vector. To expose it to R, wrap it in a `#[derive(AltrepComplex)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterComplexData;
/// use miniextendr_api::Rcomplex;
///
/// let iter = (0..5).map(|x| Rcomplex { r: x as f64, i: (x * 2) as f64 });
/// let data = IterComplexData::from_iter(iter, 5);
/// ```
IterComplexData, crate::Rcomplex
}

impl<I> AltComplexData for IterComplexData<I>
where
    I: Iterator<Item = crate::Rcomplex>,
{
    fn elt(&self, i: usize) -> crate::Rcomplex {
        self.state
            .get_element(i)
            .unwrap_or(<crate::Rcomplex as crate::RNativeType>::R_NA)
    }

    fn as_slice(&self) -> Option<&[crate::Rcomplex]> {
        self.state.as_materialized()
    }
}
