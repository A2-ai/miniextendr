//! Test: a `several_ok` list with another kind of value is
//! `Either<Vec<T>, R>` or `Either<Box<[T]>, R>` (#1612). A fixed-size array
//! under `Either<..>` has no decoder and errors at compile time.

use miniextendr_macros::miniextendr;

#[miniextendr]
fn bad_several_ok_either(#[miniextendr(match_arg, several_ok)] routes: Either<[Route; 2], List>) {}

fn main() {}
