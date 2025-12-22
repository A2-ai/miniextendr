//! High-level ALTREP data traits.
//!
//! These traits let you implement ALTREP behavior using `&self` methods instead of
//! raw `SEXP` callbacks. The library provides blanket implementations that handle
//! the SEXP extraction automatically.
//!
//! ## Quick Start
//!
//! For common types, just use them directly:
//!
//! ```ignore
//! // Vec<i32> already implements AltIntegerData
//! let altrep = create_altinteger(vec![1, 2, 3, 4, 5]);
//! ```
//!
//! For custom types, implement the relevant trait:
//!
//! ```ignore
//! struct Fibonacci { len: usize }
//!
//! impl AltrepLen for Fibonacci {
//!     fn len(&self) -> usize { self.len }
//! }
//!
//! impl AltIntegerData for Fibonacci {
//!     fn elt(&self, i: usize) -> i32 {
//!         // Compute fibonacci(i)
//!         ...
//!     }
//! }
//! ```

use crate::ffi::{Rcomplex, SEXP};

// =============================================================================
// Core trait: length
// =============================================================================

/// Base trait for ALTREP data types. All ALTREP types must provide length.
pub trait AltrepLen {
    /// Returns the length of this ALTREP vector.
    fn len(&self) -> usize;

    /// Returns true if the vector is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// =============================================================================
// Integer ALTREP
// =============================================================================

/// Trait for types that can back an ALTINTEGER vector.
///
/// Implement this to create custom integer ALTREP classes.
pub trait AltIntegerData: AltrepLen {
    /// Get the integer element at index `i`.
    fn elt(&self, i: usize) -> i32;

    /// Optional: return a pointer to contiguous data if available.
    /// Default returns None (no contiguous backing).
    fn as_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer. Returns number of elements read.
    ///
    /// # Safety Contract
    ///
    /// R guarantees that `buf.len() >= len`. Implementations must validate
    /// `start` and `len` against `self.len()` to prevent out-of-bounds access.
    /// The default implementation safely clamps to available data.
    ///
    /// # Default Implementation
    ///
    /// Uses `elt()` in a loop with bounds checking.
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }

    /// Optional: sortedness hint. Default is unknown.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum. Default returns None (use R's default).
    fn sum(&self, _na_rm: bool) -> Option<i64> {
        None
    }

    /// Optional: optimized min. Default returns None (use R's default).
    fn min(&self, _na_rm: bool) -> Option<i32> {
        None
    }

    /// Optional: optimized max. Default returns None (use R's default).
    fn max(&self, _na_rm: bool) -> Option<i32> {
        None
    }
}

// =============================================================================
// Real ALTREP
// =============================================================================

/// Trait for types that can back an ALTREAL vector.
pub trait AltRealData: AltrepLen {
    /// Get the real element at index `i`.
    fn elt(&self, i: usize) -> f64;

    /// Optional: return a pointer to contiguous data if available.
    fn as_slice(&self) -> Option<&[f64]> {
        None
    }

    /// Optional: bulk read into buffer.
    ///
    /// # Safety Contract
    ///
    /// R guarantees that `buf.len() >= len`. Implementations must validate bounds.
    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum.
    fn sum(&self, _na_rm: bool) -> Option<f64> {
        None
    }

    /// Optional: optimized min.
    fn min(&self, _na_rm: bool) -> Option<f64> {
        None
    }

    /// Optional: optimized max.
    fn max(&self, _na_rm: bool) -> Option<f64> {
        None
    }
}

// =============================================================================
// Logical ALTREP
// =============================================================================

/// Logical value: TRUE, FALSE, or NA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logical {
    False,
    True,
    Na,
}

impl Logical {
    /// Convert to R's integer representation.
    #[inline]
    pub fn to_r_int(self) -> i32 {
        self.into()
    }

    /// Convert from R's integer representation.
    #[inline]
    pub fn from_r_int(i: i32) -> Self {
        i.into()
    }

    /// Convert from Rust bool (no NA representation).
    #[inline]
    pub fn from_bool(b: bool) -> Self {
        b.into()
    }
}

/// Convert Logical to R's integer representation.
impl From<Logical> for i32 {
    fn from(logical: Logical) -> i32 {
        match logical {
            Logical::False => 0,
            Logical::True => 1,
            Logical::Na => i32::MIN,
        }
    }
}

/// Convert from R's integer representation to Logical.
impl From<i32> for Logical {
    fn from(i: i32) -> Self {
        match i {
            0 => Logical::False,
            i32::MIN => Logical::Na,
            _ => Logical::True,
        }
    }
}

/// Convert from Rust bool to Logical (no NA representation).
impl From<bool> for Logical {
    fn from(b: bool) -> Self {
        if b { Logical::True } else { Logical::False }
    }
}

/// Trait for types that can back an ALTLOGICAL vector.
pub trait AltLogicalData: AltrepLen {
    /// Get the logical element at index `i`.
    fn elt(&self, i: usize) -> Logical;

    /// Optional: return a slice if data is contiguous i32 (R's internal format).
    fn as_r_slice(&self) -> Option<&[i32]> {
        None
    }

    /// Optional: bulk read into buffer.
    ///
    /// # Safety Contract
    ///
    /// R guarantees that `buf.len() >= len`. Implementations must validate bounds.
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i).to_r_int();
        }
        actual_len
    }

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }

    /// Optional: optimized sum (count of TRUE values).
    fn sum(&self, _na_rm: bool) -> Option<i64> {
        None
    }
    // Note: R's ALTREP API does not expose min/max for logical vectors
}

// =============================================================================
// Raw ALTREP
// =============================================================================

/// Trait for types that can back an ALTRAW vector.
pub trait AltRawData: AltrepLen {
    /// Get the raw byte at index `i`.
    fn elt(&self, i: usize) -> u8;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[u8]> {
        None
    }

    /// Optional: bulk read into buffer.
    ///
    /// # Safety Contract
    ///
    /// R guarantees that `buf.len() >= len`. Implementations must validate bounds.
    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }
}

// =============================================================================
// Complex ALTREP
// =============================================================================

/// Trait for types that can back an ALTCOMPLEX vector.
pub trait AltComplexData: AltrepLen {
    /// Get the complex element at index `i`.
    fn elt(&self, i: usize) -> Rcomplex;

    /// Optional: return a slice if data is contiguous.
    fn as_slice(&self) -> Option<&[Rcomplex]> {
        None
    }

    /// Optional: bulk read into buffer.
    ///
    /// # Safety Contract
    ///
    /// R guarantees that `buf.len() >= len`. Implementations must validate bounds.
    fn get_region(&self, start: usize, len: usize, buf: &mut [Rcomplex]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, slot) in buf.iter_mut().enumerate().take(actual_len) {
            *slot = self.elt(start + i);
        }
        actual_len
    }
}

// =============================================================================
// String ALTREP
// =============================================================================

/// Trait for types that can back an ALTSTRING vector.
///
/// Note: `elt` returns a `&str` which will be converted to CHARSXP.
pub trait AltStringData: AltrepLen {
    /// Get the string element at index `i`.
    ///
    /// Return `None` for NA values.
    fn elt(&self, i: usize) -> Option<&str>;

    /// Optional: sortedness hint.
    fn is_sorted(&self) -> Option<Sortedness> {
        None
    }

    /// Optional: does this vector contain any NA values?
    fn no_na(&self) -> Option<bool> {
        None
    }
}

// =============================================================================
// List ALTREP
// =============================================================================

/// Trait for types that can back an ALTLIST vector.
///
/// List elements are arbitrary SEXPs, so this trait works with raw SEXP.
pub trait AltListData: AltrepLen {
    /// Get the list element at index `i`.
    ///
    /// Returns a SEXP (any R object).
    fn elt(&self, i: usize) -> SEXP;
}

// =============================================================================
// Iterator-backed ALTREP infrastructure
// =============================================================================

use std::cell::RefCell;
use std::sync::OnceLock;

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
    /// - `len`: The expected number of elements (must match iterator length)
    ///
    /// # Panics
    ///
    /// Panics if the iterator produces more or fewer elements than `len`.
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
    /// - `Some(T)` if element exists
    /// - `None` if index is out of bounds or iterator exhausted early
    ///
    /// # Panics
    ///
    /// May panic if iterator produces more elements than declared length.
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
        let iter = iter_opt.as_mut()?;

        // Fill cache up to and including index i
        while cache.len() <= i {
            if let Some(elem) = iter.next() {
                cache.push(elem);
            } else {
                // Iterator exhausted before reaching expected length
                return None;
            }
        }

        Some(cache[i])
    }

    /// Materialize all remaining elements from the iterator.
    ///
    /// After this call, all elements are guaranteed to be in memory and
    /// `as_materialized()` will return `Some`.
    ///
    /// # Panics
    ///
    /// Panics if iterator produces more elements than declared length.
    pub fn materialize_all(&self) -> &[T] {
        // Already materialized?
        if let Some(vec) = self.materialized.get() {
            return vec;
        }

        // Consume iterator and move cache to materialized storage
        let mut cache = self.cache.borrow_mut();
        let mut iter_opt = self.iter.borrow_mut();

        if let Some(iter) = iter_opt.take() {
            // Drain remaining elements
            cache.extend(iter);

            // Verify length matches
            assert_eq!(
                cache.len(),
                self.len,
                "iterator produced {} elements, expected {}",
                cache.len(),
                self.len
            );
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

/// Iterator-backed integer vector data.
///
/// Wraps an iterator producing `i32` values and exposes it as an ALTREP integer vector.
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
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::externalptr::TypedExternal for IterIntData<I> {
    const TYPE_NAME: &'static str = "IterIntData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterIntData\0";
}

impl<I: Iterator<Item = i32> + 'static> InferBase for IterIntData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Int;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altinteger_class(
                class_name,
                pkg_name,
                core::ptr::null_mut(),
            )
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_int::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::Altrep for IterIntData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltVec for IterIntData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltInteger for IterIntData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltIntegerData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Iterator-backed real (f64) vector data.
///
/// Wraps an iterator producing `f64` values and exposes it as an ALTREP real vector.
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
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::externalptr::TypedExternal for IterRealData<I> {
    const TYPE_NAME: &'static str = "IterRealData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterRealData\0";
}

impl<I: Iterator<Item = f64> + 'static> InferBase for IterRealData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Real;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altreal_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_real::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::Altrep for IterRealData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltVec for IterRealData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltReal for IterRealData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltRealData::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut f64,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltRealData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Iterator-backed logical vector data.
///
/// Wraps an iterator producing `bool` values and exposes it as an ALTREP logical vector.
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
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i).to_r_int();
        }
        actual_len
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::externalptr::TypedExternal for IterLogicalData<I> {
    const TYPE_NAME: &'static str = "IterLogicalData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterLogicalData\0";
}

impl<I: Iterator<Item = bool> + 'static> InferBase for IterLogicalData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Logical;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altlogical_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_lgl::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::Altrep for IterLogicalData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltVec for IterLogicalData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltLogical for IterLogicalData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltLogicalData::elt(&*d, i as usize).to_r_int())
            .unwrap_or(crate::altrep_traits::NA_LOGICAL)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltLogicalData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Iterator-backed raw (u8) vector data.
///
/// Wraps an iterator producing `u8` values and exposes it as an ALTREP raw vector.
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
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I: Iterator<Item = u8> + 'static> crate::externalptr::TypedExternal for IterRawData<I> {
    const TYPE_NAME: &'static str = "IterRawData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterRawData\0";
}

impl<I: Iterator<Item = u8> + 'static> InferBase for IterRawData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Raw;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altraw_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_raw::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::Altrep for IterRawData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::AltVec for IterRawData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::AltRaw for IterRawData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> u8 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltRawData::elt(&*d, i as usize))
            .unwrap_or(0)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut u8,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltRawData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

// =============================================================================
// Iterator-backed ALTREP with Coerce support
// =============================================================================

/// Iterator-backed integer vector with coercion from any integer-like type.
///
/// Wraps an iterator producing values that coerce to `i32` (e.g., `u16`, `i8`, etc.).
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
    fn len(&self) -> usize {
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

    fn as_slice(&self) -> Option<&[i32]> {
        // Can't return slice of i32 when cached values are type T
        // Would need a separate coerced cache
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I, T> crate::externalptr::TypedExternal for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<i32> + Copy + 'static,
{
    const TYPE_NAME: &'static str = "IterIntCoerceData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterIntCoerceData\0";
}

impl<I, T> InferBase for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<i32> + Copy + 'static,
{
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Int;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altinteger_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_int::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I, T> crate::altrep_traits::Altrep for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<i32> + Copy + 'static,
{
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I, T> crate::altrep_traits::AltVec for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<i32> + Copy + 'static,
{
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I, T> crate::altrep_traits::AltInteger for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<i32> + Copy + 'static,
{
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltIntegerData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Iterator-backed real vector with coercion from any float-like type.
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
    fn len(&self) -> usize {
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
            .unwrap_or(f64::NAN)
    }

    fn as_slice(&self) -> Option<&[f64]> {
        // Can't return slice of f64 when cached values are type T
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I, T> crate::externalptr::TypedExternal for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<f64> + Copy + 'static,
{
    const TYPE_NAME: &'static str = "IterRealCoerceData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterRealCoerceData\0";
}

impl<I, T> InferBase for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<f64> + Copy + 'static,
{
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Real;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altreal_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_real::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I, T> crate::altrep_traits::Altrep for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<f64> + Copy + 'static,
{
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I, T> crate::altrep_traits::AltVec for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<f64> + Copy + 'static,
{
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I, T> crate::altrep_traits::AltReal for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<f64> + Copy + 'static,
{
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltRealData::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut f64,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltRealData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Iterator-backed integer vector with coercion from bool.
///
/// Wraps an iterator producing `bool` values that coerce to `i32`.
/// Useful for converting boolean iterators to integer vectors.
pub struct IterIntFromBoolData<I>
where
    I: Iterator<Item = bool>,
{
    state: IterState<I, bool>,
}

impl<I> IterIntFromBoolData<I>
where
    I: Iterator<Item = bool>,
{
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I> IterIntFromBoolData<I>
where
    I: ExactSizeIterator<Item = bool>,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I> AltrepLen for IterIntFromBoolData<I>
where
    I: Iterator<Item = bool>,
{
    fn len(&self) -> usize {
        self.state.len()
    }
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

    fn as_slice(&self) -> Option<&[i32]> {
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::externalptr::TypedExternal for IterIntFromBoolData<I> {
    const TYPE_NAME: &'static str = "IterIntFromBoolData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterIntFromBoolData\0";
}

impl<I: Iterator<Item = bool> + 'static> InferBase for IterIntFromBoolData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Int;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altinteger_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_int::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::Altrep for IterIntFromBoolData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltVec for IterIntFromBoolData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltInteger for IterIntFromBoolData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut i32,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltIntegerData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Iterator-backed string vector.
///
/// Wraps an iterator producing `String` values and exposes it as an ALTREP character vector.
///
/// # Note
///
/// String elements must be materialized and stored to satisfy the `&str` borrow
/// requirement of `AltStringData::elt()`. This means strings are allocated eagerly
/// as they're accessed and kept in memory.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterStringData;
///
/// let iter = (0..5).map(|x| format!("item_{}", x));
/// let data = IterStringData::from_iter(iter, 5);
/// ```
pub struct IterStringData<I>
where
    I: Iterator<Item = String>,
{
    state: IterState<I, String>,
}

impl<I> IterStringData<I>
where
    I: Iterator<Item = String>,
{
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I> IterStringData<I>
where
    I: ExactSizeIterator<Item = String>,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I> AltrepLen for IterStringData<I>
where
    I: Iterator<Item = String>,
{
    fn len(&self) -> usize {
        self.state.len()
    }
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

impl<I: Iterator<Item = String> + 'static> crate::externalptr::TypedExternal for IterStringData<I> {
    const TYPE_NAME: &'static str = "IterStringData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterStringData\0";
}

impl<I: Iterator<Item = String> + 'static> InferBase for IterStringData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::String;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altstring_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_str::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = String> + 'static> crate::altrep_traits::Altrep for IterStringData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = String> + 'static> crate::altrep_traits::AltVec for IterStringData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = String> + 'static> crate::altrep_traits::AltString for IterStringData<I> {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .and_then(|d| {
                AltStringData::elt(&*d, i as usize).map(|s| unsafe {
                    crate::ffi::Rf_mkCharLenCE(
                        s.as_ptr().cast(),
                        s.len() as i32,
                        crate::ffi::cetype_t::CE_UTF8,
                    )
                })
            })
            .unwrap_or(unsafe { crate::ffi::R_NaString })
    }
}

/// Iterator-backed list vector.
///
/// Wraps an iterator producing R `SEXP` values and exposes it as an ALTREP list.
///
/// # Safety
///
/// The iterator must produce valid, protected SEXP values. Each SEXP must remain
/// protected for the lifetime of the ALTREP object.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterListData;
/// use miniextendr_api::IntoR;
///
/// let iter = (0..5).map(|x| vec![x, x+1, x+2].into_sexp());
/// let data = IterListData::from_iter(iter, 5);
/// ```
pub struct IterListData<I>
where
    I: Iterator<Item = SEXP>,
{
    state: IterState<I, SEXP>,
}

impl<I> IterListData<I>
where
    I: Iterator<Item = SEXP>,
{
    /// Create from an iterator with explicit length.
    ///
    /// # Safety
    ///
    /// The iterator must produce valid, protected SEXP values.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I> IterListData<I>
where
    I: ExactSizeIterator<Item = SEXP>,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    ///
    /// # Safety
    ///
    /// The iterator must produce valid, protected SEXP values.
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I> AltrepLen for IterListData<I>
where
    I: Iterator<Item = SEXP>,
{
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I> AltListData for IterListData<I>
where
    I: Iterator<Item = SEXP>,
{
    fn elt(&self, i: usize) -> SEXP {
        use crate::ffi::R_NilValue;
        self.state.get_element(i).unwrap_or(unsafe { R_NilValue })
    }
}

impl<I: Iterator<Item = SEXP> + 'static> crate::externalptr::TypedExternal for IterListData<I> {
    const TYPE_NAME: &'static str = "IterListData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterListData\0";
}

impl<I: Iterator<Item = SEXP> + 'static> InferBase for IterListData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::List;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altlist_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_list::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = SEXP> + 'static> crate::altrep_traits::Altrep for IterListData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = SEXP> + 'static> crate::altrep_traits::AltVec for IterListData<I> {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = SEXP> + 'static> crate::altrep_traits::AltList for IterListData<I> {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltListData::elt(&*d, i as usize))
            .unwrap_or(unsafe { crate::ffi::R_NilValue })
    }
}

/// Iterator-backed complex number vector.
///
/// Wraps an iterator producing `Rcomplex` values and exposes it as an ALTREP complex vector.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::altrep_data::IterComplexData;
/// use miniextendr_api::ffi::Rcomplex;
///
/// let iter = (0..5).map(|x| Rcomplex { r: x as f64, i: (x * 2) as f64 });
/// let data = IterComplexData::from_iter(iter, 5);
/// ```
pub struct IterComplexData<I>
where
    I: Iterator<Item = crate::ffi::Rcomplex>,
{
    state: IterState<I, crate::ffi::Rcomplex>,
}

impl<I> IterComplexData<I>
where
    I: Iterator<Item = crate::ffi::Rcomplex>,
{
    /// Create from an iterator with explicit length.
    pub fn from_iter(iter: I, len: usize) -> Self {
        Self {
            state: IterState::new(iter, len),
        }
    }
}

impl<I> IterComplexData<I>
where
    I: ExactSizeIterator<Item = crate::ffi::Rcomplex>,
{
    /// Create from an ExactSizeIterator (length auto-detected).
    pub fn from_exact_iter(iter: I) -> Self {
        Self {
            state: IterState::from_exact_size(iter),
        }
    }
}

impl<I> AltrepLen for IterComplexData<I>
where
    I: Iterator<Item = crate::ffi::Rcomplex>,
{
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I> AltComplexData for IterComplexData<I>
where
    I: Iterator<Item = crate::ffi::Rcomplex>,
{
    fn elt(&self, i: usize) -> crate::ffi::Rcomplex {
        self.state.get_element(i).unwrap_or(crate::ffi::Rcomplex {
            r: f64::NAN,
            i: f64::NAN,
        })
    }

    fn as_slice(&self) -> Option<&[crate::ffi::Rcomplex]> {
        self.state.as_materialized()
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [crate::ffi::Rcomplex]) -> usize {
        let actual_len = len.min(buf.len()).min(self.len().saturating_sub(start));
        for (i, buf_i) in buf.iter_mut().enumerate().take(actual_len) {
            *buf_i = self.elt(start + i);
        }
        actual_len
    }
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::externalptr::TypedExternal
    for IterComplexData<I>
{
    const TYPE_NAME: &'static str = "IterComplexData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterComplexData\0";
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> InferBase for IterComplexData<I> {
    const BASE: crate::altrep::RBase = crate::altrep::RBase::Complex;

    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t {
        unsafe {
            crate::ffi::altrep::R_make_altcomplex_class(class_name, pkg_name, core::ptr::null_mut())
        }
    }

    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t) {
        unsafe { crate::altrep_bridge::install_base::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_vec::<Self>(cls) };
        unsafe { crate::altrep_bridge::install_cplx::<Self>(cls) };
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::altrep_traits::Altrep
    for IterComplexData<I>
{
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::altrep_traits::AltVec
    for IterComplexData<I>
{
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::altrep_traits::AltComplex
    for IterComplexData<I>
{
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::Rcomplex {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltComplexData::elt(&*d, i as usize))
            .unwrap_or(crate::ffi::Rcomplex {
                r: f64::NAN,
                i: f64::NAN,
            })
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: *mut crate::ffi::Rcomplex,
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                let slice = unsafe { std::slice::from_raw_parts_mut(buf, len as usize) };
                AltComplexData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

// =============================================================================
// Sortedness enum
// =============================================================================

/// Sortedness hint for ALTREP vectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sortedness {
    /// Unknown sortedness.
    Unknown,
    /// Known to be unsorted.
    ///
    /// This corresponds to `KNOWN_UNSORTED` in R.
    KnownUnsorted,
    /// Sorted in increasing order (may have ties).
    Increasing,
    /// Sorted in decreasing order (may have ties).
    Decreasing,
    /// Sorted in increasing order, with NAs first.
    ///
    /// This corresponds to `SORTED_INCR_NA_1ST` in R.
    IncreasingNaFirst,
    /// Sorted in decreasing order, with NAs first.
    ///
    /// This corresponds to `SORTED_DECR_NA_1ST` in R.
    DecreasingNaFirst,
}

impl Sortedness {
    /// Convert to R's integer representation.
    #[inline]
    pub fn to_r_int(self) -> i32 {
        self.into()
    }

    /// Convert from R's integer representation.
    #[inline]
    pub fn from_r_int(i: i32) -> Self {
        i.into()
    }
}

/// Convert Sortedness to R's integer representation.
impl From<Sortedness> for i32 {
    fn from(s: Sortedness) -> i32 {
        match s {
            Sortedness::Unknown => i32::MIN,
            Sortedness::KnownUnsorted => 0,
            Sortedness::Increasing => 1,
            Sortedness::Decreasing => -1,
            Sortedness::IncreasingNaFirst => 2,
            Sortedness::DecreasingNaFirst => -2,
        }
    }
}

/// Convert from R's integer representation to Sortedness.
impl From<i32> for Sortedness {
    fn from(i: i32) -> Self {
        match i {
            i32::MIN => Sortedness::Unknown,
            0 => Sortedness::KnownUnsorted,
            1 => Sortedness::Increasing,
            -1 => Sortedness::Decreasing,
            2 => Sortedness::IncreasingNaFirst,
            -2 => Sortedness::DecreasingNaFirst,
            _ => Sortedness::Unknown,
        }
    }
}

// =============================================================================
// Optional dataptr trait (separate from element access)
// =============================================================================

/// Trait for types that can provide a mutable data pointer.
///
/// This is separate from element access because some ALTREP types
/// compute elements on-the-fly but can materialize to a buffer.
///
/// ## Lazy Materialization Pattern
///
/// For types that compute values lazily (e.g., arithmetic sequences, Fibonacci),
/// you can implement lazy materialization by:
///
/// 1. Store an `Option<Vec<T>>` for the materialized buffer
/// 2. In `dataptr()`, compute all values and cache them
/// 3. In `dataptr_or_null()`, return `None` until materialized
///
/// ```ignore
/// struct LazySequence {
///     start: i32,
///     step: i32,
///     len: usize,
///     materialized: Option<Vec<i32>>,
/// }
///
/// impl AltrepDataptr<i32> for LazySequence {
///     fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
///         // Materialize on first access
///         if self.materialized.is_none() {
///             let data: Vec<i32> = (0..self.len)
///                 .map(|i| self.start + (i as i32) * self.step)
///                 .collect();
///             self.materialized = Some(data);
///         }
///         self.materialized.as_mut().map(|v| v.as_mut_ptr())
///     }
///
///     fn dataptr_or_null(&self) -> Option<*const i32> {
///         // Only return pointer if already materialized
///         self.materialized.as_ref().map(|v| v.as_ptr())
///     }
/// }
/// ```
pub trait AltrepDataptr<T> {
    /// Get a mutable pointer to the underlying data.
    ///
    /// If `writable` is true, R may modify the data.
    /// Return `None` if data cannot be accessed as a contiguous buffer.
    ///
    /// This method may trigger materialization of lazy data.
    fn dataptr(&mut self, writable: bool) -> Option<*mut T>;

    /// Get a read-only pointer without forcing materialization.
    ///
    /// Return `None` if data is not already materialized or cannot provide
    /// a contiguous buffer. R will fall back to element-by-element access
    /// via `Elt` when this returns `None`.
    fn dataptr_or_null(&self) -> Option<*const T> {
        None
    }
}

// =============================================================================
// Serialization trait
// =============================================================================

/// Trait for ALTREP types that support serialization.
///
/// When an ALTREP object is saved (e.g., with `saveRDS()`), R calls `serialized_state`
/// to get a representation that can be saved. When loaded, R calls `unserialize`
/// to reconstruct the ALTREP object from that state.
///
/// ## How It Works
///
/// 1. **Saving**: R calls `serialized_state(x)` which should return an R object
///    (typically a list or vector) containing all data needed to reconstruct the ALTREP.
///
/// 2. **Loading**: R calls `unserialize(class, state)` where `state` is what
///    `serialized_state` returned. You reconstruct your ALTREP object from this.
///
/// ## Example
///
/// ```ignore
/// use miniextendr_api::ffi::{SEXP, Rf_allocVector, INTSXP, SET_INTEGER_ELT, INTEGER_ELT};
///
/// impl AltrepSerialize for ArithSeqData {
///     fn serialized_state(&self) -> SEXP {
///         // Store start, step, len in an integer vector
///         unsafe {
///             let state = Rf_allocVector(INTSXP, 3);
///             SET_INTEGER_ELT(state, 0, self.start);
///             SET_INTEGER_ELT(state, 1, self.step);
///             SET_INTEGER_ELT(state, 2, self.len as i32);
///             state
///         }
///     }
///
///     fn unserialize(state: SEXP) -> Option<Self> {
///         unsafe {
///             let start = INTEGER_ELT(state, 0);
///             let step = INTEGER_ELT(state, 1);
///             let len = INTEGER_ELT(state, 2) as usize;
///             Some(ArithSeqData { start, step, len })
///         }
///     }
/// }
/// ```
///
/// ## Notes
///
/// - The serialized state should be a standard R object (list, vector, etc.)
/// - Avoid storing pointers or handles that won't survive serialization
/// - For lazy types, decide whether to serialize the computed values or the parameters
pub trait AltrepSerialize: Sized {
    /// Convert the ALTREP data to a serializable R object.
    ///
    /// This is called when R needs to save the ALTREP (e.g., `saveRDS()`).
    /// Return an R object that contains all information needed to reconstruct
    /// the ALTREP on load.
    fn serialized_state(&self) -> SEXP;

    /// Reconstruct the ALTREP data from a serialized state.
    ///
    /// This is called when R loads a serialized ALTREP (e.g., `readRDS()`).
    /// The `state` parameter is what `serialized_state()` returned.
    ///
    /// Return `None` if the state is invalid or cannot be deserialized.
    fn unserialize(state: SEXP) -> Option<Self>;
}

// =============================================================================
// Extract_subset optimization trait
// =============================================================================

/// Trait for ALTREP types that can provide optimized subsetting.
///
/// When R subsets an ALTREP (e.g., `x[1:10]`), it can call `Extract_subset` to get
/// an optimized result. This is useful for:
///
/// - **Arithmetic sequences**: `seq(1, 1000000)[1:10]` can return a new sequence
///   instead of materializing the full million elements
/// - **Lazy types**: Can return another lazy object covering just the subset
/// - **Memory-mapped files**: Can return a view without loading everything
///
/// ## Example
///
/// ```ignore
/// impl AltrepExtractSubset for ArithSeqData {
///     fn extract_subset(&self, indices: &[i32]) -> Option<SEXP> {
///         // For simple contiguous subsets like 1:10, we could return a new ArithSeq
///         // For general subsets, return None to let R handle it
///         None
///     }
/// }
/// ```
///
/// ## Notes
///
/// - `indices` contains 1-based R indices (may include NA as i32::MIN)
/// - Return `None` to let R use default subsetting
/// - Return `Some(sexp)` with the subset result
pub trait AltrepExtractSubset {
    /// Extract a subset of this ALTREP.
    ///
    /// `indices` contains the 1-based indices to extract.
    /// Return `None` to fall back to R's default subsetting.
    fn extract_subset(&self, indices: &[i32]) -> Option<SEXP>;
}

// =============================================================================
// InferBase trait - automatic base type inference from data traits
// =============================================================================

/// Trait for inferring the R base type from a data type's implemented traits.
///
/// This is automatically implemented via blanket impls for types that implement
/// one of the `Alt*Data` traits. It allows the `#[miniextendr]` macro to infer
/// the base type without requiring an explicit `base = "..."` attribute.
///
/// # Example
///
/// ```ignore
/// // ConstantIntData implements AltIntegerData, so InferBase is auto-implemented
/// impl AltIntegerData for ConstantIntData { ... }
///
/// // Now the macro can infer the base type:
/// #[miniextendr(class = "ConstantInt", pkg = "rpkg")]  // No base needed!
/// pub struct ConstantIntClass(ConstantIntData);
/// ```
pub trait InferBase {
    /// The inferred R base type.
    const BASE: crate::altrep::RBase;

    /// Create the ALTREP class handle.
    ///
    /// # Safety
    /// Must be called during R initialization.
    unsafe fn make_class(
        class_name: *const i8,
        pkg_name: *const i8,
    ) -> crate::ffi::altrep::R_altrep_class_t;

    /// Install ALTREP methods on the class.
    ///
    /// # Safety
    /// Must be called during R initialization with a valid class handle.
    unsafe fn install_methods(cls: crate::ffi::altrep::R_altrep_class_t);
}

/// Implement `InferBase` for an integer ALTREP data type.
///
/// This macro should be called after `impl_altinteger_from_data!` to enable
/// automatic base type inference in the `#[miniextendr]` macro.
#[macro_export]
macro_rules! impl_inferbase_integer {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Int;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altinteger_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_int::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a real ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_real {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Real;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altreal_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_real::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a logical ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_logical {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Logical;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altlogical_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_lgl::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a raw ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_raw {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Raw;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altraw_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_raw::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a string ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_string {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::String;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altstring_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_str::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a complex ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_complex {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::Complex;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altcomplex_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_cplx::<$ty>(cls) };
            }
        }
    };
}

/// Implement `InferBase` for a list ALTREP data type.
#[macro_export]
macro_rules! impl_inferbase_list {
    ($ty:ty) => {
        impl $crate::altrep_data::InferBase for $ty {
            const BASE: $crate::altrep::RBase = $crate::altrep::RBase::List;

            unsafe fn make_class(
                class_name: *const i8,
                pkg_name: *const i8,
            ) -> $crate::ffi::altrep::R_altrep_class_t {
                unsafe {
                    $crate::ffi::altrep::R_make_altlist_class(
                        class_name,
                        pkg_name,
                        core::ptr::null_mut(),
                    )
                }
            }

            unsafe fn install_methods(cls: $crate::ffi::altrep::R_altrep_class_t) {
                unsafe { $crate::altrep_bridge::install_base::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_vec::<$ty>(cls) };
                unsafe { $crate::altrep_bridge::install_list::<$ty>(cls) };
            }
        }
    };
}

// =============================================================================
// Built-in implementations for Vec<T>
// =============================================================================

impl AltrepLen for Vec<i32> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltIntegerData for Vec<i32> {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.contains(&i32::MIN))
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let mut sum: i64 = 0;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None; // NA propagates
                }
            } else {
                sum += x as i64;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        let mut min = i32::MAX;
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found { Some(min) } else { None }
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        let mut max = i32::MIN + 1; // Avoid NA sentinel
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found { Some(max) } else { None }
    }
}

impl AltrepDataptr<i32> for Vec<i32> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Vec<f64> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltRealData for Vec<f64> {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }

    fn sum(&self, na_rm: bool) -> Option<f64> {
        let mut sum = 0.0;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                sum += x;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<f64> {
        let mut min = f64::INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found { Some(min) } else { None }
    }

    fn max(&self, na_rm: bool) -> Option<f64> {
        let mut max = f64::NEG_INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found { Some(max) } else { None }
    }
}

impl AltrepDataptr<f64> for Vec<f64> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Vec<u8> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltRawData for Vec<u8> {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let end = (start + len).min(self.len());
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl AltrepDataptr<u8> for Vec<u8> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Vec<String> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltStringData for Vec<String> {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // String vectors don't have NA
    }
}

impl AltrepLen for Vec<Option<String>> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltStringData for Vec<Option<String>> {
    fn elt(&self, i: usize) -> Option<&str> {
        self[i].as_deref()
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_none()))
    }
}

impl AltrepLen for Vec<bool> {
    fn len(&self) -> usize {
        Vec::len(self)
    }
}

impl AltLogicalData for Vec<bool> {
    fn elt(&self, i: usize) -> Logical {
        if self[i] {
            Logical::True
        } else {
            Logical::False
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool can't be NA
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        Some(self.iter().filter(|&&x| x).count() as i64)
    }
}

// =============================================================================
// Built-in implementations for Box<[T]> (owned slices)
// =============================================================================
// Box<[T]> is a fat pointer (Sized) that wraps a DST slice.
// Unlike Vec<T>, it has no capacity field - just ptr + len (2 words).
// This makes it more memory-efficient for fixed-size data.
//
// Box<[T]> CAN be used directly with ALTREP via the proc-macro:
// ```
// #[miniextendr(class = "BoxedInts", pkg = "mypkg")]
// pub struct BoxedIntsClass(Box<[i32]>);
// ```
//
// Or use these trait implementations in custom wrapper structs.

impl AltrepLen for Box<[i32]> {
    fn len(&self) -> usize {
        <[i32]>::len(self)
    }
}

impl AltIntegerData for Box<[i32]> {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(<[i32]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.contains(&i32::MIN))
    }

    fn sum(&self, na_rm: bool) -> Option<i64> {
        let mut sum: i64 = 0;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                sum += x as i64;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<i32> {
        let mut min = i32::MAX;
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found { Some(min) } else { None }
    }

    fn max(&self, na_rm: bool) -> Option<i32> {
        let mut max = i32::MIN + 1; // i32::MIN is NA
        let mut found = false;
        for &x in self.iter() {
            if x == i32::MIN {
                if !na_rm {
                    return None;
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found { Some(max) } else { None }
    }
}

impl AltrepDataptr<i32> for Box<[i32]> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut i32> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Box<[f64]> {
    fn len(&self) -> usize {
        <[f64]>::len(self)
    }
}

impl AltRealData for Box<[f64]> {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let end = (start + len).min(<[f64]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }

    fn sum(&self, na_rm: bool) -> Option<f64> {
        let mut sum = 0.0;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                sum += x;
            }
        }
        Some(sum)
    }

    fn min(&self, na_rm: bool) -> Option<f64> {
        let mut min = f64::INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                found = true;
                min = min.min(x);
            }
        }
        if found { Some(min) } else { None }
    }

    fn max(&self, na_rm: bool) -> Option<f64> {
        let mut max = f64::NEG_INFINITY;
        let mut found = false;
        for &x in self.iter() {
            if x.is_nan() {
                if !na_rm {
                    return Some(f64::NAN);
                }
            } else {
                found = true;
                max = max.max(x);
            }
        }
        if found { Some(max) } else { None }
    }
}

impl AltrepDataptr<f64> for Box<[f64]> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut f64> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const f64> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Box<[u8]> {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl AltRawData for Box<[u8]> {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let end = (start + len).min(<[u8]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl AltrepDataptr<u8> for Box<[u8]> {
    fn dataptr(&mut self, _writable: bool) -> Option<*mut u8> {
        Some(self.as_mut_ptr())
    }

    fn dataptr_or_null(&self) -> Option<*const u8> {
        Some(self.as_ptr())
    }
}

impl AltrepLen for Box<[bool]> {
    fn len(&self) -> usize {
        <[bool]>::len(self)
    }
}

impl AltLogicalData for Box<[bool]> {
    fn elt(&self, i: usize) -> Logical {
        if self[i] {
            Logical::True
        } else {
            Logical::False
        }
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool can't be NA
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        Some(self.iter().filter(|&&x| x).count() as i64)
    }
}

impl AltrepLen for Box<[String]> {
    fn len(&self) -> usize {
        <[String]>::len(self)
    }
}

impl AltStringData for Box<[String]> {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // String can't be NA
    }
}

// =============================================================================
// Built-in implementations for Range types
// =============================================================================

use std::ops::Range;

impl AltrepLen for Range<i32> {
    fn len(&self) -> usize {
        if self.end > self.start {
            (self.end - self.start) as usize
        } else {
            0
        }
    }
}

impl AltIntegerData for Range<i32> {
    fn elt(&self, i: usize) -> i32 {
        self.start + i as i32
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::Increasing)
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }
        // Sum of arithmetic sequence: n/2 * (first + last)
        let first = self.start as i64;
        let last = (self.end - 1) as i64;
        Some(n * (first + last) / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            Some(self.start)
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            Some(self.end - 1)
        } else {
            None
        }
    }
}

impl AltrepLen for Range<i64> {
    fn len(&self) -> usize {
        if self.end > self.start {
            (self.end - self.start) as usize
        } else {
            0
        }
    }
}

impl AltIntegerData for Range<i64> {
    fn elt(&self, i: usize) -> i32 {
        let val = self.start.saturating_add(i as i64);
        // Bounds check: return NA_INTEGER for values outside i32 range
        if val > i32::MAX as i64 || val < i32::MIN as i64 {
            crate::altrep_traits::NA_INTEGER
        } else {
            val as i32
        }
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::Increasing)
    }

    fn no_na(&self) -> Option<bool> {
        // May contain NA if range exceeds i32 bounds
        let start_ok = self.start >= i32::MIN as i64 && self.start <= i32::MAX as i64;
        let end_ok = self.end >= i32::MIN as i64 && self.end <= i32::MAX as i64 + 1;
        Some(start_ok && end_ok)
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        let n = AltrepLen::len(self) as i64;
        if n == 0 {
            return Some(0);
        }
        let first = self.start;
        let last = self.end - 1;

        // Use checked arithmetic to detect overflow
        // Formula: n * (first + last) / 2
        let sum_endpoints = first.checked_add(last)?;
        let product = n.checked_mul(sum_endpoints)?;
        Some(product / 2)
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            let val = self.start;
            if val > i32::MAX as i64 || val < i32::MIN as i64 {
                None // Out of range, let R compute
            } else {
                Some(val as i32)
            }
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if AltrepLen::len(self) > 0 {
            let val = self.end - 1;
            if val > i32::MAX as i64 || val < i32::MIN as i64 {
                None // Out of range, let R compute
            } else {
                Some(val as i32)
            }
        } else {
            None
        }
    }
}

impl AltrepLen for Range<f64> {
    fn len(&self) -> usize {
        // For f64 ranges, assume step of 1.0
        if self.end > self.start {
            (self.end - self.start).ceil() as usize
        } else {
            0
        }
    }
}

impl AltRealData for Range<f64> {
    fn elt(&self, i: usize) -> f64 {
        self.start + i as f64
    }

    fn is_sorted(&self) -> Option<Sortedness> {
        Some(Sortedness::Increasing)
    }

    fn no_na(&self) -> Option<bool> {
        Some(true)
    }

    fn sum(&self, _na_rm: bool) -> Option<f64> {
        let n = AltrepLen::len(self) as f64;
        if n == 0.0 {
            return Some(0.0);
        }
        let first = self.start;
        let last = self.start + (n - 1.0);
        Some(n * (first + last) / 2.0)
    }

    fn min(&self, _na_rm: bool) -> Option<f64> {
        if AltrepLen::len(self) > 0 {
            Some(self.start)
        } else {
            None
        }
    }

    fn max(&self, _na_rm: bool) -> Option<f64> {
        if AltrepLen::len(self) > 0 {
            Some(self.start + (AltrepLen::len(self) - 1) as f64)
        } else {
            None
        }
    }
}

// =============================================================================
// Built-in implementations for slices (read-only)
// =============================================================================

impl AltrepLen for &[i32] {
    fn len(&self) -> usize {
        <[i32]>::len(self)
    }
}

impl AltIntegerData for &[i32] {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self)
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(<[i32]>::len(self));
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        // i32 slices have NA as i32::MIN
        Some(!self.contains(&i32::MIN))
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        // Check for NA (i32::MIN)
        if self.contains(&i32::MIN) {
            if _na_rm {
                Some(
                    self.iter()
                        .filter(|&&x| x != i32::MIN)
                        .map(|&x| x as i64)
                        .sum(),
                )
            } else {
                None // Return NA
            }
        } else {
            Some(self.iter().map(|&x| x as i64).sum())
        }
    }

    fn min(&self, _na_rm: bool) -> Option<i32> {
        if self.is_empty() {
            return None;
        }
        if _na_rm {
            self.iter().filter(|&&x| x != i32::MIN).copied().min()
        } else if self.contains(&i32::MIN) {
            None // NA present
        } else {
            self.iter().copied().min()
        }
    }

    fn max(&self, _na_rm: bool) -> Option<i32> {
        if self.is_empty() {
            return None;
        }
        if _na_rm {
            self.iter().filter(|&&x| x != i32::MIN).copied().max()
        } else if self.contains(&i32::MIN) {
            None // NA present
        } else {
            self.iter().copied().max()
        }
    }
}

impl AltrepLen for &[f64] {
    fn len(&self) -> usize {
        <[f64]>::len(self)
    }
}

impl AltRealData for &[f64] {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self)
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }

    fn sum(&self, na_rm: bool) -> Option<f64> {
        if na_rm {
            Some(self.iter().filter(|x| !x.is_nan()).sum())
        } else if self.iter().any(|x| x.is_nan()) {
            None // Return NA
        } else {
            Some(self.iter().sum())
        }
    }

    fn min(&self, na_rm: bool) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        if na_rm {
            self.iter()
                .filter(|x| !x.is_nan())
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        } else if self.iter().any(|x| x.is_nan()) {
            None
        } else {
            self.iter()
                .copied()
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        }
    }

    fn max(&self, na_rm: bool) -> Option<f64> {
        if self.is_empty() {
            return None;
        }
        if na_rm {
            self.iter()
                .filter(|x| !x.is_nan())
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        } else if self.iter().any(|x| x.is_nan()) {
            None
        } else {
            self.iter()
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        }
    }
}

impl AltrepLen for &[u8] {
    fn len(&self) -> usize {
        <[u8]>::len(self)
    }
}

impl AltRawData for &[u8] {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self)
    }
}

impl AltrepLen for &[bool] {
    fn len(&self) -> usize {
        <[bool]>::len(self)
    }
}

impl AltLogicalData for &[bool] {
    fn elt(&self, i: usize) -> Logical {
        Logical::from_bool(self[i])
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool can't be NA
    }

    fn sum(&self, _na_rm: bool) -> Option<i64> {
        Some(self.iter().filter(|&&x| x).count() as i64)
    }
}

impl AltrepLen for &[String] {
    fn len(&self) -> usize {
        <[String]>::len(self)
    }
}

impl AltStringData for &[String] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }
}

impl AltrepLen for &[&str] {
    fn len(&self) -> usize {
        <[&str]>::len(self)
    }
}

impl AltStringData for &[&str] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i])
    }
}

// =============================================================================
// NOTE on &'static [T] (static slices)
// =============================================================================
//
// `&'static [T]` is Sized (fat pointer: ptr + len) and satisfies 'static,
// so it can be used DIRECTLY with ALTREP via ExternalPtr.
//
// The data trait implementations above for `&[T]` already cover `&'static [T]`
// since `&'static [T]` is a subtype of `&[T]`. The ALTREP trait implementations
// (Altrep, AltVec, AltInteger, etc.) are provided separately in altrep_impl.rs.
//
// Use cases:
// - Const arrays: `static DATA: [i32; 5] = [1, 2, 3, 4, 5]; create_altrep(&DATA[..])`
// - Leaked data: `let s: &'static [i32] = Box::leak(vec.into_boxed_slice());`
// - Memory-mapped files with 'static lifetime

// =============================================================================
// Built-in implementations for arrays (owned, fixed-size)
// =============================================================================

impl<const N: usize> AltrepLen for [i32; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltIntegerData for [i32; N] {
    fn elt(&self, i: usize) -> i32 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[i32]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.contains(&i32::MIN))
    }
}

impl<const N: usize> AltrepLen for [f64; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltRealData for [f64; N] {
    fn elt(&self, i: usize) -> f64 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[f64]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [f64]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }

    fn no_na(&self) -> Option<bool> {
        Some(!self.iter().any(|x| x.is_nan()))
    }
}

impl<const N: usize> AltrepLen for [bool; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltLogicalData for [bool; N] {
    fn elt(&self, i: usize) -> Logical {
        Logical::from_bool(self[i])
    }

    fn no_na(&self) -> Option<bool> {
        Some(true) // bool arrays can't have NA
    }
}

impl<const N: usize> AltrepLen for [u8; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltRawData for [u8; N] {
    fn elt(&self, i: usize) -> u8 {
        self[i]
    }

    fn as_slice(&self) -> Option<&[u8]> {
        Some(self.as_slice())
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [u8]) -> usize {
        let end = (start + len).min(N);
        let actual_len = end.saturating_sub(start);
        if actual_len > 0 {
            buf[..actual_len].copy_from_slice(&self[start..end]);
        }
        actual_len
    }
}

impl<const N: usize> AltrepLen for [String; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize> AltStringData for [String; N] {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(self[i].as_str())
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Logical enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_logical_to_r_int() {
        assert_eq!(Logical::False.to_r_int(), 0);
        assert_eq!(Logical::True.to_r_int(), 1);
        assert_eq!(Logical::Na.to_r_int(), i32::MIN);
    }

    #[test]
    fn test_logical_from_r_int() {
        assert_eq!(Logical::from_r_int(0), Logical::False);
        assert_eq!(Logical::from_r_int(1), Logical::True);
        assert_eq!(Logical::from_r_int(42), Logical::True); // Non-zero is TRUE
        assert_eq!(Logical::from_r_int(-1), Logical::True);
        assert_eq!(Logical::from_r_int(i32::MIN), Logical::Na);
    }

    #[test]
    fn test_logical_from_bool() {
        assert_eq!(Logical::from_bool(false), Logical::False);
        assert_eq!(Logical::from_bool(true), Logical::True);
    }

    // -------------------------------------------------------------------------
    // Sortedness enum tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sortedness_to_r_int() {
        assert_eq!(Sortedness::Unknown.to_r_int(), i32::MIN);
        assert_eq!(Sortedness::KnownUnsorted.to_r_int(), 0);
        assert_eq!(Sortedness::Increasing.to_r_int(), 1);
        assert_eq!(Sortedness::Decreasing.to_r_int(), -1);
        assert_eq!(Sortedness::IncreasingNaFirst.to_r_int(), 2);
        assert_eq!(Sortedness::DecreasingNaFirst.to_r_int(), -2);
    }

    #[test]
    fn test_sortedness_from_r_int() {
        assert_eq!(Sortedness::from_r_int(i32::MIN), Sortedness::Unknown);
        assert_eq!(Sortedness::from_r_int(0), Sortedness::KnownUnsorted);
        assert_eq!(Sortedness::from_r_int(1), Sortedness::Increasing);
        assert_eq!(Sortedness::from_r_int(-1), Sortedness::Decreasing);
        assert_eq!(Sortedness::from_r_int(2), Sortedness::IncreasingNaFirst);
        assert_eq!(Sortedness::from_r_int(-2), Sortedness::DecreasingNaFirst);
        // Invalid values map to Unknown
        assert_eq!(Sortedness::from_r_int(99), Sortedness::Unknown);
    }

    // -------------------------------------------------------------------------
    // Vec<i32> AltIntegerData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_i32_len() {
        let v: Vec<i32> = vec![1, 2, 3, 4, 5];
        assert_eq!(AltrepLen::len(&v), 5);
        assert!(!AltrepLen::is_empty(&v));

        let empty: Vec<i32> = vec![];
        assert_eq!(AltrepLen::len(&empty), 0);
        assert!(AltrepLen::is_empty(&empty));
    }

    #[test]
    fn test_vec_i32_elt() {
        let v = vec![10, 20, 30];
        assert_eq!(AltIntegerData::elt(&v, 0), 10);
        assert_eq!(AltIntegerData::elt(&v, 1), 20);
        assert_eq!(AltIntegerData::elt(&v, 2), 30);
    }

    #[test]
    fn test_vec_i32_as_slice() {
        let v = vec![1, 2, 3];
        assert_eq!(AltIntegerData::as_slice(&v), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn test_vec_i32_get_region() {
        let v = vec![10, 20, 30, 40, 50];
        let mut buf = [0i32; 3];

        // Normal region
        let n = AltIntegerData::get_region(&v, 1, 3, &mut buf);
        assert_eq!(n, 3);
        assert_eq!(buf, [20, 30, 40]);

        // Region at end (partial)
        let n = AltIntegerData::get_region(&v, 3, 5, &mut buf);
        assert_eq!(n, 2);
        assert_eq!(buf[..2], [40, 50]);

        // Start beyond length
        let n = AltIntegerData::get_region(&v, 10, 3, &mut buf);
        assert_eq!(n, 0);
    }

    #[test]
    fn test_vec_i32_no_na() {
        let v = vec![1, 2, 3];
        assert_eq!(AltIntegerData::no_na(&v), Some(true));

        let v_with_na = vec![1, i32::MIN, 3]; // i32::MIN is NA
        assert_eq!(AltIntegerData::no_na(&v_with_na), Some(false));
    }

    #[test]
    fn test_vec_i32_sum() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(AltIntegerData::sum(&v, false), Some(15));
        assert_eq!(AltIntegerData::sum(&v, true), Some(15));

        // With NA
        let v_na = vec![1, 2, i32::MIN, 4, 5];
        assert_eq!(AltIntegerData::sum(&v_na, false), None); // NA propagates
        assert_eq!(AltIntegerData::sum(&v_na, true), Some(12)); // na.rm=TRUE
    }

    #[test]
    fn test_vec_i32_min_max() {
        let v = vec![5, 2, 8, 1, 9];
        assert_eq!(AltIntegerData::min(&v, false), Some(1));
        assert_eq!(AltIntegerData::max(&v, false), Some(9));

        // With NA
        let v_na = vec![5, 2, i32::MIN, 1, 9];
        assert_eq!(AltIntegerData::min(&v_na, false), None);
        assert_eq!(AltIntegerData::max(&v_na, false), None);
        assert_eq!(AltIntegerData::min(&v_na, true), Some(1));
        assert_eq!(AltIntegerData::max(&v_na, true), Some(9));
    }

    // -------------------------------------------------------------------------
    // Vec<f64> AltRealData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_f64_sum() {
        let v = vec![1.0, 2.0, 3.0];
        assert_eq!(AltRealData::sum(&v, false), Some(6.0));

        let v_nan = vec![1.0, f64::NAN, 3.0];
        assert!(AltRealData::sum(&v_nan, false).unwrap().is_nan());
        assert_eq!(AltRealData::sum(&v_nan, true), Some(4.0));
    }

    #[test]
    fn test_vec_f64_min_max() {
        let v = vec![3.0, 1.0, 4.0, 1.5];
        assert_eq!(AltRealData::min(&v, false), Some(1.0));
        assert_eq!(AltRealData::max(&v, false), Some(4.0));
    }

    // -------------------------------------------------------------------------
    // Box<[T]> tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_box_slice_i32() {
        let b: Box<[i32]> = vec![1, 2, 3, 4, 5].into_boxed_slice();
        assert_eq!(AltrepLen::len(&b), 5);
        assert_eq!(AltIntegerData::elt(&b, 2), 3);
        assert_eq!(AltIntegerData::sum(&b, false), Some(15));
        assert_eq!(AltIntegerData::min(&b, false), Some(1));
        assert_eq!(AltIntegerData::max(&b, false), Some(5));
    }

    #[test]
    fn test_box_slice_f64() {
        let b: Box<[f64]> = vec![1.0, 2.0, 3.0].into_boxed_slice();
        assert_eq!(AltrepLen::len(&b), 3);
        assert_eq!(AltRealData::elt(&b, 1), 2.0);
        assert_eq!(AltRealData::sum(&b, false), Some(6.0));
    }

    // -------------------------------------------------------------------------
    // Range<i32> tests
    // -------------------------------------------------------------------------

    #[test]
    #[allow(clippy::reversed_empty_ranges)] // Intentionally testing empty range handling
    fn test_range_i32_len() {
        let r = 1..10;
        assert_eq!(AltrepLen::len(&r), 9);

        let empty = 10..5;
        assert_eq!(AltrepLen::len(&empty), 0);
    }

    #[test]
    fn test_range_i32_elt() {
        let r = 5..10;
        assert_eq!(AltIntegerData::elt(&r, 0), 5);
        assert_eq!(AltIntegerData::elt(&r, 4), 9);
    }

    #[test]
    fn test_range_i32_sum() {
        // Sum of 1..11 (1 to 10) = 55
        let r = 1..11;
        assert_eq!(AltIntegerData::sum(&r, false), Some(55));

        // Sum of 1..101 (1 to 100) = 5050
        let r = 1..101;
        assert_eq!(AltIntegerData::sum(&r, false), Some(5050));
    }

    #[test]
    fn test_range_i32_min_max() {
        let r = 5..15;
        assert_eq!(AltIntegerData::min(&r, false), Some(5));
        assert_eq!(AltIntegerData::max(&r, false), Some(14)); // end is exclusive
    }

    #[test]
    fn test_range_i32_is_sorted() {
        let r = 1..10;
        assert_eq!(AltIntegerData::is_sorted(&r), Some(Sortedness::Increasing));
    }

    // -------------------------------------------------------------------------
    // Static slice tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_static_slice_i32() {
        static DATA: [i32; 5] = [10, 20, 30, 40, 50];
        let s: &[i32] = &DATA;

        assert_eq!(AltrepLen::len(&s), 5);
        assert_eq!(AltIntegerData::elt(&s, 0), 10);
        assert_eq!(AltIntegerData::elt(&s, 4), 50);
        assert_eq!(AltIntegerData::sum(&s, false), Some(150));
        assert_eq!(AltIntegerData::min(&s, false), Some(10));
        assert_eq!(AltIntegerData::max(&s, false), Some(50));
    }

    #[test]
    fn test_static_slice_with_na() {
        let s: &[i32] = &[1, 2, i32::MIN, 4];
        assert_eq!(AltIntegerData::no_na(&s), Some(false));
        assert_eq!(AltIntegerData::sum(&s, false), None); // NA propagates
        assert_eq!(AltIntegerData::sum(&s, true), Some(7)); // na.rm=TRUE
    }

    #[test]
    fn test_static_slice_f64() {
        static DATA: [f64; 4] = [1.5, 2.5, 3.5, 4.5];
        let s: &[f64] = &DATA;

        assert_eq!(AltrepLen::len(&s), 4);
        assert_eq!(AltRealData::sum(&s, false), Some(12.0));
        assert_eq!(AltRealData::min(&s, false), Some(1.5));
        assert_eq!(AltRealData::max(&s, false), Some(4.5));
    }

    // -------------------------------------------------------------------------
    // Array tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_array_i32() {
        let arr: [i32; 3] = [100, 200, 300];
        assert_eq!(AltrepLen::len(&arr), 3);
        assert_eq!(AltIntegerData::elt(&arr, 1), 200);
        assert_eq!(AltIntegerData::as_slice(&arr), Some(&[100, 200, 300][..]));
    }

    #[test]
    fn test_array_f64() {
        let arr: [f64; 2] = [1.1, 2.2];
        assert_eq!(AltrepLen::len(&arr), 2);
        assert_eq!(AltRealData::elt(&arr, 0), 1.1);
    }

    // -------------------------------------------------------------------------
    // Vec<bool> AltLogicalData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_bool_logical() {
        let v = vec![true, false, true, true];
        assert_eq!(AltrepLen::len(&v), 4);
        assert_eq!(AltLogicalData::elt(&v, 0), Logical::True);
        assert_eq!(AltLogicalData::elt(&v, 1), Logical::False);
        assert_eq!(AltLogicalData::no_na(&v), Some(true));
        assert_eq!(AltLogicalData::sum(&v, false), Some(3)); // Count of TRUE
    }

    // -------------------------------------------------------------------------
    // Vec<String> AltStringData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_string() {
        let v = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(AltrepLen::len(&v), 2);
        assert_eq!(AltStringData::elt(&v, 0), Some("hello"));
        assert_eq!(AltStringData::elt(&v, 1), Some("world"));
        assert_eq!(AltStringData::no_na(&v), Some(true));
    }

    #[test]
    fn test_vec_option_string() {
        let v: Vec<Option<String>> = vec![Some("a".to_string()), None, Some("b".to_string())];
        assert_eq!(AltrepLen::len(&v), 3);
        assert_eq!(AltStringData::elt(&v, 0), Some("a"));
        assert_eq!(AltStringData::elt(&v, 1), None); // NA
        assert_eq!(AltStringData::elt(&v, 2), Some("b"));
        assert_eq!(AltStringData::no_na(&v), Some(false)); // Has NA
    }

    // -------------------------------------------------------------------------
    // Vec<u8> AltRawData tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vec_u8() {
        let v: Vec<u8> = vec![0x01, 0x02, 0xFF];
        assert_eq!(AltrepLen::len(&v), 3);
        assert_eq!(AltRawData::elt(&v, 0), 0x01);
        assert_eq!(AltRawData::elt(&v, 2), 0xFF);
        assert_eq!(AltRawData::as_slice(&v), Some(&[0x01, 0x02, 0xFF][..]));
    }

    // -------------------------------------------------------------------------
    // Edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_vec() {
        let v: Vec<i32> = vec![];
        assert_eq!(AltrepLen::len(&v), 0);
        assert!(AltrepLen::is_empty(&v));
        assert_eq!(AltIntegerData::sum(&v, false), Some(0));
        assert_eq!(AltIntegerData::min(&v, false), None);
        assert_eq!(AltIntegerData::max(&v, false), None);
    }

    #[test]
    fn test_single_element() {
        let v = vec![42];
        assert_eq!(AltIntegerData::sum(&v, false), Some(42));
        assert_eq!(AltIntegerData::min(&v, false), Some(42));
        assert_eq!(AltIntegerData::max(&v, false), Some(42));
    }

    #[test]
    fn test_large_sum_overflow() {
        // Sum that exceeds i32 range but fits in i64
        let v: Vec<i32> = vec![i32::MAX, i32::MAX];
        let sum = AltIntegerData::sum(&v, false).unwrap();
        assert_eq!(sum, 2 * i32::MAX as i64);
    }

    // -------------------------------------------------------------------------
    // Iterator-backed ALTREP tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_iter_int_basic() {
        // Use from_iter with explicit length since Map doesn't preserve ExactSizeIterator
        let iter = (1..=5).map(|x| x * 2);
        let data = IterIntData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltIntegerData::elt(&data, 0), 2);
        assert_eq!(AltIntegerData::elt(&data, 4), 10);

        // Out of bounds
        assert_eq!(
            AltIntegerData::elt(&data, 5),
            crate::altrep_traits::NA_INTEGER
        );
    }

    #[test]
    fn test_iter_int_random_access() {
        let iter = (0..10).map(|x| x * x);
        let data = IterIntData::from_iter(iter, 10);

        // Access in non-sequential order (tests caching)
        assert_eq!(AltIntegerData::elt(&data, 5), 25);
        assert_eq!(AltIntegerData::elt(&data, 2), 4);
        assert_eq!(AltIntegerData::elt(&data, 5), 25); // Cached
        assert_eq!(AltIntegerData::elt(&data, 9), 81);
    }

    #[test]
    fn test_iter_real_basic() {
        let iter = (1..=5).map(|x| x as f64 * 1.5);
        let data = IterRealData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltRealData::elt(&data, 0), 1.5);
        assert_eq!(AltRealData::elt(&data, 4), 7.5);
    }

    #[test]
    fn test_iter_logical_basic() {
        let iter = (0..5).map(|x| x % 2 == 0);
        let data = IterLogicalData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltLogicalData::elt(&data, 0), Logical::True);
        assert_eq!(AltLogicalData::elt(&data, 1), Logical::False);
        assert_eq!(AltLogicalData::elt(&data, 2), Logical::True);
    }

    #[test]
    fn test_iter_raw_basic() {
        let iter = (0..5u8).map(|x| x * 10);
        let data = IterRawData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltRawData::elt(&data, 0), 0);
        assert_eq!(AltRawData::elt(&data, 2), 20);
        assert_eq!(AltRawData::elt(&data, 4), 40);
    }

    #[test]
    fn test_iter_get_region() {
        let iter = (1..=10).map(|x| x * 10);
        let data = IterIntData::from_iter(iter, 10);

        let mut buf = [0i32; 5];
        let n = AltIntegerData::get_region(&data, 2, 5, &mut buf);

        assert_eq!(n, 5);
        assert_eq!(buf, [30, 40, 50, 60, 70]);
    }

    #[test]
    fn test_iter_state_materialization() {
        let iter = (1..=3).map(|x| x * 2);
        let data = IterIntData::from_iter(iter, 3);

        // Initially not materialized
        assert!(data.state.as_materialized().is_none());

        // Access some elements
        assert_eq!(AltIntegerData::elt(&data, 0), 2);
        assert!(data.state.as_materialized().is_none()); // Still not fully materialized

        // Access all elements
        assert_eq!(AltIntegerData::elt(&data, 2), 6);

        // Materialize
        let materialized = data.state.materialize_all();
        assert_eq!(materialized, &[2, 4, 6]);

        // Now as_slice works
        assert_eq!(data.as_slice(), Some(&[2, 4, 6][..]));
    }

    #[test]
    fn test_iter_explicit_length() {
        // Create with explicit length (not ExactSizeIterator)
        let iter = vec![10, 20, 30].into_iter();
        let data = IterIntData::from_iter(iter, 3);

        assert_eq!(AltrepLen::len(&data), 3);
        assert_eq!(AltIntegerData::elt(&data, 1), 20);
    }

    // -------------------------------------------------------------------------
    // Iterator-backed ALTREP with Coerce tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_iter_int_coerce_u16() {
        // Iterator of u16 values coerced to i32
        let iter = (0..5u16).map(|x| x * 1000);
        let data = IterIntCoerceData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltIntegerData::elt(&data, 0), 0);
        assert_eq!(AltIntegerData::elt(&data, 2), 2000);
        assert_eq!(AltIntegerData::elt(&data, 4), 4000);
    }

    #[test]
    fn test_iter_int_coerce_i8() {
        // Iterator of i8 values coerced to i32
        let iter = -5i8..5i8;
        let data = IterIntCoerceData::from_exact_iter(iter);

        assert_eq!(AltrepLen::len(&data), 10);
        assert_eq!(AltIntegerData::elt(&data, 0), -5);
        assert_eq!(AltIntegerData::elt(&data, 5), 0);
        assert_eq!(AltIntegerData::elt(&data, 9), 4);
    }

    #[test]
    fn test_iter_real_coerce_f32() {
        // Iterator of f32 values coerced to f64
        let iter = (0..5).map(|x| x as f32 * 1.5);
        let data = IterRealCoerceData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert!((AltRealData::elt(&data, 0) - 0.0).abs() < 0.001);
        assert!((AltRealData::elt(&data, 2) - 3.0).abs() < 0.001);
        assert!((AltRealData::elt(&data, 4) - 6.0).abs() < 0.001);
    }

    #[test]
    fn test_iter_real_coerce_i32() {
        // Iterator of i32 values coerced to f64
        let iter = 1..=5;
        let data = IterRealCoerceData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltRealData::elt(&data, 0), 1.0);
        assert_eq!(AltRealData::elt(&data, 4), 5.0);
    }

    #[test]
    fn test_iter_int_from_bool() {
        // Iterator of bool values coerced to i32
        let iter = (0..10).map(|x| x % 3 == 0);
        let data = IterIntFromBoolData::from_iter(iter, 10);

        assert_eq!(AltrepLen::len(&data), 10);
        assert_eq!(AltIntegerData::elt(&data, 0), 1); // TRUE
        assert_eq!(AltIntegerData::elt(&data, 1), 0); // FALSE
        assert_eq!(AltIntegerData::elt(&data, 3), 1); // TRUE
    }

    #[test]
    fn test_iter_coerce_get_region() {
        // Test get_region with coerced types
        let iter = (0..10u16).map(|x| x * 10);
        let data = IterIntCoerceData::from_iter(iter, 10);

        let mut buf = [0i32; 5];
        let n = AltIntegerData::get_region(&data, 3, 5, &mut buf);

        assert_eq!(n, 5);
        assert_eq!(buf, [30, 40, 50, 60, 70]);
    }

    #[test]
    fn test_iter_real_coerce_option() {
        // Iterator of Option<f64> coerced to f64 with None → NA (NaN)
        let iter = (0..5).map(|x| if x % 2 == 0 { Some(x as f64) } else { None });
        let data = IterRealCoerceData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltRealData::elt(&data, 0), 0.0); // Some(0.0)
        assert!(AltRealData::elt(&data, 1).is_nan()); // None → NaN
        assert_eq!(AltRealData::elt(&data, 2), 2.0); // Some(2.0)
        assert!(AltRealData::elt(&data, 3).is_nan()); // None → NaN
        assert_eq!(AltRealData::elt(&data, 4), 4.0); // Some(4.0)
    }

    #[test]
    fn test_iter_int_coerce_option() {
        // Iterator of Option<i32> coerced to i32 with None → NA (i32::MIN)
        let iter = (0..5).map(|x| if x % 2 == 0 { Some(x) } else { None });
        let data = IterIntCoerceData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);
        assert_eq!(AltIntegerData::elt(&data, 0), 0); // Some(0)
        assert_eq!(AltIntegerData::elt(&data, 1), i32::MIN); // None → NA
        assert_eq!(AltIntegerData::elt(&data, 2), 2); // Some(2)
        assert_eq!(AltIntegerData::elt(&data, 3), i32::MIN); // None → NA
        assert_eq!(AltIntegerData::elt(&data, 4), 4); // Some(4)
    }

    #[test]
    fn test_iter_string_basic() {
        let iter = (0..3).map(|x| format!("item_{}", x));
        let data = IterStringData::from_iter(iter, 3);

        assert_eq!(AltrepLen::len(&data), 3);
        assert_eq!(AltStringData::elt(&data, 0), Some("item_0"));
        assert_eq!(AltStringData::elt(&data, 1), Some("item_1"));
        assert_eq!(AltStringData::elt(&data, 2), Some("item_2"));
    }

    #[test]
    fn test_iter_complex_basic() {
        use crate::ffi::Rcomplex;

        let iter = (0..5).map(|x| Rcomplex {
            r: x as f64,
            i: (x * 2) as f64,
        });
        let data = IterComplexData::from_iter(iter, 5);

        assert_eq!(AltrepLen::len(&data), 5);

        let z0 = AltComplexData::elt(&data, 0);
        assert_eq!(z0.r, 0.0);
        assert_eq!(z0.i, 0.0);

        let z2 = AltComplexData::elt(&data, 2);
        assert_eq!(z2.r, 2.0);
        assert_eq!(z2.i, 4.0);
    }
}
