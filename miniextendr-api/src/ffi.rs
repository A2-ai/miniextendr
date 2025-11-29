pub mod altrep;

#[allow(non_camel_case_types)]
pub type R_xlen_t = isize;
pub type Rbyte = ::std::os::raw::c_uchar;

#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum SEXPTYPE {
    #[doc = " nil = NULL"]
    NILSXP = 0,
    #[doc = " symbols"]
    SYMSXP = 1,
    #[doc = " lists of dotted pairs"]
    LISTSXP = 2,
    #[doc = " closures"]
    CLOSXP = 3,
    #[doc = " environments"]
    ENVSXP = 4,
    #[doc = r" promises: \[un\]evaluated closure arguments"]
    PROMSXP = 5,
    #[doc = " language constructs (special lists)"]
    LANGSXP = 6,
    #[doc = " special forms"]
    SPECIALSXP = 7,
    #[doc = " builtin non-special forms"]
    BUILTINSXP = 8,
    #[doc = " \"scalar\" string type (internal only)"]
    CHARSXP = 9,
    #[doc = " logical vectors"]
    LGLSXP = 10,
    #[doc = " integer vectors"]
    INTSXP = 13,
    #[doc = " real variables"]
    REALSXP = 14,
    #[doc = " complex variables"]
    CPLXSXP = 15,
    #[doc = " string vectors"]
    STRSXP = 16,
    #[doc = " dot-dot-dot object"]
    DOTSXP = 17,
    #[doc = " make \"any\" args work"]
    ANYSXP = 18,
    #[doc = " generic vectors"]
    VECSXP = 19,
    #[doc = " expressions vectors"]
    EXPRSXP = 20,
    #[doc = " byte code"]
    BCODESXP = 21,
    #[doc = " external pointer"]
    EXTPTRSXP = 22,
    #[doc = " weak reference"]
    WEAKREFSXP = 23,
    #[doc = " raw bytes"]
    RAWSXP = 24,
    #[doc = " S4 non-vector"]
    S4SXP = 25,
    #[doc = " fresh node created in new page"]
    NEWSXP = 30,
    #[doc = " node released by GC"]
    FREESXP = 31,
    #[doc = " Closure or Builtin"]
    FUNSXP = 99,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct SEXPREC(::std::os::raw::c_void);
pub type SEXP = *mut SEXPREC;

/// Extension trait for SEXP providing safe(r) accessors.
pub(crate) trait SexpExt {
    /// Get the type of this SEXP.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid (not null and not freed).
    fn type_of(&self) -> SEXPTYPE;

    /// Check if this SEXP is null or R_NilValue.
    fn is_null_or_nil(&self) -> bool;

    /// Get the length of this SEXP.
    ///
    /// # Safety
    ///
    /// The SEXP must be valid.
    fn xlength(&self) -> R_xlen_t;

    /// Get a slice view of this SEXP's data.
    ///
    /// # Safety
    ///
    /// - The SEXP must be valid and of the correct type for T
    /// - The returned slice borrows from R's memory; the SEXP must remain protected
    fn as_slice<T: RNativeType>(&self) -> &'static [T];
}

impl SexpExt for SEXP {
    #[inline]
    fn type_of(&self) -> SEXPTYPE {
        unsafe { TYPEOF(*self) }
    }

    #[inline]
    fn is_null_or_nil(&self) -> bool {
        self.is_null() || std::ptr::addr_eq(*self, unsafe { R_NilValue })
    }

    #[inline]
    fn xlength(&self) -> R_xlen_t {
        unsafe { Rf_xlength(*self) }
    }

    #[inline]
    fn as_slice<T: RNativeType>(&self) -> &'static [T] {
        debug_assert!(
            self.type_of() == T::SEXP_TYPE,
            "SEXP type mismatch: expected {:?}, got {:?}",
            T::SEXP_TYPE,
            self.type_of()
        );
        let len = self.xlength() as usize;
        if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(DATAPTR_RO(*self).cast(), len) }
        }
    }
}

/// Marker trait for types that correspond to R's native vector element types.
///
/// This enables blanket implementations for `TryFromSexp` and safe conversions.
pub(crate) trait RNativeType: Sized + Copy + 'static {
    /// The SEXPTYPE for vectors containing this element type.
    const SEXP_TYPE: SEXPTYPE;
}

impl RNativeType for i32 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::INTSXP;
}

impl RNativeType for f64 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::REALSXP;
}

impl RNativeType for u8 {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::RAWSXP;
}

impl RNativeType for Rboolean {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::LGLSXP;
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Rboolean {
    FALSE = 0,
    TRUE = 1,
}

impl From<bool> for Rboolean {
    fn from(value: bool) -> Self {
        match value {
            true => Rboolean::TRUE,
            false => Rboolean::FALSE,
        }
    }
}

impl From<Rboolean> for bool {
    fn from(value: Rboolean) -> Self {
        match value {
            Rboolean::FALSE => false,
            Rboolean::TRUE => true,
        }
    }
}

#[allow(non_camel_case_types)]
pub type R_CFinalizer_t = ::std::option::Option<unsafe extern "C-unwind" fn(arg1: SEXP)>;

#[allow(non_camel_case_types)]
pub type R_CFinalizer_t_C_unwind = ::std::option::Option<unsafe extern "C-unwind" fn(arg1: SEXP)>;

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum cetype_t {
    CE_NATIVE = 0,
    CE_UTF8 = 1,
}
pub use cetype_t::CE_UTF8;
use miniextendr_macros::r_ffi_checked;

// Unchecked variadic functions (internal use only, no thread check)
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    #[link_name = "Rf_error"]
    pub fn Rf_error_unchecked(arg1: *const ::std::os::raw::c_char, ...) -> !;
    #[link_name = "Rf_errorcall"]
    pub fn Rf_errorcall_unchecked(arg1: SEXP, arg2: *const ::std::os::raw::c_char, ...) -> !;
    #[link_name = "Rf_warning"]
    pub fn Rf_warning_unchecked(arg1: *const ::std::os::raw::c_char, ...);
    #[link_name = "Rprintf"]
    pub fn Rprintf_unchecked(arg1: *const ::std::os::raw::c_char, ...);
}

/// Checked wrapper for `Rf_error` - panics if called from non-main thread.
/// Common usage: `Rf_error(c"%s".as_ptr(), message.as_ptr())`
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_error(
    fmt: *const ::std::os::raw::c_char,
    arg1: *const ::std::os::raw::c_char,
) -> ! {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_error called from non-main thread");
    }
    unsafe { Rf_error_unchecked(fmt, arg1) }
}

/// Checked wrapper for `Rf_errorcall` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `call` must be a valid SEXP or R_NilValue
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_errorcall(
    call: SEXP,
    fmt: *const ::std::os::raw::c_char,
    arg1: *const ::std::os::raw::c_char,
) -> ! {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_errorcall called from non-main thread");
    }
    unsafe { Rf_errorcall_unchecked(call, fmt, arg1) }
}

/// Checked wrapper for `Rf_warning` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rf_warning(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("Rf_warning called from non-main thread");
    }
    unsafe { Rf_warning_unchecked(fmt, arg1) }
}

/// Checked wrapper for `Rprintf` - panics if called from non-main thread.
///
/// # Safety
///
/// - Must be called from the R main thread
/// - `fmt` and `arg1` must be valid null-terminated C strings
#[inline(always)]
#[allow(non_snake_case)]
pub unsafe fn Rprintf(fmt: *const ::std::os::raw::c_char, arg1: *const ::std::os::raw::c_char) {
    if !crate::worker::is_r_main_thread() {
        panic!("Rprintf called from non-main thread");
    }
    unsafe { Rprintf_unchecked(fmt, arg1) }
}

#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    #[allow(dead_code)]
    pub static R_NilValue: SEXP;

    pub static R_NaString: SEXP;

    // Rinternals.h
    pub fn Rf_mkCharLen(s: *const ::std::os::raw::c_char, len: i32) -> SEXP;
    pub fn Rf_mkCharLenCE(
        x: *const ::std::os::raw::c_char,
        len: ::std::os::raw::c_int,
        ce: cetype_t,
    ) -> SEXP;
    pub fn Rf_xlength(x: SEXP) -> R_xlen_t;
    pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn Rf_translateCharUTF8(x: SEXP) -> *const ::std::os::raw::c_char;

    pub fn R_MakeUnwindCont() -> SEXP;
    pub fn R_ContinueUnwind(cont: SEXP) -> !;
    pub fn R_UnwindProtect(
        fun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void) -> SEXP,
        >,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    /// Version of `R_UnwindProtect` that accepts `extern "C-unwind"` function pointers
    #[link_name = "R_UnwindProtect"]
    pub fn R_UnwindProtect_C_unwind(
        fun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void) -> SEXP,
        >,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C-unwind" fn(*mut ::std::os::raw::c_void, Rboolean),
        >,
        cleanfun_data: *mut ::std::os::raw::c_void,
        cont: SEXP,
    ) -> SEXP;

    // Rinternals.h
    #[doc = " External pointer interface"]
    pub fn R_MakeExternalPtr(p: *mut ::std::os::raw::c_void, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddr(s: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn R_ExternalPtrTag(s: SEXP) -> SEXP;
    pub fn R_ExternalPtrProtected(s: SEXP) -> SEXP;
    pub fn R_ClearExternalPtr(s: SEXP);
    pub fn R_SetExternalPtrAddr(s: SEXP, p: *mut ::std::os::raw::c_void);
    pub fn R_SetExternalPtrTag(s: SEXP, tag: SEXP);
    pub fn R_SetExternalPtrProtected(s: SEXP, p: SEXP);
    #[doc = " Added in R 3.4.0"]
    pub fn R_MakeExternalPtrFn(p: DL_FUNC, tag: SEXP, prot: SEXP) -> SEXP;
    #[link_name = "R_MakeExternalPtrFn"]
    pub fn R_MakeExternalPtrFn_C_unwind(p: DL_FUNC_C_unwind, tag: SEXP, prot: SEXP) -> SEXP;
    pub fn R_ExternalPtrAddrFn(s: SEXP) -> DL_FUNC;
    #[link_name = "R_ExternalPtrAddrFn"]
    pub fn R_ExternalPtrAddrFn_C_unwind(s: SEXP) -> DL_FUNC_C_unwind;
    pub fn R_RegisterFinalizer(s: SEXP, fun: SEXP);
    pub fn R_RegisterCFinalizer(s: SEXP, fun: R_CFinalizer_t);
    pub fn R_RegisterFinalizerEx(s: SEXP, fun: SEXP, onexit: Rboolean);
    pub fn R_RegisterCFinalizerEx(s: SEXP, fun: R_CFinalizer_t, onexit: Rboolean);

    // Rinternals.h
    pub fn R_PreserveObject(arg1: SEXP);
    pub fn R_ReleaseObject(arg1: SEXP);

    pub fn Rf_protect(arg1: SEXP) -> SEXP;
    pub fn Rf_unprotect(arg1: ::std::os::raw::c_int);
    pub fn Rf_allocVector(arg1: SEXPTYPE, arg2: R_xlen_t) -> SEXP;

    // Rinternals.h
    // pub fn Rf_ScalarComplex(arg1: Rcomplex) -> SEXP;
    pub fn Rf_ScalarInteger(arg1: ::std::os::raw::c_int) -> SEXP;
    pub fn Rf_ScalarLogical(arg1: ::std::os::raw::c_int) -> SEXP;
    pub fn Rf_ScalarRaw(arg1: Rbyte) -> SEXP;
    pub fn Rf_ScalarReal(arg1: f64) -> SEXP;
    pub fn Rf_ScalarString(arg1: SEXP) -> SEXP;

    // Rinternals.h
    pub fn DATAPTR(x: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn DATAPTR_RO(x: SEXP) -> *const ::std::os::raw::c_void;
    pub fn DATAPTR_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_void;

    // Cons cell accessors
    pub fn CAR(e: SEXP) -> SEXP;
    pub fn CDR(e: SEXP) -> SEXP;
    pub fn SETCAR(x: SEXP, y: SEXP) -> SEXP;
    pub fn LOGICAL_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn INTEGER_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_int;
    pub fn REAL_OR_NULL(x: SEXP) -> *const f64;
    // pub fn COMPLEX_OR_NULL(x: SEXP) -> *const Rcomplex;
    // pub fn RAW_OR_NULL(x: SEXP) -> *const Rbyte;
    // pub fn INTEGER_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    // pub fn REAL_ELT(x: SEXP, i: R_xlen_t) -> f64;
    // pub fn LOGICAL_ELT(x: SEXP, i: R_xlen_t) -> ::std::os::raw::c_int;
    // pub fn COMPLEX_ELT(x: SEXP, i: R_xlen_t) -> Rcomplex;
    // pub fn RAW_ELT(x: SEXP, i: R_xlen_t) -> Rbyte;
    pub fn VECTOR_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    // pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    // pub fn SET_LOGICAL_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    // pub fn SET_INTEGER_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
    // pub fn SET_REAL_ELT(x: SEXP, i: R_xlen_t, v: f64);
    // pub fn SET_COMPLEX_ELT(x: SEXP, i: R_xlen_t, v: Rcomplex);
    // pub fn SET_RAW_ELT(x: SEXP, i: R_xlen_t, v: Rbyte);
    pub fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP);

    pub fn ALTREP_CLASS(x: SEXP) -> SEXP;
    pub fn R_altrep_data1(x: SEXP) -> SEXP;
    pub fn R_altrep_data2(x: SEXP) -> SEXP;
    pub fn R_set_altrep_data1(x: SEXP, v: SEXP);
    pub fn R_set_altrep_data2(x: SEXP, v: SEXP);
    pub fn LOGICAL(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn INTEGER(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn REAL(x: SEXP) -> *mut f64;
    // pub fn COMPLEX(x: SEXP) -> *mut Rcomplex;
    pub fn RAW(x: SEXP) -> *mut Rbyte;
    pub fn ALTREP(x: SEXP) -> ::std::os::raw::c_int;

    // utils.h
    pub fn R_CheckUserInterrupt();

    // Type checking
    pub fn TYPEOF(x: SEXP) -> SEXPTYPE;

    // Symbol creation
    pub fn Rf_install(name: *const ::std::os::raw::c_char) -> SEXP;
}

/// Check if a SEXP is an S4 object.
///
/// # Safety
///
/// - `arg1` must be a valid SEXP
#[allow(non_snake_case)]
pub unsafe fn Rf_isS4(arg1: SEXP) -> Rboolean {
    unsafe extern "C-unwind" {
        #[link_name = "Rf_isS4"]
        pub fn Rf_isS4_original(arg1: SEXP) -> u32;
    }

    unsafe {
        if Rf_isS4_original(arg1) == 0 {
            Rboolean::FALSE
        } else {
            Rboolean::TRUE
        }
    }
}

// region: registration!

#[repr(C)]
#[derive(Debug)]
pub struct DllInfo(::std::os::raw::c_void);

#[allow(non_camel_case_types)]
pub type DL_FUNC = ::std::option::Option<unsafe extern "C-unwind" fn(...) -> SEXP>;

#[allow(non_camel_case_types)]
pub type DL_FUNC_C_unwind = ::std::option::Option<unsafe extern "C-unwind" fn(...) -> SEXP>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC,
    pub numArgs: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef_C_unwind {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC_C_unwind,
    pub numArgs: ::std::os::raw::c_int,
}

// SAFETY: R_CallMethodDef contains raw pointers which don't impl Sync by default.
// However, Sync is required to store these in static arrays for R's method registration.
// This is safe because:
// 1. The name pointer points to static C string literals (&'static CStr)
// 2. The fun pointer is a static function pointer
// 3. These are read-only after initialization during R_init_*
unsafe impl Sync for R_CallMethodDef {}

#[r_ffi_checked]
#[allow(clashing_extern_declarations)]
#[allow(non_snake_case)]
unsafe extern "C-unwind" {
    pub fn R_registerRoutines(
        info: *mut DllInfo,
        croutines: *const ::std::os::raw::c_void,
        callRoutines: *const R_CallMethodDef,
        fortranRoutines: *const ::std::os::raw::c_void,
        externalRoutines: *const ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;

    #[link_name = "R_registerRoutines"]
    pub fn R_registerRoutines_C_unwind(
        info: *mut DllInfo,
        croutines: *const ::std::os::raw::c_void,
        callRoutines: *const R_CallMethodDef_C_unwind,
        fortranRoutines: *const ::std::os::raw::c_void,
        externalRoutines: *const ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;

    pub fn R_useDynamicSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
    pub fn R_forceSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
}

// endregion
