//! A method-level option must be gated with the parameter it names.
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Host;

#[miniextendr(env)]
impl Host {
    #[miniextendr(no_na(hidden))]
    pub fn read(&self, #[cfg(any())] hidden: i32) -> i32 { 0 }
}

#[miniextendr]
pub trait Implemented { fn read(&self) -> i32; }

#[miniextendr(env)]
impl Implemented for Host {
    #[miniextendr(no_na(hidden))]
    fn read(&self, #[cfg(any())] hidden: i32) -> i32 { 0 }
}

fn main() {}
