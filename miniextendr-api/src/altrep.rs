//! ALTREP "from scratch" core for miniextendr-api: one class per base kind
//! (INT, REAL, STRING). No libR-sys/extendr dependencies; only raw FFI.

use core::ffi::{c_char, c_void};
use core::slice;
use std::sync::OnceLock;

// Use the project's FFI definitions and types.
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
}

/// String backend — provides UTF-8. Return None for NA.
pub trait StringBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str>;
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
}

/// List backend — general VECSXP; returns owned SEXP references.
pub trait ListBackend: Send + Sync + 'static {
    fn len(&self) -> R_xlen_t;
    fn elt(&self, i: R_xlen_t) -> SEXP;
}

// -- helpers to store/retrieve Box<dyn Backend> behind an external ptr --
unsafe fn make_eptr<T: ?Sized>(b: Box<T>, fin: unsafe extern "C" fn(SEXP)) -> SEXP {
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
    unsafe { ep_as::<Box<dyn IntBackend>>(ep).as_ref() }
}

unsafe extern "C" fn int_finalizer(ep: SEXP) {
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
unsafe extern "C" fn real_finalizer(ep: SEXP) {
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
unsafe extern "C" fn str_finalizer(ep: SEXP) {
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
unsafe extern "C" fn lgl_finalizer(ep: SEXP) {
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
unsafe extern "C" fn raw_finalizer(ep: SEXP) {
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
unsafe extern "C" fn list_finalizer(ep: SEXP) {
    let raw = unsafe { R_ExternalPtrAddr(ep) };
    if !raw.is_null() {
        drop(unsafe { Box::<Box<dyn ListBackend>>::from_raw(raw.cast()) });
    }
}

// ========= Class registration =========
fn cstr(s: &str) -> *const c_char {
    std::ffi::CString::new(s).unwrap().into_raw()
}

/// Must be called once (lazy-initialized on first constructor use).
unsafe fn ensure_classes() {
    ALTINT.get_or_init(|| unsafe { register_altinteger_class::<AltIntClass>() });
    ALTREAL.get_or_init(|| unsafe { register_altreal_class::<AltRealClass>() });
    ALTSTR.get_or_init(|| unsafe { register_altstring_class::<AltStrClass>() });
    ALTLOG.get_or_init(|| unsafe { register_altlogical_class::<AltLogicalClass>() });
    ALTRAW.get_or_init(|| unsafe { register_altraw_class::<AltRawClass>() });
    ALTLIST.get_or_init(|| unsafe { register_altlist_class::<AltListClass>() });
}

// ========= Public constructors =========

/// Create an INT ALTREP from a trait object.
pub unsafe fn new_altrep_int(b: Box<dyn IntBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), int_finalizer) };
    unsafe { R_new_altrep(*ALTINT.get().unwrap(), ep, R_NilValue) }
}
/// Create a REAL ALTREP from a trait object.
pub unsafe fn new_altrep_real(b: Box<dyn RealBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), real_finalizer) };
    unsafe { R_new_altrep(*ALTREAL.get().unwrap(), ep, R_NilValue) }
}
/// Create a STRING ALTREP from a trait object.
pub unsafe fn new_altrep_str(b: Box<dyn StringBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), str_finalizer) };
    unsafe { R_new_altrep(*ALTSTR.get().unwrap(), ep, R_NilValue) }
}

/// Create a LOGICAL ALTREP from a trait object.
pub unsafe fn new_altrep_lgl(b: Box<dyn LogicalBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), lgl_finalizer) };
    unsafe { R_new_altrep(*ALTLOG.get().unwrap(), ep, R_NilValue) }
}
/// Create a RAW ALTREP from a trait object.
pub unsafe fn new_altrep_raw(b: Box<dyn RawBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), raw_finalizer) };
    unsafe { R_new_altrep(*ALTRAW.get().unwrap(), ep, R_NilValue) }
}
/// Create a LIST ALTREP from a trait object.
pub unsafe fn new_altrep_list(b: Box<dyn ListBackend>) -> SEXP {
    unsafe { ensure_classes() };
    let ep = unsafe { make_eptr(Box::new(b), list_finalizer) };
    unsafe { R_new_altrep(*ALTLIST.get().unwrap(), ep, R_NilValue) }
}

// ========= Example backends =========

/// start + i * step sequence
pub struct CompactIntSeq {
    len: R_xlen_t,
    start: i32,
    step: i32,
}
impl CompactIntSeq {
    pub fn new(len: R_xlen_t, start: i32, step: i32) -> Self {
        Self { len, start, step }
    }
}
impl IntBackend for CompactIntSeq {
    fn len(&self) -> R_xlen_t {
        self.len
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.start.wrapping_add(self.step.wrapping_mul(i as i32))
    }
    fn is_sorted(&self) -> i32 {
        if self.step < 0 { -1 } else { 1 }
    }
    fn no_na(&self) -> i32 {
        1
    }
}

/// Owned contiguous f64 buffer
pub struct OwnedReal {
    data: Box<[f64]>,
}
impl OwnedReal {
    pub fn from_reals_sexp(x: SEXP) -> Self {
        unsafe {
            let n = Rf_xlength(x);
            let ptr = DATAPTR_RO(x) as *const f64;
            let slice = slice::from_raw_parts(ptr, n as usize);
            Self {
                data: slice.to_vec().into_boxed_slice(),
            }
        }
    }
}
impl RealBackend for OwnedReal {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        Some(&self.data)
    }
    fn no_na(&self) -> i32 {
        1
    }
}

/// Owns the UTF-8 strings
pub struct Utf8Vec {
    data: Vec<String>,
}
impl Utf8Vec {
    pub fn from_strs_sexp(x: SEXP) -> Self {
        unsafe {
            unsafe extern "C" {
                fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
                fn Rf_translateCharUTF8(x: SEXP) -> *const c_char;
            }
            let n = Rf_xlength(x);
            let mut v = Vec::with_capacity(n as usize);
            for i in 0..n {
                let ch = STRING_ELT(x, i);
                if ch.is_null() {
                    v.push(String::new());
                } else {
                    let c = Rf_translateCharUTF8(ch);
                    // Safety: R returns NUL-terminated UTF-8
                    let s = std::ffi::CStr::from_ptr(c).to_string_lossy().into_owned();
                    v.push(s);
                }
            }
            Self { data: v }
        }
    }
}
impl StringBackend for Utf8Vec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str> {
        Some(self.data[i as usize].as_str())
    }
}

/// Owned contiguous i32 buffer for LOGICALSXP.
pub struct OwnedLogical {
    data: Box<[i32]>,
}
impl OwnedLogical {
    pub fn from_lgls_sexp(x: SEXP) -> Self {
        unsafe {
            let n = Rf_xlength(x) as usize;
            let ptr = LOGICAL_OR_NULL(x);
            let slice = if ptr.is_null() {
                &[]
            } else {
                core::slice::from_raw_parts(ptr, n)
            };
            Self { data: slice.to_vec().into_boxed_slice() }
        }
    }
}
impl LogicalBackend for OwnedLogical {
    fn len(&self) -> R_xlen_t { self.data.len() as R_xlen_t }
    fn elt(&self, i: R_xlen_t) -> i32 { self.data[i as usize] }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> { Some(&self.data) }
    fn no_na(&self) -> i32 { 0 }
}

/// Owned contiguous raw buffer for RAWSXP.
pub struct OwnedRaw {
    data: Box<[Rbyte]>,
}
impl OwnedRaw {
    pub fn from_raw_sexp(x: SEXP) -> Self {
        unsafe {
            let n = Rf_xlength(x) as usize;
            let ptr = DATAPTR_RO(x) as *const Rbyte;
            let slice = core::slice::from_raw_parts(ptr, n);
            Self { data: slice.to_vec().into_boxed_slice() }
        }
    }
}
impl RawBackend for OwnedRaw {
    fn len(&self) -> R_xlen_t { self.data.len() as R_xlen_t }
    fn elt(&self, i: R_xlen_t) -> Rbyte { self.data[i as usize] }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> { Some(&self.data) }
}

/// Owned list of SEXP values for VECSXP.
pub struct OwnedList {
    data: Vec<SEXP>,
}
impl OwnedList {
    pub fn from_sexps(v: Vec<SEXP>) -> Self { Self { data: v } }
}
impl ListBackend for OwnedList {
    fn len(&self) -> R_xlen_t { self.data.len() as R_xlen_t }
    fn elt(&self, i: R_xlen_t) -> SEXP { self.data[i as usize] }
}

// ========= R-callable C wrappers (no macros, pure .Call) =========

#[unsafe(no_mangle)]
pub unsafe extern "C" fn C_altrep_compact_int(
    _call: SEXP,
    n_: SEXP,
    start_: SEXP,
    step_: SEXP,
) -> SEXP {
    // Expect INTSXP scalars; read via DATAPTR_RO
    let n = unsafe { *DATAPTR_RO(n_).cast() };
    let start = unsafe { *DATAPTR_RO(start_).cast() };
    let step = unsafe { *DATAPTR_RO(step_).cast() };
    if step != 1 && step != -1 {
        return unsafe { R_NilValue };
    }
    unsafe { new_altrep_int(Box::new(CompactIntSeq::new(n, start, step))) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn C_altrep_from_doubles(_call: SEXP, x: SEXP) -> SEXP {
    let b = OwnedReal::from_reals_sexp(x);
    unsafe { new_altrep_real(Box::new(b)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn C_altrep_from_strings(_call: SEXP, x: SEXP) -> SEXP {
    let b = Utf8Vec::from_strs_sexp(x);
    unsafe { new_altrep_str(Box::new(b)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn C_altrep_from_logicals(_call: SEXP, x: SEXP) -> SEXP {
    let b = OwnedLogical::from_lgls_sexp(x);
    unsafe { new_altrep_lgl(Box::new(b)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn C_altrep_from_raw(_call: SEXP, x: SEXP) -> SEXP {
    let b = OwnedRaw::from_raw_sexp(x);
    unsafe { new_altrep_raw(Box::new(b)) }
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
pub trait AltrepClass {
    const CLASS_NAME: &'static str;
    const PKG_NAME: &'static str;
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

/// Vector-level hooks.
pub trait AltVec: AltrepClass {
    unsafe fn dataptr(_x: SEXP, _writable: bool) -> *mut c_void {
        core::ptr::null_mut()
    }
    unsafe fn dataptr_or_null(_x: SEXP) -> *const c_void {
        core::ptr::null()
    }
    unsafe fn extract_subset(_x: SEXP, _indx: SEXP, _call: SEXP) -> Option<SEXP> {
        None
    }
}

/// INT methods.
pub trait AltInteger: AltVec {
    unsafe fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!()
    }
    unsafe fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        0
    }
    unsafe fn is_sorted(_x: SEXP) -> i32 {
        /* UNKNOWN_SORTEDNESS */
        0
    }
    unsafe fn no_na(_x: SEXP) -> i32 {
        0
    }
    unsafe fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
    unsafe fn min(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
    unsafe fn max(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
}

/// REAL methods.
pub trait AltReal: AltVec {
    unsafe fn elt(_x: SEXP, _i: R_xlen_t) -> f64 {
        unreachable!()
    }
    unsafe fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut f64) -> R_xlen_t {
        0
    }
    unsafe fn is_sorted(_x: SEXP) -> i32 {
        0
    }
    unsafe fn no_na(_x: SEXP) -> i32 {
        0
    }
    unsafe fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
    unsafe fn min(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
    unsafe fn max(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
}

/// LOGICAL methods.
pub trait AltLogical: AltVec {
    unsafe fn elt(_x: SEXP, _i: R_xlen_t) -> i32 {
        unreachable!()
    }
    unsafe fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut i32) -> R_xlen_t {
        0
    }
    unsafe fn is_sorted(_x: SEXP) -> i32 {
        0
    }
    unsafe fn no_na(_x: SEXP) -> i32 {
        0
    }
    unsafe fn sum(_x: SEXP, _narm: bool) -> Option<SEXP> {
        None
    }
}

/// RAW methods.
pub trait AltRaw: AltVec {
    unsafe fn elt(_x: SEXP, _i: R_xlen_t) -> Rbyte {
        unreachable!()
    }
    unsafe fn get_region(_x: SEXP, _i: R_xlen_t, _n: R_xlen_t, _buf: *mut Rbyte) -> R_xlen_t {
        0
    }
}

/// STRING methods.
pub trait AltString: AltVec {
    const HAS_SET_ELT: bool = false;
    unsafe fn elt(_x: SEXP, _i: R_xlen_t) -> SEXP {
        unreachable!()
    }
    unsafe fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        // default: unsupported
    }
    unsafe fn is_sorted(_x: SEXP) -> i32 {
        0
    }
    unsafe fn no_na(_x: SEXP) -> i32 {
        0
    }
}

/// LIST (VECSXP) methods.
pub trait AltList: AltVec {
    const HAS_SET_ELT: bool = false;
    unsafe fn elt(_x: SEXP, _i: R_xlen_t) -> SEXP {
        unreachable!()
    }
    unsafe fn set_elt(_x: SEXP, _i: R_xlen_t, _v: SEXP) {
        // default: unsupported
    }
}

// AltComplex intentionally omitted for now: FFI method types are not exposed.

// ========= Generic registration helpers =========

// Generic trampolines for AltrepClass and AltVec
unsafe extern "C" fn g_length<T: AltrepClass>(x: SEXP) -> R_xlen_t {
    unsafe { T::length(x) }
}
unsafe extern "C" fn g_dataptr<T: AltVec>(x: SEXP, w: Rboolean) -> *mut c_void {
    unsafe { T::dataptr(x, matches!(w, Rboolean::TRUE)) }
}
unsafe extern "C" fn g_dataptr_or_null<T: AltVec>(x: SEXP) -> *const c_void {
    unsafe { T::dataptr_or_null(x) }
}

// Integer family trampolines
unsafe extern "C" fn g_int_elt<T: AltInteger>(x: SEXP, i: R_xlen_t) -> i32 {
    unsafe { T::elt(x, i) }
}
unsafe extern "C" fn g_int_get_region<T: AltInteger>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut i32,
) -> R_xlen_t {
    unsafe { T::get_region(x, i, n, buf) }
}
unsafe extern "C" fn g_int_is_sorted<T: AltInteger>(x: SEXP) -> i32 {
    unsafe { T::is_sorted(x) }
}
unsafe extern "C" fn g_int_no_na<T: AltInteger>(x: SEXP) -> i32 {
    unsafe { T::no_na(x) }
}

// Real family trampolines
unsafe extern "C" fn g_real_elt<T: AltReal>(x: SEXP, i: R_xlen_t) -> f64 {
    unsafe { T::elt(x, i) }
}
unsafe extern "C" fn g_real_get_region<T: AltReal>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut f64,
) -> R_xlen_t {
    unsafe { T::get_region(x, i, n, buf) }
}
unsafe extern "C" fn g_real_is_sorted<T: AltReal>(x: SEXP) -> i32 {
    unsafe { T::is_sorted(x) }
}
unsafe extern "C" fn g_real_no_na<T: AltReal>(x: SEXP) -> i32 {
    unsafe { T::no_na(x) }
}

// Logical family trampolines
unsafe extern "C" fn g_lgl_elt<T: AltLogical>(x: SEXP, i: R_xlen_t) -> i32 {
    unsafe { T::elt(x, i) }
}
unsafe extern "C" fn g_lgl_get_region<T: AltLogical>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut i32,
) -> R_xlen_t {
    unsafe { T::get_region(x, i, n, buf) }
}
unsafe extern "C" fn g_lgl_is_sorted<T: AltLogical>(x: SEXP) -> i32 {
    unsafe { T::is_sorted(x) }
}
unsafe extern "C" fn g_lgl_no_na<T: AltLogical>(x: SEXP) -> i32 {
    unsafe { T::no_na(x) }
}

// Raw family trampolines
unsafe extern "C" fn g_raw_elt<T: AltRaw>(x: SEXP, i: R_xlen_t) -> Rbyte {
    unsafe { T::elt(x, i) }
}
unsafe extern "C" fn g_raw_get_region<T: AltRaw>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut Rbyte,
) -> R_xlen_t {
    unsafe { T::get_region(x, i, n, buf) }
}

// String family trampolines
unsafe extern "C" fn g_str_elt<T: AltString>(x: SEXP, i: R_xlen_t) -> SEXP {
    unsafe { T::elt(x, i) }
}
unsafe extern "C" fn g_str_is_sorted<T: AltString>(x: SEXP) -> i32 {
    unsafe { T::is_sorted(x) }
}
unsafe extern "C" fn g_str_no_na<T: AltString>(x: SEXP) -> i32 {
    unsafe { T::no_na(x) }
}
unsafe extern "C" fn g_str_set_elt<T: AltString>(x: SEXP, i: R_xlen_t, v: SEXP) {
    unsafe { T::set_elt(x, i, v) }
}

// List family trampolines
unsafe extern "C" fn g_list_elt<T: AltList>(x: SEXP, i: R_xlen_t) -> SEXP {
    unsafe { T::elt(x, i) }
}
unsafe extern "C" fn g_list_set_elt<T: AltList>(x: SEXP, i: R_xlen_t, v: SEXP) {
    unsafe { T::set_elt(x, i, v) }
}

/// Register an ALTREP class for integer vectors backed by `T`.
pub unsafe fn register_altinteger_class<T: AltrepClass + AltVec + AltInteger>() -> R_altrep_class_t
{
    let cls = unsafe {
        R_make_altinteger_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    unsafe {
        R_set_altrep_Length_method(cls, Some(g_length::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_method(cls, Some(g_dataptr::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_or_null_method(cls, Some(g_dataptr_or_null::<T>));
    }
    unsafe {
        R_set_altinteger_Elt_method(cls, Some(g_int_elt::<T>));
    }
    unsafe {
        R_set_altinteger_Get_region_method(cls, Some(g_int_get_region::<T>));
    }
    unsafe {
        R_set_altinteger_Is_sorted_method(cls, Some(g_int_is_sorted::<T>));
    }
    unsafe {
        R_set_altinteger_No_NA_method(cls, Some(g_int_no_na::<T>));
    }
    cls
}

/// Register an ALTREP class for real vectors backed by `T`.
pub unsafe fn register_altreal_class<T: AltrepClass + AltVec + AltReal>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altreal_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    unsafe {
        R_set_altrep_Length_method(cls, Some(g_length::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_method(cls, Some(g_dataptr::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_or_null_method(cls, Some(g_dataptr_or_null::<T>));
    }
    unsafe {
        R_set_altreal_Elt_method(cls, Some(g_real_elt::<T>));
    }
    unsafe {
        R_set_altreal_Get_region_method(cls, Some(g_real_get_region::<T>));
    }
    unsafe {
        R_set_altreal_Is_sorted_method(cls, Some(g_real_is_sorted::<T>));
    }
    unsafe {
        R_set_altreal_No_NA_method(cls, Some(g_real_no_na::<T>));
    }
    cls
}

/// Register an ALTREP class for logical vectors backed by `T`.
pub unsafe fn register_altlogical_class<T: AltrepClass + AltVec + AltLogical>() -> R_altrep_class_t
{
    let cls = unsafe {
        R_make_altlogical_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    unsafe {
        R_set_altrep_Length_method(cls, Some(g_length::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_method(cls, Some(g_dataptr::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_or_null_method(cls, Some(g_dataptr_or_null::<T>));
    }
    unsafe {
        R_set_altlogical_Elt_method(cls, Some(g_lgl_elt::<T>));
    }
    unsafe {
        R_set_altlogical_Get_region_method(cls, Some(g_lgl_get_region::<T>));
    }
    unsafe {
        R_set_altlogical_Is_sorted_method(cls, Some(g_lgl_is_sorted::<T>));
    }
    unsafe {
        R_set_altlogical_No_NA_method(cls, Some(g_lgl_no_na::<T>));
    }
    cls
}

/// Register an ALTREP class for raw vectors backed by `T`.
pub unsafe fn register_altraw_class<T: AltrepClass + AltVec + AltRaw>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altraw_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    unsafe {
        R_set_altrep_Length_method(cls, Some(g_length::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_method(cls, Some(g_dataptr::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_or_null_method(cls, Some(g_dataptr_or_null::<T>));
    }
    unsafe {
        R_set_altraw_Elt_method(cls, Some(g_raw_elt::<T>));
    }
    unsafe {
        R_set_altraw_Get_region_method(cls, Some(g_raw_get_region::<T>));
    }
    cls
}

/// Register an ALTREP class for string vectors backed by `T`.
pub unsafe fn register_altstring_class<T: AltrepClass + AltVec + AltString>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altstring_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    unsafe {
        R_set_altrep_Length_method(cls, Some(g_length::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_method(cls, Some(g_dataptr::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_or_null_method(cls, Some(g_dataptr_or_null::<T>));
    }
    unsafe {
        R_set_altstring_Elt_method(cls, Some(g_str_elt::<T>));
    }
    unsafe {
        R_set_altstring_Is_sorted_method(cls, Some(g_str_is_sorted::<T>));
    }
    unsafe {
        R_set_altstring_No_NA_method(cls, Some(g_str_no_na::<T>));
    }
    if T::HAS_SET_ELT {
        unsafe {
            R_set_altstring_Set_elt_method(cls, Some(g_str_set_elt::<T>));
        }
    }
    cls
}

/// Register an ALTREP class for generic lists (VECSXP) backed by `T`.
pub unsafe fn register_altlist_class<T: AltrepClass + AltVec + AltList>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altlist_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    unsafe {
        R_set_altrep_Length_method(cls, Some(g_length::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_method(cls, Some(g_dataptr::<T>));
    }
    unsafe {
        R_set_altvec_Dataptr_or_null_method(cls, Some(g_dataptr_or_null::<T>));
    }
    unsafe {
        R_set_altlist_Elt_method(cls, Some(g_list_elt::<T>));
    }
    if T::HAS_SET_ELT {
        unsafe {
            R_set_altlist_Set_elt_method(cls, Some(g_list_set_elt::<T>));
        }
    }
    cls
}

// ========= Built-in class adapters using dynamic Backends =========

struct AltIntClass;
impl AltrepClass for AltIntClass {
    const CLASS_NAME: &'static str = "rust_altint";
    const PKG_NAME: &'static str = "miniextendr";
    const BASE: RBase = RBase::Int;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { int_backend(x).len() }
    }
}
impl AltVec for AltIntClass {
    unsafe fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        unsafe {
            int_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *mut c_void)
                .unwrap_or(core::ptr::null_mut())
        }
    }
    unsafe fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            int_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *const c_void)
                .unwrap_or(core::ptr::null())
        }
    }
}
impl AltInteger for AltIntClass {
    unsafe fn elt(x: SEXP, i: R_xlen_t) -> i32 {
        unsafe { int_backend(x).elt(i) }
    }
    unsafe fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { int_backend(x).get_region(i, n, out) }
    }
    unsafe fn is_sorted(x: SEXP) -> i32 {
        unsafe { int_backend(x).is_sorted() }
    }
    unsafe fn no_na(x: SEXP) -> i32 {
        unsafe { int_backend(x).no_na() }
    }
}

struct AltRealClass;
impl AltrepClass for AltRealClass {
    const CLASS_NAME: &'static str = "rust_altreal";
    const PKG_NAME: &'static str = "miniextendr";
    const BASE: RBase = RBase::Real;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { real_backend(x).len() }
    }
}
impl AltVec for AltRealClass {
    unsafe fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        unsafe {
            real_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *mut c_void)
                .unwrap_or(core::ptr::null_mut())
        }
    }
    unsafe fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            real_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *const c_void)
                .unwrap_or(core::ptr::null())
        }
    }
}
impl AltReal for AltRealClass {
    unsafe fn elt(x: SEXP, i: R_xlen_t) -> f64 {
        unsafe { real_backend(x).elt(i) }
    }
    unsafe fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut f64) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { real_backend(x).get_region(i, n, out) }
    }
    unsafe fn is_sorted(x: SEXP) -> i32 {
        unsafe { real_backend(x).is_sorted() }
    }
    unsafe fn no_na(x: SEXP) -> i32 {
        unsafe { real_backend(x).no_na() }
    }
}

struct AltStrClass;
impl AltrepClass for AltStrClass {
    const CLASS_NAME: &'static str = "rust_altstr";
    const PKG_NAME: &'static str = "miniextendr";
    const BASE: RBase = RBase::String;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { str_backend(x).len() }
    }
}
impl AltVec for AltStrClass {}
impl AltString for AltStrClass {
    unsafe fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        match unsafe { str_backend(x).utf8_at(i) } {
            None => unsafe { NA_STRING },
            Some(s) => {
                let cs = std::ffi::CString::new(s).unwrap();
                unsafe { Rf_mkCharLen(cs.as_ptr(), s.len() as i32) }
            }
        }
    }
}
