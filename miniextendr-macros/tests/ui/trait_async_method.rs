//! Test: async trait method should fail.

use miniextendr_macros::miniextendr;

#[miniextendr]
pub trait BadAsync {
    async fn foo(&self);
}

fn main() {}
