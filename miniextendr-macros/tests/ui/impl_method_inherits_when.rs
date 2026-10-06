//! Test: rejected method-level `when(...)` hints of `inherits(p(...))`
//! (#1824): a hint without a message, and a hint class that `inherits`
//! requires.

use miniextendr_macros::miniextendr;

struct Holder;

#[miniextendr]
impl Holder {
    #[miniextendr(inherits(model(class = "pkg_model", when(class = "data.frame"))))]
    fn fit(&self, model: Vec<f64>) -> i32 {
        0
    }
}

struct Other;

#[miniextendr]
impl Other {
    #[miniextendr(inherits(model(
        class = "pkg_model, pkg_fit",
        when(class = "data.frame, pkg_fit", message = "a data frame")
    )))]
    fn fit(&self, model: Vec<f64>) -> i32 {
        0
    }
}

fn main() {}
