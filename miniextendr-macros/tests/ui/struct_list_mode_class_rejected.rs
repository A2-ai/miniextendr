//! Test: `class` in `#[miniextendr(list)]` points at `#[into_list(class = ...)]`.

use miniextendr_macros::miniextendr;

#[miniextendr(list, class = "fit")]
struct Fit {
    x: f64,
}

fn main() {}
