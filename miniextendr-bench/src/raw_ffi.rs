//! Raw FFI declarations for benchmarking.
//!
//! These re-declare R C API functions that miniextendr-api has privatized.
//! Bench code needs direct access to measure overhead of safe wrappers.

use miniextendr_api::ffi::{R_xlen_t, Rboolean, Rcomplex, SEXP, SEXPTYPE};

#[allow(non_snake_case, dead_code)]
unsafe extern "C-unwind" {
    pub fn Rf_ScalarInteger(x: i32) -> SEXP;
    pub fn Rf_ScalarReal(x: f64) -> SEXP;
    pub fn Rf_ScalarLogical(x: i32) -> SEXP;
    pub fn Rf_ScalarString(x: SEXP) -> SEXP;
    pub fn Rf_ScalarComplex(x: Rcomplex) -> SEXP;
    pub fn Rf_xlength(x: SEXP) -> R_xlen_t;
    pub fn Rf_asInteger(x: SEXP) -> i32;
    pub fn Rf_allocVector(sexptype: SEXPTYPE, length: R_xlen_t) -> SEXP;
    pub fn Rf_protect(x: SEXP) -> SEXP;
    pub fn Rf_unprotect(n: i32);
    pub fn R_PreserveObject(x: SEXP);
    pub fn R_ReleaseObject(x: SEXP);
    pub fn STRING_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn SET_STRING_ELT(x: SEXP, i: R_xlen_t, v: SEXP);
    pub fn VECTOR_ELT(x: SEXP, i: R_xlen_t) -> SEXP;
    pub fn SET_VECTOR_ELT(x: SEXP, i: R_xlen_t, v: SEXP) -> SEXP;
    pub fn R_CHAR(x: SEXP) -> *const std::os::raw::c_char;
    pub fn LENGTH(x: SEXP) -> i32;
    pub fn Rf_mkCharLenCE(
        s: *const std::os::raw::c_char,
        len: i32,
        encoding: miniextendr_api::ffi::cetype_t,
    ) -> SEXP;
    pub fn Rf_isNewList(s: SEXP) -> Rboolean;
    pub fn Rf_setAttrib(vec: SEXP, name: SEXP, val: SEXP) -> SEXP;
    pub fn Rf_getAttrib(vec: SEXP, name: SEXP) -> SEXP;
    pub fn INTEGER(x: SEXP) -> *mut i32;
    pub fn REAL(x: SEXP) -> *mut f64;
    pub fn LOGICAL(x: SEXP) -> *mut i32;
    pub fn RAW(x: SEXP) -> *mut u8;
    pub fn COMPLEX(x: SEXP) -> *mut Rcomplex;
    pub fn Rf_allocMatrix(sexptype: SEXPTYPE, nrow: i32, ncol: i32) -> SEXP;
    pub fn Rf_Scalar(x: f64) -> SEXP;
    pub fn Rf_lang2(s: SEXP, t: SEXP) -> SEXP;
    pub fn Rf_lang3(s: SEXP, t: SEXP, u: SEXP) -> SEXP;
    pub fn Rf_unprotect_ptr(s: SEXP);
    pub fn Rf_duplicate(x: SEXP) -> SEXP;
    pub fn DATAPTR_RO(x: SEXP) -> *const std::os::raw::c_void;
    pub fn ALTREP(x: SEXP) -> i32;
}

use std::os::raw::c_char;

#[allow(non_snake_case, dead_code)]
unsafe extern "C" {
    pub static R_NilValue: SEXP;
    pub static R_NamesSymbol: SEXP;
    pub static R_BlankString: SEXP;
    pub static R_NaString: SEXP;

    pub fn Rf_install(name: *const c_char) -> SEXP;
    pub fn Rf_mkString(name: *const c_char) -> SEXP;
    pub fn Rf_eval(expr: SEXP, env: SEXP) -> SEXP;
    pub static R_GlobalEnv: SEXP;
}
