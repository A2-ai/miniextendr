//! Same-named traits in distinct modules must fail before linking.
use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr)]
pub struct Host;

mod first {
    use miniextendr_api::miniextendr;
    #[miniextendr]
    pub trait Counter { fn first(&self) -> i32; }
    #[miniextendr(env)]
    impl Counter for crate::Host { fn first(&self) -> i32 { 1 } }
}

mod second {
    use miniextendr_api::miniextendr;
    #[miniextendr]
    pub trait Counter { fn second(&self) -> i32; }
    #[miniextendr(env)]
    impl Counter for crate::Host { fn second(&self) -> i32 { 2 } }
}

fn main() {}
