//! One trait on two same-named types has the same exported vtable name.
use miniextendr_api::miniextendr;

#[miniextendr]
pub trait Counter { fn value(&self) -> i32; }

mod first {
    use miniextendr_api::{ExternalPtr, miniextendr};
    #[derive(ExternalPtr)]
    pub struct Host;
    #[miniextendr(env)]
    impl crate::Counter for Host { fn value(&self) -> i32 { 1 } }
}

mod second {
    use miniextendr_api::{ExternalPtr, miniextendr};
    #[derive(ExternalPtr)]
    pub struct Host;
    #[miniextendr(env)]
    impl crate::Counter for Host { fn value(&self) -> i32 { 2 } }
}

fn main() {}
