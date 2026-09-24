//! Test: the choice type is the left arm of `Either<T, R>`, under the other
//! layers. `Either<Option<T>, R>` puts `Option` below `Either`, where it would
//! never see `NULL` (the split sends `NULL` to `R`), so it errors at compile
//! time; `Option<Either<T, R>>` is the optional form.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_either_layer_order(#[miniextendr(match_arg)] route: Either<Option<Route>, List>) {}

fn main() {}
