//! ALTREP "from scratch" core for miniextendr-api: one class per base kind
//! (INT, REAL, STRING). No libR-sys/extendr dependencies; only raw FFI.
//!
//! # ALTREP Method Return Conventions
//!
//! R's ALTREP API uses two distinct "no result" values with different semantics:
//!
//! - **`NULL` (C null pointer, `core::ptr::null_mut()` in Rust)**: Signals "I cannot
//!   compute this efficiently; please use the default fallback." R will then compute
//!   the result itself using DATAPTR/Elt methods.
//!
//! - **`R_NilValue`**: A valid SEXP pointing to R's nil singleton. Returning this
//!   means "the result is nil" — a legitimate return value, not a fallback signal.
//!
//! This distinction matters for methods like Sum, Min, Max, Duplicate, Coerce, and
//! Extract_subset. For example, in `do_summary` (summary.c), R checks:
//!
//! ```c
//! if (toret != NULL) {
//!     return toret;  // Use ALTREP's result
//! }
//! // ... fall through to default computation
//! ```
//!
//! If Sum returns `R_NilValue`, R would use nil as the sum result (wrong!).
//! If Sum returns `NULL`, R computes the sum itself (correct fallback).
//!
//! R's default ALTREP methods follow this pattern:
//! ```c
//! static SEXP altreal_Sum_default(SEXP x, Rboolean narm) { return NULL; }
//! static SEXP altrep_Duplicate_default(SEXP x, Rboolean deep) { return NULL; }
//! ```

use core::ffi::c_void;
use core::slice;
use std::sync::{Arc, OnceLock};

// Use the project's FFI definitions and types.
use crate::altrep_traits as traits;
use crate::ffi::altrep::*;
use crate::ffi::*;

// ALTREP class handles are global opaque pointers provided by R
// and can be safely shared across threads in this context.
unsafe impl Send for R_altrep_class_t {}
unsafe impl Sync for R_altrep_class_t {}
impl Copy for R_altrep_class_t {}
impl Clone for R_altrep_class_t {
    fn clone(&self) -> Self {
        *self
    }
}

// Global class handles
static ALTINT: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTREAL: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTSTR: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTLOG: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTRAW: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTLIST: OnceLock<R_altrep_class_t> = OnceLock::new();

/// Integer backend trait — implement this for any Rust struct to back an INTSXP ALTREP.
pub trait IntBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn elt(&self, i: R_xlen_t) -> i32;
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let ncopy = n.min(self.len().saturating_sub(i)).max(0);
        for k in 0..ncopy {
            out[k as usize] = self.elt(i + k);
        }
        ncopy
    }
    fn dataptr(&self) -> Option<&[i32]> {
        None
    }
    fn is_sorted(&self) -> i32 {
        0
    } // UNKNOWN_SORTEDNESS
    fn no_na(&self) -> i32 {
        0
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Optional O(1) sum computation. Return None to use default O(n) iteration.
    fn sum(&self) -> Option<f64> {
        None
    }
    /// Optional O(1) min computation. Return None to use default O(n) iteration.
    fn min(&self) -> Option<i32> {
        None
    }
    /// Optional O(1) max computation. Return None to use default O(n) iteration.
    fn max(&self) -> Option<i32> {
        None
    }
    /// For serialization: return compact representation (len, start, step) if this is
    /// a compact integer sequence. Return None to use default materialized serialization.
    fn as_compact_seq(&self) -> Option<(R_xlen_t, i32, i32)> {
        None
    }
    /// For extract_subset optimization: extract a contiguous subsequence.
    /// Returns None to use default O(n) extraction.
    /// `start` is 0-based, `count` is the number of elements.
    fn extract_contiguous(
        &self,
        _start: R_xlen_t,
        _count: R_xlen_t,
    ) -> Option<Box<dyn IntBackend>> {
        None
    }
}

/// Real backend
pub trait RealBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn elt(&self, i: R_xlen_t) -> f64;
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let ncopy = n.min(self.len().saturating_sub(i)).max(0);
        for k in 0..ncopy {
            out[k as usize] = self.elt(i + k);
        }
        ncopy
    }
    fn dataptr(&self) -> Option<&[f64]> {
        None
    }
    fn is_sorted(&self) -> i32 {
        0
    }
    fn no_na(&self) -> i32 {
        0
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// String backend — provides UTF-8. Return None for NA.
pub trait StringBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Logical backend — values are R logical ints (0/1/NA_LOGICAL).
pub trait LogicalBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn elt(&self, i: R_xlen_t) -> i32;
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let ncopy = n.min(self.len().saturating_sub(i)).max(0);
        for k in 0..ncopy {
            out[k as usize] = self.elt(i + k);
        }
        ncopy
    }
    fn dataptr(&self) -> Option<&[i32]> {
        None
    }
    fn is_sorted(&self) -> i32 {
        0
    }
    fn no_na(&self) -> i32 {
        0
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Raw backend — bytes.
pub trait RawBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn elt(&self, i: R_xlen_t) -> Rbyte;
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let ncopy = n.min(self.len().saturating_sub(i)).max(0);
        for k in 0..ncopy {
            out[k as usize] = self.elt(i + k);
        }
        ncopy
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        None
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// List backend — general VECSXP; returns owned SEXP references.
pub trait ListBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn elt(&self, i: R_xlen_t) -> SEXP;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// -- helpers to store/retrieve Box<dyn Backend> behind an external ptr --
unsafe fn make_eptr<T: ?Sized>(b: Box<T>, fin: unsafe extern "C-unwind" fn(SEXP)) -> SEXP {
    let ep = unsafe { R_MakeExternalPtr(Box::into_raw(b).cast(), R_NilValue, R_NilValue) };
    unsafe { R_RegisterCFinalizerEx(ep, Some(fin), Rboolean::TRUE) };
    ep
}
unsafe fn ep_as<'a, T>(ep: SEXP) -> &'a T {
    unsafe { R_ExternalPtrAddr(ep).cast::<T>().as_ref() }.unwrap()
}

// ========= INT class + trampolines =========
unsafe fn int_backend<'a>(x: SEXP) -> &'a dyn IntBackend {
    let ep = unsafe { R_altrep_data1(x) };
    // Double-boxing is necessary: Box<dyn Trait> is a fat pointer (2 words),
    // but R's external pointer can only store a thin pointer (1 word).
    // So we store Box<Box<dyn Trait>> which is a thin pointer to heap-allocated fat pointer.
    unsafe { ep_as::<Box<dyn IntBackend>>(ep).as_ref() }
}
/// # Safety
/// `x` must be an ALTREP INTSXP created by this crate, with data1
/// holding a valid `Box<Box<dyn IntBackend>>` pointer.
pub unsafe fn altrep_int_backend<'a>(x: SEXP) -> &'a dyn IntBackend {
    unsafe { int_backend(x) }
}

unsafe extern "C-unwind" fn int_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        drop(unsafe { Box::<Box<dyn IntBackend>>::from_raw(raw.cast()) });
    }
}

// ========= REAL class + trampolines =========
unsafe fn real_backend<'a>(x: SEXP) -> &'a dyn RealBackend {
    let ep = unsafe { R_altrep_data1(x) };
    unsafe { ep_as::<Box<dyn RealBackend>>(ep).as_ref() }
}
/// # Safety
/// `x` must be an ALTREP REALSXP created by this crate, with data1
/// holding a valid `Box<Box<dyn RealBackend>>` pointer.
pub unsafe fn altrep_real_backend<'a>(x: SEXP) -> &'a dyn RealBackend {
    unsafe { real_backend(x) }
}
unsafe extern "C-unwind" fn real_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        drop(unsafe { Box::<Box<dyn RealBackend>>::from_raw(raw.cast()) });
    }
}

// ========= STRING class + trampolines =========
unsafe fn str_backend<'a>(x: SEXP) -> &'a dyn StringBackend {
    let ep = unsafe { R_altrep_data1(x) };
    unsafe { ep_as::<Box<dyn StringBackend>>(ep).as_ref() }
}
/// # Safety
/// `x` must be an ALTREP STRSXP created by this crate, with data1
/// holding a valid `Box<Box<dyn StringBackend>>` pointer.
pub unsafe fn altrep_str_backend<'a>(x: SEXP) -> &'a dyn StringBackend {
    unsafe { str_backend(x) }
}
unsafe extern "C-unwind" fn str_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        unsafe { drop(Box::<Box<dyn StringBackend>>::from_raw(raw.cast())) };
    }
}

// ========= LOGICAL class + trampolines =========
unsafe fn lgl_backend<'a>(x: SEXP) -> &'a dyn LogicalBackend {
    let ep = unsafe { R_altrep_data1(x) };
    unsafe { ep_as::<Box<dyn LogicalBackend>>(ep).as_ref() }
}
/// # Safety
/// `x` must be an ALTREP LGLSXP created by this crate, with data1
/// holding a valid `Box<Box<dyn LogicalBackend>>` pointer.
pub unsafe fn altrep_lgl_backend<'a>(x: SEXP) -> &'a dyn LogicalBackend {
    unsafe { lgl_backend(x) }
}
unsafe extern "C-unwind" fn lgl_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        drop(unsafe { Box::<Box<dyn LogicalBackend>>::from_raw(raw.cast()) });
    }
}

// ========= RAW class + trampolines =========
unsafe fn raw_backend<'a>(x: SEXP) -> &'a dyn RawBackend {
    let ep = unsafe { R_altrep_data1(x) };
    unsafe { ep_as::<Box<dyn RawBackend>>(ep).as_ref() }
}
/// # Safety
/// `x` must be an ALTREP `RAWSXP` created by this crate, with data1
/// holding a valid `Box<Box<dyn RawBackend>>` pointer.
pub unsafe fn altrep_raw_backend<'a>(x: SEXP) -> &'a dyn RawBackend {
    unsafe { raw_backend(x) }
}
unsafe extern "C-unwind" fn raw_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        drop(unsafe { Box::<Box<dyn RawBackend>>::from_raw(raw.cast()) });
    }
}

// ========= LIST class + trampolines =========
unsafe fn list_backend<'a>(x: SEXP) -> &'a dyn ListBackend {
    let ep = unsafe { R_altrep_data1(x) };
    unsafe { ep_as::<Box<dyn ListBackend>>(ep).as_ref() }
}
/// # Safety
/// `x` must be an ALTREP VECSXP created by this crate, with data1
/// holding a valid `Box<Box<dyn ListBackend>>` pointer.
pub unsafe fn altrep_list_backend<'a>(x: SEXP) -> &'a dyn ListBackend {
    unsafe { list_backend(x) }
}
unsafe extern "C-unwind" fn list_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        drop(unsafe { Box::<Box<dyn ListBackend>>::from_raw(raw.cast()) });
    }
}

// ========= Class registration =========

/// Must be called once (lazy-initialized on first constructor use).
unsafe fn ensure_classes() {
    ALTINT.get_or_init(|| unsafe { register_altinteger_class::<AltIntClass>() });
    ALTREAL.get_or_init(|| unsafe { register_altreal_class::<AltRealClass>() });
    ALTSTR.get_or_init(|| unsafe { register_altstring_class::<AltStrClass>() });
    ALTLOG.get_or_init(|| unsafe { register_altlogical_class::<AltLogicalClass>() });
    ALTRAW.get_or_init(|| unsafe { register_altraw_class::<AltRawClass>() });
    ALTLIST.get_or_init(|| unsafe { register_altlist_class::<AltListClass>() });
}

/// Initialize and register all built-in ALTREP classes.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_altrep_init() {
    unsafe { ensure_classes() };
}

// ========= Public constructors =========

/// Create an INT ALTREP from a trait object.
/// # Safety
/// Call only when R is initialized and from the R main thread.
/// The provided backend must remain valid for the lifetime of the ALTREP object.
pub unsafe fn new_altrep_int(b: Box<dyn IntBackend>) -> SEXP {
    unsafe { ensure_classes() };
    // Double-box: Box<dyn Trait> is a fat pointer, R's external pointer only holds thin pointers
    let ep = unsafe { make_eptr(Box::new(b), int_finalizer) };
    unsafe { R_new_altrep(*ALTINT.get().unwrap(), ep, R_NilValue) }
}
/// Create a REAL ALTREP from a trait object.
/// # Safety
/// Call only when R is initialized and from the R main thread.
/// The provided backend must remain valid for the lifetime of the ALTREP object.
pub unsafe fn new_altrep_real(b: Box<dyn RealBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), real_finalizer) };
    unsafe { R_new_altrep(*ALTREAL.get().unwrap(), ep, R_NilValue) }
}
/// Create a STRING ALTREP from a trait object.
/// # Safety
/// Call only when R is initialized and from the R main thread.
/// The provided backend must remain valid for the lifetime of the ALTREP object.
pub unsafe fn new_altrep_str(b: Box<dyn StringBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), str_finalizer) };
    unsafe { R_new_altrep(*ALTSTR.get().unwrap(), ep, R_NilValue) }
}

/// Create a LOGICAL ALTREP from a trait object.
/// # Safety
/// Call only when R is initialized and from the R main thread.
/// The provided backend must remain valid for the lifetime of the ALTREP object.
pub unsafe fn new_altrep_lgl(b: Box<dyn LogicalBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), lgl_finalizer) };
    unsafe { R_new_altrep(*ALTLOG.get().unwrap(), ep, R_NilValue) }
}
/// Create a RAW ALTREP from a trait object.
/// # Safety
/// Call only when R is initialized and from the R main thread.
/// The provided backend must remain valid for the lifetime of the ALTREP object.
pub unsafe fn new_altrep_raw(b: Box<dyn RawBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), raw_finalizer) };
    unsafe { R_new_altrep(*ALTRAW.get().unwrap(), ep, R_NilValue) }
}
/// Create a LIST ALTREP from a trait object.
/// # Safety
/// Call only when R is initialized and from the R main thread.
/// The provided backend must remain valid for the lifetime of the ALTREP object.
pub unsafe fn new_altrep_list(b: Box<dyn ListBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), list_finalizer) };
    unsafe { R_new_altrep(*ALTLIST.get().unwrap(), ep, R_NilValue) }
}

// Convenience constructors for stock backends
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_int_from_vec(v: Vec<i32>) -> SEXP {
    unsafe { new_altrep_int(Box::new(IntVec::from(v))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_int_from_arc(a: Arc<[i32]>) -> SEXP {
    unsafe { new_altrep_int(Box::new(IntArc::from(a))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_int_from_slice_static(s: &'static [i32]) -> SEXP {
    unsafe { new_altrep_int(Box::new(IntSliceMat::new(s))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_int_from_mmap(
    ptr: *const i32,
    len: usize,
    cleanup: Option<unsafe extern "C-unwind" fn(*const i32, usize)>,
) -> SEXP {
    unsafe { new_altrep_int(Box::new(IntMmap::new(ptr, len, cleanup))) }
}

/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_real_from_vec(v: Vec<f64>) -> SEXP {
    unsafe { new_altrep_real(Box::new(RealVec::from(v))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_real_from_arc(a: Arc<[f64]>) -> SEXP {
    unsafe { new_altrep_real(Box::new(RealArc::from(a))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_real_from_slice_static(s: &'static [f64]) -> SEXP {
    unsafe { new_altrep_real(Box::new(RealSliceMat::new(s))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_real_from_mmap(
    ptr: *const f64,
    len: usize,
    cleanup: Option<unsafe extern "C-unwind" fn(*const f64, usize)>,
) -> SEXP {
    unsafe { new_altrep_real(Box::new(RealMmap::new(ptr, len, cleanup))) }
}

/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_str_from_vec(v: Vec<String>) -> SEXP {
    let na = vec![false; v.len()];
    unsafe { new_altrep_str(Box::new(Utf8Vec { data: v, na })) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_str_from_arc(a: Arc<[String]>) -> SEXP {
    unsafe { new_altrep_str(Box::new(Utf8Arc::from(a))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_str_from_slice_static(s: &'static [&'static str]) -> SEXP {
    unsafe { new_altrep_str(Box::new(Utf8Slice::new(s))) }
}

/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_lgl_from_vec(v: Vec<i32>) -> SEXP {
    unsafe { new_altrep_lgl(Box::new(LogicalVec::from(v))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_lgl_from_arc(a: Arc<[i32]>) -> SEXP {
    unsafe { new_altrep_lgl(Box::new(LogicalArc::from(a))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_lgl_from_slice_static(s: &'static [i32]) -> SEXP {
    unsafe { new_altrep_lgl(Box::new(LogicalSliceMat::new(s))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_lgl_from_mmap(
    ptr: *const i32,
    len: usize,
    cleanup: Option<unsafe extern "C-unwind" fn(*const i32, usize)>,
) -> SEXP {
    unsafe { new_altrep_lgl(Box::new(LogicalMmap::new(ptr, len, cleanup))) }
}

/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_raw_from_vec(v: Vec<Rbyte>) -> SEXP {
    unsafe { new_altrep_raw(Box::new(RawVec::from(v))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_raw_from_arc(a: Arc<[Rbyte]>) -> SEXP {
    unsafe { new_altrep_raw(Box::new(RawArc::from(a))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_raw_from_slice_static(s: &'static [Rbyte]) -> SEXP {
    unsafe { new_altrep_raw(Box::new(RawSliceMat::new(s))) }
}
/// # Safety
/// Call only when R is initialized and from the R main thread.
pub unsafe fn new_altrep_raw_from_mmap(
    ptr: *const Rbyte,
    len: usize,
    cleanup: Option<unsafe extern "C-unwind" fn(*const Rbyte, usize)>,
) -> SEXP {
    unsafe { new_altrep_raw(Box::new(RawMmap::new(ptr, len, cleanup))) }
}

// ========= Standard backends re-exported from altrep_std_impls =========
pub use crate::altrep_std_impls::*;

// ========= R-callable C wrappers (no macros, pure .Call) =========

#[unsafe(no_mangle)]
/// # Safety
/// Must be called by R with valid SEXP arguments. Panics or errors
/// in this function must not unwind across the FFI boundary.
pub unsafe extern "C-unwind" fn C_altrep_compact_int(n_: SEXP, start_: SEXP, step_: SEXP) -> SEXP {
    // Expect INTSXP scalars; read via DATAPTR_RO
    let n: i32 = unsafe { *DATAPTR_RO(n_).cast() };
    let start: i32 = unsafe { *DATAPTR_RO(start_).cast() };
    let step: i32 = unsafe { *DATAPTR_RO(step_).cast() };
    unsafe { new_altrep_int(Box::new(CompactIntSeq::new(n as R_xlen_t, start, step))) }
}

#[unsafe(no_mangle)]
/// # Safety
/// Must be called by R with a REALSXP `x` value; must not unwind across FFI.
pub unsafe extern "C-unwind" fn C_altrep_from_doubles(x: SEXP) -> SEXP {
    let b = unsafe { OwnedReal::from_reals_sexp(x) };
    unsafe { new_altrep_real(Box::new(b)) }
}

#[unsafe(no_mangle)]
/// # Safety
/// Must be called by R with a STRSXP `x` value; must not unwind across FFI.
pub unsafe extern "C-unwind" fn C_altrep_from_strings(x: SEXP) -> SEXP {
    let b = unsafe { Utf8Vec::from_strs_sexp(x) };
    unsafe { new_altrep_str(Box::new(b)) }
}

#[unsafe(no_mangle)]
/// # Safety
/// Must be called by R with a LGLSXP `x` value; must not unwind across FFI.
pub unsafe extern "C-unwind" fn C_altrep_from_logicals(x: SEXP) -> SEXP {
    let b = unsafe { OwnedLogical::from_lgls_sexp(x) };
    unsafe { new_altrep_lgl(Box::new(b)) }
}

#[unsafe(no_mangle)]
/// # Safety
/// Must be called by R with a RAWSXP `x` value; must not unwind across FFI.
pub unsafe extern "C-unwind" fn C_altrep_from_raw(x: SEXP) -> SEXP {
    let b = unsafe { OwnedRaw::from_raw_sexp(x) };
    unsafe { new_altrep_raw(Box::new(b)) }
}

#[unsafe(no_mangle)]
/// # Safety
/// Must be called by R with a VECSXP `x` value; must not unwind across FFI.
pub unsafe extern "C-unwind" fn C_altrep_from_list(x: SEXP) -> SEXP {
    let b = unsafe { OwnedList::from_list_sexp(x) };
    unsafe { new_altrep_list(Box::new(b)) }
}

#[repr(C)]
pub struct RAltrepClass {
    pub ptr: SEXP,
} // mirrors R_altrep_class_t { SEXP ptr; }
// pub type R_xlen_t = isize; // or i64 depending on your libR-sys cfg
// // ... similarly AltLogical, AltRaw, AltComplex, AltString, AltList
#[derive(Copy, Clone, Debug)]
pub enum RBase {
    Int,
    Real,
    Logical,
    Raw,
    Complex,
    String,
    List,
}

/// Base spec every ALTREP class must provide.
#[allow(clippy::missing_safety_doc)]
pub trait AltrepClass {
    const CLASS_NAME: &'static std::ffi::CStr;
    const PKG_NAME: &'static std::ffi::CStr;
    const BASE: RBase;

    /// Called to compute Length(x).
    unsafe fn length(x: SEXP) -> R_xlen_t;

    /// Optional: serialization hooks.
    unsafe fn serialized_state(_x: SEXP) -> Option<SEXP> {
        None
    }
    unsafe fn unserialize_ex(
        _class: RAltrepClass,
        _state: SEXP,
        _attr: SEXP,
        _objf: i32,
        _levs: i32,
    ) -> Option<SEXP> {
        None
    }
    unsafe fn unserialize(_class: RAltrepClass, _state: SEXP) -> Option<SEXP> {
        None
    }

    /// Optional: Duplicate/Coerce/Inspect hooks
    unsafe fn duplicate(_x: SEXP, _deep: bool) -> Option<SEXP> {
        None
    }
    unsafe fn duplicate_ex(x: SEXP, deep: bool) -> Option<SEXP> {
        // default: delegate to duplicate()
        unsafe { Self::duplicate(x, deep) }
    }
    unsafe fn coerce(_x: SEXP, _to_type: i32) -> Option<SEXP> {
        None
    }
    unsafe fn inspect(_x: SEXP, _pre: i32, _deep: i32, _pvec: i32) -> bool {
        false
    }
}

// Vector-level hooks.
// Old Alt* trait scaffolding has been replaced by safe traits in `altrep_traits`.

// AltComplex intentionally omitted for now: FFI method types are not exposed.

// Local helper macro to set a method only if a feature flag is true.
macro_rules! set_if {
    ($cond:expr, $setter:path, $tramp:expr, $cls:expr) => {
        if $cond {
            unsafe { $setter($cls, Some($tramp)) };
        }
    };
}

/// Register an ALTREP class for integer vectors backed by `T`.
/// # Safety
/// Registers callbacks with the R ALTREP system; must be called with R initialized.
pub unsafe fn register_altinteger_class<T: AltrepClass + traits::AltVec + traits::AltInteger>()
-> R_altrep_class_t {
    let cls = unsafe {
        R_make_altinteger_class(
            T::CLASS_NAME.as_ptr(),
            T::PKG_NAME.as_ptr(),
            core::ptr::null_mut(),
        )
    };
    {
        use crate::altrep_bridge as bridge;
        use crate::ffi::altrep::*;
        // Base (Length, Duplicate, Inspect, Serialization)
        set_if!(
            <T as traits::Altrep>::HAS_LENGTH,
            R_set_altrep_Length_method,
            bridge::t_length::<T>,
            cls
        );
        set_if!(
            <T as traits::Altrep>::HAS_DUPLICATE,
            R_set_altrep_Duplicate_method,
            bridge::t_duplicate::<T>,
            cls
        );
        set_if!(
            <T as traits::Altrep>::HAS_INSPECT,
            R_set_altrep_Inspect_method,
            bridge::t_inspect::<T>,
            cls
        );
        set_if!(
            <T as traits::Altrep>::HAS_SERIALIZED_STATE,
            R_set_altrep_Serialized_state_method,
            bridge::t_serialized_state::<T>,
            cls
        );
        set_if!(
            <T as traits::Altrep>::HAS_UNSERIALIZE,
            R_set_altrep_Unserialize_method,
            bridge::t_unserialize::<T>,
            cls
        );
        // Vec
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR,
            R_set_altvec_Dataptr_method,
            bridge::t_dataptr::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR_OR_NULL,
            R_set_altvec_Dataptr_or_null_method,
            bridge::t_dataptr_or_null::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_EXTRACT_SUBSET,
            R_set_altvec_Extract_subset_method,
            bridge::t_extract_subset::<T>,
            cls
        );
        // Int family
        set_if!(
            <T as traits::AltInteger>::HAS_ELT,
            R_set_altinteger_Elt_method,
            bridge::t_int_elt::<T>,
            cls
        );
        set_if!(
            <T as traits::AltInteger>::HAS_GET_REGION,
            R_set_altinteger_Get_region_method,
            bridge::t_int_get_region::<T>,
            cls
        );
        set_if!(
            <T as traits::AltInteger>::HAS_IS_SORTED,
            R_set_altinteger_Is_sorted_method,
            bridge::t_int_is_sorted::<T>,
            cls
        );
        set_if!(
            <T as traits::AltInteger>::HAS_NO_NA,
            R_set_altinteger_No_NA_method,
            bridge::t_int_no_na::<T>,
            cls
        );
        set_if!(
            <T as traits::AltInteger>::HAS_SUM,
            R_set_altinteger_Sum_method,
            bridge::t_int_sum::<T>,
            cls
        );
        set_if!(
            <T as traits::AltInteger>::HAS_MIN,
            R_set_altinteger_Min_method,
            bridge::t_int_min::<T>,
            cls
        );
        set_if!(
            <T as traits::AltInteger>::HAS_MAX,
            R_set_altinteger_Max_method,
            bridge::t_int_max::<T>,
            cls
        );
    }
    cls
}

/// Register an ALTREP class for real vectors backed by `T`.
/// # Safety
/// Registers callbacks with the R ALTREP system; must be called with R initialized.
pub unsafe fn register_altreal_class<T: AltrepClass + traits::AltVec + traits::AltReal>()
-> R_altrep_class_t {
    let cls = unsafe {
        R_make_altreal_class(
            T::CLASS_NAME.as_ptr(),
            T::PKG_NAME.as_ptr(),
            core::ptr::null_mut(),
        )
    };
    {
        use crate::altrep_bridge as bridge;
        use crate::ffi::altrep::*;
        // Base
        set_if!(
            <T as traits::Altrep>::HAS_LENGTH,
            R_set_altrep_Length_method,
            bridge::t_length::<T>,
            cls
        );
        // Vec
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR,
            R_set_altvec_Dataptr_method,
            bridge::t_dataptr::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR_OR_NULL,
            R_set_altvec_Dataptr_or_null_method,
            bridge::t_dataptr_or_null::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_EXTRACT_SUBSET,
            R_set_altvec_Extract_subset_method,
            bridge::t_extract_subset::<T>,
            cls
        );
        // Real family
        set_if!(
            <T as traits::AltReal>::HAS_ELT,
            R_set_altreal_Elt_method,
            bridge::t_real_elt::<T>,
            cls
        );
        set_if!(
            <T as traits::AltReal>::HAS_GET_REGION,
            R_set_altreal_Get_region_method,
            bridge::t_real_get_region::<T>,
            cls
        );
        set_if!(
            <T as traits::AltReal>::HAS_IS_SORTED,
            R_set_altreal_Is_sorted_method,
            bridge::t_real_is_sorted::<T>,
            cls
        );
        set_if!(
            <T as traits::AltReal>::HAS_NO_NA,
            R_set_altreal_No_NA_method,
            bridge::t_real_no_na::<T>,
            cls
        );
        set_if!(
            <T as traits::AltReal>::HAS_SUM,
            R_set_altreal_Sum_method,
            bridge::t_real_sum::<T>,
            cls
        );
        set_if!(
            <T as traits::AltReal>::HAS_MIN,
            R_set_altreal_Min_method,
            bridge::t_real_min::<T>,
            cls
        );
        set_if!(
            <T as traits::AltReal>::HAS_MAX,
            R_set_altreal_Max_method,
            bridge::t_real_max::<T>,
            cls
        );
    }
    cls
}

/// Register an ALTREP class for logical vectors backed by `T`.
/// # Safety
/// Registers callbacks with the R ALTREP system; must be called with R initialized.
pub unsafe fn register_altlogical_class<T: AltrepClass + traits::AltVec + traits::AltLogical>()
-> R_altrep_class_t {
    let cls = unsafe {
        R_make_altlogical_class(
            T::CLASS_NAME.as_ptr(),
            T::PKG_NAME.as_ptr(),
            core::ptr::null_mut(),
        )
    };
    {
        use crate::altrep_bridge as bridge;
        use crate::ffi::altrep::*;
        // Base
        set_if!(
            <T as traits::Altrep>::HAS_LENGTH,
            R_set_altrep_Length_method,
            bridge::t_length::<T>,
            cls
        );
        // Vec
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR,
            R_set_altvec_Dataptr_method,
            bridge::t_dataptr::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR_OR_NULL,
            R_set_altvec_Dataptr_or_null_method,
            bridge::t_dataptr_or_null::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_EXTRACT_SUBSET,
            R_set_altvec_Extract_subset_method,
            bridge::t_extract_subset::<T>,
            cls
        );
        // Logical family
        set_if!(
            <T as traits::AltLogical>::HAS_ELT,
            R_set_altlogical_Elt_method,
            bridge::t_lgl_elt::<T>,
            cls
        );
        set_if!(
            <T as traits::AltLogical>::HAS_GET_REGION,
            R_set_altlogical_Get_region_method,
            bridge::t_lgl_get_region::<T>,
            cls
        );
        set_if!(
            <T as traits::AltLogical>::HAS_IS_SORTED,
            R_set_altlogical_Is_sorted_method,
            bridge::t_lgl_is_sorted::<T>,
            cls
        );
        set_if!(
            <T as traits::AltLogical>::HAS_NO_NA,
            R_set_altlogical_No_NA_method,
            bridge::t_lgl_no_na::<T>,
            cls
        );
    }
    cls
}

/// Register an ALTREP class for raw vectors backed by `T`.
/// # Safety
/// Registers callbacks with the R ALTREP system; must be called with R initialized.
pub unsafe fn register_altraw_class<T: AltrepClass + traits::AltVec + traits::AltRaw>()
-> R_altrep_class_t {
    let cls = unsafe {
        R_make_altraw_class(
            T::CLASS_NAME.as_ptr(),
            T::PKG_NAME.as_ptr(),
            core::ptr::null_mut(),
        )
    };
    {
        use crate::altrep_bridge as bridge;
        use crate::ffi::altrep::*;
        // Base
        set_if!(
            <T as traits::Altrep>::HAS_LENGTH,
            R_set_altrep_Length_method,
            bridge::t_length::<T>,
            cls
        );
        // Vec
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR,
            R_set_altvec_Dataptr_method,
            bridge::t_dataptr::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR_OR_NULL,
            R_set_altvec_Dataptr_or_null_method,
            bridge::t_dataptr_or_null::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_EXTRACT_SUBSET,
            R_set_altvec_Extract_subset_method,
            bridge::t_extract_subset::<T>,
            cls
        );
        // Raw family
        set_if!(
            <T as traits::AltRaw>::HAS_ELT,
            R_set_altraw_Elt_method,
            bridge::t_raw_elt::<T>,
            cls
        );
        set_if!(
            <T as traits::AltRaw>::HAS_GET_REGION,
            R_set_altraw_Get_region_method,
            bridge::t_raw_get_region::<T>,
            cls
        );
    }
    cls
}

/// Register an ALTREP class for string vectors backed by `T`.
/// # Safety
/// Registers callbacks with the R ALTREP system; must be called with R initialized.
pub unsafe fn register_altstring_class<T: AltrepClass + traits::AltVec + traits::AltString>()
-> R_altrep_class_t {
    let cls = unsafe {
        R_make_altstring_class(
            T::CLASS_NAME.as_ptr(),
            T::PKG_NAME.as_ptr(),
            core::ptr::null_mut(),
        )
    };
    {
        use crate::altrep_bridge as bridge;
        use crate::ffi::altrep::*;
        // Base
        set_if!(
            <T as traits::Altrep>::HAS_LENGTH,
            R_set_altrep_Length_method,
            bridge::t_length::<T>,
            cls
        );
        // Vec
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR,
            R_set_altvec_Dataptr_method,
            bridge::t_dataptr::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR_OR_NULL,
            R_set_altvec_Dataptr_or_null_method,
            bridge::t_dataptr_or_null::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_EXTRACT_SUBSET,
            R_set_altvec_Extract_subset_method,
            bridge::t_extract_subset::<T>,
            cls
        );
        // String family
        set_if!(
            <T as traits::AltString>::HAS_ELT,
            R_set_altstring_Elt_method,
            bridge::t_str_elt::<T>,
            cls
        );
        set_if!(
            <T as traits::AltString>::HAS_IS_SORTED,
            R_set_altstring_Is_sorted_method,
            bridge::t_str_is_sorted::<T>,
            cls
        );
        set_if!(
            <T as traits::AltString>::HAS_NO_NA,
            R_set_altstring_No_NA_method,
            bridge::t_str_no_na::<T>,
            cls
        );
        set_if!(
            <T as traits::AltString>::HAS_SET_ELT,
            R_set_altstring_Set_elt_method,
            bridge::t_str_set_elt::<T>,
            cls
        );
    }
    cls
}

/// Register an ALTREP class for generic lists (VECSXP) backed by `T`.
/// # Safety
/// Registers callbacks with the R ALTREP system; must be called with R initialized.
pub unsafe fn register_altlist_class<T: AltrepClass + traits::AltVec + traits::AltList>()
-> R_altrep_class_t {
    let cls = unsafe {
        R_make_altlist_class(
            T::CLASS_NAME.as_ptr(),
            T::PKG_NAME.as_ptr(),
            core::ptr::null_mut(),
        )
    };
    {
        use crate::altrep_bridge as bridge;
        use crate::ffi::altrep::*;
        // Base
        set_if!(
            <T as traits::Altrep>::HAS_LENGTH,
            R_set_altrep_Length_method,
            bridge::t_length::<T>,
            cls
        );
        // Vec
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR,
            R_set_altvec_Dataptr_method,
            bridge::t_dataptr::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_DATAPTR_OR_NULL,
            R_set_altvec_Dataptr_or_null_method,
            bridge::t_dataptr_or_null::<T>,
            cls
        );
        set_if!(
            <T as traits::AltVec>::HAS_EXTRACT_SUBSET,
            R_set_altvec_Extract_subset_method,
            bridge::t_extract_subset::<T>,
            cls
        );
        // List family
        set_if!(
            <T as traits::AltList>::HAS_ELT,
            R_set_altlist_Elt_method,
            bridge::t_list_elt::<T>,
            cls
        );
        set_if!(
            <T as traits::AltList>::HAS_SET_ELT,
            R_set_altlist_Set_elt_method,
            bridge::t_list_set_elt::<T>,
            cls
        );
    }
    cls
}

// ========= Built-in class adapters using dynamic Backends =========

struct AltIntClass;
impl AltrepClass for AltIntClass {
    const CLASS_NAME: &'static std::ffi::CStr = c"rust_altint";
    const PKG_NAME: &'static std::ffi::CStr = c"miniextendr";
    const BASE: RBase = RBase::Int;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { int_backend(x).len() }
    }
}
impl traits::Altrep for AltIntClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { int_backend(x).len() }
    }

    const HAS_SERIALIZED_STATE: bool = true;
    fn serialized_state(x: SEXP) -> SEXP {
        // If backend supports compact serialization, return [len, start, step]
        // Otherwise return NULL to let R materialize and serialize normally
        let b = unsafe { int_backend(x) };
        if let Some((len, start, step)) = b.as_compact_seq() {
            // Only serialize compactly if len fits in i32 (most cases)
            if len <= i32::MAX as R_xlen_t && len >= i32::MIN as R_xlen_t {
                unsafe {
                    let state = Rf_allocVector(SEXPTYPE::INTSXP, 3);
                    Rf_protect(state);
                    let p = INTEGER(state);
                    *p = len as i32;
                    *p.add(1) = start;
                    *p.add(2) = step;
                    Rf_unprotect(1);
                    return state;
                }
            }
        }
        // Return NULL to use default materialized serialization
        core::ptr::null_mut()
    }

    const HAS_UNSERIALIZE: bool = true;
    fn unserialize(_class: SEXP, state: SEXP) -> SEXP {
        // Reconstruct CompactIntSeq from serialized state [len, start, step]
        unsafe {
            let p = DATAPTR_RO(state).cast::<i32>();
            let len = *p as R_xlen_t;
            let start = *p.add(1);
            let step = *p.add(2);
            new_altrep_int(Box::new(CompactIntSeq::new(len, start, step)))
        }
    }

    const HAS_DUPLICATE: bool = true;
    fn duplicate(x: SEXP, deep: bool) -> SEXP {
        // If deep copy requested or already expanded, materialize
        unsafe {
            let expanded = R_altrep_data2(x);
            if deep || expanded != R_NilValue {
                // Materialize: allocate and copy
                let n = int_backend(x).len();
                let val = Rf_allocVector(SEXPTYPE::INTSXP, n);
                Rf_protect(val);
                let buf = INTEGER(val);
                int_backend(x).get_region(0, n, slice::from_raw_parts_mut(buf, n as usize));
                Rf_unprotect(1);
                val
            } else {
                // Return NULL to let R use default duplication
                core::ptr::null_mut()
            }
        }
    }

    const HAS_INSPECT: bool = true;
    fn inspect(
        x: SEXP,
        _pre: i32,
        _deep: i32,
        _pvec: i32,
        _inspect_subtree: Option<unsafe extern "C-unwind" fn(SEXP, i32, i32, i32)>,
    ) -> bool {
        unsafe {
            let n = int_backend(x).len();
            let expanded = R_altrep_data2(x);
            let status = if expanded == R_NilValue {
                c"compact"
            } else {
                c"expanded"
            };
            // Print info using Rprintf
            let fmt = c" rust_altint [len=%ld, %s]\n".as_ptr();
            Rprintf_unchecked(fmt, n as std::os::raw::c_long, status.as_ptr());
        }
        true
    }
}
impl traits::AltVec for AltIntClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    const HAS_EXTRACT_SUBSET: bool = true;
    fn extract_subset(x: SEXP, indx: SEXP, _call: SEXP) -> SEXP {
        // Check if indx is an integer vector representing a contiguous range
        // e.g., c(5, 6, 7, 8, 9) -> can extract compact subset
        unsafe {
            let idx_type = indx.type_of();
            // Only handle integer indices for now
            if idx_type != SEXPTYPE::INTSXP {
                return core::ptr::null_mut();
            }

            let idx_len = indx.len();
            if idx_len == 0 {
                // Empty subset - return NULL to let R handle
                return core::ptr::null_mut();
            }

            // Get pointer to index data
            let idx_ptr = INTEGER_OR_NULL(indx);
            if idx_ptr.is_null() {
                return core::ptr::null_mut();
            }

            // Check if indices form a contiguous range starting at first[0]
            let first_idx = *idx_ptr;
            if first_idx <= 0 {
                // Negative or zero indices - let R handle
                return core::ptr::null_mut();
            }

            // Check for NA in first element
            if first_idx == i32::MIN {
                // NA_INTEGER
                return core::ptr::null_mut();
            }

            // Verify it's a contiguous increasing sequence: first, first+1, first+2, ...
            for i in 1..idx_len {
                let expected = first_idx.wrapping_add(i as i32);
                let actual = *idx_ptr.add(i as usize);
                if actual != expected {
                    return core::ptr::null_mut();
                }
            }

            // It's contiguous! Try to extract a compact subset
            let b = int_backend(x);
            // Convert to 0-based index
            let start_0based = (first_idx - 1) as R_xlen_t;
            if let Some(new_backend) = b.extract_contiguous(start_0based, idx_len as R_xlen_t) {
                return new_altrep_int(new_backend);
            }
        }
        // Fall back to default extraction
        core::ptr::null_mut()
    }
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        // Materialize the data if not already expanded
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = int_backend(x).len();
                let val = Rf_allocVector(SEXPTYPE::INTSXP, n);
                Rf_protect(val);
                // Fill using get_region
                let buf = INTEGER(val);
                int_backend(x).get_region(0, n, slice::from_raw_parts_mut(buf, n as usize));
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                INTEGER(expanded).cast()
            }
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                INTEGER(expanded).cast()
            }
        }
    }
}
impl traits::AltInteger for AltIntClass {
    const HAS_ELT: bool = true;
    const HAS_GET_REGION: bool = true;
    const HAS_IS_SORTED: bool = true;
    const HAS_NO_NA: bool = true;
    const HAS_SUM: bool = true;
    const HAS_MIN: bool = true;
    const HAS_MAX: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        unsafe { int_backend(x).elt(i) }
    }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { int_backend(x).get_region(i, n, out) }
    }
    fn is_sorted(x: SEXP) -> i32 {
        unsafe { int_backend(x).is_sorted() }
    }
    fn no_na(x: SEXP) -> i32 {
        unsafe { int_backend(x).no_na() }
    }
    fn sum(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { int_backend(x) };
        // Return NULL (not R_NilValue!) to signal "use default fallback".
        // See module docs for NULL vs R_NilValue semantics.
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut();
        }
        // Try O(1) backend optimization first (e.g., arithmetic series formula)
        if let Some(sum) = b.sum() {
            return unsafe { Rf_ScalarReal(sum) };
        }
        // Fall back to O(n) iteration
        let mut acc: i64 = 0;
        let n = b.len();
        for i in 0..n {
            acc = acc.wrapping_add(b.elt(i) as i64);
        }
        unsafe { Rf_ScalarReal(acc as f64) }
    }
    fn min(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { int_backend(x) };
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut();
        }
        // Try O(1) backend optimization first
        if let Some(m) = b.min() {
            return unsafe { Rf_ScalarInteger(m) };
        }
        // Fall back to O(n) iteration
        let n = b.len();
        if n == 0 {
            return core::ptr::null_mut();
        }
        let mut m = b.elt(0);
        for i in 1..n {
            let v = b.elt(i);
            if v < m {
                m = v;
            }
        }
        unsafe { Rf_ScalarInteger(m) }
    }
    fn max(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { int_backend(x) };
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut();
        }
        // Try O(1) backend optimization first
        if let Some(m) = b.max() {
            return unsafe { Rf_ScalarInteger(m) };
        }
        // Fall back to O(n) iteration
        let n = b.len();
        if n == 0 {
            return core::ptr::null_mut();
        }
        let mut m = b.elt(0);
        for i in 1..n {
            let v = b.elt(i);
            if v > m {
                m = v;
            }
        }
        unsafe { Rf_ScalarInteger(m) }
    }
}

// RegisterAltrep is provided via blanket impls in altrep_registration.rs

struct AltRealClass;
impl AltrepClass for AltRealClass {
    const CLASS_NAME: &'static std::ffi::CStr = c"rust_altreal";
    const PKG_NAME: &'static std::ffi::CStr = c"miniextendr";
    const BASE: RBase = RBase::Real;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { real_backend(x).len() }
    }
}
impl traits::Altrep for AltRealClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { real_backend(x).len() }
    }
}
impl traits::AltVec for AltRealClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        // Materialize the data if not already expanded
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = real_backend(x).len();
                let val = Rf_allocVector(SEXPTYPE::REALSXP, n);
                Rf_protect(val);
                let buf = REAL(val);
                real_backend(x).get_region(0, n, slice::from_raw_parts_mut(buf, n as usize));
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                REAL(expanded).cast()
            }
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                REAL(expanded).cast()
            }
        }
    }
}
impl traits::AltReal for AltRealClass {
    const HAS_ELT: bool = true;
    const HAS_GET_REGION: bool = true;
    const HAS_IS_SORTED: bool = true;
    const HAS_NO_NA: bool = true;
    const HAS_SUM: bool = true;
    const HAS_MIN: bool = true;
    const HAS_MAX: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> f64 {
        unsafe { real_backend(x).elt(i) }
    }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut f64) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { real_backend(x).get_region(i, n, out) }
    }
    fn is_sorted(x: SEXP) -> i32 {
        unsafe { real_backend(x).is_sorted() }
    }
    fn no_na(x: SEXP) -> i32 {
        unsafe { real_backend(x).no_na() }
    }
    fn sum(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { real_backend(x) };
        // Only use fast path if no NA handling needed
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut(); // Let R handle it
        }
        let mut acc = 0.0;
        let n = b.len();
        for i in 0..n {
            acc += b.elt(i);
        }
        unsafe { Rf_ScalarReal(acc) }
    }
    fn min(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { real_backend(x) };
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut();
        }
        let n = b.len();
        if n == 0 {
            return core::ptr::null_mut();
        }
        let mut m = b.elt(0);
        for i in 1..n {
            let v = b.elt(i);
            if v < m {
                m = v;
            }
        }
        unsafe { Rf_ScalarReal(m) }
    }
    fn max(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { real_backend(x) };
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut();
        }
        let n = b.len();
        if n == 0 {
            return core::ptr::null_mut();
        }
        let mut m = b.elt(0);
        for i in 1..n {
            let v = b.elt(i);
            if v > m {
                m = v;
            }
        }
        unsafe { Rf_ScalarReal(m) }
    }
}

// RegisterAltrep via blanket impl

struct AltStrClass;
impl AltrepClass for AltStrClass {
    const CLASS_NAME: &'static std::ffi::CStr = c"rust_altstr";
    const PKG_NAME: &'static std::ffi::CStr = c"miniextendr";
    const BASE: RBase = RBase::String;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { str_backend(x).len() }
    }
}
impl traits::Altrep for AltStrClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { str_backend(x).len() }
    }
}
impl traits::AltVec for AltStrClass {}

impl traits::AltString for AltStrClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        match unsafe { str_backend(x).utf8_at(i) } {
            None => unsafe { R_NaString },
            Some(s) => {
                let cs = std::ffi::CString::new(s).unwrap();
                unsafe { Rf_mkCharLenCE(cs.as_ptr(), s.len() as i32, CE_UTF8) }
            }
        }
    }
}

// RegisterAltrep via blanket impl

struct AltLogicalClass;
impl AltrepClass for AltLogicalClass {
    const CLASS_NAME: &'static std::ffi::CStr = c"rust_altlgl";
    const PKG_NAME: &'static std::ffi::CStr = c"miniextendr";
    const BASE: RBase = RBase::Logical;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { lgl_backend(x).len() }
    }
}
impl traits::Altrep for AltLogicalClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { lgl_backend(x).len() }
    }
}
impl traits::AltVec for AltLogicalClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        // Materialize the data if not already expanded
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = lgl_backend(x).len();
                let val = Rf_allocVector(SEXPTYPE::LGLSXP, n);
                Rf_protect(val);
                let buf = LOGICAL(val);
                lgl_backend(x).get_region(0, n, slice::from_raw_parts_mut(buf, n as usize));
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                LOGICAL(expanded).cast()
            }
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                LOGICAL(expanded).cast()
            }
        }
    }
}
impl traits::AltLogical for AltLogicalClass {
    const HAS_ELT: bool = true;
    const HAS_GET_REGION: bool = true;
    const HAS_IS_SORTED: bool = true;
    const HAS_NO_NA: bool = true;
    const HAS_SUM: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        unsafe { lgl_backend(x).elt(i) }
    }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { lgl_backend(x).get_region(i, n, out) }
    }
    fn is_sorted(x: SEXP) -> i32 {
        unsafe { lgl_backend(x).is_sorted() }
    }
    fn no_na(x: SEXP) -> i32 {
        unsafe { lgl_backend(x).no_na() }
    }
    fn sum(x: SEXP, narm: bool) -> SEXP {
        let b = unsafe { lgl_backend(x) };
        // Only use fast path if no NA handling needed
        if narm || b.no_na() == 0 {
            return core::ptr::null_mut(); // Let R handle it
        }
        let mut acc: i64 = 0;
        let n = b.len();
        for i in 0..n {
            acc += b.elt(i) as i64;
        }
        // Logical sum returns integer
        unsafe { Rf_ScalarInteger(acc as i32) }
    }
}

// RegisterAltrep via blanket impl

struct AltRawClass;
impl AltrepClass for AltRawClass {
    const CLASS_NAME: &'static std::ffi::CStr = c"rust_altraw";
    const PKG_NAME: &'static std::ffi::CStr = c"miniextendr";
    const BASE: RBase = RBase::Raw;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { raw_backend(x).len() }
    }
}
impl traits::Altrep for AltRawClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { raw_backend(x).len() }
    }
}
impl traits::AltVec for AltRawClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        // Materialize the data if not already expanded
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                let n = raw_backend(x).len();
                let val = Rf_allocVector(SEXPTYPE::RAWSXP, n);
                Rf_protect(val);
                let buf = RAW(val);
                raw_backend(x).get_region(0, n, slice::from_raw_parts_mut(buf, n as usize));
                R_set_altrep_data2(x, val);
                Rf_unprotect(1);
                buf.cast()
            } else {
                RAW(expanded).cast()
            }
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            let expanded = R_altrep_data2(x);
            if expanded == R_NilValue {
                core::ptr::null()
            } else {
                RAW(expanded).cast()
            }
        }
    }
}
impl traits::AltRaw for AltRawClass {
    const HAS_ELT: bool = true;
    const HAS_GET_REGION: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> Rbyte {
        unsafe { raw_backend(x).elt(i) }
    }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut Rbyte) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { raw_backend(x).get_region(i, n, out) }
    }
}

// RegisterAltrep via blanket impl

struct AltListClass;
impl AltrepClass for AltListClass {
    const CLASS_NAME: &'static std::ffi::CStr = c"rust_altlist";
    const PKG_NAME: &'static std::ffi::CStr = c"miniextendr";
    const BASE: RBase = RBase::List;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { list_backend(x).len() }
    }
}
impl traits::Altrep for AltListClass {
    const HAS_LENGTH: bool = true;
    fn length(x: SEXP) -> R_xlen_t {
        unsafe { list_backend(x).len() }
    }
}
impl traits::AltVec for AltListClass {}
impl traits::AltList for AltListClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        unsafe { list_backend(x).elt(i) }
    }
}

// RegisterAltrep via blanket impl
