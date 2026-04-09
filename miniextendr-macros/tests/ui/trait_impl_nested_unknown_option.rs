//! Test: unknown top-level key in #[miniextendr(...)] on a trait impl method.
//! The nested (env/r6/s3/s4/s7) unknown key is also validated but harder to test
//! in isolation because it requires a full trait ABI setup.

use miniextendr_macros::miniextendr;

struct Dummy;

trait MyTrait {
    fn value(&self) -> i32;
}

#[miniextendr(env)]
impl MyTrait for Dummy {
    #[miniextendr(typo_toplevel)]
    fn value(&self) -> i32 { 42 }
}

fn main() {}
