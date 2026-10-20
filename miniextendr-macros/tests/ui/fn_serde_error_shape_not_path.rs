//! Test: `serde_error(shape = ..)` takes the path of a `SerdeErrorShape`
//! const, not a string (#2207).

use miniextendr_macros::miniextendr;

struct E;

#[miniextendr(serde_error(shape = "ENGINE"))]
fn bad_serde_error_shape_string() -> Result<(), E> {
    Ok(())
}

fn main() {}
