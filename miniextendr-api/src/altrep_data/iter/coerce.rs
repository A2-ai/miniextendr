//! Iterator-backed ALTREP with coercion support.
//!
//! Provides `IterIntCoerceData`, `IterRealCoerceData`, and `IterIntFromBoolData`
//! for iterators that produce values coercible to the target R type.

use super::IterState;
use crate::altrep_data::{
    AltComplexData, AltIntegerData, AltListData, AltRealData, AltStringData, AltrepLen, InferBase,
    fill_region,
};
use crate::ffi::SEXP;

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
                let slice = unsafe { crate::altrep_impl::altrep_region_buf(buf, len as usize) };
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
                let slice = unsafe { crate::altrep_impl::altrep_region_buf(buf, len as usize) };
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

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::Altrep for IterIntFromBoolData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = bool> + 'static> crate::altrep_traits::AltVec for IterIntFromBoolData<I> {}

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
                let slice = unsafe { crate::altrep_impl::altrep_region_buf(buf, len as usize) };
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

impl<I: Iterator<Item = String> + 'static> crate::altrep_traits::Altrep for IterStringData<I> {
    // String ALTREP elt calls Rf_mkCharLenCE (R API) — must use RUnwind to catch longjmps.
    const GUARD: crate::altrep_traits::AltrepGuard = crate::altrep_traits::AltrepGuard::RUnwind;

    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = String> + 'static> crate::altrep_traits::AltVec for IterStringData<I> {}

impl<I: Iterator<Item = String> + 'static> crate::altrep_traits::AltString for IterStringData<I> {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .and_then(|d| {
                AltStringData::elt(&*d, i as usize)
                    .map(|s| unsafe { crate::altrep_impl::checked_mkchar(s) })
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
        use crate::ffi::SEXP;
        self.state.get_element(i).unwrap_or(SEXP::nil())
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

impl<I: Iterator<Item = SEXP> + 'static> crate::altrep_traits::Altrep for IterListData<I> {
    fn length(x: crate::ffi::SEXP) -> crate::ffi::R_xlen_t {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| d.len() as crate::ffi::R_xlen_t)
            .unwrap_or(0)
    }
}

impl<I: Iterator<Item = SEXP> + 'static> crate::altrep_traits::AltVec for IterListData<I> {}

impl<I: Iterator<Item = SEXP> + 'static> crate::altrep_traits::AltList for IterListData<I> {
    fn elt(x: crate::ffi::SEXP, i: crate::ffi::R_xlen_t) -> crate::ffi::SEXP {
        unsafe { crate::altrep_data1_as::<Self>(x) }
            .map(|d| AltListData::elt(&*d, i as usize))
            .unwrap_or(crate::ffi::SEXP::nil())
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
                let slice = unsafe { crate::altrep_impl::altrep_region_buf(buf, len as usize) };
                AltComplexData::get_region(&*d, start as usize, len as usize, slice)
                    as crate::ffi::R_xlen_t
            })
            .unwrap_or(0)
    }
}
// endregion
