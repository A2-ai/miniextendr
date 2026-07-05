//! Test: `#[miniextendr(prefer = "...")]` combined with `unwrap_in_r` on a `Result<T, E>`
//! return (a second BUG4-shaped no-op site, caught while fixing BUG4).
//!
//! In `unwrap_in_r` mode the whole `Result<T, E>` is converted via a single `IntoR` impl
//! (a tagged list for R to decode) rather than the inner `T`, so `prefer=` — which always
//! targets the inner `T` — has nothing to wrap here. Previously this was silently ignored;
//! now it's a hard compile error.

use miniextendr_macros::miniextendr;

#[miniextendr(prefer = "list", unwrap_in_r)]
fn bad_prefer_unwrap_in_r(x: i32) -> Result<i32, String> {
    Ok(x)
}

fn main() {}
