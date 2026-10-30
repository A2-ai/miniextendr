//! Test: invalid `#[into_list(class = ...)]` uses.

use miniextendr_macros::IntoList;

#[derive(IntoList)]
#[into_list(ignore)]
struct IgnoreOnStruct {
    x: f64,
}

#[derive(IntoList)]
struct ClassOnField {
    #[into_list(class = "a")]
    x: f64,
}

#[derive(IntoList)]
#[into_list(class = [])]
struct NoClasses {
    x: f64,
}

#[derive(IntoList)]
#[into_list(class = ["a", ""])]
struct EmptyClass {
    x: f64,
}

#[derive(IntoList)]
#[into_list(class = ["a", "factor"])]
struct FactorClass {
    x: f64,
}

#[derive(IntoList)]
#[into_list(class = "a", class = "b")]
struct TwoClassOptions {
    x: f64,
}

fn main() {}
