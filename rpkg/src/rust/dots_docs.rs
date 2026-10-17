//! Standalone and shared-page dots documentation regressions (#1819).

use miniextendr_api::dots::Dots;
use miniextendr_api::{LazyDots, miniextendr};

/// Return a value while accepting unnamed dots.
/// @param x Value returned unchanged.
#[miniextendr]
pub fn dots_docs_unnamed(x: i32, _: ...) -> i32 {
    x
}

/// Count named Rust dots without an explicit dots doc tag.
#[miniextendr]
pub fn dots_docs_named(sources: ...) -> i32 {
    i32::try_from(sources.len()).expect("fewer than 2^31 dots")
}

/// Count explicit dots when the following flag is enabled.
#[miniextendr]
pub fn dots_docs_explicit(rest: &Dots, enabled: bool) -> i32 {
    if enabled {
        i32::try_from(rest.len()).expect("fewer than 2^31 dots")
    } else {
        0
    }
}

/// Count lazy dots without evaluating them.
#[miniextendr]
pub fn dots_docs_lazy(rest: LazyDots) -> i32 {
    i32::try_from(rest.len()).expect("fewer than 2^31 dots")
}

/// Exercise inherited dots documentation alongside an undocumented regular argument.
/// @inheritDotParams own_page_round
#[miniextendr]
pub fn dots_docs_inherited(label: i32, _: ...) -> i32 {
    label
}

/// @title Shared dots documentation
/// @name dots_docs
/// @rdname dots_docs
/// @description Count arguments on a shared file-stem page.
/// @param ... Shared extra arguments, counted without inspecting their values.
#[miniextendr]
pub fn dots_docs_group_first(rest: &Dots) -> i32 {
    i32::try_from(rest.len()).expect("fewer than 2^31 dots")
}

/// Another counter on the shared page.
/// @rdname dots_docs
#[miniextendr]
pub fn dots_docs_group_second(rest: &Dots) -> i32 {
    i32::try_from(rest.len()).expect("fewer than 2^31 dots")
}
