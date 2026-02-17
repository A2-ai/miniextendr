//! Test: path-style use is not supported in miniextendr_module.

use miniextendr_macros::miniextendr_module;

miniextendr_module! {
    mod test_pkg;
    use foo::bar;
}

fn main() {}
