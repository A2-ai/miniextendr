//! #1427: a borrowed `StrVec` / `RCow` parameter is bound to a root that lives
//! only as long as the generated call, so a `'static` view parameter cannot
//! compile; the macro says so at the lifetime. Before, the views' raw
//! `TryFromSexp` impl let the wrapper pick `'static`, and the body could keep
//! the view after R had collected it. `rooted_view_outlives_static.rs` covers
//! the borrow-checker tie behind this message.

use miniextendr_api::{RCow, StrVec, miniextendr};

static mut KEPT_STRINGS: Option<StrVec<'static>> = None;
static mut KEPT_VALUES: Option<RCow<'static, i32>> = None;

#[miniextendr]
pub fn keep_strings(strings: StrVec<'static>) {
    unsafe { KEPT_STRINGS = Some(strings) };
}

#[miniextendr]
pub fn keep_values(values: RCow<'static, i32>) {
    unsafe { KEPT_VALUES = Some(values) };
}

fn main() {}
