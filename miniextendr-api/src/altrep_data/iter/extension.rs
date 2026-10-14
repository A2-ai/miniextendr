//! Iterator capability-based construction; no per-adapter implementations.

use super::{
    IterComplexData, IterIntData, IterLogicalData, IterRLogicalData, IterRawData, IterRealData,
    IterStringData,
};
use crate::altrep_data::AltrepLen;

/// Choose the ALTREP data family for an iterator's element type.
///
/// Built-in mappings cover native numeric values, logicals and owned strings.
/// A local element type may implement this trait to select its own data adapter
/// (for example, an existing coercing adapter). The resulting data still needs
/// a concrete registered class, which owns duplication and serialization policy.
///
/// Raw SEXP is deliberately not inferred: storing one requires a GC-root owner.
pub trait AltrepIteratorItem: Sized {
    /// Data adapter retaining the concrete iterator type.
    type Data<I: Iterator<Item = Self>>: AltrepLen;

    /// Store the iterator without advancing it.
    fn into_data<I: Iterator<Item = Self>>(iter: I, len: usize) -> Self::Data<I>;
}

/// ALTREP data constructors for every iterator, including third-party adapters.
///
/// Exact-size iterators can supply their remaining length. Other iterators need
/// an explicit finite prefix length; neither constructor counts or collects them.
/// Borrowed iterators can be used as data adapters, but an R-owned class must own
/// its source for the whole R lifetime.
///
/// The item type picks the data family, so an unsuffixed integer literal can't
/// fall back to `i32` here: write `0..6i32`, not `0..6`.
///
/// ```
/// use miniextendr_api::{AltrepIteratorExt, AltIntegerData, AltrepLen};
/// let data = (0..6i32).map(|x| x * 2).into_altrep_data_exact();
/// assert_eq!(data.altrep_len(), 6);
/// assert_eq!(data.elt(2), 4);
/// ```
pub trait AltrepIteratorExt: Iterator + Sized {
    /// Cache a finite prefix in the item's chosen ALTREP data family.
    fn into_altrep_data(self, len: usize) -> <Self::Item as AltrepIteratorItem>::Data<Self>
    where
        Self::Item: AltrepIteratorItem,
    {
        Self::Item::into_data(self, len)
    }

    /// Use the iterator's exact remaining length without advancing it.
    fn into_altrep_data_exact(self) -> <Self::Item as AltrepIteratorItem>::Data<Self>
    where
        Self: ExactSizeIterator,
        Self::Item: AltrepIteratorItem,
    {
        let len = self.len();
        self.into_altrep_data(len)
    }
}

impl<I: Iterator> AltrepIteratorExt for I {}

macro_rules! iterator_item {
    ($item:ty => $data:ident) => {
        impl AltrepIteratorItem for $item {
            type Data<I: Iterator<Item = Self>> = $data<I>;

            fn into_data<I: Iterator<Item = Self>>(iter: I, len: usize) -> Self::Data<I> {
                $data::from_iter(iter, len)
            }
        }
    };
}

iterator_item!(i32 => IterIntData);
iterator_item!(f64 => IterRealData);
iterator_item!(u8 => IterRawData);
iterator_item!(crate::Rcomplex => IterComplexData);
iterator_item!(crate::RLogical => IterRLogicalData);
iterator_item!(bool => IterLogicalData);
iterator_item!(String => IterStringData);

// IsNA excludes raw bytes: None must not silently become a real zero byte.
impl<T: crate::IsNA + AltrepIteratorItem> AltrepIteratorItem for Option<T> {
    type Data<I: Iterator<Item = Self>> = T::Data<std::iter::Map<I, fn(Option<T>) -> T>>;

    fn into_data<I: Iterator<Item = Self>>(iter: I, len: usize) -> Self::Data<I> {
        let native: fn(Option<T>) -> T = |value| value.unwrap_or(T::R_NA);
        T::into_data(iter.map(native), len)
    }
}
