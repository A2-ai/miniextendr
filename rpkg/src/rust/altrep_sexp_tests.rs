use miniextendr_api::IntoR;
use miniextendr_api::altrep_sexp::{AltrepSexp, ensure_materialized};
use miniextendr_api::ffi::{self, SEXP, SEXPTYPE, SexpExt};
use miniextendr_api::miniextendr;

/// Check if a SEXP is ALTREP and return info about it.
///
/// Takes raw SEXP (extern "C-unwind") to bypass the ALTREP rejection.
/// Returns a character vector: c(is_altrep, sexptype, length).
///
/// @param x A SEXP to inspect.
/// @export
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_altrep_sexp_check(x: SEXP) -> SEXP {
    let is_altrep = unsafe { ffi::ALTREP(x) } != 0;
    let sexptype = x.type_of();
    let len = (x.xlength()) as usize;

    vec![
        format!("is_altrep={}", is_altrep),
        format!("sexptype={:?}", sexptype),
        format!("length={}", len),
    ]
    .into_sexp()
}

/// Try to wrap a SEXP in AltrepSexp and materialize it as integers.
///
/// Uses AltrepSexp parameter -- only accepts ALTREP vectors.
///
/// @param x An ALTREP integer vector.
/// @export
#[miniextendr]
pub fn altrep_sexp_materialize_int(x: AltrepSexp) -> Vec<i32> {
    assert_eq!(x.sexptype(), SEXPTYPE::INTSXP, "expected INTSXP");
    let slice = unsafe { x.materialize_integer() };
    slice.to_vec()
}

/// Try to wrap a SEXP in AltrepSexp and materialize it as doubles.
///
/// Uses AltrepSexp parameter -- only accepts ALTREP vectors.
///
/// @param x An ALTREP real vector.
/// @export
#[miniextendr]
pub fn altrep_sexp_materialize_real(x: AltrepSexp) -> Vec<f64> {
    assert_eq!(x.sexptype(), SEXPTYPE::REALSXP, "expected REALSXP");
    let slice = unsafe { x.materialize_real() };
    slice.to_vec()
}

/// Try to wrap a SEXP in AltrepSexp and materialize strings.
///
/// Uses AltrepSexp parameter -- only accepts ALTREP vectors.
///
/// @param x An ALTREP string vector.
/// @export
#[miniextendr]
pub fn altrep_sexp_materialize_strings(x: AltrepSexp) -> Vec<Option<String>> {
    assert_eq!(x.sexptype(), SEXPTYPE::STRSXP, "expected STRSXP");
    unsafe { x.materialize_strings() }
}

/// Use ensure_materialized on any SEXP, then extract as integer vector.
///
/// Takes raw SEXP (extern "C-unwind") to bypass the ALTREP rejection,
/// then materializes and converts normally.
///
/// @param x A SEXP to materialize.
/// @export
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_altrep_ensure_materialized_int(x: SEXP) -> SEXP {
    let materialized = unsafe { ensure_materialized(x) };
    let result: Vec<i32> =
        miniextendr_api::from_r::TryFromSexp::try_from_sexp(materialized).unwrap();
    result.into_sexp()
}

/// Use ensure_materialized on any SEXP, then extract as string vector.
///
/// Takes raw SEXP (extern "C-unwind") to bypass the ALTREP rejection.
///
/// @param x A SEXP to materialize.
/// @export
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_altrep_ensure_materialized_str(x: SEXP) -> SEXP {
    let materialized = unsafe { ensure_materialized(x) };
    let result: Vec<Option<String>> =
        miniextendr_api::from_r::TryFromSexp::try_from_sexp(materialized).unwrap();
    result.into_sexp()
}

/// Return whether a SEXP is ALTREP.
///
/// Takes raw SEXP (extern "C-unwind") to bypass the ALTREP rejection.
///
/// @param x A SEXP to check.
/// @export
#[miniextendr]
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "C-unwind" fn C_altrep_sexp_is_altrep(x: SEXP) -> SEXP {
    let is_altrep = AltrepSexp::try_wrap(x).is_some();
    is_altrep.into_sexp()
}
