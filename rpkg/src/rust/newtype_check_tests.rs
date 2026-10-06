//! Fixtures for `#[try_from_sexp(validate = ...)]` (#1815): a newtype's check
//! runs on the R value in every shape (scalar, `Option`, `Vec`,
//! `Vec<Option>`, an `Either` arm), and its refusal is the argument error:
//! the check's classes and fields, `e$param`, `e$rust_type` and the wrapper's
//! call. `newtype_check_raised` is the other way, a conversion that raises
//! with `rust_error!`.

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::SexpError;
use miniextendr_api::{SEXP, SexpExt, TryFromSexp, miniextendr};

/// Refuse a number that carries a unit class, whose number drops the unit
/// (`difftime`) or counts from 1970 (`Date`, `POSIXct` / `POSIXlt`).
fn plain_number(x: SEXP) -> Result<(), RError> {
    for class in [c"difftime", c"Date", c"POSIXt"] {
        if x.inherits_class(class) {
            let class = class.to_str().expect("an ASCII class name");
            return Err(RError::new(format!(
                "got a {class}; give a plain number in the data's time unit"
            ))
            .class(["mx_unit_refused", "mx_newtype_check"])
            .data("unit_class", class));
        }
    }
    Ok(())
}

/// An elapsed time, as a plain number.
#[derive(TryFromSexp)]
#[try_from_sexp(validate = plain_number)]
pub struct Elapsed(f64);

/// A validated newtype as a scalar.
/// @param x A plain number.
#[miniextendr(noexport)]
pub fn newtype_check_scalar(x: Elapsed) -> f64 {
    x.0
}

/// A validated newtype under `Option`: `NULL` is not checked.
/// @param x `NULL`, or a plain number.
#[miniextendr(noexport)]
pub fn newtype_check_option(#[miniextendr(default = "NULL")] x: Option<Elapsed>) -> String {
    match x {
        None => "NULL".to_string(),
        Some(x) => x.0.to_string(),
    }
}

/// A vector of a validated newtype: the check sees the whole vector.
/// @param x Plain numbers.
#[miniextendr(noexport)]
pub fn newtype_check_vec(x: Vec<Elapsed>) -> f64 {
    x.iter().map(|x| x.0).sum()
}

/// A vector of an optional validated newtype: the check sees the whole
/// vector, and `NA` is `None`.
/// @param x Plain numbers, `NA` allowed.
#[miniextendr(noexport)]
pub fn newtype_check_vec_option(x: Vec<Option<Elapsed>>) -> i32 {
    i32::try_from(x.iter().filter(|x| x.is_none()).count()).expect("a count that fits R")
}

/// A duration read like `as.numeric()`, as a plain number.
#[cfg(feature = "either")]
#[derive(TryFromSexp)]
#[try_from_sexp(validate = plain_number)]
pub struct Tau(miniextendr_api::AsNumeric);

/// A validated newtype as an `Either` arm, which has no R guard: the check's
/// refusal is reported when the other arm refused the kind of value.
/// @param tau `NULL`, a plain number, or a data frame.
#[cfg(feature = "either")]
#[miniextendr(noexport)]
pub fn newtype_check_either(
    tau: miniextendr_api::either_impl::Either<Option<Tau>, miniextendr_api::DataFrame>,
) -> String {
    use miniextendr_api::either_impl::Either;
    match tau {
        Either::Left(None) => "NULL".to_string(),
        Either::Left(Some(Tau(n))) => format!("{:?}", n.0),
        Either::Right(_) => "data frame".to_string(),
    }
}

/// A number whose hand-written conversion raises its refusal with
/// `rust_error!` instead of returning it.
pub struct RaisedElapsed(f64);

impl TryFromSexp for RaisedElapsed {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, SexpError> {
        if sexp.inherits_class(c"difftime") {
            miniextendr_api::rust_error!(class = "mx_unit_refused", "got a difftime");
        }
        f64::try_from_sexp(sexp).map(RaisedElapsed)
    }
}

/// A conversion that raises from inside `try_from_sexp`.
/// @param x A plain number.
#[miniextendr(noexport)]
pub fn newtype_check_raised(x: RaisedElapsed) -> f64 {
    x.0
}
