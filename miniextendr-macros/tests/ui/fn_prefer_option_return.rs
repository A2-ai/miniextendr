//! Test: `#[miniextendr(prefer = "...")]` on an `Option<T>`-returning function (BUG4).
//!
//! Previously this was silently accepted and dropped: auto-detection produces
//! `OptionIntoR` for `Option<T>` returns, which passed through `apply_return_pref`
//! unchanged, so `prefer = "list"` had no effect and no diagnostic. Now it's a hard
//! compile error — `prefer=` only has a plain `T` to wrap when the return type is a
//! bare `T: IntoR`.

use miniextendr_macros::miniextendr;

#[miniextendr(prefer = "list")]
fn bad_prefer_option(x: Option<i32>) -> Option<i32> {
    x
}

fn main() {}
