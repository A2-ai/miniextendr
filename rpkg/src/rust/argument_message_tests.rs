//! A type that words its whole argument error once, for every parameter of
//! that type (#1833), driven by `tests/testthat/test-argument-message.R`.
//!
//! `ModelArg` checks its class in `#[try_from_sexp(validate = ...)]`, and the
//! refusal's `RError::argument_message` is the condition's whole message.
//! `argument_message_guard` writes the same check as an
//! `inherits(..., when(...))` guard on one parameter. The two raise the same
//! message, classes, `kind`, `e$param` and call; the conversion adds
//! `e$rust_type`, as for any conversion error. `HandModel` is the same check
//! in a hand-written `TryFromSexp`, `DerivedModel` the same with a
//! `#[derive(RConditionError)]` error (`#[condition(argument_message = ...)]`),
//! and `Spacing` a scalar-backed type whose check also runs once on a whole
//! `Vec`.

use miniextendr_api::condition::{RConditionError, RError};
use miniextendr_api::{List, SEXP, SexpExt, TryFromSexp, miniextendr};

/// The model check: an `mx_model` object, and advice for a data frame.
fn model_object(x: SEXP) -> Result<(), RError> {
    if x.inherits_class(c"mx_model") {
        return Ok(());
    }
    if x.inherits_class(c"data.frame") {
        return Err(RError::new("got a data frame")
            .argument_message("use model_from_df() for a data frame"));
    }
    Err(RError::new("got no model object").argument_message("expected a model object"))
}

// region: the check declared on the type

/// A model object, checked once on the type.
#[derive(TryFromSexp)]
#[try_from_sexp(validate = model_object)]
pub struct ModelArg(List);

/// A model as a plain parameter.
/// @param fit A model object.
#[miniextendr(noexport)]
pub fn argument_message_model(fit: ModelArg) -> i32 {
    i32::try_from(fit.0.len()).expect("list length fits i32")
}

/// A model under `Option`: `NULL` is not checked.
/// @param fit `NULL`, or a model object.
#[miniextendr(noexport)]
pub fn argument_message_model_option(
    #[miniextendr(default = "NULL")] fit: Option<ModelArg>,
) -> bool {
    fit.is_some()
}

/// A second function taking the type: the same check, declared nowhere here.
/// @param fit A model object.
/// @param extra A number.
#[miniextendr(noexport)]
pub fn argument_message_model_other(extra: f64, fit: ModelArg) -> f64 {
    extra + f64::from(i32::try_from(fit.0.len()).expect("list length fits i32"))
}

// endregion

// region: the same check as an R guard on one parameter

/// The model check written as `inherits(..., when(...))` on the parameter.
/// @param fit A model object.
#[miniextendr(noexport)]
pub fn argument_message_guard(
    #[miniextendr(inherits(
        class = "mx_model",
        message = "expected a model object",
        when(class = "data.frame", message = "use model_from_df() for a data frame")
    ))]
    fit: List,
) -> i32 {
    i32::try_from(fit.len()).expect("list length fits i32")
}

// endregion

// region: a hand-written conversion

/// The model check in a hand-written `TryFromSexp` whose error is the
/// `RError` itself.
pub struct HandModel(List);

impl TryFromSexp for HandModel {
    type Error = RError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, RError> {
        model_object(sexp)?;
        List::try_from_sexp(sexp)
            .map(HandModel)
            .map_err(|e| RError::new(e.to_string()))
    }
}

/// A model through a hand-written conversion.
/// @param fit A model object.
#[miniextendr(noexport)]
pub fn argument_message_hand(fit: HandModel) -> i32 {
    i32::try_from(fit.0.len()).expect("list length fits i32")
}

/// The refusals of `DerivedModel`: `argument_message` on one variant of a
/// derived `RConditionError`, none on the other.
#[derive(RConditionError)]
#[condition(class = "mx_model_error")]
pub enum ModelRefusal {
    #[condition(
        message = "got a data frame",
        argument_message = "use model_from_df() for a data frame"
    )]
    DataFrame,
    #[condition(message = "got no model object")]
    NotAModel,
}

/// The model check in a hand-written `TryFromSexp` whose error is derived.
pub struct DerivedModel(List);

impl TryFromSexp for DerivedModel {
    type Error = ModelRefusal;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, ModelRefusal> {
        if sexp.inherits_class(c"data.frame") {
            return Err(ModelRefusal::DataFrame);
        }
        if !sexp.inherits_class(c"mx_model") {
            return Err(ModelRefusal::NotAModel);
        }
        List::try_from_sexp(sexp)
            .map(DerivedModel)
            .map_err(|_| ModelRefusal::NotAModel)
    }
}

/// A model through a conversion whose error is a derived `RConditionError`.
/// @param fit A model object.
#[miniextendr(noexport)]
pub fn argument_message_derived(fit: DerivedModel) -> i32 {
    i32::try_from(fit.0.len()).expect("list length fits i32")
}

// endregion

// region: a scalar-backed type in every container

/// Refuse a number that carries a unit class, with advice.
fn plain_spacing(x: SEXP) -> Result<(), RError> {
    if x.inherits_class(c"difftime") {
        return Err(RError::new("got a difftime")
            .class("mx_spacing_refused")
            .argument_message("give the spacing as a plain number in hours"));
    }
    Ok(())
}

/// A spacing in hours, checked once on the type.
#[derive(TryFromSexp)]
#[try_from_sexp(validate = plain_spacing)]
pub struct Spacing(f64);

/// A spacing as a scalar.
/// @param x A plain number.
#[miniextendr(noexport)]
pub fn argument_message_spacing(x: Spacing) -> f64 {
    x.0
}

/// Spacings as a vector: the check sees the whole vector.
/// @param x Plain numbers.
#[miniextendr(noexport)]
pub fn argument_message_spacing_vec(x: Vec<Spacing>) -> f64 {
    x.iter().map(|s| s.0).sum()
}

/// Optional spacings as a vector, `NA` allowed.
/// @param x Plain numbers.
#[miniextendr(noexport)]
pub fn argument_message_spacing_vec_option(x: Vec<Option<Spacing>>) -> i32 {
    i32::try_from(x.iter().filter(|s| s.is_none()).count()).expect("a count that fits R")
}

// endregion
