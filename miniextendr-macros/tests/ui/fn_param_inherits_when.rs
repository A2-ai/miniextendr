//! Test: rejected `when(...)` hints of the parameter-level `inherits` check
//! (#1824): a hint without a message, a hint class that `inherits` requires
//! (also through another attribute), a hint without the check's classes, and
//! a hint in `not_inherits`.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub fn hint_without_message(
    #[miniextendr(inherits(class = "pkg_model", when(class = "data.frame")))] model: Vec<f64>,
) {
}

#[miniextendr]
pub fn hint_class_required(
    #[miniextendr(inherits(
        class = "pkg_model",
        when(class = "data.frame", class = "pkg_model", message = "a data frame")
    ))]
    model: Vec<f64>,
) {
}

#[miniextendr]
pub fn hint_class_required_elsewhere(
    #[miniextendr(inherits(class = "pkg_model", when(class = "pkg_fit", message = "a fit")))]
    #[miniextendr(inherits = "pkg_fit")]
    model: Vec<f64>,
) {
}

#[miniextendr]
pub fn hint_without_class_to_check(
    #[miniextendr(inherits(when(class = "data.frame", message = "a data frame")))] model: Vec<f64>,
) {
}

#[miniextendr]
pub fn hint_on_not_inherits(
    #[miniextendr(not_inherits(class = "Date", when(class = "POSIXt", message = "m")))] x: Vec<f64>,
) {
}

fn main() {}
