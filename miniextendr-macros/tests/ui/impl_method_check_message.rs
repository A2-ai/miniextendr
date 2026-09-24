//! Test: rejected method-level `inherits(p(...))` / `no_na(p(...))` forms:
//! a class check without `class = "..."`, and an unknown key.

use miniextendr_macros::miniextendr;

struct Holder;

#[miniextendr]
impl Holder {
    #[miniextendr(inherits(x(message = "`x` must be a pkg_obj")))]
    fn check(&self, x: Vec<f64>) -> i32 {
        0
    }
}

struct Other;

#[miniextendr]
impl Other {
    #[miniextendr(no_na(y(msg = "m")))]
    fn check(&self, y: f64) -> i32 {
        0
    }
}

fn main() {}
