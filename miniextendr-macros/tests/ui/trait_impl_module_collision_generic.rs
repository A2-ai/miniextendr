//! Concrete generic-trait shims must get the same early collision error.
use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr)]
pub struct Host;

mod first {
    use miniextendr_api::miniextendr;
    use crate::Host;
    #[miniextendr]
    pub trait Counter<T> { fn first(&self) -> T; }
    #[miniextendr(env)]
    impl Counter<i32> for crate::Host { fn first(&self) -> i32 { 1 } }
}

mod second {
    use miniextendr_api::miniextendr;
    use crate::Host;
    #[miniextendr]
    pub trait Counter<T> { fn second(&self) -> T; }
    #[miniextendr(env)]
    impl Counter<i32> for crate::Host { fn second(&self) -> i32 { 2 } }
}

fn main() {}
