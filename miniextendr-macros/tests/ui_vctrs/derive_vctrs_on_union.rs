//! Test: Vctrs can only be applied to structs; on a union the error points
//! at the `union` keyword.

use miniextendr_macros::Vctrs;

#[derive(Vctrs)]
#[vctrs(class = "bad")]
union Bad {
    x: f64,
    y: i32,
}

fn main() {}
