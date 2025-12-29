// =============================================================================
// Iterator-backed ALTREP infrastructure
// =============================================================================

use std::cell::RefCell;
use std::sync::OnceLock;

use crate::ffi::SEXP;

use super::{
    AltComplexData, AltIntegerData, AltListData, AltLogicalData, AltRawData, AltRealData,
    AltStringData, AltrepLen, InferBase, Logical, fill_region,
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I, T> crate::externalptr::TypedExternal for IterIntCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<i32> + Copy + 'static,
{
    const TYPE_NAME: &'static str = "IterIntCoerceData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterIntCoerceData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterIntCoerceData\0";
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I, T> crate::externalptr::TypedExternal for IterRealCoerceData<I, T>
where
    I: Iterator<Item = T> + 'static,
    T: crate::coerce::Coerce<f64> + Copy + 'static,
{
    const TYPE_NAME: &'static str = "IterRealCoerceData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterRealCoerceData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterRealCoerceData\0";
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::externalptr::TypedExternal
    for IterIntFromBoolData<I>
{
    const TYPE_NAME: &'static str = "IterIntFromBoolData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterIntFromBoolData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterIntFromBoolData\0";
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
impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltInteger
    for IterIntFromBoolData<I>
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
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterStringData\0";
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
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterListData\0";
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
        fill_region(start, len, self.len(), buf, |idx| self.elt(idx))
    }
}

impl<I: Iterator<Item = crate::ffi::Rcomplex> + 'static> crate::externalptr::TypedExternal
    for IterComplexData<I>
{
    const TYPE_NAME: &'static str = "IterComplexData";
    const TYPE_NAME_CSTR: &'static [u8] = b"IterComplexData\0";
    const TYPE_ID_CSTR: &'static [u8] = b"miniextendr_api::altrep::IterComplexData\0";
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
