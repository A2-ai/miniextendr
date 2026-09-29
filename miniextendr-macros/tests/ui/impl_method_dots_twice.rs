//! Test: a method takes at most one `...` too.

use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Collector;

#[miniextendr(env)]
impl Collector {
    pub fn collect(
        &self,
        a: &miniextendr_api::dots::Dots,
        n: i32,
        b: &miniextendr_api::dots::Dots,
    ) -> i32 {
        let _ = (a, b);
        n
    }
}

fn main() {}
