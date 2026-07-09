//! Core iterator-backed ALTREP data adaptors.
//!
//! Provides `IterState<I, T>` (the shared lazy-caching state machine) and
//! data-adaptor types for the integer/real/logical/raw ALTREP families:
//! `IterIntData`, `IterRealData`, `IterLogicalData`, `IterRawData`. The
//! string/list/complex adaptors (`IterStringData`, `IterListData`,
//! `IterComplexData`) live in `super::coerce`.
//!
//! See the [`super`](crate::altrep_data::iter) module docs for how to expose
//! these adaptors to R: they implement only the data-level traits
//! ([`AltrepLen`] + `Alt*Data`) and must be wrapped in a concrete
//! `#[derive(Altrep*)]` + `#[altrep(manual)]` struct to back a live ALTREP
//! vector.

use std::cell::RefCell;
use std::sync::OnceLock;

use crate::altrep_data::{
    AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepLen, Logical, fill_region,
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
/// - **Safe:** Uses `RefCell` for interior mutability, protected by R's GC
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
    /// If the iterator produces a different number of elements than `len`:
    /// - Fewer elements: Missing indices return `None`/NA/default values
    /// - More elements: Extra elements are ignored (truncated to `len`)
    ///
    /// A warning is printed to stderr when a mismatch is detected.
    pub fn new(iter: I, len: usize) -> Self {
        Self {
            len,
            iter: RefCell::new(Some(iter)),
            cache: RefCell::new(Vec::with_capacity(len.min(1024))),
            materialized: OnceLock::new(),
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
        {
            let iter = iter_opt.as_mut()?;

            // Fill cache up to and including index i. `?` bails with `None`
            // when the iterator is exhausted before reaching the expected length.
            while cache.len() <= i {
                let elem = iter.next()?;
                cache.push(elem);
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

    /// Materialize all remaining elements from the iterator.
    ///
    /// After this call, all elements are guaranteed to be in memory and
    /// `as_materialized()` will return `Some`.
    ///
    /// # Length Mismatch Handling
    ///
    /// If the iterator produces fewer elements than declared `len`, the missing
    /// elements are left uninitialized in the cache (callers should handle this
    /// via bounds checking). If the iterator produces more elements than declared,
    /// extra elements are silently ignored (truncated to `len`).
    ///
    /// A warning is printed to stderr if a length mismatch is detected.
    pub fn materialize_all(&self) -> &[T] {
        // Already materialized?
        if let Some(vec) = self.materialized.get() {
            return vec;
        }

        // Consume iterator and move cache to materialized storage
        let mut cache = self.cache.borrow_mut();
        let mut iter_opt = self.iter.borrow_mut();

        if let Some(iter) = iter_opt.take() {
            // Drain remaining elements (up to self.len to avoid memory issues)
            for elem in iter {
                if cache.len() >= self.len {
                    // Iterator produced more than expected - truncate and warn
                    eprintln!(
                        "[miniextendr warning] iterator ALTREP: iterator produced more elements than declared length ({}), truncating",
                        self.len
                    );
                    break;
                }
                cache.push(elem);
            }

            // Check if iterator exhausted early
            if cache.len() < self.len {
                eprintln!(
                    "[miniextendr warning] iterator ALTREP: iterator produced {} elements, expected {} - accessing missing indices will return NA/default",
                    cache.len(),
                    self.len
                );
            }
        }

        // Move cache to materialized (take ownership)
        let vec = std::mem::take(&mut *cache);
        drop(cache);
        drop(iter_opt);

        // Store in OnceLock and return reference
        self.materialized.get_or_init(|| vec)
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

/// Iterator-backed integer vector data adaptor.
///
/// Wraps an iterator producing `i32` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltIntegerData`]) for backing an ALTREP integer
/// vector. To expose it to R, wrap it in a `#[derive(AltrepInteger)]` +
/// `#[altrep(manual)]` struct (see the [module docs](crate::altrep_data::iter)).
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterIntData;
///
/// // Create from an iterator
/// let data = IterIntData::from_iter((1..=10).map(|x| x * 2), 10);
/// ```
pub struct IterIntData<I: Iterator<Item = i32>> {
    state: IterState<I, i32>,
}

impl<I: Iterator<Item = i32>> IterIntData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = i32>> IterIntData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = i32>> AltrepLen for IterIntData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
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

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

/// Iterator-backed real (f64) vector data adaptor.
///
/// Wraps an iterator producing `f64` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltRealData`]) for backing an ALTREP real vector.
/// To expose it to R, wrap it in a `#[derive(AltrepReal)]` +
/// `#[altrep(manual)]` struct (see the [module docs](crate::altrep_data::iter)).
pub struct IterRealData<I: Iterator<Item = f64>> {
    state: IterState<I, f64>,
}

impl<I: Iterator<Item = f64>> IterRealData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = f64>> IterRealData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = f64>> AltrepLen for IterRealData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = f64>> AltRealData for IterRealData<I> {
    fn elt(&self, i: usize) -> f64 {
        self.state.get_element(i).unwrap_or(f64::NAN)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        self.state.as_materialized()
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

/// Iterator-backed logical vector data adaptor.
///
/// Wraps an iterator producing `bool` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltLogicalData`]) for backing an ALTREP logical
/// vector. To expose it to R, wrap it in a `#[derive(AltrepLogical)]` +
/// `#[altrep(manual)]` struct (see the [module docs](crate::altrep_data::iter)).
pub struct IterLogicalData<I: Iterator<Item = bool>> {
    state: IterState<I, bool>,
}

impl<I: Iterator<Item = bool>> IterLogicalData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = bool>> IterLogicalData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = bool>> AltrepLen for IterLogicalData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = bool>> AltLogicalData for IterLogicalData<I> {
    fn elt(&self, i: usize) -> Logical {
        self.state
            .get_element(i)
            .map(Logical::from_bool)
            .unwrap_or(Logical::Na)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx).to_r_int())
    }
}

/// Iterator-backed raw (u8) vector data adaptor.
///
/// Wraps an iterator producing `u8` values and implements the data-level
/// traits ([`AltrepLen`] + [`AltRawData`]) for backing an ALTREP raw vector.
/// To expose it to R, wrap it in a `#[derive(AltrepRaw)]` +
/// `#[altrep(manual)]` struct (see the [module docs](crate::altrep_data::iter)).
pub struct IterRawData<I: Iterator<Item = u8>> {
    state: IterState<I, u8>,
}

impl<I: Iterator<Item = u8>> IterRawData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = u8>> IterRawData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = u8>> AltrepLen for IterRawData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = u8>> AltRawData for IterRawData<I> {
    fn elt(&self, i: usize) -> u8 {
        self.state.get_element(i).unwrap_or(0)
    }

    fn as_slice(&self) -> Option<&[u8]> {
        self.state.as_materialized()
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}
