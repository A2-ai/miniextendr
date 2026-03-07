use miniextendr_api::altrep_sexp::{AltrepSexp, ensure_materialized};
use miniextendr_api::ffi::{self, SEXP, SEXPTYPE};
use miniextendr_api::{miniextendr, miniextendr_module};

/// Check if a SEXP is ALTREP and return info about it.
///
/// Returns a character vector: c(is_altrep, sexptype, length).
#[miniextendr]
pub fn altrep_sexp_check(x: SEXP) -> Vec<String> {
    let is_altrep = unsafe { ffi::ALTREP(x) } != 0;
    let sexptype = (unsafe { ffi::TYPEOF(x) }) as SEXPTYPE;
    let len = (unsafe { ffi::Rf_xlength(x) }) as usize;

    vec![
        format!("is_altrep={}", is_altrep),
        format!("sexptype={:?}", sexptype),
        format!("length={}", len),
    ]
}

/// Try to wrap a SEXP in AltrepSexp and materialize it as integers.
///
/// Returns the materialized Vec<i32>, or errors if not ALTREP or not INTSXP.
#[miniextendr]
pub fn altrep_sexp_materialize_int(x: SEXP) -> Vec<i32> {
    let altrep = AltrepSexp::try_wrap(x)
        .expect("expected an ALTREP vector");

    assert_eq!(altrep.sexptype(), SEXPTYPE::INTSXP, "expected INTSXP");

    let slice = unsafe { altrep.materialize_integer() };
    slice.to_vec()
}

/// Try to wrap a SEXP in AltrepSexp and materialize it as doubles.
///
/// Returns the materialized Vec<f64>, or errors if not ALTREP or not REALSXP.
#[miniextendr]
pub fn altrep_sexp_materialize_real(x: SEXP) -> Vec<f64> {
    let altrep = AltrepSexp::try_wrap(x)
        .expect("expected an ALTREP vector");

    assert_eq!(altrep.sexptype(), SEXPTYPE::REALSXP, "expected REALSXP");

    let slice = unsafe { altrep.materialize_real() };
    slice.to_vec()
}

/// Try to wrap a SEXP in AltrepSexp and materialize strings.
///
/// Returns character vector, with NA preserved.
#[miniextendr]
pub fn altrep_sexp_materialize_strings(x: SEXP) -> Vec<Option<String>> {
    let altrep = AltrepSexp::try_wrap(x)
        .expect("expected an ALTREP vector");

    assert_eq!(altrep.sexptype(), SEXPTYPE::STRSXP, "expected STRSXP");

    unsafe { altrep.materialize_strings() }
}

/// Use ensure_materialized on any SEXP, then extract as integer vector.
///
/// Works whether or not the input is ALTREP — it materializes if needed
/// and passes through if not.
#[miniextendr]
pub fn altrep_sexp_ensure_materialized_int(x: SEXP) -> Vec<i32> {
    let materialized = unsafe { ensure_materialized(x) };

    // After ensure_materialized, it's safe to convert normally
    miniextendr_api::from_r::TryFromSexp::try_from_sexp(materialized).unwrap()
}

/// Use ensure_materialized on any SEXP, then extract as string vector.
///
/// Works for both ALTREP and non-ALTREP STRSXP.
#[miniextendr]
pub fn altrep_sexp_ensure_materialized_str(x: SEXP) -> Vec<Option<String>> {
    let materialized = unsafe { ensure_materialized(x) };

    miniextendr_api::from_r::TryFromSexp::try_from_sexp(materialized).unwrap()
}

/// Return whether try_wrap returns Some (ALTREP) or None (non-ALTREP).
#[miniextendr]
pub fn altrep_sexp_is_altrep(x: SEXP) -> bool {
    AltrepSexp::try_wrap(x).is_some()
}

miniextendr_module! {
    mod altrep_sexp_tests;
    fn altrep_sexp_check;
    fn altrep_sexp_materialize_int;
    fn altrep_sexp_materialize_real;
    fn altrep_sexp_materialize_strings;
    fn altrep_sexp_ensure_materialized_int;
    fn altrep_sexp_ensure_materialized_str;
    fn altrep_sexp_is_altrep;
}
