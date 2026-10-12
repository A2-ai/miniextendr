//! Core iterator-backed ALTREP data adaptors.
//!
//! Provides `IterState<I, T>` (the shared lazy-caching state machine) and
//! data-adaptor types for the integer/real/logical/raw ALTREP families:
//! `IterIntData`, `IterRealData`, `IterLogicalData`, `IterRawData`. The
//! string/complex adaptors live in `super::coerce`; the rooted list adaptor
//! lives in `super::list`.
//!
//! See the iterator-adaptor section in the [`altrep_data`](crate::altrep_data)
//! module docs for how to expose
//! these adaptors to R: they implement only the data-level traits
//! ([`AltrepLen`] + `Alt*Data`) and must be wrapped in a concrete
//! `#[derive(Altrep*)]` + `#[altrep(manual)]` struct to back a live ALTREP
//! vector.

use super::macros::define_iter_data;
use std::cell::RefCell;
use std::sync::OnceLock;

use crate::altrep_data::{
    AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepLen, Logical,
};

/// Core state for iterator-backed ALTREP vectors.
///
/// Provides lazy element generation with caching for random-access semantics.
/// Iterator elements are cached as they're accessed, enabling repeatable reads.
///
/// # Type Parameters
///
/// - `I`: The iterator type (must be `ExactSizeIterator` or provide explicit length)
/// - `T`: The element type produced by the iterator
///
/// # Design
///
/// - **Lazy:** Elements generated on-demand via `elt(i)`
/// - **Cached:** Once generated, elements stored in cache for repeat access
/// - **Materializable:** Can be fully materialized for `Dataptr` or serialization
/// - **Borrow-checked:** Uses `RefCell` for interior mutability on the R thread
#[derive(Clone)]
pub struct IterState<I, T> {
    /// Vector length (from `ExactSizeIterator::len()` or explicit)
    len: usize,
    /// Iterator state (consumed as we advance)
    iter: RefCell<Option<I>>,
    /// Cache of generated elements (prefix of the vector)
    cache: RefCell<Vec<T>>,
    /// Full materialization (when all elements have been generated)
    materialized: OnceLock<Vec<T>>,
}

impl<I, T> IterState<I, T>
where
    I: Iterator<Item = T>,
{
    /// Create a new iterator state with an explicit length.
    ///
    /// # Arguments
    ///
    /// - `iter`: The iterator to wrap
    /// - `len`: The expected number of elements
    ///
    /// # Length Mismatch
    ///
    /// [`get_element`](Self::get_element) returns `None` for a missing index.
    /// [`materialize_all`](Self::materialize_all) requires the complete declared
    /// prefix and panics if the source ends early. Neither reads past `len`.
    pub fn new(iter: I, len: usize) -> Self {
        Self {
            len,
            iter: RefCell::new(Some(iter)),
            cache: RefCell::new(Vec::with_capacity(len.min(1024))),
            materialized: OnceLock::new(),
        }
    }

    /// Restore a realized sequence without constructing or advancing an iterator.
    ///
    /// The iterator type is retained, but its state is absent: all reads use the
    /// supplied values. This supports deserialization even when `I: !Default`.
    pub fn from_values(values: Vec<T>) -> Self {
        Self {
            len: values.len(),
            iter: RefCell::new(None),
            cache: RefCell::new(Vec::new()),
            materialized: OnceLock::from(values),
        }
    }

    /// Ensure the element at index `i` is in the cache and return it by value.
    ///
    /// Advances the iterator as needed. Only works for `Copy` types.
    ///
    /// # Returns
    ///
    /// - `Some(T)` if element exists and has been generated
    /// - `None` if index is out of bounds or iterator exhausted before reaching index `i`
    pub fn get_element(&self, i: usize) -> Option<T>
    where
        T: Copy,
    {
        // Check bounds
        if i >= self.len {
            return None;
        }

        // If fully materialized, return from materialized vec
        if let Some(vec) = self.materialized.get() {
            return vec.get(i).copied();
        }

        // Otherwise, check cache and advance iterator if needed
        let mut cache = self.cache.borrow_mut();

        // Already in cache?
        if i < cache.len() {
            return Some(cache[i]);
        }

        // Need to advance iterator to index i
        let mut iter_opt = self.iter.borrow_mut();
        // Fill cache up to and including index i. A source that ends early is
        // dropped at once: a non-fused iterator could otherwise yield again on
        // a later call, giving an index that read as missing a value.
        while cache.len() <= i {
            match iter_opt.as_mut()?.next() {
                Some(value) => cache.push(value),
                None => {
                    iter_opt.take();
                    return None;
                }
            }
        }

        let value = cache[i];

        // If we've generated the full vector via random-access, promote the cache
        // to the materialized storage so `as_slice()` can expose it.
        if cache.len() == self.len {
            iter_opt.take();

            let vec = std::mem::take(&mut *cache);
            drop(cache);
            drop(iter_opt);

            let _ = self.materialized.set(vec);
        }

        Some(value)
    }

    /// Materialize exactly the declared prefix, including already cached elements.
    ///
    /// Does not request an element beyond `len`, including when `len` is zero
    /// or the source is infinite. On success, [`Self::as_materialized`] returns
    /// a slice whose length is exactly [`Self::len`].
    ///
    /// # Panics
    ///
    /// If the source ends before the declared length. The partial prefix stays
    /// cached, but is never published as a complete buffer. A failed attempt
    /// drops the iterator; another attempt cannot resume an exhausted source.
    pub fn materialize_all(&self) -> &[T] {
        match self.try_materialize_all() {
            Ok(vec) => vec,
            Err(cached) => panic!(
                "ALTREP iterator ended before element {} of its declared length {}",
                cached + 1,
                self.len
            ),
        }
    }

    /// [`Self::materialize_all`] without the panic: `Err` carries the number
    /// of elements the source produced before it ended.
    pub fn try_materialize_all(&self) -> Result<&[T], usize> {
        if let Some(vec) = self.materialized.get() {
            return Ok(vec);
        }

        let mut cache = self.cache.borrow_mut();
        let mut iter_opt = self.iter.borrow_mut();
        if let Some(mut iter) = iter_opt.take() {
            while cache.len() < self.len {
                let Some(value) = iter.next() else {
                    break;
                };
                cache.push(value);
            }
        }
        if cache.len() != self.len {
            return Err(cache.len());
        }

        let vec = std::mem::take(&mut *cache);
        drop(cache);
        drop(iter_opt);
        Ok(self.materialized.get_or_init(|| vec))
    }

    /// Get the materialized vector if all elements have been generated.
    ///
    /// Returns `None` if not yet fully materialized.
    pub fn as_materialized(&self) -> Option<&[T]> {
        self.materialized.get().map(|v| v.as_slice())
    }

    /// Get the current length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<I, T> IterState<I, T>
where
    I: ExactSizeIterator<Item = T>,
{
    /// Create a new iterator state from an `ExactSizeIterator`.
    ///
    /// The length is automatically determined from `iter.len()`.
    pub fn from_exact_size(iter: I) -> Self {
        let len = iter.len();
        Self::new(iter, len)
    }
}

define_iter_data! {
/// Iterator-backed integer vector data adaptor.
///
/// Wraps an iterator producing `i32` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltIntegerData`]) for backing an ALTREP integer
/// vector. To expose it to R, wrap it in a `#[derive(AltrepInteger)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterIntData;
///
/// // Create from an iterator
/// let data = IterIntData::from_iter((1..=10).map(|x| x * 2), 10);
/// ```
IterIntData, i32
}

impl<I: Iterator<Item = i32>> AltIntegerData for IterIntData<I> {
    fn elt(&self, i: usize) -> i32 {
        self.state
            .get_element(i)
            .unwrap_or(crate::altrep_traits::NA_INTEGER)
    }

    fn as_slice(&self) -> Option<&[i32]> {
        self.state.as_materialized()
    }
}

define_iter_data! {
/// Iterator-backed real (f64) vector data adaptor.
///
/// Wraps an iterator producing `f64` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltRealData`]) for backing an ALTREP real vector.
/// To expose it to R, wrap it in a `#[derive(AltrepReal)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
IterRealData, f64
}

impl<I: Iterator<Item = f64>> AltRealData for IterRealData<I> {
    fn elt(&self, i: usize) -> f64 {
        self.state
            .get_element(i)
            .unwrap_or(crate::altrep_traits::NA_REAL)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        self.state.as_materialized()
    }
}

define_iter_data! {
/// Iterator-backed logical vector data adaptor.
///
/// Wraps an iterator producing `bool` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltLogicalData`]) for backing an ALTREP logical
/// vector. To expose it to R, wrap it in a `#[derive(AltrepLogical)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
IterLogicalData, bool
}

impl<I: Iterator<Item = bool>> AltLogicalData for IterLogicalData<I> {
    fn elt(&self, i: usize) -> Logical {
        self.state
            .get_element(i)
            .map(Logical::from_bool)
            .unwrap_or(Logical::Na)
    }
}

define_iter_data! {
/// Iterator-backed raw (u8) vector data adaptor.
///
/// Wraps an iterator producing `u8` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltRawData`]) for backing an ALTREP raw vector.
/// To expose it to R, wrap it in a `#[derive(AltrepRaw)]` +
/// `#[altrep(manual)]` struct (see the iterator module documentation).
IterRawData, u8
}

impl<I: Iterator<Item = u8>> AltRawData for IterRawData<I> {
    fn elt(&self, i: usize) -> u8 {
        self.state.get_element(i).unwrap_or(0)
    }

    fn as_slice(&self) -> Option<&[u8]> {
        self.state.as_materialized()
    }
}

define_iter_data! {
/// Iterator-backed logical data retaining R's native logical bit patterns.
IterRLogicalData, crate::RLogical
}

impl<I: Iterator<Item = crate::RLogical>> AltLogicalData for IterRLogicalData<I> {
    fn elt(&self, i: usize) -> Logical {
        self.state
            .get_element(i)
            .map(Logical::from)
            .unwrap_or(Logical::Na)
    }

    fn as_r_slice(&self) -> Option<&[i32]> {
        let values = self.state.as_materialized()?;
        // RLogical is transparent over i32 and accepts every bit pattern.
        Some(unsafe { std::slice::from_raw_parts(values.as_ptr().cast(), values.len()) })
    }

    fn get_region(&self, start: usize, len: usize, out: &mut [i32]) -> usize {
        crate::altrep_data::fill_region(start, len, self.altrep_len(), out, |i| {
            self.state
                .get_element(i)
                .unwrap_or(crate::RLogical::NA)
                .to_i32()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::IterState;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn materialization_never_probes_past_the_declared_prefix() {
        for len in [0, 3] {
            let source = (0..len)
                .map(|i| i.to_string())
                .chain(std::iter::once_with(|| panic!("read beyond the prefix")));
            let state = IterState::new(source, len);
            let expected: Vec<_> = (0..len).map(|i| i.to_string()).collect();
            assert_eq!(state.materialize_all(), expected);
            assert_eq!(state.materialize_all(), expected);
        }
    }

    #[test]
    fn materialization_continues_after_a_cached_prefix() {
        let source = (0..4).chain(std::iter::once_with(|| panic!("read beyond the prefix")));
        let state = IterState::new(source, 4);
        assert_eq!(state.get_element(1), Some(1));
        assert_eq!(state.materialize_all(), [0, 1, 2, 3]);
    }

    #[test]
    fn an_exhausted_source_is_never_polled_again() {
        // Yields 0, then None, then 1 if polled again (not fused).
        struct Flaky(u8);
        impl Iterator for Flaky {
            type Item = i32;
            fn next(&mut self) -> Option<i32> {
                self.0 += 1;
                match self.0 {
                    1 => Some(0),
                    3 => Some(1),
                    _ => None,
                }
            }
        }
        let state = IterState::new(Flaky(0), 2);
        assert_eq!(state.get_element(1), None);
        assert_eq!(state.get_element(1), None);
        assert_eq!(state.get_element(0), Some(0));
    }

    #[test]
    fn early_exhaustion_never_publishes_a_partial_buffer() {
        let state = IterState::new(0..2, 3);
        for _ in 0..2 {
            assert!(catch_unwind(AssertUnwindSafe(|| state.materialize_all())).is_err());
            assert!(state.as_materialized().is_none());
            assert_eq!(state.get_element(0), Some(0));
            assert_eq!(state.get_element(2), None);
        }
        assert_eq!(state.try_materialize_all(), Err(2));
    }

    #[test]
    fn clone_preserves_a_partial_prefix_without_advancing_the_source() {
        let original = IterState::new(10..20i32, 6);
        assert_eq!(original.get_element(2), Some(12));
        let cloned = original.clone();
        assert!(original.as_materialized().is_none());
        assert!(cloned.as_materialized().is_none());
        assert_eq!(cloned.materialize_all(), &[10, 11, 12, 13, 14, 15]);
        assert!(original.as_materialized().is_none());
        assert_eq!(original.get_element(3), Some(13));
        assert_eq!(original.materialize_all(), cloned.materialize_all());
    }

    #[test]
    fn restored_values_need_no_iterator_constructor() {
        type Source = std::iter::Take<std::iter::Repeat<i32>>;
        let restored = IterState::<Source, i32>::from_values(vec![4, 5, 6]);
        assert_eq!(restored.len(), 3);
        assert_eq!(restored.get_element(1), Some(5));
        assert_eq!(restored.materialize_all(), &[4, 5, 6]);
    }
}
