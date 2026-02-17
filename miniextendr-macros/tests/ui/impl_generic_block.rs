//! Test: generic impl blocks are not supported.

use miniextendr_macros::miniextendr;

struct Wrapper<T>(T);

#[miniextendr]
impl<T: Clone> Wrapper<T> {
    fn get(&self) -> String {
        String::new()
    }
}

fn main() {}
