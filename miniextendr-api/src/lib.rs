pub use miniextendr_macros::miniextendr;
pub use miniextendr_macros::miniextendr_module;

pub mod ffi {

    #[non_exhaustive]
    #[repr(transparent)]
    #[derive(Debug)]
    pub struct SEXPREC(std::ffi::c_void);
    pub type SEXP = *mut SEXPREC;

    #[repr(i32)]
    #[non_exhaustive]
    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub enum Rboolean {
        FALSE = 0,
        TRUE = 1,
    }

    unsafe extern "C" {
        #[allow(dead_code)]
        pub static R_NilValue: SEXP;

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
        // pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
        // pub fn SET_LOGICAL_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
        // pub fn SET_INTEGER_ELT(x: SEXP, i: R_xlen_t, v: ::std::os::raw::c_int);
        // pub fn SET_REAL_ELT(x: SEXP, i: R_xlen_t, v: f64);
        // pub fn SET_COMPLEX_ELT(x: SEXP, i: R_xlen_t, v: Rcomplex);
        // pub fn SET_RAW_ELT(x: SEXP, i: R_xlen_t, v: Rbyte);

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
    }

    // region: registration!

    #[repr(C)]
    #[derive(Debug)]
    pub struct DllInfo(std::ffi::c_void);

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

    // necessary for calling R_init_<module name>
    unsafe impl Sync for R_CallMethodDef {}

    // FIXME: move to an ffi crate or similar..
    unsafe extern "C" {
        pub fn R_registerRoutines(
            info: *mut DllInfo,
            // croutines: *const R_CMethodDef,
            croutines: *const std::ffi::c_void,
            callRoutines: *const R_CallMethodDef,
            // fortranRoutines: *const R_FortranMethodDef,
            fortranRoutines: *const std::ffi::c_void,
            // externalRoutines: *const R_ExternalMethodDef,
            externalRoutines: *const std::ffi::c_void,
        ) -> ::std::os::raw::c_int;

        pub fn R_useDynamicSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
        pub fn R_forceSymbols(info: *mut DllInfo, value: Rboolean) -> Rboolean;
    }

    // endregion
}

// #[doc(hidden)]
// #[repr(transparent)]
// #[derive(Debug)]
// pub struct SEXPREC(std::ffi::c_void);
// pub type SEXP = *mut SEXPREC;
