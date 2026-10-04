//! Compile-pass test: `_: ...` is unnamed dots (#1743). rustc rejects a bare
//! `...` by default (`varargs_without_pattern`) and suggests `_: ...`, which
//! binds the synthetic `__miniextendr_dots`; the R formal is plain `...`.

#![allow(dead_code)]

use miniextendr_api::{miniextendr, typed_list};

/// `f(x, ...)`, ignoring the dots.
#[miniextendr]
pub fn wild_dots(x: i32, _: ...) -> i32 {
    x
}

/// `f(...)`, ignoring the dots.
#[miniextendr(noexport)]
pub fn wild_dots_only(_: ...) {}

/// `typed_list!` sugar reads `_: ...` through its synthetic binding.
#[miniextendr(noexport, dots = typed_list!(a => numeric()))]
pub fn wild_dots_typed(_: ...) -> f64 {
    dots_typed.get("a").expect("a")
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct WildDotsCollector {
    base: i32,
}

#[miniextendr(env)]
impl WildDotsCollector {
    pub fn new(base: i32, _: ...) -> Self {
        Self { base }
    }

    /// `collect(n, ...)`, ignoring the dots.
    pub fn collect(&self, n: i32, _: ...) -> i32 {
        self.base + n
    }

    /// `typed_list!` sugar on a method's `_: ...`.
    #[miniextendr(dots = typed_list!(bump => numeric()))]
    pub fn bumped(&self, _: ...) -> f64 {
        let bump: f64 = dots_typed.get("bump").expect("bump");
        f64::from(self.base) + bump
    }
}

fn main() {}
