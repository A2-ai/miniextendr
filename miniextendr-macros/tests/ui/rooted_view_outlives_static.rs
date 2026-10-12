//! #1427: the borrow checker ties a borrowed view parameter to the generated
//! call's argument slot. A lifetime the signature bounds by `'static` names no
//! `'static` view, so only that tie refuses it: the slot does not live that
//! long. Before, the views' raw `TryFromSexp` impl accepted any lifetime and
//! this compiled, so the body could keep the view after R collected it.

use miniextendr_api::{RCow, StrVec, miniextendr};

static mut KEPT_STRINGS: Option<StrVec<'static>> = None;
static mut KEPT_VALUES: Option<RCow<'static, i32>> = None;

#[miniextendr]
pub fn keep_strings<'a: 'static>(strings: StrVec<'a>) {
    unsafe { KEPT_STRINGS = Some(strings) };
}

#[miniextendr]
pub fn keep_values<'a: 'static>(values: RCow<'a, i32>) {
    unsafe { KEPT_VALUES = Some(values) };
}

fn main() {}
