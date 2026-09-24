//! Compile-pass test for choice parameters that report omission (#1551):
//! `Missing<T>` and `Missing<Option<T>>` under `match_arg` / `choices`, and a
//! `Missing<Vec<T>>` / `Missing<Box<[T]>>` `several_ok` list, on standalone
//! functions and on an impl method. The R formal keeps the choice vector; the
//! generated C wrapper decodes an omitted argument as `Missing::Absent`.

#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, MatchArg, Missing, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
pub enum Mode {
    Fast,
    Safe,
}

#[miniextendr]
pub fn omitted_plain(#[miniextendr(match_arg)] mode: Missing<Mode>) -> bool {
    mode.is_missing()
}

#[miniextendr]
pub fn omitted_optional(#[miniextendr(match_arg)] mode: Missing<Option<Mode>>) -> bool {
    matches!(mode, Missing::Present(None))
}

#[miniextendr]
pub fn omitted_several(#[miniextendr(match_arg, several_ok)] modes: Missing<Vec<Mode>>) -> bool {
    modes.is_present()
}

#[miniextendr]
pub fn omitted_several_boxed(
    #[miniextendr(match_arg, several_ok)] modes: Missing<Box<[Mode]>>,
) -> bool {
    modes.is_present()
}

#[miniextendr]
pub fn omitted_literal(
    #[miniextendr(choices("red", "green"))] color: Missing<Option<String>>,
) -> bool {
    color.is_present()
}

#[derive(ExternalPtr)]
pub struct Picker;

#[miniextendr(env)]
impl Picker {
    pub fn new() -> Self {
        Picker
    }

    #[miniextendr(match_arg(mode))]
    pub fn pick(&self, mode: Missing<Option<Mode>>) -> bool {
        mode.is_missing()
    }
}

fn main() {}
