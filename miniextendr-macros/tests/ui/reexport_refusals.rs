//! Test: `reexport!` reports every problem of an invocation at once, each on
//! its own tokens (#1910).

use miniextendr_api::reexport;

reexport!(
    x::tidy,
    generics::_hidden,
    generics::"a b",
    generics::glance,
    broom::glance,
    generics::tidy::x,
    stats,
);

fn main() {}
