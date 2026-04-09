//! Sparse iterator-backed ALTREP with skipping support.
//!
//! Provides `SparseIterState<I, T>` which uses `Iterator::nth()` to skip elements
//! efficiently, and wrapper types for each ALTREP family.

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::altrep_data::{
    AltComplexData, AltIntegerData, AltLogicalData, AltRawData, AltRealData, AltrepLen, InferBase,
    Logical, fill_region,
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

/// Sparse iterator-backed integer vector data.
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

impl<I: Iterator<Item = i32> + 'static> crate::externalptr::TypedExternal for SparseIterIntData<I> {
    const TYPE_NAME: &'static str = "SparseIterIntData";
    const TYPE_NAME_CSTR: &'static [u8] = b"SparseIterIntData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::SparseIterIntData\0";
}

impl<I: Iterator<Item = i32> + 'static> InferBase for SparseIterIntData<I> {
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

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::Altrep for SparseIterIntData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltVec for SparseIterIntData<I> {}

impl<I: Iterator<Item = i32> + 'static> crate::altrep_traits::AltInteger for SparseIterIntData<I> {
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
        buf: &mut [i32],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                AltIntegerData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Sparse iterator-backed real (f64) vector data.
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

impl<I: Iterator<Item = f64> + 'static> crate::externalptr::TypedExternal
    for SparseIterRealData<I>
{
    const TYPE_NAME: &'static str = "SparseIterRealData";
    const TYPE_NAME_CSTR: &'static [u8] = b"SparseIterRealData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::SparseIterRealData\0";
}

impl<I: Iterator<Item = f64> + 'static> InferBase for SparseIterRealData<I> {
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

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::Altrep for SparseIterRealData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltVec for SparseIterRealData<I> {}

impl<I: Iterator<Item = f64> + 'static> crate::altrep_traits::AltReal for SparseIterRealData<I> {
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
        buf: &mut [f64],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                AltRealData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Sparse iterator-backed logical vector data.
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

impl<I: Iterator<Item = bool> + 'static> crate::externalptr::TypedExternal
    for SparseIterLogicalData<I>
{
    const TYPE_NAME: &'static str = "SparseIterLogicalData";
    const TYPE_NAME_CSTR: &'static [u8] = b"SparseIterLogicalData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::SparseIterLogicalData\0";
}

impl<I: Iterator<Item = bool> + 'static> InferBase for SparseIterLogicalData<I> {
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

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::Altrep for SparseIterLogicalData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltVec for SparseIterLogicalData<I> {}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltLogical
    for SparseIterLogicalData<I>
{
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
        buf: &mut [i32],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                AltLogicalData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Sparse iterator-backed raw (u8) vector data.
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

impl<I: Iterator<Item = u8> + 'static> crate::externalptr::TypedExternal for SparseIterRawData<I> {
    const TYPE_NAME: &'static str = "SparseIterRawData";
    const TYPE_NAME_CSTR: &'static [u8] = b"SparseIterRawData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::SparseIterRawData\0";
}

impl<I: Iterator<Item = u8> + 'static> InferBase for SparseIterRawData<I> {
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

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::Altrep for SparseIterRawData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::AltVec for SparseIterRawData<I> {}

impl<I: Iterator<Item = u8> + 'static> crate::altrep_traits::AltRaw for SparseIterRawData<I> {
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
        buf: &mut [u8],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                AltRawData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}

/// Sparse iterator-backed complex number vector.
pub struct SparseIterComplexData<I>
where
    I: Iterator<Item = crate::ffi::Rcomplex>,
{
    state: SparseIterState<I, crate::ffi::Rcomplex>,
}

impl<I> SparseIterComplexData<I>
where
    I: Iterator<Item = crate::ffi::Rcomplex>,
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
    I: ExactSizeIterator<Item = crate::ffi::Rcomplex>,
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
    I: Iterator<Item = crate::ffi::Rcomplex>,
{
    fn len(&self) -> usize {
        self.state.len()
    }
}

impl<I> AltComplexData for SparseIterComplexData<I>
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
        None
    }

    fn get_region(&self, start: usize, len: usize, buf: &mut [crate::ffi::Rcomplex]) -> usize {
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::externalptr::TypedExternal
    for SparseIterComplexData<I>
{
    const TYPE_NAME: &'static str = "SparseIterComplexData";
    const TYPE_NAME_CSTR: &'static [u8] = b"SparseIterComplexData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::SparseIterComplexData\0";
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> InferBase for SparseIterComplexData<I> {
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

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::altrep_traits::Altrep
    for SparseIterComplexData<I>
{
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::altrep_traits::AltVec
    for SparseIterComplexData<I>
{
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::altrep_traits::AltComplex
    for SparseIterComplexData<I>
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
        buf: &mut [crate::ffi::Rcomplex],
    ) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| {
                AltComplexData::get_region(&*d, start as usize, len as usize, buf)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}
// endregion
