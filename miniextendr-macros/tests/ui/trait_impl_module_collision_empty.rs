//! Empty annotated impls use the same uniqueness check as populated ones.
use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr)]
pub struct Host;

mod first {
    use miniextendr_api::miniextendr;
    use crate::Host;
    #[miniextendr]
    pub trait Counter { fn first(&self) -> i32 { 1 } }
    impl<T> Counter for T {}
    #[miniextendr(env)]
    impl Counter for crate::Host {}
}

mod second {
    use miniextendr_api::miniextendr;
    use crate::Host;
    #[miniextendr]
    pub trait Counter { fn second(&self) -> i32 { 2 } }
    impl<T> Counter for T {}
    #[miniextendr(env)]
    impl Counter for crate::Host {}
}

fn main() {}
