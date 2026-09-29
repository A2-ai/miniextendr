//! Test: the check reads the arm's `TryFromSexp`, not its name, so a type
//! alias of a character conversion is rejected too. `Option<String>` reads
//! `NULL` and character input only; the message names the alias and points to
//! `Option<Either<..>>` for a `NULL` alternative.

use miniextendr_api::either_impl::Either;
use miniextendr_api::miniextendr;

type Label = Option<String>;

#[miniextendr]
pub fn level_or_label(
    #[miniextendr(choices("low", "high"))] level: Either<String, Label>,
) -> bool {
    level.is_left()
}

fn main() {}
