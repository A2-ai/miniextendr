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
    #[doc = " promises: [un]evaluated closure arguments"]
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

/// Send-only handle to a SEXP pointer.
#[repr(transparent)]
#[derive(Debug)]
pub struct SendSEXP {
    pub inner: SEXP,
    /// PhantomData<Cell<()>> forces !Sync.
    pub _not_sync: std::marker::PhantomData<std::cell::Cell<()>>,
}
unsafe impl Send for SendSEXP {}

impl SendSEXP {
    #[inline(always)]
    /// # Safety
    /// Caller must supply a valid SEXP pointer whose ownership can be
    /// transferred to this sendable wrapper.
    pub unsafe fn new(inner: SEXP) -> Self {
        Self {
            inner,
            _not_sync: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(self) -> SEXP {
        self.inner
    }
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum Rboolean {
    FALSE = 0,
    TRUE = 1,
}

// TODO: I don't think `R_CFinalizer_t` can be None, so maybe it ought to be NonNull
#[allow(non_camel_case_types)]
pub type R_CFinalizer_t = ::std::option::Option<unsafe extern "C" fn(arg1: SEXP)>;

unsafe extern "C" {
    #[allow(dead_code)]
    pub static R_NilValue: SEXP;

    pub static R_NaString: SEXP;

    // Rinternals.h
    pub fn Rf_errorcall(arg1: SEXP, arg2: *const ::std::os::raw::c_char, ...) -> !;
    pub fn Rf_mkCharLen(s: *const ::std::os::raw::c_char, len: i32) -> SEXP;
    pub fn Rf_xlength(x: SEXP) -> R_xlen_t;

    // R_ext/Error.h
    pub fn Rf_error(arg1: *const ::std::os::raw::c_char, ...) -> !;
    pub fn Rprintf(arg1: *const ::std::os::raw::c_char, ...);

    pub fn R_MakeUnwindCont() -> SEXP;
    pub fn R_ContinueUnwind(cont: SEXP) -> !;
    pub fn R_UnwindProtect(
        fun: ::std::option::Option<unsafe extern "C" fn(*mut ::std::os::raw::c_void) -> SEXP>,
        fun_data: *mut ::std::os::raw::c_void,
        cleanfun: ::std::option::Option<
            unsafe extern "C" fn(*mut ::std::os::raw::c_void, Rboolean),
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
    pub fn R_ExternalPtrAddrFn(s: SEXP) -> DL_FUNC;
    pub fn R_RegisterFinalizer(s: SEXP, fun: SEXP);
    pub fn R_RegisterCFinalizer(s: SEXP, fun: R_CFinalizer_t);
    pub fn R_RegisterFinalizerEx(s: SEXP, fun: SEXP, onexit: Rboolean);
    pub fn R_RegisterCFinalizerEx(s: SEXP, fun: R_CFinalizer_t, onexit: Rboolean);

    // Rinternals.h
    pub fn R_PreserveObject(arg1: SEXP);
    pub fn R_ReleaseObject(arg1: SEXP);

    pub fn Rf_protect(arg1: SEXP) -> SEXP;
    pub fn Rf_unprotect(arg1: ::std::os::raw::c_int);

    // Rinternals.h
    // pub fn Rf_ScalarComplex(arg1: Rcomplex) -> SEXP;
    pub fn Rf_ScalarInteger(arg1: ::std::os::raw::c_int) -> SEXP;
    pub fn Rf_ScalarLogical(arg1: ::std::os::raw::c_int) -> SEXP;
    // pub fn Rf_ScalarRaw(arg1: Rbyte) -> SEXP;
    pub fn Rf_ScalarReal(arg1: f64) -> SEXP;
    pub fn Rf_ScalarString(arg1: SEXP) -> SEXP;

    // Rinternals.h
    pub fn DATAPTR(x: SEXP) -> *mut ::std::os::raw::c_void;
    pub fn DATAPTR_RO(x: SEXP) -> *const ::std::os::raw::c_void;
    pub fn DATAPTR_OR_NULL(x: SEXP) -> *const ::std::os::raw::c_void;
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
    pub fn LOGICAL0(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn INTEGER0(x: SEXP) -> *mut ::std::os::raw::c_int;
    pub fn REAL0(x: SEXP) -> *mut f64;
    // pub fn COMPLEX0(x: SEXP) -> *mut Rcomplex;
    // pub fn RAW0(x: SEXP) -> *mut Rbyte;
    pub fn ALTREP(x: SEXP) -> ::std::os::raw::c_int;

    // utils.h
    pub fn R_CheckUserInterrupt();
}

// region: registration!

#[repr(C)]
#[derive(Debug)]
pub struct DllInfo(::std::os::raw::c_void);

#[allow(non_camel_case_types)]
pub type DL_FUNC = ::std::option::Option<unsafe extern "C" fn(...) -> SEXP>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct R_CallMethodDef {
    pub name: *const ::std::os::raw::c_char,
    pub fun: DL_FUNC,
    pub numArgs: ::std::os::raw::c_int,
}
// TODO: investigate why Sync is necessary...
unsafe impl Sync for R_CallMethodDef {}

// FIXME: move to an ffi crate or similar..
unsafe extern "C" {
    pub fn R_registerRoutines(
        info: *mut DllInfo,
        // croutines: *const R_CMethodDef,
        croutines: *const ::std::os::raw::c_void,
        callRoutines: *const R_CallMethodDef,
        // fortranRoutines: *const R_FortranMethodDef,
        fortranRoutines: *const ::std::os::raw::c_void,
        // externalRoutines: *const R_ExternalMethodDef,
        externalRoutines: *const ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;

    pub fn R_useDynamicSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
    pub fn R_forceSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
}

// endregion
