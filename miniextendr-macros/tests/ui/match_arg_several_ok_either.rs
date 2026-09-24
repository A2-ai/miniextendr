//! Test: `several_ok` does not take an `Either<..>`. The choice-or-other split
//! (character or factor input is the choice, anything else goes to the `R`
//! arm) applies to a scalar choice, `Either<T, R>`.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_several_ok_either(#[miniextendr(match_arg, several_ok)] routes: Either<Vec<Route>, List>) {}

fn main() {}
