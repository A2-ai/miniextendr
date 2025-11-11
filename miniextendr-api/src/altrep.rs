//! ALTREP "from scratch" core for miniextendr-api: one class per base kind
//! (INT, REAL, STRING). No libR-sys/extendr dependencies; only raw FFI.

use core::ffi::{c_char, c_void};
use core::ptr;
use core::slice;
use std::sync::OnceLock;

#[allow(non_camel_case_types)]
pub type R_xlen_t = isize;

// Reuse the project's SEXP type.
use crate::ffi::{SEXP, SendSEXP};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct R_altrep_class_t {
    pub ptr: SEXP,
}

// Minimal FFI we need that isn't in crate::ffi yet.
// All of these symbols are provided by libR.dylib / libR.so.
extern "C" {
    // ALTREP class constructors (R_ext/Altrep.h)
    fn R_make_altinteger_class(
        cname: *const c_char,
        pname: *const c_char,
        dll: *mut c_void,
    ) -> R_altrep_class_t;
    fn R_make_altreal_class(
        cname: *const c_char,
        pname: *const c_char,
        dll: *mut c_void,
    ) -> R_altrep_class_t;
    fn R_make_altstring_class(
        cname: *const c_char,
        pname: *const c_char,
        dll: *mut c_void,
    ) -> R_altrep_class_t;

    // ALTREP object constructor
    fn R_new_altrep(cls: R_altrep_class_t, data1: SEXP, data2: SEXP) -> SEXP;

    // ALTREP method setters (subset)
    fn R_set_altrep_Length_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP) -> R_xlen_t>,
    );
    fn R_set_altvec_Dataptr_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP, i32) -> *mut c_void>,
    );
    fn R_set_altvec_Dataptr_or_null_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP) -> *const c_void>,
    );

    fn R_set_altinteger_Elt_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP, R_xlen_t) -> i32>,
    );
    fn R_set_altinteger_Get_region_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP, R_xlen_t, R_xlen_t, *mut i32) -> R_xlen_t>,
    );
    fn R_set_altinteger_Is_sorted_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP) -> i32>,
    );
    fn R_set_altinteger_No_NA_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP) -> i32>,
    );

    fn R_set_altreal_Elt_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP, R_xlen_t) -> f64>,
    );
    fn R_set_altreal_Get_region_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP, R_xlen_t, R_xlen_t, *mut f64) -> R_xlen_t>,
    );
    fn R_set_altreal_Is_sorted_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP) -> i32>,
    );
    fn R_set_altreal_No_NA_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP) -> i32>,
    );

    fn R_set_altstring_Elt_method(
        cls: R_altrep_class_t,
        f: Option<unsafe extern "C" fn(SEXP, R_xlen_t) -> SEXP>,
    );

    // External pointers
    fn R_MakeExternalPtr(p: *mut c_void, tag: SEXP, prot: SEXP) -> SEXP;
    fn R_ExternalPtrAddr(p: SEXP) -> *mut c_void;
    fn R_RegisterCFinalizerEx(p: SEXP, f: Option<unsafe extern "C" fn(SEXP)>, onexit: i32);

    // R internals we need
    static R_NilValue: SEXP;
    static NA_STRING: SEXP;
    fn Rf_xlength(x: SEXP) -> R_xlen_t;
    fn Rf_mkCharLen(s: *const c_char, len: i32) -> SEXP;

    // Handy memory accessors already declared in crate::ffi, but duplicate here for isolation.
    fn DATAPTR(x: SEXP) -> *mut c_void;
    fn DATAPTR_RO(x: SEXP) -> *const c_void;
    fn DATAPTR_OR_NULL(x: SEXP) -> *const c_void;
    fn R_altrep_data1(x: SEXP) -> SEXP;
}

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

// Global class handles
static ALTINT: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTREAL: OnceLock<R_altrep_class_t> = OnceLock::new();
static ALTSTR: OnceLock<R_altrep_class_t> = OnceLock::new();

// -- helpers to store/retrieve Box<dyn Backend> behind an external ptr --
unsafe fn make_eptr<T: ?Sized>(b: Box<T>, fin: unsafe extern "C" fn(SEXP)) -> SEXP {
    let ep = R_MakeExternalPtr(Box::into_raw(b).cast(), R_NilValue, R_NilValue);
    R_RegisterCFinalizerEx(ep, Some(fin), 1);
    ep
}
unsafe fn ep_as<'a, T: ?Sized>(ep: SEXP) -> &'a T {
    &*(R_ExternalPtrAddr(ep) as *const T)
}

// ========= INT class + trampolines =========
unsafe fn int_backend<'a>(x: SEXP) -> &'a dyn IntBackend {
    let ep = R_altrep_data1(x);
    ep_as::<Box<dyn IntBackend>>(ep).as_ref()
}

unsafe extern "C" fn int_len(x: SEXP) -> R_xlen_t {
    int_backend(x).len()
}
unsafe extern "C" fn int_elt(x: SEXP, i: R_xlen_t) -> i32 {
    int_backend(x).elt(i)
}
unsafe extern "C" fn int_get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut i32) -> R_xlen_t {
    let out = slice::from_raw_parts_mut(buf, n as usize);
    int_backend(x).get_region(i, n, out)
}
unsafe extern "C" fn int_is_sorted(x: SEXP) -> i32 {
    int_backend(x).is_sorted()
}
unsafe extern "C" fn int_no_na(x: SEXP) -> i32 {
    int_backend(x).no_na()
}
unsafe extern "C" fn int_dataptr_or_null(x: SEXP) -> *const c_void {
    int_backend(x)
        .dataptr()
        .map(|s| s.as_ptr() as *const c_void)
        .unwrap_or(ptr::null())
}
unsafe extern "C" fn int_dataptr(x: SEXP, _w: i32) -> *mut c_void {
    int_dataptr_or_null(x) as *mut c_void
}
unsafe extern "C" fn int_finalizer(ep: SEXP) {
    let raw = R_ExternalPtrAddr(ep);
    if !raw.is_null() {
        drop(Box::<Box<dyn IntBackend>>::from_raw(raw.cast()));
    }
}

// ========= REAL class + trampolines =========
unsafe fn real_backend<'a>(x: SEXP) -> &'a dyn RealBackend {
    let ep = R_altrep_data1(x);
    ep_as::<Box<dyn RealBackend>>(ep).as_ref()
}
unsafe extern "C" fn real_len(x: SEXP) -> R_xlen_t {
    real_backend(x).len()
}
unsafe extern "C" fn real_elt(x: SEXP, i: R_xlen_t) -> f64 {
    real_backend(x).elt(i)
}
unsafe extern "C" fn real_get_region(x: SEXP, i: R_xlen_t, n: R_xlen_t, buf: *mut f64) -> R_xlen_t {
    let out = slice::from_raw_parts_mut(buf, n as usize);
    real_backend(x).get_region(i, n, out)
}
unsafe extern "C" fn real_is_sorted(x: SEXP) -> i32 {
    real_backend(x).is_sorted()
}
unsafe extern "C" fn real_no_na(x: SEXP) -> i32 {
    real_backend(x).no_na()
}
unsafe extern "C" fn real_dataptr_or_null(x: SEXP) -> *const c_void {
    real_backend(x)
        .dataptr()
        .map(|s| s.as_ptr() as *const c_void)
        .unwrap_or(ptr::null())
}
unsafe extern "C" fn real_dataptr(x: SEXP, _w: i32) -> *mut c_void {
    real_dataptr_or_null(x) as *mut c_void
}
unsafe extern "C" fn real_finalizer(ep: SEXP) {
    let raw = R_ExternalPtrAddr(ep);
    if !raw.is_null() {
        drop(Box::<Box<dyn RealBackend>>::from_raw(raw.cast()));
    }
}

// ========= STRING class + trampolines =========
unsafe fn str_backend<'a>(x: SEXP) -> &'a dyn StringBackend {
    let ep = R_altrep_data1(x);
    ep_as::<Box<dyn StringBackend>>(ep).as_ref()
}
unsafe extern "C" fn str_len(x: SEXP) -> R_xlen_t {
    str_backend(x).len()
}
unsafe extern "C" fn str_elt(x: SEXP, i: R_xlen_t) -> SEXP {
    match str_backend(x).utf8_at(i) {
        None => NA_STRING,
        Some(s) => {
            // NB: mkCharLen uses native encoding; swap to mkCharLenCE with CE_UTF8 if needed.
            let cs = std::ffi::CString::new(s).unwrap();
            Rf_mkCharLen(cs.as_ptr(), s.len() as i32)
        }
    }
}
unsafe extern "C" fn str_finalizer(ep: SEXP) {
    let raw = R_ExternalPtrAddr(ep);
    if !raw.is_null() {
        drop(Box::<Box<dyn StringBackend>>::from_raw(raw.cast()));
    }
}

// ========= Class registration =========
fn cstr(s: &str) -> *const c_char {
    std::ffi::CString::new(s).unwrap().into_raw()
}

/// Must be called once (lazy-initialized on first constructor use).
unsafe fn ensure_classes() {
    ALTINT.get_or_init(|| {
        let cls =
            R_make_altinteger_class(cstr("rust_altint"), cstr("miniextendr"), ptr::null_mut());
        R_set_altrep_Length_method(cls, Some(int_len));
        R_set_altvec_Dataptr_method(cls, Some(int_dataptr));
        R_set_altvec_Dataptr_or_null_method(cls, Some(int_dataptr_or_null));
        R_set_altinteger_Elt_method(cls, Some(int_elt));
        R_set_altinteger_Get_region_method(cls, Some(int_get_region));
        R_set_altinteger_Is_sorted_method(cls, Some(int_is_sorted));
        R_set_altinteger_No_NA_method(cls, Some(int_no_na));
        cls
    });
    ALTREAL.get_or_init(|| {
        let cls = R_make_altreal_class(cstr("rust_altreal"), cstr("miniextendr"), ptr::null_mut());
        R_set_altrep_Length_method(cls, Some(real_len));
        R_set_altvec_Dataptr_method(cls, Some(real_dataptr));
        R_set_altvec_Dataptr_or_null_method(cls, Some(real_dataptr_or_null));
        R_set_altreal_Elt_method(cls, Some(real_elt));
        R_set_altreal_Get_region_method(cls, Some(real_get_region));
        R_set_altreal_Is_sorted_method(cls, Some(real_is_sorted));
        R_set_altreal_No_NA_method(cls, Some(real_no_na));
        cls
    });
    ALTSTR.get_or_init(|| {
        let cls = R_make_altstring_class(cstr("rust_altstr"), cstr("miniextendr"), ptr::null_mut());
        R_set_altrep_Length_method(cls, Some(str_len));
        R_set_altstring_Elt_method(cls, Some(str_elt));
        cls
    });
}

// ========= Public constructors =========

/// Create an INT ALTREP from a trait object.
pub unsafe fn new_altrep_int(b: Box<dyn IntBackend>) -> SEXP {
    ensure_classes();
    let ep = make_eptr(Box::new(b), int_finalizer);
    R_new_altrep(*ALTINT.get().unwrap(), ep, R_NilValue)
}
/// Create a REAL ALTREP from a trait object.
pub unsafe fn new_altrep_real(b: Box<dyn RealBackend>) -> SEXP {
    ensure_classes();
    let ep = make_eptr(Box::new(b), real_finalizer);
    R_new_altrep(*ALTREAL.get().unwrap(), ep, R_NilValue)
}
/// Create a STRING ALTREP from a trait object.
pub unsafe fn new_altrep_str(b: Box<dyn StringBackend>) -> SEXP {
    ensure_classes();
    let ep = make_eptr(Box::new(b), str_finalizer);
    R_new_altrep(*ALTSTR.get().unwrap(), ep, R_NilValue)
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
            extern "C" {
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

// ========= R-callable C wrappers (no macros, pure .Call) =========

#[no_mangle]
pub unsafe extern "C" fn C_altrep_compact_int(
    _call: SEXP,
    n_: SEXP,
    start_: SEXP,
    step_: SEXP,
) -> SEXP {
    // Expect INTSXP scalars; read via DATAPTR_RO
    let n = *(DATAPTR_RO(n_) as *const i32) as R_xlen_t;
    let start = *(DATAPTR_RO(start_) as *const i32);
    let step = *(DATAPTR_RO(step_) as *const i32);
    if step != 1 && step != -1 {
        return R_NilValue;
    }
    new_altrep_int(Box::new(CompactIntSeq::new(n, start, step)))
}

#[no_mangle]
pub unsafe extern "C" fn C_altrep_from_doubles(_call: SEXP, x: SEXP) -> SEXP {
    let b = OwnedReal::from_reals_sexp(x);
    new_altrep_real(Box::new(b))
}

#[no_mangle]
pub unsafe extern "C" fn C_altrep_from_strings(_call: SEXP, x: SEXP) -> SEXP {
    let b = Utf8Vec::from_strs_sexp(x);
    new_altrep_str(Box::new(b))
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
// TODO: ... similarly AltLogical, AltRaw, AltComplex, AltString, AltList
