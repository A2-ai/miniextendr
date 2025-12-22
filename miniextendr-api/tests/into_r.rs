//! Integration tests for IntoR conversions that require an embedded R runtime.

use miniextendr_api::altrep_traits::NA_LOGICAL;
use miniextendr_api::ffi::{
    LOGICAL, R_NaString, R_xlen_t, RLogical, Rboolean, Rf_translateCharUTF8, Rf_xlength, SEXP,
    SEXPTYPE, STRING_ELT, TYPEOF,
};
use miniextendr_api::into_r::IntoR;
use std::ffi::CStr;
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_r() {
    INIT.call_once(|| unsafe {
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as rpkg/src/entrypoint.c.in
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::worker::miniextendr_worker_init();
        std::mem::forget(engine);
    });
}

unsafe fn scalar_logical(sexp: SEXP) -> i32 {
    unsafe { *LOGICAL(sexp) }
}

unsafe fn string_elt(sexp: SEXP, idx: usize) -> Option<String> {
    unsafe {
        let charsxp = STRING_ELT(sexp, idx as R_xlen_t);
        if charsxp == R_NaString {
            None
        } else {
            let c_str = Rf_translateCharUTF8(charsxp);
            Some(CStr::from_ptr(c_str).to_string_lossy().into_owned())
        }
    }
}

#[test]
fn into_r_suite() {
    initialize_r();

    test_option_rlogical_scalar();
    test_vec_option_rlogical();
    test_vec_option_rboolean();
    test_string_slice();
}

fn test_option_rlogical_scalar() {
    let sexp = Option::<RLogical>::Some(RLogical::TRUE).into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::LGLSXP);
    assert_eq!(unsafe { scalar_logical(sexp) }, 1);

    let sexp_na = Option::<RLogical>::None.into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp_na) }, SEXPTYPE::LGLSXP);
    assert_eq!(unsafe { scalar_logical(sexp_na) }, NA_LOGICAL);
}

fn test_vec_option_rlogical() {
    let sexp = vec![Some(RLogical::TRUE), None, Some(RLogical::FALSE)].into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::LGLSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 3);

    let slice = unsafe { std::slice::from_raw_parts(LOGICAL(sexp), 3) };
    assert_eq!(slice, &[1, NA_LOGICAL, 0]);
}

fn test_vec_option_rboolean() {
    let sexp = vec![Some(Rboolean::TRUE), None, Some(Rboolean::FALSE)].into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::LGLSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 3);

    let slice = unsafe { std::slice::from_raw_parts(LOGICAL(sexp), 3) };
    assert_eq!(slice, &[1, NA_LOGICAL, 0]);
}

fn test_string_slice() {
    let items = vec!["alpha".to_string(), "beta".to_string()];
    let sexp = items.as_slice().into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::STRSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 2);

    assert_eq!(unsafe { string_elt(sexp, 0) }, Some("alpha".to_string()));
    assert_eq!(unsafe { string_elt(sexp, 1) }, Some("beta".to_string()));
}
