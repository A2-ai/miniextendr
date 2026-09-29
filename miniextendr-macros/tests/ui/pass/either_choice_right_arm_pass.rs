//! Compile-pass test: the `R` arm of an `Either<T, R>` choice parameter may be
//! anything that reads more than character or factor input, and the
//! `TryFromSexp::CHARACTER_ONLY` metadata the macro checks is set, forwarded or
//! left `false` as documented.
//!
//! `AsCharacter` / `AsCharacterVec` read numbers too, so they stay allowed; a
//! hand-written string conversion that leaves the default is not checked; a
//! plain `Either` parameter (no `match_arg` / `choices`) is never checked.

#![allow(dead_code)]

use miniextendr_api::either_impl::Either;
use miniextendr_api::from_r::SexpError;
use miniextendr_api::{
    AsCharacter, AsCharacterVec, DataFrame, ExternalPtr, List, MatchArg, RFactor, SEXP,
    TryFromSexp, miniextendr,
};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Mode {
    Fast,
    Safe,
}

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Unit {
    Mg,
    Ug,
}

#[derive(Copy, Clone, Debug, RFactor)]
pub enum Level {
    Low,
    High,
}

/// A newtype of a string: forwards `CHARACTER_ONLY`.
#[derive(TryFromSexp)]
pub struct Tag(String);

/// A newtype of a number: forwards `false`.
#[derive(TryFromSexp)]
pub struct Scaled(f64);

/// Reads only strings, but leaves the default: the check is skipped.
pub struct Loose(String);

impl TryFromSexp for Loose {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        String::try_from_sexp(sexp).map(Loose)
    }
}

// The derives set or forward the const.
const _: () = assert!(<Unit as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Level as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Tag as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Vec<Tag> as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(<Option<Tag> as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(!<Scaled as TryFromSexp>::CHARACTER_ONLY);
const _: () = assert!(!<Loose as TryFromSexp>::CHARACTER_ONLY);

#[miniextendr]
pub fn mode_or_text(#[miniextendr(match_arg)] mode: Either<Mode, AsCharacter>) -> bool {
    mode.is_left()
}

#[miniextendr]
pub fn mode_or_frame(#[miniextendr(match_arg)] mode: Either<Mode, DataFrame>) -> bool {
    mode.is_left()
}

#[miniextendr]
pub fn mode_or_list(#[miniextendr(match_arg)] mode: Either<Mode, List>) -> bool {
    mode.is_left()
}

#[miniextendr]
pub fn modes_or_number(
    #[miniextendr(match_arg, several_ok)] modes: Either<Vec<Mode>, f64>,
) -> bool {
    modes.is_left()
}

#[miniextendr]
pub fn mode_or_values(#[miniextendr(match_arg)] mode: Option<Either<Mode, &[f64]>>) -> bool {
    mode.is_some()
}

#[miniextendr]
pub fn level_or_text(
    #[miniextendr(choices("low", "high"))] level: Either<String, AsCharacterVec>,
) -> bool {
    level.is_left()
}

#[miniextendr]
pub fn mode_or_loose(#[miniextendr(match_arg)] mode: Either<Mode, Loose>) -> bool {
    mode.is_left()
}

/// Not a choice parameter: the left-first `TryFromSexp for Either` decodes it.
#[miniextendr]
pub fn int_or_text(x: Either<i32, String>) -> bool {
    x.is_left()
}

#[derive(ExternalPtr)]
pub struct Picker;

#[miniextendr(env)]
impl Picker {
    pub fn new() -> Self {
        Picker
    }

    #[miniextendr(match_arg(mode))]
    pub fn pick(&self, mode: Either<Mode, Vec<f64>>) -> bool {
        mode.is_left()
    }
}

fn main() {}
