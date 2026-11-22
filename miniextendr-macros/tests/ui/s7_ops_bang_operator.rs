//! Test: an S7 method on `!` is a compile error. S7's `!` generic is unary,
//! and miniextendr generates methods for the binary operators only.

use miniextendr_macros::miniextendr;

struct Flag {
    on: bool,
}

#[miniextendr(s7)]
impl Flag {
    #[miniextendr(r_name = "!")]
    fn not(&self) -> bool {
        !self.on
    }
}

fn main() {}
