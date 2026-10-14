//! #1427: the borrow checker ties `Cow` and shared-reference parameters to the
//! generated call's argument slot. A lifetime bounded by `'static` names no
//! `'static`, so only that tie refuses it. Before, the raw `TryFromSexp` impls
//! accepted any lifetime and this compiled, so the body could keep R memory
//! after the call.

use miniextendr_api::miniextendr;
use std::borrow::Cow;

static mut KEPT_SLICE: Option<&'static [i32]> = None;
static mut KEPT_TEXT: Option<Cow<'static, str>> = None;

#[miniextendr]
pub fn keep_slice<'a: 'static>(values: &'a [i32]) {
    unsafe { KEPT_SLICE = Some(values) };
}

#[miniextendr]
pub fn keep_text<'a: 'static>(text: Cow<'a, str>) {
    unsafe { KEPT_TEXT = Some(text) };
}

fn main() {}
