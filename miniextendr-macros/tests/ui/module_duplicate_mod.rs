//! Test: miniextendr_module with duplicate mod declaration

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    mod first;
    mod second;
}

fn main() {}
