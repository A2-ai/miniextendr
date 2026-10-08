//! Compile-pass test for `#[match_arg(alias = "...")]` (#1843): repeatable
//! on a variant, next to `rename` / `rename_all`, and a prefix of its own
//! choice is allowed. The aliases land in `MatchArg::ALIASES`, and the type
//! works as a `match_arg` parameter on every layer.

#![allow(dead_code)]

use miniextendr_api::{MatchArg, Missing, miniextendr};

#[derive(Copy, Clone, Debug, MatchArg)]
#[match_arg(rename_all = "lower")]
pub enum Shade {
    Red,
    #[match_arg(alias = "grey", alias = "gr")]
    Gray,
    #[match_arg(rename = "navy", alias = "Blue")]
    Blue,
}

const _: () = assert!(Shade::ALIASES.len() == 3);

#[miniextendr]
pub fn alias_plain(#[miniextendr(match_arg)] color: Shade) -> String {
    format!("{color:?}")
}

#[miniextendr]
pub fn alias_layers(
    #[miniextendr(match_arg)] maybe: Option<Shade>,
    #[miniextendr(match_arg)] omitted: Missing<Shade>,
    #[miniextendr(match_arg, several_ok)] many: Vec<Shade>,
) -> usize {
    usize::from(maybe.is_some()) + usize::from(omitted.is_missing()) + many.len()
}

fn main() {}
