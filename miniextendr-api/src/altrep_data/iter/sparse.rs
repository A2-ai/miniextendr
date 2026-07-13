//! Sparse iterator-backed ALTREP data adaptors with skipping support.
//!
//! Provides `SparseIterState<I, T>` which uses `Iterator::nth()` to skip elements
//! efficiently, and data-adaptor types for each ALTREP family.
//!
//! See the iterator-adaptor section in the [`altrep_data`](crate::altrep_data)
//! module docs for how to expose
//! these adaptors to R (wrap in a `#[derive(Altrep*)]` + `#[altrep(manual)]`
//! struct).

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::altrep_data::{
    AltComplexData, AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepLen, Logical,
    fill_region,
};

/// Core state for sparse iterator-backed ALTREP vectors.
///
/// Unlike [`super::IterState`], this variant uses `Iterator::nth()` to skip elements
/// efficiently, only caching the elements that are actually accessed.
///
/// # Type Parameters
///
/// - `I`: The iterator type
/// - `T`: The element type produced by the iterator
///
/// # Design
///
/// - **Sparse:** Only accessed elements are cached (uses `BTreeMap`)
/// - **Skipping:** Uses `nth()` to skip directly to requested indices
/// - **Trade-off:** Skipped elements are gone forever (iterator is consumed)
/// - **Best for:** Large iterators where only a small subset of elements are accessed
///
/// # Comparison with `IterState`
///
/// | Feature | `IterState` | `SparseIterState` |
/// |---------|-------------|-------------------|
/// | Cache storage | Contiguous `Vec<T>` | Sparse `BTreeMap<usize, T>` |
/// | Access pattern | Prefix (0..=i) cached | Only accessed indices cached |
/// | Skipped elements | All cached | Gone forever (return NA) |
/// | Memory for sparse access | O(max_index) | O(num_accessed) |
/// | `as_slice()` support | Yes (after full materialization) | No (sparse) |
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::SparseIterIntData;
///
/// // Create from an infinite-ish iterator
/// let data = SparseIterIntData::from_iter((0..).map(|x| x * 2), 1_000_000);
///
/// // Access only element 999_999 - skips directly there
/// let last = data.elt(999_999);  // Only this element is generated
///
/// // Element 0 was skipped and is now inaccessible
/// let first = data.elt(0);  // Returns NA_INTEGER
/// ```
pub struct SparseIterState<I, T> {
    /// Vector length
    len: usize,
    /// Iterator state: (iterator, next index the iterator will produce)
    iter: RefCell<Option<(I, usize)>>,
    /// Sparse cache of accessed elements
    cache: RefCell<BTreeMap<usize, T>>,
}

impl<I, T> SparseIterState<I, T>
where
    I: Iterator<Item = T>,
{
    /// Create a new sparse iterator state with an explicit length.
    ///
    /// # Arguments
    ///
    /// - `iter`: The iterator to wrap
    /// - `len`: The expected number of elements
    pub fn new(iter: I, len: usize) -> Self {
        Self {
            len,
            iter: RefCell::new(Some((iter, 0))),
            cache: RefCell::new(BTreeMap::new()),
        }
    }

    /// Get an element, skipping intermediate elements if needed.
    ///
    /// Uses `Iterator::nth()` to skip efficiently. Skipped elements are
    /// consumed from the iterator and cannot be retrieved later.
    ///
    /// # Returns
    ///
    /// - `Some(T)` if element exists and is accessible
    /// - `None` if:
    ///   - Index is out of bounds
    ///   - Element was already skipped (iterator advanced past it)
    ///   - Iterator exhausted before reaching the index
    pub fn get_element(&self, i: usize) -> Option<T>
    where
        T: Copy,
    {
        // Check bounds
        if i >= self.len {
            return None;
        }

        // Check cache first
        {
            let cache = self.cache.borrow();
            if let Some(&val) = cache.get(&i) {
                return Some(val);
            }
        }

        // Need to get from iterator
        let mut iter_opt = self.iter.borrow_mut();
        let (iter, pos) = iter_opt.as_mut()?;

        // Element already passed? It was skipped.
        if i < *pos {
            return None;
        }

        // Skip to element i using nth()
        let skip_count = i - *pos;
        let elem = iter.nth(skip_count)?;
        *pos = i + 1;

        // Cache the element
        drop(iter_opt);
        self.cache.borrow_mut().insert(i, elem);

        Some(elem)
    }

    /// Get the current iterator position (next index to be produced).
    ///
    /// Returns `None` if the iterator has been exhausted.
    pub fn iterator_position(&self) -> Option<usize> {
        self.iter.borrow().as_ref().map(|(_, pos)| *pos)
    }

    /// Check if an element has been cached.
    pub fn is_cached(&self, i: usize) -> bool {
        self.cache.borrow().contains_key(&i)
    }

    /// Get the number of cached elements.
    pub fn cached_count(&self) -> usize {
        self.cache.borrow().len()
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

impl<I, T> SparseIterState<I, T>
where
    I: ExactSizeIterator<Item = T>,
{
    /// Create a new sparse iterator state from an `ExactSizeIterator`.
    pub fn from_exact_size(iter: I) -> Self {
        let len = iter.len();
        Self::new(iter, len)
    }
}

/// Sparse iterator-backed integer vector data adaptor.
///
/// Uses `Iterator::nth()` to skip directly to requested indices.
/// Only accessed elements are cached; skipped elements return `NA_INTEGER`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::SparseIterIntData;
///
/// // Access only specific elements from a large range
/// let data = SparseIterIntData::from_iter(0..1_000_000, 1_000_000);
/// let elem = data.elt(500_000);  // Skips 0..499_999
/// ```
pub struct SparseIterIntData<I: Iterator<Item = i32>> {
    state: SparseIterState<I, i32>,
}

impl<I: Iterator<Item = i32>> SparseIterIntData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: SparseIterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = i32>> SparseIterIntData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: SparseIterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = i32>> AltrepLen for SparseIterIntData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = i32>> AltIntegerData for SparseIterIntData<I> {
    fn elt(&self, i: usize) -> i32 {
        self.state
            .get_element(i)
            .unwrap_or(crate::altrep_traits::NA_INTEGER)
    }

    fn as_slice(&self) -> Option<&[i32]> {
        // Sparse storage cannot provide contiguous slice
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

/// Sparse iterator-backed real (f64) vector data adaptor.
///
/// Uses `Iterator::nth()` to skip directly to requested indices.
/// Only accessed elements are cached; skipped elements return `NaN`.
pub struct SparseIterRealData<I: Iterator<Item = f64>> {
    state: SparseIterState<I, f64>,
}

impl<I: Iterator<Item = f64>> SparseIterRealData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: SparseIterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = f64>> SparseIterRealData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: SparseIterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = f64>> AltrepLen for SparseIterRealData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = f64>> AltRealData for SparseIterRealData<I> {
    fn elt(&self, i: usize) -> f64 {
        self.state.get_element(i).unwrap_or(f64::NAN)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

/// Sparse iterator-backed logical vector data adaptor.
pub struct SparseIterLogicalData<I: Iterator<Item = bool>> {
    state: SparseIterState<I, bool>,
}

impl<I: Iterator<Item = bool>> SparseIterLogicalData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: SparseIterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = bool>> SparseIterLogicalData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: SparseIterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = bool>> AltrepLen for SparseIterLogicalData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = bool>> AltLogicalData for SparseIterLogicalData<I> {
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

/// Sparse iterator-backed raw (u8) vector data adaptor.
pub struct SparseIterRawData<I: Iterator<Item = u8>> {
    state: SparseIterState<I, u8>,
}

impl<I: Iterator<Item = u8>> SparseIterRawData<I> {
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: SparseIterState::new(iter, len),
        }
    }
}

impl<I: ExactSizeIterator<Item = u8>> SparseIterRawData<I> {
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: SparseIterState::from_exact_size(iter),
        }
    }
}

impl<I: Iterator<Item = u8>> AltrepLen for SparseIterRawData<I> {
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I: Iterator<Item = u8>> AltRawData for SparseIterRawData<I> {
    fn elt(&self, i: usize) -> u8 {
        self.state.get_element(i).unwrap_or(0)
    }

    fn as_slice(&self) -> Option<&[u8]> {
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

/// Sparse iterator-backed complex number vector data adaptor.
pub struct SparseIterComplexData<I>
where
    I: Iterator<Item = crate::Rcomplex>,
{
    state: SparseIterState<I, crate::Rcomplex>,
}

impl<I> SparseIterComplexData<I>
where
    I: Iterator<Item = crate::Rcomplex>,
{
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: SparseIterState::new(iter, len),
        }
    }
}

impl<I> SparseIterComplexData<I>
where
    I: ExactSizeIterator<Item = crate::Rcomplex>,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: SparseIterState::from_exact_size(iter),
        }
    }
}

impl<I> AltrepLen for SparseIterComplexData<I>
where
    I: Iterator<Item = crate::Rcomplex>,
{
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I> AltComplexData for SparseIterComplexData<I>
where
    I: Iterator<Item = crate::Rcomplex>,
{
    fn elt(&self, i: usize) -> crate::Rcomplex {
        self.state.get_element(i).unwrap_or(crate::Rcomplex {
            r: f64::NAN,
            i: f64::NAN,
        })
    }

    fn as_slice(&self) -> Option<&[crate::Rcomplex]> {
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [crate::Rcomplex]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}
