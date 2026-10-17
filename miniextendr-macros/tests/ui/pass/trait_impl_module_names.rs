//! Module-local traits may share a name when their implementing types differ.
#![allow(dead_code)]
#![deny(nonstandard_style)]
use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr)]
pub struct First;
#[derive(ExternalPtr)]
pub struct Second;

mod first {
    use crate::First;
    use miniextendr_api::miniextendr;
    #[miniextendr]
    pub trait Counter { fn value(&self) -> i32; }
    #[miniextendr(env)]
    impl Counter for First { fn value(&self) -> i32 { 1 } }
}

mod second {
    use crate::Second;
    use miniextendr_api::miniextendr;
    #[miniextendr]
    pub trait Counter { fn value(&self) -> i32 { 2 } }
    impl<T> Counter for T {}
    #[miniextendr(env)]
    impl Counter for Second {}
}

fn main() {}
