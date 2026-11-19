//! Test: `#[derive(Altrep)]` can only be applied to structs. On an enum with
//! doc comments, the error points at the `enum` keyword, not at the first
//! `///` line or at `pub`.

use miniextendr_macros::Altrep;

/// The faces of a die.
///
/// An enum, so no ALTREP class.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Altrep)]
pub enum Die {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

fn main() {}
