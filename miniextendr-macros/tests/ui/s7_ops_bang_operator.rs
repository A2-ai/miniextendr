//! Test: an S7 method on `!` is a compile error. S7 dispatches the binary Ops
//! operators only, so no signature can register a method for `!`.

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
