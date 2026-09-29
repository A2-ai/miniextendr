//! Compile-pass test: a `&Dots` parameter is R's `...` at any position. The
//! formals after it are matched by name only in R; `typed_list!` sugar reads
//! an explicit `&Dots` the same way it reads Rust `...`.

#![allow(dead_code)]

use miniextendr_api::dots::Dots;
use miniextendr_api::{miniextendr, typed_list};

/// `f(x, ..., overwrite = FALSE)`.
#[miniextendr]
pub fn dots_mid(x: i32, rest: &Dots, #[miniextendr(default = "FALSE")] overwrite: bool) -> i32 {
    x + i32::try_from(rest.len()).unwrap() + i32::from(overwrite)
}

/// `f(..., x)`.
#[miniextendr(noexport)]
pub fn dots_first(rest: &Dots, x: i32) -> i32 {
    x + i32::try_from(rest.len()).unwrap()
}

/// `typed_list!` on an explicit trailing `&Dots`.
#[miniextendr(noexport, dots = typed_list!(a => numeric()))]
pub fn dots_trailing_typed(x: i32, rest: &Dots) -> f64 {
    let _ = rest;
    let a: f64 = dots_typed.get("a").expect("a");
    f64::from(x) + a
}

/// `typed_list!` on a middle `&Dots`.
#[miniextendr(noexport, dots = typed_list!(a => numeric()))]
pub fn dots_mid_typed(x: i32, rest: &Dots, flag: bool) -> f64 {
    let _ = rest;
    let a: f64 = dots_typed.get("a").expect("a");
    f64::from(x) + a + f64::from(u8::from(flag))
}

#[derive(miniextendr_api::ExternalPtr)]
pub struct MidDotsCollector {
    base: i32,
}

#[miniextendr(env)]
impl MidDotsCollector {
    pub fn new(base: i32) -> Self {
        Self { base }
    }

    /// `collect(n, ..., flag = FALSE)`.
    #[miniextendr(defaults(flag = "FALSE"))]
    pub fn collect(&self, n: i32, rest: &Dots, flag: bool) -> i32 {
        self.base + n + i32::try_from(rest.len()).unwrap() + i32::from(flag)
    }
}

fn main() {}
