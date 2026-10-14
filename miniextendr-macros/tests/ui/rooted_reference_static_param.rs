//! #1427: `Cow` and shared-reference parameters are bound to the generated
//! call's argument slot, as `StrVec` / `RCow` are, so a `'static` one cannot
//! compile; the macro says so at the lifetime, also when it sits inside a
//! container. Before, their raw `TryFromSexp` impls handed out `'static`
//! borrows, and the body could keep R memory after the call.

use miniextendr_api::miniextendr;
use std::borrow::Cow;

static mut KEPT: Option<&'static [i32]> = None;

#[miniextendr]
pub fn keep_slice(values: &'static [i32]) {
    unsafe { KEPT = Some(values) };
}

#[miniextendr]
pub fn keep_text(text: Option<&'static str>) {
    let _ = text;
}

#[miniextendr]
pub fn keep_cow(text: Cow<'static, str>) {
    let _ = text;
}

#[miniextendr]
pub fn keep_words(words: Vec<Option<Cow<'static, str>>>) {
    let _ = words;
}

fn main() {}
