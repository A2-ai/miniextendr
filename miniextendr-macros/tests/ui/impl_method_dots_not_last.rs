//! Test: a method whose Rust `...` is not the last parameter gets the same
//! `&Dots` error as a function (#1737).

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Collector;

#[miniextendr(env)]
impl Collector {
    pub fn collect(&self, rest: ..., n: i32) -> i32 {
        n
    }
}

fn main() {}
