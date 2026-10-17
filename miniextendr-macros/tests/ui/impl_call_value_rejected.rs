//! Methods accept only the no-call option; attribution markers stay standalone.
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
struct Widget;

#[miniextendr(env, call = caller)]
impl Widget {
    pub fn block(&self) {}
}

#[miniextendr(env)]
impl Widget {
    #[miniextendr(call = wrapper)]
    pub fn method(&self) {}
}

fn main() {}
