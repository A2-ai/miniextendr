//! Integration tests for IntoR conversions that require an embedded R runtime.

mod r_test_utils;

use miniextendr_api::altrep_traits::NA_LOGICAL;
use miniextendr_api::ffi::{
    LOGICAL, R_NaString, R_xlen_t, RLogical, Rboolean, Rf_translateCharUTF8, Rf_xlength, SEXP,
    SEXPTYPE, STRING_ELT, TYPEOF,
};
use miniextendr_api::into_r::IntoR;
use std::ffi::CStr;

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
    r_test_utils::with_r_thread(|| {
        test_option_rlogical_scalar();
        test_vec_option_rlogical();
        test_vec_option_rboolean();
        test_string_slice();
        test_as_named_list_vec();
        test_as_named_list_array();
        test_as_named_vector_vec();
        test_as_named_vector_array();
        test_as_named_vector_option();
        test_as_named_list_slice();
        test_as_named_vector_slice();
    });
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

// ── AsNamedList / AsNamedVector tests ────────────────────────────────────────

use miniextendr_api::ffi::{INTEGER, R_NamesSymbol, REAL, Rf_getAttrib, VECTOR_ELT};
use miniextendr_api::{AsNamedList, AsNamedListExt, AsNamedVector, AsNamedVectorExt};

/// Extract names from an R SEXP as Vec<String>.
unsafe fn extract_names(sexp: SEXP) -> Vec<String> {
    unsafe {
        let names = Rf_getAttrib(sexp, R_NamesSymbol);
        let n = Rf_xlength(names);
        (0..n)
            .map(|i| {
                let charsxp = STRING_ELT(names, i);
                let c_str = Rf_translateCharUTF8(charsxp);
                CStr::from_ptr(c_str).to_string_lossy().into_owned()
            })
            .collect()
    }
}

fn test_as_named_list_vec() {
    let pairs: Vec<(String, i32)> = vec![("a".into(), 1), ("b".into(), 2), ("c".into(), 3)];
    let sexp = AsNamedList(pairs).into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::VECSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 3);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["a", "b", "c"]);

    // Values are length-1 integer vectors
    assert_eq!(unsafe { *INTEGER(VECTOR_ELT(sexp, 0)) }, 1);
    assert_eq!(unsafe { *INTEGER(VECTOR_ELT(sexp, 1)) }, 2);
    assert_eq!(unsafe { *INTEGER(VECTOR_ELT(sexp, 2)) }, 3);
}

fn test_as_named_list_array() {
    let pairs = [("x", 1.0f64), ("y", 2.0)];
    let sexp = pairs.as_named_list().into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::VECSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 2);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["x", "y"]);
}

fn test_as_named_vector_vec() {
    let pairs: Vec<(&str, i32)> = vec![("alice", 95), ("bob", 87)];
    let sexp = AsNamedVector(pairs).into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::INTSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 2);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["alice", "bob"]);
    let data = unsafe { std::slice::from_raw_parts(INTEGER(sexp), 2) };
    assert_eq!(data, &[95, 87]);
}

fn test_as_named_vector_array() {
    let pairs = [("x", 1.0f64), ("y", 2.0), ("z", 3.0)];
    let sexp = pairs.as_named_vector().into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::REALSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 3);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["x", "y", "z"]);

    let data = unsafe { std::slice::from_raw_parts(REAL(sexp), 3) };
    assert_eq!(data, &[1.0, 2.0, 3.0]);
}

fn test_as_named_vector_option() {
    let pairs: Vec<(&str, Option<i32>)> = vec![("a", Some(1)), ("b", None), ("c", Some(3))];
    let sexp = AsNamedVector(pairs).into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::INTSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 3);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["a", "b", "c"]);

    let data = unsafe { std::slice::from_raw_parts(INTEGER(sexp), 3) };
    assert_eq!(data[0], 1);
    assert_eq!(data[1], i32::MIN); // NA_integer_
    assert_eq!(data[2], 3);
}

fn test_as_named_list_slice() {
    let pairs: &[(&str, i32)] = &[("a", 1), ("b", 2)];
    let sexp = AsNamedList(pairs).into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::VECSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 2);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["a", "b"]);
}

fn test_as_named_vector_slice() {
    let pairs: &[(&str, f64)] = &[("x", 1.0), ("y", 2.0)];
    let sexp = pairs.as_named_vector().into_sexp();
    assert_eq!(unsafe { TYPEOF(sexp) }, SEXPTYPE::REALSXP);
    assert_eq!(unsafe { Rf_xlength(sexp) }, 2);

    let names = unsafe { extract_names(sexp) };
    assert_eq!(names, vec!["x", "y"]);

    let data = unsafe { std::slice::from_raw_parts(REAL(sexp), 2) };
    assert_eq!(data, &[1.0, 2.0]);
}
