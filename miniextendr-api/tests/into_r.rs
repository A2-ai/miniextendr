//! Integration tests for IntoR conversions that require an embedded R runtime.

mod r_test_utils;

use miniextendr_api::altrep_traits::NA_LOGICAL;
use miniextendr_api::ffi::{R_xlen_t, RLogical, Rboolean, SEXP, SEXPTYPE, SexpExt};
use miniextendr_api::into_r::IntoR;

fn scalar_logical(sexp: SEXP) -> i32 {
    sexp.logical_elt(0)
}

fn string_elt_val(sexp: SEXP, idx: usize) -> Option<String> {
    sexp.string_elt_str(idx as R_xlen_t).map(|s| s.to_string())
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
    assert_eq!(sexp.type_of(), SEXPTYPE::LGLSXP);
    assert_eq!(scalar_logical(sexp), 1);

    let sexp_na = Option::<RLogical>::None.into_sexp();
    assert_eq!(sexp_na.type_of(), SEXPTYPE::LGLSXP);
    assert_eq!(scalar_logical(sexp_na), NA_LOGICAL);
}

fn test_vec_option_rlogical() {
    let sexp = vec![Some(RLogical::TRUE), None, Some(RLogical::FALSE)].into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::LGLSXP);
    assert_eq!(sexp.xlength(), 3);

    let elts: Vec<i32> = (0..3).map(|i| sexp.logical_elt(i)).collect();
    assert_eq!(elts, &[1, NA_LOGICAL, 0]);
}

fn test_vec_option_rboolean() {
    let sexp = vec![Some(Rboolean::TRUE), None, Some(Rboolean::FALSE)].into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::LGLSXP);
    assert_eq!(sexp.xlength(), 3);

    let elts: Vec<i32> = (0..3).map(|i| sexp.logical_elt(i)).collect();
    assert_eq!(elts, &[1, NA_LOGICAL, 0]);
}

fn test_string_slice() {
    let items = vec!["alpha".to_string(), "beta".to_string()];
    let sexp = items.as_slice().into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::STRSXP);
    assert_eq!(sexp.xlength(), 2);

    assert_eq!(string_elt_val(sexp, 0), Some("alpha".to_string()));
    assert_eq!(string_elt_val(sexp, 1), Some("beta".to_string()));
}

// region: AsNamedList / AsNamedVector tests

// SexpExt already imported above
use miniextendr_api::{AsNamedList, AsNamedListExt, AsNamedVector, AsNamedVectorExt};

/// Extract names from an R SEXP as Vec<String>.
fn extract_names(sexp: SEXP) -> Vec<String> {
    let names = sexp.get_names();
    let n = names.len();
    (0..n)
        .map(|i| {
            names
                .string_elt_str(i as R_xlen_t)
                .unwrap_or("")
                .to_string()
        })
        .collect()
}

fn test_as_named_list_vec() {
    let pairs: Vec<(String, i32)> = vec![("a".into(), 1), ("b".into(), 2), ("c".into(), 3)];
    let sexp = AsNamedList(pairs).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::VECSXP);
    assert_eq!(sexp.xlength(), 3);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["a", "b", "c"]);

    // Values are length-1 integer vectors
    assert_eq!(sexp.vector_elt(0).as_integer(), Some(1));
    assert_eq!(sexp.vector_elt(1).as_integer(), Some(2));
    assert_eq!(sexp.vector_elt(2).as_integer(), Some(3));
}

fn test_as_named_list_array() {
    let pairs = [("x", 1.0f64), ("y", 2.0)];
    let sexp = pairs.wrap_named_list().into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::VECSXP);
    assert_eq!(sexp.xlength(), 2);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["x", "y"]);
}

fn test_as_named_vector_vec() {
    let pairs: Vec<(&str, i32)> = vec![("alice", 95), ("bob", 87)];
    let sexp = AsNamedVector(pairs).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    assert_eq!(sexp.xlength(), 2);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["alice", "bob"]);
    let data: &[i32] = unsafe { sexp.as_slice() };
    assert_eq!(data, &[95, 87]);
}

fn test_as_named_vector_array() {
    let pairs = [("x", 1.0f64), ("y", 2.0), ("z", 3.0)];
    let sexp = pairs.wrap_named_vector().into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    assert_eq!(sexp.xlength(), 3);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["x", "y", "z"]);

    let data: &[f64] = unsafe { sexp.as_slice() };
    assert_eq!(data, &[1.0, 2.0, 3.0]);
}

fn test_as_named_vector_option() {
    let pairs: Vec<(&str, Option<i32>)> = vec![("a", Some(1)), ("b", None), ("c", Some(3))];
    let sexp = AsNamedVector(pairs).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
    assert_eq!(sexp.xlength(), 3);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["a", "b", "c"]);

    let data: &[i32] = unsafe { sexp.as_slice() };
    assert_eq!(data[0], 1);
    assert_eq!(data[1], i32::MIN); // NA_integer_
    assert_eq!(data[2], 3);
}

fn test_as_named_list_slice() {
    let pairs: &[(&str, i32)] = &[("a", 1), ("b", 2)];
    let sexp = AsNamedList(pairs).into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::VECSXP);
    assert_eq!(sexp.xlength(), 2);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["a", "b"]);
}

fn test_as_named_vector_slice() {
    let pairs: &[(&str, f64)] = &[("x", 1.0), ("y", 2.0)];
    let sexp = pairs.wrap_named_vector().into_sexp();
    assert_eq!(sexp.type_of(), SEXPTYPE::REALSXP);
    assert_eq!(sexp.xlength(), 2);

    let names = extract_names(sexp);
    assert_eq!(names, vec!["x", "y"]);

    let data: &[f64] = unsafe { sexp.as_slice() };
    assert_eq!(data, &[1.0, 2.0]);
}
// endregion
