//! Test: contradictory `serde_error` field options on one function (#2207).
//!
//! A field takes one `as_*` type and cannot be both skipped and typed. Every
//! conflict is reported at once, each on the name that repeats an earlier
//! option.

use miniextendr_macros::miniextendr;

struct E;

#[miniextendr(serde_error(
    as_integer("n"),
    as_double("n"),
    skip("message"),
    as_character("rule", "message")
))]
fn bad_serde_error_as_conflicts() -> Result<(), E> {
    Ok(())
}

fn main() {}
