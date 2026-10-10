//! Test: a `LazyDots` parameter on a `#[miniextendr]` trait method.
//!
//! Trait-method arguments cross the vtable as converted values, as for
//! `&Dots`, which trait methods don't take either (#1892).

use miniextendr_api::{LazyDots, miniextendr};

#[miniextendr]
pub trait Reporter {
    fn report(&self, rest: LazyDots) -> i32;
}

fn main() {}
