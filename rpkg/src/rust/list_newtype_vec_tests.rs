//! A list of checked objects as an argument (#1837), driven by
//! `tests/testthat/test-list-newtype-vec.R`.
//!
//! `Model` is a `#[derive(TryFromSexp)]` newtype over `List` with a check.
//! `Vec<Model>` and `Vec<Option<Model>>` run that check on each element of
//! the list, where each object keeps its class, and report every failing
//! element in one argument error that names its position. `Vec<List>` reads a
//! list of lists the same way, with no check.

use miniextendr_api::condition::RError;
use miniextendr_api::prelude::{OwnedProtect, SEXP, SexpExt};
use miniextendr_api::{IntoR, List, TryFromSexp, miniextendr};

/// The model check: an `mx_model` object, with advice for a data frame.
fn model_object(x: SEXP) -> Result<(), RError> {
    if x.inherits_class(c"mx_model") {
        return Ok(());
    }
    if x.inherits_class(c"data.frame") {
        return Err(RError::new("got a data frame")
            .class("mx_model_refused")
            .data("advice", "model_from_df()")
            .argument_message("use model_from_df() for a data frame"));
    }
    Err(RError::new("got no model object").class("mx_model_refused"))
}

/// A model object: a classed list, checked once on the type.
#[derive(TryFromSexp, IntoR)]
#[try_from_sexp(validate = model_object)]
pub struct Model(List);

/// The length of one model.
/// @param fit A model object.
#[miniextendr(noexport)]
pub fn list_newtype_one(fit: Model) -> i32 {
    i32::try_from(fit.0.len()).expect("list length fits i32")
}

/// How many models a list holds; each element is checked as a model.
/// @param fits A list of model objects.
#[miniextendr(noexport)]
pub fn list_newtype_count(fits: Vec<Model>) -> i32 {
    i32::try_from(fits.len()).expect("a count that fits R")
}

/// Which elements of a list of models and `NULL`s hold a model.
/// @param fits A list of model objects and `NULL`s.
#[miniextendr(noexport)]
pub fn list_newtype_present(fits: Vec<Option<Model>>) -> Vec<bool> {
    fits.iter().map(Option::is_some).collect()
}

/// A list of models, back to R as a list.
/// @param fits A list of model objects.
#[miniextendr(noexport)]
pub fn list_newtype_round_trip(fits: Vec<Model>) -> Vec<Model> {
    fits
}

/// A list of models and `NULL`s, back to R as a list.
/// @param fits A list of model objects and `NULL`s.
#[miniextendr(noexport)]
pub fn list_newtype_round_trip_option(fits: Vec<Option<Model>>) -> Vec<Option<Model>> {
    fits
}

/// The length of each list in a list of lists.
/// @param x A list of lists.
#[miniextendr(noexport)]
pub fn list_newtype_lengths(x: Vec<List>) -> Vec<i32> {
    x.iter()
        .map(|list| i32::try_from(list.len()).expect("list length fits i32"))
        .collect()
}

/// Read lists of models as `Vec<Model>` and `Vec<Option<Model>>` under GC
/// pressure, allocate while holding them, read every model back, and return
/// the models as a list.
///
/// The models live in a `Vec` that only the input list roots: each element is
/// stored in the protected list before anything else allocates, and the
/// conversion keeps the element `SEXP`s, not copies. Under `gctorture(TRUE)` a
/// model the list did not root would be collected and the readback would see
/// a reused cell.
///
/// No arguments: picked up by the `gctorture(TRUE)` no-arg sweep (#430).
#[miniextendr(noexport)]
pub fn gc_stress_list_newtype_vec() -> SEXP {
    use miniextendr_api::SEXPTYPE::VECSXP;
    use miniextendr_api::sys::Rf_allocVector;

    let n: usize = 24;
    let len = isize::try_from(n).expect("a small length");
    let models = unsafe { OwnedProtect::new(Rf_allocVector(VECSXP, len)) };
    let with_nulls = unsafe { OwnedProtect::new(Rf_allocVector(VECSXP, len)) };
    for i in 0..n {
        let index = isize::try_from(i).expect("a small index");
        let value = i32::try_from(i).expect("a small value");
        let model = List::from_values(vec![value]).set_class_str(&["mx_model"]);
        // No allocation between building the model and rooting it.
        models.get().set_vector_elt(index, model.as_sexp());
        if i % 3 != 0 {
            with_nulls.get().set_vector_elt(index, model.as_sexp());
        }
    }

    let read: Vec<Model> = TryFromSexp::try_from_sexp(models.get()).expect("a list of models");
    let maybe: Vec<Option<Model>> =
        TryFromSexp::try_from_sexp(with_nulls.get()).expect("a list of models and NULLs");

    // Churn the GC while holding both vectors.
    for i in 0..n {
        let _throwaway = List::from_values(vec![i32::try_from(i).expect("a small value")]);
    }

    for (i, model) in read.iter().enumerate() {
        let expected = i32::try_from(i).expect("a small value");
        assert_eq!(model.0.get_index::<i32>(0), Some(expected), "model {i}");
        assert!(model.0.as_sexp().inherits_class(c"mx_model"), "model {i}");
    }
    for (i, model) in maybe.iter().enumerate() {
        assert_eq!(model.is_some(), i % 3 != 0, "element {i}");
    }

    // The return direction, while the input list still roots the models.
    read.into_sexp()
}
