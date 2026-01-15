//! Test: impl method with self by value should fail (needs &self or &mut self).

use miniextendr_macros::miniextendr;

struct Counter(i32);

#[miniextendr]
impl Counter {
    fn consume(self) -> i32 {
        self.0
    }
}

fn main() {}
