//! Core iterator-backed ALTREP infrastructure.
//!
//! Provides `IterState<I, T>` (the shared lazy-caching state machine) and
//! wrapper types for each ALTREP family: `IterIntData`, `IterRealData`,
//! `IterLogicalData`, `IterRawData`, `IterStringData`, `IterListData`,
//! `IterComplexData`.

use std::cell::RefCell;
use std::sync::OnceLock;

use crate::altrep_data::{
    AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepLen, InferBase, Logical,
    fill_region,
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

            // Fill cache up to and including index i
            while cache.len() <= i {
                if let Some(elem) = iter.next() {
                    cache.push(elem);
                } else {
                    // Iterator exhausted before reaching expected length
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::externalptr::TypedExternal for IterIntData<I> {
    const TYPE_NAME: &'static str = "IterIntData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterIntData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterIntData\0";
}

impl<I: Iterator<Item = i32> + 'static> InferBase for IterIntData<I> {
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

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::Altrep for IterIntData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltVec for IterIntData<I> {}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltInteger for IterIntData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| AltIntegerData::elt(&*d, i as usize))
            .unwrap_or(i32::MIN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [i32],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| {
                AltIntegerData::get_region(&*d, start as usize, len as usize, buf)
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::externalptr::TypedExternal for IterRealData<I> {
    const TYPE_NAME: &'static str = "IterRealData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterRealData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterRealData\0";
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

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::Altrep for IterRealData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltVec for IterRealData<I> {}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltReal for IterRealData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> f64 {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| AltRealData::elt(&*d, i as usize))
            .unwrap_or(f64::NAN)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [f64],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| {
                AltRealData::get_region(&*d, start as usize, len as usize, buf)
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx).to_r_int())
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::externalptr::TypedExternal for IterLogicalData<I> {
    const TYPE_NAME: &'static str = "IterLogicalData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterLogicalData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterLogicalData\0";
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

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::Altrep for IterLogicalData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltVec for IterLogicalData<I> {}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltLogical for IterLogicalData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> i32 {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| AltLogicalData::elt(&*d, i as usize).to_r_int())
            .unwrap_or(crate::altrep_traits::NA_LOGICAL)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [i32],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| {
                AltLogicalData::get_region(&*d, start as usize, len as usize, buf)
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = u8> + 'static> crate::externalptr::TypedExternal for IterRawData<I> {
    const TYPE_NAME: &'static str = "IterRawData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterRawData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterRawData\0";
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

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::Altrep for IterRawData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::AltVec for IterRawData<I> {}

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::AltRaw for IterRawData<I> {
    const HAS_ELT: bool = true;

    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> u8 {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| AltRawData::elt(&*d, i as usize))
            .unwrap_or(0)
    }

    const HAS_GET_REGION: bool = true;

    fn get_region(
        x: crate::ffi::SEXP,
        start: crate::ffi::R_xlen_t,
        len: crate::ffi::R_xlen_t,
        buf: &mut [u8],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_ext::AltrepSexpExt::altrep_data1::<Self>(&x) }
            .map(|d| {
                AltRawData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}
// endregion
