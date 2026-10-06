//! Compile-pass test (#1815): `#[derive(TryFromSexp)]` with
//! `#[try_from_sexp(validate = path)]`. The check may return an `RError`
//! (a classed refusal) or a plain `SexpError`; the newtype's scalar error is
//! then `SexpError`, as is every container's, and each shape works as a
//! `#[miniextendr]` parameter.

#![allow(dead_code)]

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::SexpError;
use miniextendr_api::{SEXP, SexpExt, TryFromSexp, TryFromSexpElement, miniextendr};

mod checks {
    use miniextendr_api::from_r::SexpError;
    use miniextendr_api::{SEXP, SexpExt};

    /// A path with segments, refusing with a plain `SexpError`.
    pub fn not_a_date(x: SEXP) -> Result<(), SexpError> {
        if x.inherits_class(c"Date") {
            return Err(SexpError::InvalidValue("give a number, not a date".into()));
        }
        Ok(())
    }
}

/// A classed refusal.
fn plain_number(x: SEXP) -> Result<(), RError> {
    if x.inherits_class(c"difftime") {
        return Err(RError::new("give a plain number").class("pkg_unit_error"));
    }
    Ok(())
}

/// Tuple newtype over a type whose `Vec` error is `SexpTypeError`.
#[derive(TryFromSexp)]
#[try_from_sexp(validate = plain_number)]
pub struct Elapsed(f64);

/// Named-field newtype, a checking path with segments.
#[derive(TryFromSexp)]
#[try_from_sexp(validate = checks::not_a_date)]
pub struct Level {
    value: f64,
}

/// A newtype of a validated newtype: the outer check runs, and the inner
/// one through the inner type's conversions.
#[derive(TryFromSexp)]
pub struct Outer(Elapsed);

fn assert_sexp_error<T: TryFromSexp<Error = SexpError>>() {}
fn assert_element<T: TryFromSexpElement>() {}

fn _check() {
    assert_sexp_error::<Elapsed>();
    assert_sexp_error::<Option<Elapsed>>();
    assert_sexp_error::<Vec<Elapsed>>();
    assert_sexp_error::<Vec<Option<Elapsed>>>();
    assert_sexp_error::<Level>();
    assert_sexp_error::<Vec<Level>>();
    assert_sexp_error::<Outer>();
    assert_sexp_error::<Vec<Option<Outer>>>();
    assert_element::<Elapsed>();
    let _: fn(SEXP) -> Result<(), SexpError> = <Elapsed as TryFromSexpElement>::check_sexp;
}

#[miniextendr]
pub fn elapsed_scalar(x: Elapsed) -> f64 {
    x.0
}

#[miniextendr]
pub fn elapsed_option(#[miniextendr(default = "NULL")] x: Option<Elapsed>) -> f64 {
    x.map_or(0.0, |x| x.0)
}

#[miniextendr]
pub fn elapsed_vec(x: Vec<Elapsed>) -> f64 {
    x.iter().map(|x| x.0).sum()
}

#[miniextendr]
pub fn elapsed_vec_option(x: Vec<Option<Elapsed>>) -> i32 {
    x.iter().filter(|x| x.is_none()).count() as i32
}

#[miniextendr]
pub fn level_scalar(x: Level) -> f64 {
    x.value
}

fn main() {}
