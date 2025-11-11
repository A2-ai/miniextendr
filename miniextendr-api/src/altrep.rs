//! ALTREP "from scratch" core for miniextendr-api: one class per base kind
//! (INT, REAL, STRING). No libR-sys/extendr dependencies; only raw FFI.

use core::ffi::{c_char, c_void};
use core::slice;
use std::sync::{Arc, OnceLock};

// Use the project's FFI definitions and types.
use crate::ffi::altrep::*;
use crate::ffi::*;
use crate::altrep_traits as traits;

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

// Convenience constructors for stock backends
pub unsafe fn new_altrep_int_from_vec(v: Vec<i32>) -> SEXP {
    new_altrep_int(Box::new(IntVec::from(v)))
}
pub unsafe fn new_altrep_int_from_arc(a: Arc<[i32]>) -> SEXP {
    new_altrep_int(Box::new(IntArc::from(a)))
}
pub unsafe fn new_altrep_int_from_slice_static(s: &'static [i32]) -> SEXP {
    new_altrep_int(Box::new(IntSliceMat::new(s)))
}
pub unsafe fn new_altrep_int_from_mmap(
    ptr: *const i32,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const i32, usize)>,
) -> SEXP {
    new_altrep_int(Box::new(IntMmap::new(ptr, len, cleanup)))
}

pub unsafe fn new_altrep_real_from_vec(v: Vec<f64>) -> SEXP {
    new_altrep_real(Box::new(RealVec::from(v)))
}
pub unsafe fn new_altrep_real_from_arc(a: Arc<[f64]>) -> SEXP {
    new_altrep_real(Box::new(RealArc::from(a)))
}
pub unsafe fn new_altrep_real_from_slice_static(s: &'static [f64]) -> SEXP {
    new_altrep_real(Box::new(RealSliceMat::new(s)))
}
pub unsafe fn new_altrep_real_from_mmap(
    ptr: *const f64,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const f64, usize)>,
) -> SEXP {
    new_altrep_real(Box::new(RealMmap::new(ptr, len, cleanup)))
}

pub unsafe fn new_altrep_str_from_vec(v: Vec<String>) -> SEXP {
    new_altrep_str(Box::new(Utf8Vec { data: v }))
}
pub unsafe fn new_altrep_str_from_arc(a: Arc<[String]>) -> SEXP {
    new_altrep_str(Box::new(Utf8Arc::from(a)))
}
pub unsafe fn new_altrep_str_from_slice_static(s: &'static [&'static str]) -> SEXP {
    new_altrep_str(Box::new(Utf8Slice::new(s)))
}

pub unsafe fn new_altrep_lgl_from_vec(v: Vec<i32>) -> SEXP {
    new_altrep_lgl(Box::new(LogicalVec::from(v)))
}
pub unsafe fn new_altrep_lgl_from_arc(a: Arc<[i32]>) -> SEXP {
    new_altrep_lgl(Box::new(LogicalArc::from(a)))
}
pub unsafe fn new_altrep_lgl_from_slice_static(s: &'static [i32]) -> SEXP {
    new_altrep_lgl(Box::new(LogicalSliceMat::new(s)))
}
pub unsafe fn new_altrep_lgl_from_mmap(
    ptr: *const i32,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const i32, usize)>,
) -> SEXP {
    new_altrep_lgl(Box::new(LogicalMmap::new(ptr, len, cleanup)))
}

pub unsafe fn new_altrep_raw_from_vec(v: Vec<Rbyte>) -> SEXP {
    new_altrep_raw(Box::new(RawVec::from(v)))
}
pub unsafe fn new_altrep_raw_from_arc(a: Arc<[Rbyte]>) -> SEXP {
    new_altrep_raw(Box::new(RawArc::from(a)))
}
pub unsafe fn new_altrep_raw_from_slice_static(s: &'static [Rbyte]) -> SEXP {
    new_altrep_raw(Box::new(RawSliceMat::new(s)))
}
pub unsafe fn new_altrep_raw_from_mmap(
    ptr: *const Rbyte,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const Rbyte, usize)>,
) -> SEXP {
    new_altrep_raw(Box::new(RawMmap::new(ptr, len, cleanup)))
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

// ========= Stock numeric/string backends =========

// INT backends
pub struct IntVec {
    data: Box<[i32]>,
}
impl From<Vec<i32>> for IntVec {
    fn from(v: Vec<i32>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl IntBackend for IntVec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

pub struct IntArc {
    data: Arc<[i32]>,
}
impl From<Arc<[i32]>> for IntArc {
    fn from(data: Arc<[i32]>) -> Self {
        Self { data }
    }
}
impl IntBackend for IntArc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

pub struct IntSliceMat {
    src: &'static [i32],
    materialized: OnceLock<Box<[i32]>>,
}
impl IntSliceMat {
    pub fn new(src: &'static [i32]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl IntBackend for IntSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct IntMmap {
    ptr: *const i32,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const i32, usize)>,
}
unsafe impl Send for IntMmap {}
unsafe impl Sync for IntMmap {}
impl IntMmap {
    pub unsafe fn new(
        ptr: *const i32,
        len: usize,
        cleanup: Option<unsafe extern "C" fn(*const i32, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for IntMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl IntBackend for IntMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { core::slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(unsafe { core::slice::from_raw_parts(self.ptr, self.len) })
    }
}

// REAL backends
pub struct RealVec {
    data: Box<[f64]>,
}
impl From<Vec<f64>> for RealVec {
    fn from(v: Vec<f64>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl RealBackend for RealVec {
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
}

pub struct RealArc {
    data: Arc<[f64]>,
}
impl From<Arc<[f64]>> for RealArc {
    fn from(data: Arc<[f64]>) -> Self {
        Self { data }
    }
}
impl RealBackend for RealArc {
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
}

pub struct RealSliceMat {
    src: &'static [f64],
    materialized: OnceLock<Box<[f64]>>,
}
impl RealSliceMat {
    pub fn new(src: &'static [f64]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl RealBackend for RealSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct RealMmap {
    ptr: *const f64,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const f64, usize)>,
}
unsafe impl Send for RealMmap {}
unsafe impl Sync for RealMmap {}
impl RealMmap {
    pub unsafe fn new(
        ptr: *const f64,
        len: usize,
        cleanup: Option<unsafe extern "C" fn(*const f64, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for RealMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl RealBackend for RealMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> f64 {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [f64]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { core::slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[f64]> {
        Some(unsafe { core::slice::from_raw_parts(self.ptr, self.len) })
    }
}

// LOGICAL backends (i32 with NA support)
pub struct LogicalVec {
    data: Box<[i32]>,
}
impl From<Vec<i32>> for LogicalVec {
    fn from(v: Vec<i32>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl LogicalBackend for LogicalVec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

pub struct LogicalArc {
    data: Arc<[i32]>,
}
impl From<Arc<[i32]>> for LogicalArc {
    fn from(data: Arc<[i32]>) -> Self {
        Self { data }
    }
}
impl LogicalBackend for LogicalArc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
}

pub struct LogicalSliceMat {
    src: &'static [i32],
    materialized: OnceLock<Box<[i32]>>,
}
impl LogicalSliceMat {
    pub fn new(src: &'static [i32]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl LogicalBackend for LogicalSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct LogicalMmap {
    ptr: *const i32,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const i32, usize)>,
}
unsafe impl Send for LogicalMmap {}
unsafe impl Sync for LogicalMmap {}
impl LogicalMmap {
    pub unsafe fn new(
        ptr: *const i32,
        len: usize,
        cleanup: Option<unsafe extern "C" fn(*const i32, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for LogicalMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl LogicalBackend for LogicalMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { core::slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(unsafe { core::slice::from_raw_parts(self.ptr, self.len) })
    }
}

// RAW backends (Rbyte)
pub struct RawVec {
    data: Box<[Rbyte]>,
}
impl From<Vec<Rbyte>> for RawVec {
    fn from(v: Vec<Rbyte>) -> Self {
        Self {
            data: v.into_boxed_slice(),
        }
    }
}
impl RawBackend for RawVec {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(&self.data)
    }
}

pub struct RawArc {
    data: Arc<[Rbyte]>,
}
impl From<Arc<[Rbyte]>> for RawArc {
    fn from(data: Arc<[Rbyte]>) -> Self {
        Self { data }
    }
}
impl RawBackend for RawArc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(&self.data)
    }
}

pub struct RawSliceMat {
    src: &'static [Rbyte],
    materialized: OnceLock<Box<[Rbyte]>>,
}
impl RawSliceMat {
    pub fn new(src: &'static [Rbyte]) -> Self {
        Self {
            src,
            materialized: OnceLock::new(),
        }
    }
}
impl RawBackend for RawSliceMat {
    fn len(&self) -> R_xlen_t {
        self.src.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.src[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.src[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        let bx = self
            .materialized
            .get_or_init(|| self.src.to_vec().into_boxed_slice());
        Some(&**bx)
    }
}

pub struct RawMmap {
    ptr: *const Rbyte,
    len: usize,
    cleanup: Option<unsafe extern "C" fn(*const Rbyte, usize)>,
}
unsafe impl Send for RawMmap {}
unsafe impl Sync for RawMmap {}
impl RawMmap {
    pub unsafe fn new(
        ptr: *const Rbyte,
        len: usize,
        cleanup: Option<unsafe extern "C" fn(*const Rbyte, usize)>,
    ) -> Self {
        Self { ptr, len, cleanup }
    }
}
impl Drop for RawMmap {
    fn drop(&mut self) {
        if let Some(f) = self.cleanup {
            unsafe { f(self.ptr, self.len) }
        }
    }
}
impl RawBackend for RawMmap {
    fn len(&self) -> R_xlen_t {
        self.len as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        unsafe { *self.ptr.add(i as usize) }
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let start = i as usize;
        let end = ((i + n).min(self.len())) as usize;
        let src = unsafe { core::slice::from_raw_parts(self.ptr.add(start), end - start) };
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(unsafe { core::slice::from_raw_parts(self.ptr, self.len) })
    }
}

// STRING backends
pub struct Utf8Arc {
    data: Arc<[String]>,
}
impl From<Arc<[String]>> for Utf8Arc {
    fn from(data: Arc<[String]>) -> Self {
        Self { data }
    }
}
impl StringBackend for Utf8Arc {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str> {
        Some(self.data[i as usize].as_str())
    }
}

pub struct Utf8Slice {
    data: &'static [&'static str],
}
impl Utf8Slice {
    pub fn new(data: &'static [&'static str]) -> Self {
        Self { data }
    }
}
impl StringBackend for Utf8Slice {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn utf8_at(&self, i: R_xlen_t) -> Option<&str> {
        Some(self.data[i as usize])
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
            Self {
                data: slice.to_vec().into_boxed_slice(),
            }
        }
    }
}
impl LogicalBackend for OwnedLogical {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> i32 {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [i32]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[i32]> {
        Some(&self.data)
    }
    fn no_na(&self) -> i32 {
        0
    }
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
            Self {
                data: slice.to_vec().into_boxed_slice(),
            }
        }
    }
}
impl RawBackend for OwnedRaw {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> Rbyte {
        self.data[i as usize]
    }
    fn get_region(&self, i: R_xlen_t, n: R_xlen_t, out: &mut [Rbyte]) -> R_xlen_t {
        let end = (i + n).min(self.len()) as usize;
        let src = &self.data[i as usize..end];
        out[..src.len()].copy_from_slice(src);
        src.len() as R_xlen_t
    }
    fn dataptr(&self) -> Option<&[Rbyte]> {
        Some(&self.data)
    }
}

/// Owned list of SEXP values for VECSXP.
pub struct OwnedList {
    data: Vec<SEXP>,
}
impl OwnedList {
    pub fn from_sexps(v: Vec<SEXP>) -> Self {
        Self { data: v }
    }
    pub fn from_list_sexp(x: SEXP) -> Self {
        unsafe {
            let n = Rf_xlength(x);
            let mut v = Vec::with_capacity(n as usize);
            for i in 0..n {
                let elt = VECTOR_ELT(x, i);
                v.push(elt);
            }
            Self { data: v }
        }
    }
}
unsafe impl Send for OwnedList {}
unsafe impl Sync for OwnedList {}
impl ListBackend for OwnedList {
    fn len(&self) -> R_xlen_t {
        self.data.len() as R_xlen_t
    }
    fn elt(&self, i: R_xlen_t) -> SEXP {
        self.data[i as usize]
    }
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn C_altrep_from_list(_call: SEXP, x: SEXP) -> SEXP {
    let b = OwnedList::from_list_sexp(x);
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
// Old Alt* trait scaffolding has been replaced by safe traits in `altrep_traits`.

// AltComplex intentionally omitted for now: FFI method types are not exposed.

// ========= Generic registration helpers =========

// Generic trampolines for Altrep/AltVec (safe traits)
unsafe extern "C" fn t_length<T: traits::Altrep>(x: SEXP) -> R_xlen_t { T::length(x) }
unsafe extern "C" fn t_dataptr<T: traits::AltVec>(x: SEXP, w: Rboolean) -> *mut c_void {
    T::dataptr(x, matches!(w, Rboolean::TRUE))
}
unsafe extern "C" fn t_dataptr_or_null<T: traits::AltVec>(x: SEXP) -> *const c_void {
    T::dataptr_or_null(x)
}
unsafe extern "C" fn t_extract_subset<T: traits::AltVec>(x: SEXP, indx: SEXP, call: SEXP) -> SEXP {
    T::extract_subset(x, indx, call)
}

// Integer family trampolines (safe traits)
unsafe extern "C" fn t_int_elt<T: traits::AltInteger>(x: SEXP, i: R_xlen_t) -> i32 { T::elt(x, i) }
unsafe extern "C" fn t_int_get_region<T: traits::AltInteger>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut i32,
) -> R_xlen_t {
    T::get_region(x, i, n, buf)
}
unsafe extern "C" fn t_int_is_sorted<T: traits::AltInteger>(x: SEXP) -> i32 { T::is_sorted(x) }
unsafe extern "C" fn t_int_no_na<T: traits::AltInteger>(x: SEXP) -> i32 { T::no_na(x) }
unsafe extern "C" fn t_int_sum<T: traits::AltInteger>(x: SEXP, narm: Rboolean) -> SEXP { T::sum(x, matches!(narm, Rboolean::TRUE)) }
unsafe extern "C" fn t_int_min<T: traits::AltInteger>(x: SEXP, narm: Rboolean) -> SEXP { T::min(x, matches!(narm, Rboolean::TRUE)) }
unsafe extern "C" fn t_int_max<T: traits::AltInteger>(x: SEXP, narm: Rboolean) -> SEXP { T::max(x, matches!(narm, Rboolean::TRUE)) }

// Real family trampolines
unsafe extern "C" fn t_real_elt<T: traits::AltReal>(x: SEXP, i: R_xlen_t) -> f64 { T::elt(x, i) }
unsafe extern "C" fn t_real_get_region<T: traits::AltReal>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut f64,
) -> R_xlen_t {
    T::get_region(x, i, n, buf)
}
unsafe extern "C" fn t_real_is_sorted<T: traits::AltReal>(x: SEXP) -> i32 { T::is_sorted(x) }
unsafe extern "C" fn t_real_no_na<T: traits::AltReal>(x: SEXP) -> i32 { T::no_na(x) }
unsafe extern "C" fn t_real_sum<T: traits::AltReal>(x: SEXP, narm: Rboolean) -> SEXP { T::sum(x, matches!(narm, Rboolean::TRUE)) }
unsafe extern "C" fn t_real_min<T: traits::AltReal>(x: SEXP, narm: Rboolean) -> SEXP { T::min(x, matches!(narm, Rboolean::TRUE)) }
unsafe extern "C" fn t_real_max<T: traits::AltReal>(x: SEXP, narm: Rboolean) -> SEXP { T::max(x, matches!(narm, Rboolean::TRUE)) }

// Logical family trampolines
unsafe extern "C" fn t_lgl_elt<T: traits::AltLogical>(x: SEXP, i: R_xlen_t) -> i32 { T::elt(x, i) }
unsafe extern "C" fn t_lgl_get_region<T: traits::AltLogical>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut i32,
) -> R_xlen_t {
    T::get_region(x, i, n, buf)
}
unsafe extern "C" fn t_lgl_is_sorted<T: traits::AltLogical>(x: SEXP) -> i32 { T::is_sorted(x) }
unsafe extern "C" fn t_lgl_no_na<T: traits::AltLogical>(x: SEXP) -> i32 { T::no_na(x) }

// Raw family trampolines
unsafe extern "C" fn t_raw_elt<T: traits::AltRaw>(x: SEXP, i: R_xlen_t) -> Rbyte { T::elt(x, i) }
unsafe extern "C" fn t_raw_get_region<T: traits::AltRaw>(
    x: SEXP,
    i: R_xlen_t,
    n: R_xlen_t,
    buf: *mut Rbyte,
) -> R_xlen_t {
    T::get_region(x, i, n, buf)
}

// String family trampolines
unsafe extern "C" fn t_str_elt<T: traits::AltString>(x: SEXP, i: R_xlen_t) -> SEXP { T::elt(x, i) }
unsafe extern "C" fn t_str_is_sorted<T: traits::AltString>(x: SEXP) -> i32 { T::is_sorted(x) }
unsafe extern "C" fn t_str_no_na<T: traits::AltString>(x: SEXP) -> i32 { T::no_na(x) }
unsafe extern "C" fn t_str_set_elt<T: traits::AltString>(x: SEXP, i: R_xlen_t, v: SEXP) { T::set_elt(x, i, v) }

// List family trampolines
unsafe extern "C" fn t_list_elt<T: traits::AltList>(x: SEXP, i: R_xlen_t) -> SEXP { T::elt(x, i) }
unsafe extern "C" fn t_list_set_elt<T: traits::AltList>(x: SEXP, i: R_xlen_t, v: SEXP) { T::set_elt(x, i, v) }

/// Register an ALTREP class for integer vectors backed by `T`.
pub unsafe fn register_altinteger_class<T: AltrepClass + traits::AltVec + traits::AltInteger>() -> R_altrep_class_t
{
    let cls = unsafe {
        R_make_altinteger_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    if <T as traits::Altrep>::HAS_LENGTH { R_set_altrep_Length_method(cls, Some(t_length::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)); }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)); }
    if <T as traits::AltInteger>::HAS_ELT { R_set_altinteger_Elt_method(cls, Some(t_int_elt::<T>)); }
    if <T as traits::AltInteger>::HAS_GET_REGION { R_set_altinteger_Get_region_method(cls, Some(t_int_get_region::<T>)); }
    if <T as traits::AltInteger>::HAS_IS_SORTED { R_set_altinteger_Is_sorted_method(cls, Some(t_int_is_sorted::<T>)); }
    if <T as traits::AltInteger>::HAS_NO_NA { R_set_altinteger_No_NA_method(cls, Some(t_int_no_na::<T>)); }
    if <T as traits::AltInteger>::HAS_SUM { R_set_altinteger_Sum_method(cls, Some(t_int_sum::<T>)); }
    if <T as traits::AltInteger>::HAS_MIN { R_set_altinteger_Min_method(cls, Some(t_int_min::<T>)); }
    if <T as traits::AltInteger>::HAS_MAX { R_set_altinteger_Max_method(cls, Some(t_int_max::<T>)); }
    cls
}

/// Register an ALTREP class for real vectors backed by `T`.
pub unsafe fn register_altreal_class<T: AltrepClass + traits::AltVec + traits::AltReal>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altreal_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    if <T as traits::Altrep>::HAS_LENGTH { R_set_altrep_Length_method(cls, Some(t_length::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)); }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)); }
    if <T as traits::AltReal>::HAS_ELT { R_set_altreal_Elt_method(cls, Some(t_real_elt::<T>)); }
    if <T as traits::AltReal>::HAS_GET_REGION { R_set_altreal_Get_region_method(cls, Some(t_real_get_region::<T>)); }
    if <T as traits::AltReal>::HAS_IS_SORTED { R_set_altreal_Is_sorted_method(cls, Some(t_real_is_sorted::<T>)); }
    if <T as traits::AltReal>::HAS_NO_NA { R_set_altreal_No_NA_method(cls, Some(t_real_no_na::<T>)); }
    if <T as traits::AltReal>::HAS_SUM { R_set_altreal_Sum_method(cls, Some(t_real_sum::<T>)); }
    if <T as traits::AltReal>::HAS_MIN { R_set_altreal_Min_method(cls, Some(t_real_min::<T>)); }
    if <T as traits::AltReal>::HAS_MAX { R_set_altreal_Max_method(cls, Some(t_real_max::<T>)); }
    cls
}

/// Register an ALTREP class for logical vectors backed by `T`.
pub unsafe fn register_altlogical_class<T: AltrepClass + traits::AltVec + traits::AltLogical>() -> R_altrep_class_t
{
    let cls = unsafe {
        R_make_altlogical_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    if <T as traits::Altrep>::HAS_LENGTH { R_set_altrep_Length_method(cls, Some(t_length::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)); }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)); }
    if <T as traits::AltLogical>::HAS_ELT { R_set_altlogical_Elt_method(cls, Some(t_lgl_elt::<T>)); }
    if <T as traits::AltLogical>::HAS_GET_REGION { R_set_altlogical_Get_region_method(cls, Some(t_lgl_get_region::<T>)); }
    if <T as traits::AltLogical>::HAS_IS_SORTED { R_set_altlogical_Is_sorted_method(cls, Some(t_lgl_is_sorted::<T>)); }
    if <T as traits::AltLogical>::HAS_NO_NA { R_set_altlogical_No_NA_method(cls, Some(t_lgl_no_na::<T>)); }
    cls
}

/// Register an ALTREP class for raw vectors backed by `T`.
pub unsafe fn register_altraw_class<T: AltrepClass + traits::AltVec + traits::AltRaw>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altraw_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    if <T as traits::Altrep>::HAS_LENGTH { R_set_altrep_Length_method(cls, Some(t_length::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)); }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)); }
    if <T as traits::AltRaw>::HAS_ELT { R_set_altraw_Elt_method(cls, Some(t_raw_elt::<T>)); }
    if <T as traits::AltRaw>::HAS_GET_REGION { R_set_altraw_Get_region_method(cls, Some(t_raw_get_region::<T>)); }
    cls
}

/// Register an ALTREP class for string vectors backed by `T`.
pub unsafe fn register_altstring_class<T: AltrepClass + traits::AltVec + traits::AltString>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altstring_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    if <T as traits::Altrep>::HAS_LENGTH { R_set_altrep_Length_method(cls, Some(t_length::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)); }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)); }
    if <T as traits::AltString>::HAS_ELT { R_set_altstring_Elt_method(cls, Some(t_str_elt::<T>)); }
    if <T as traits::AltString>::HAS_IS_SORTED { R_set_altstring_Is_sorted_method(cls, Some(t_str_is_sorted::<T>)); }
    if <T as traits::AltString>::HAS_NO_NA { R_set_altstring_No_NA_method(cls, Some(t_str_no_na::<T>)); }
    if <T as traits::AltString>::HAS_SET_ELT { R_set_altstring_Set_elt_method(cls, Some(t_str_set_elt::<T>)); }
    cls
}

/// Register an ALTREP class for generic lists (VECSXP) backed by `T`.
pub unsafe fn register_altlist_class<T: AltrepClass + traits::AltVec + traits::AltList>() -> R_altrep_class_t {
    let cls = unsafe {
        R_make_altlist_class(
            cstr(T::CLASS_NAME),
            cstr(T::PKG_NAME),
            core::ptr::null_mut(),
        )
    };
    if <T as traits::Altrep>::HAS_LENGTH { R_set_altrep_Length_method(cls, Some(t_length::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR { R_set_altvec_Dataptr_method(cls, Some(t_dataptr::<T>)); }
    if <T as traits::AltVec>::HAS_DATAPTR_OR_NULL { R_set_altvec_Dataptr_or_null_method(cls, Some(t_dataptr_or_null::<T>)); }
    if <T as traits::AltVec>::HAS_EXTRACT_SUBSET { R_set_altvec_Extract_subset_method(cls, Some(t_extract_subset::<T>)); }
    if <T as traits::AltList>::HAS_ELT { R_set_altlist_Elt_method(cls, Some(t_list_elt::<T>)); }
    if <T as traits::AltList>::HAS_SET_ELT { R_set_altlist_Set_elt_method(cls, Some(t_list_set_elt::<T>)); }
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
impl traits::Altrep for AltIntClass { const HAS_LENGTH: bool = true; fn length(x: SEXP) -> R_xlen_t { unsafe { int_backend(x).len() } } }
impl traits::AltVec for AltIntClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        unsafe {
            int_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *mut c_void)
                .unwrap_or(core::ptr::null_mut())
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            int_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *const c_void)
                .unwrap_or(core::ptr::null())
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
    fn elt(x: SEXP, i: R_xlen_t) -> i32 { unsafe { int_backend(x).elt(i) } }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32) -> R_xlen_t {
        let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) };
        unsafe { int_backend(x).get_region(i, n, out) }
    }
    fn is_sorted(x: SEXP) -> i32 { unsafe { int_backend(x).is_sorted() } }
    fn no_na(x: SEXP) -> i32 { unsafe { int_backend(x).no_na() } }
    fn sum(x: SEXP, _narm: bool) -> SEXP {
        let b = unsafe { int_backend(x) };
        let mut acc: i64 = 0; let n = b.len();
        for i in 0..n { acc = acc.wrapping_add(b.elt(i) as i64); }
        unsafe { Rf_ScalarReal(acc as f64) }
    }
    fn min(x: SEXP, _narm: bool) -> SEXP {
        let b = unsafe { int_backend(x) }; let n = b.len(); let mut m = b.elt(0);
        for i in 1..n { let v = b.elt(i); if v < m { m = v; } }
        unsafe { Rf_ScalarInteger(m) }
    }
    fn max(x: SEXP, _narm: bool) -> SEXP {
        let b = unsafe { int_backend(x) }; let n = b.len(); let mut m = b.elt(0);
        for i in 1..n { let v = b.elt(i); if v > m { m = v; } }
        unsafe { Rf_ScalarInteger(m) }
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
impl traits::Altrep for AltRealClass { const HAS_LENGTH: bool = true; fn length(x: SEXP) -> R_xlen_t { unsafe { real_backend(x).len() } } }
impl traits::AltVec for AltRealClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        unsafe {
            real_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *mut c_void)
                .unwrap_or(core::ptr::null_mut())
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            real_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *const c_void)
                .unwrap_or(core::ptr::null())
        }
    }
}
impl traits::AltReal for AltRealClass {
    const HAS_ELT: bool = true; const HAS_GET_REGION: bool = true; const HAS_IS_SORTED: bool = true; const HAS_NO_NA: bool = true; const HAS_SUM: bool = true; const HAS_MIN: bool = true; const HAS_MAX: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> f64 { unsafe { real_backend(x).elt(i) } }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut f64) -> R_xlen_t { let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) }; unsafe { real_backend(x).get_region(i, n, out) } }
    fn is_sorted(x: SEXP) -> i32 { unsafe { real_backend(x).is_sorted() } }
    fn no_na(x: SEXP) -> i32 { unsafe { real_backend(x).no_na() } }
    fn sum(x: SEXP, _narm: bool) -> SEXP { let b = unsafe { real_backend(x) }; let mut acc = 0.0; let n = b.len(); for i in 0..n { acc += b.elt(i); } unsafe { Rf_ScalarReal(acc) } }
    fn min(x: SEXP, _narm: bool) -> SEXP { let b = unsafe { real_backend(x) }; let n = b.len(); let mut m = b.elt(0); for i in 1..n { let v = b.elt(i); if v < m { m = v; } } unsafe { Rf_ScalarReal(m) } }
    fn max(x: SEXP, _narm: bool) -> SEXP { let b = unsafe { real_backend(x) }; let n = b.len(); let mut m = b.elt(0); for i in 1..n { let v = b.elt(i); if v > m { m = v; } } unsafe { Rf_ScalarReal(m) } }
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
impl traits::Altrep for AltStrClass { const HAS_LENGTH: bool = true; fn length(x: SEXP) -> R_xlen_t { unsafe { str_backend(x).len() } } }
impl traits::AltVec for AltStrClass {}
impl traits::AltString for AltStrClass {
    const HAS_ELT: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> SEXP {
        match unsafe { str_backend(x).utf8_at(i) } {
            None => unsafe { NA_STRING },
            Some(s) => { let cs = std::ffi::CString::new(s).unwrap(); unsafe { Rf_mkCharLen(cs.as_ptr(), s.len() as i32) } }
        }
    }
}

struct AltLogicalClass;
impl AltrepClass for AltLogicalClass {
    const CLASS_NAME: &'static str = "rust_altlgl";
    const PKG_NAME: &'static str = "miniextendr";
    const BASE: RBase = RBase::Logical;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { lgl_backend(x).len() }
    }
}
impl traits::Altrep for AltLogicalClass { const HAS_LENGTH: bool = true; fn length(x: SEXP) -> R_xlen_t { unsafe { lgl_backend(x).len() } } }
impl traits::AltVec for AltLogicalClass {
    const HAS_DATAPTR: bool = true;
    const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        unsafe {
            lgl_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *mut c_void)
                .unwrap_or(core::ptr::null_mut())
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            lgl_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *const c_void)
                .unwrap_or(core::ptr::null())
        }
    }
}
impl traits::AltLogical for AltLogicalClass {
    const HAS_ELT: bool = true; const HAS_GET_REGION: bool = true; const HAS_IS_SORTED: bool = true; const HAS_NO_NA: bool = true;
    fn elt(x: SEXP, i: R_xlen_t) -> i32 { unsafe { lgl_backend(x).elt(i) } }
    fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32) -> R_xlen_t { let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) }; unsafe { lgl_backend(x).get_region(i, n, out) } }
    fn is_sorted(x: SEXP) -> i32 { unsafe { lgl_backend(x).is_sorted() } }
    fn no_na(x: SEXP) -> i32 { unsafe { lgl_backend(x).no_na() } }
}

struct AltRawClass;
impl AltrepClass for AltRawClass {
    const CLASS_NAME: &'static str = "rust_altraw";
    const PKG_NAME: &'static str = "miniextendr";
    const BASE: RBase = RBase::Raw;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { raw_backend(x).len() }
    }
}
impl traits::Altrep for AltRawClass { const HAS_LENGTH: bool = true; fn length(x: SEXP) -> R_xlen_t { unsafe { raw_backend(x).len() } } }
impl traits::AltVec for AltRawClass { const HAS_DATAPTR: bool = true; const HAS_DATAPTR_OR_NULL: bool = true;
    fn dataptr(x: SEXP, _writable: bool) -> *mut c_void {
        unsafe {
            raw_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *mut c_void)
                .unwrap_or(core::ptr::null_mut())
        }
    }
    fn dataptr_or_null(x: SEXP) -> *const c_void {
        unsafe {
            raw_backend(x)
                .dataptr()
                .map(|s| s.as_ptr() as *const c_void)
                .unwrap_or(core::ptr::null())
        }
    }
}
impl traits::AltRaw for AltRawClass { const HAS_ELT: bool = true; const HAS_GET_REGION: bool = true; fn elt(x: SEXP, i: R_xlen_t) -> Rbyte { unsafe { raw_backend(x).elt(i) } } fn get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut Rbyte) -> R_xlen_t { let out = unsafe { slice::from_raw_parts_mut(buf, n as usize) }; unsafe { raw_backend(x).get_region(i, n, out) } } }

struct AltListClass;
impl AltrepClass for AltListClass {
    const CLASS_NAME: &'static str = "rust_altlist";
    const PKG_NAME: &'static str = "miniextendr";
    const BASE: RBase = RBase::List;
    unsafe fn length(x: SEXP) -> R_xlen_t {
        unsafe { list_backend(x).len() }
    }
}
impl traits::Altrep for AltListClass { const HAS_LENGTH: bool = true; fn length(x: SEXP) -> R_xlen_t { unsafe { list_backend(x).len() } } }
impl traits::AltVec for AltListClass {}
impl traits::AltList for AltListClass { const HAS_ELT: bool = true; fn elt(x: SEXP, i: R_xlen_t) -> SEXP { unsafe { list_backend(x).elt(i) } } }
