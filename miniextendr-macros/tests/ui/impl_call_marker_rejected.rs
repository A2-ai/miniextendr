//! Test: a `Call` marker parameter on a class method.
//!
//! Condition-call markers are a standalone-function feature; class and trait
//! methods attribute conditions to the wrapper's own call (#1566).

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Widget {
    value: i32,
}

#[miniextendr(env)]
impl Widget {
    fn new(value: i32) -> Self {
        Self { value }
    }

    fn bump(&self, by: i32, _call: miniextendr_api::Call) -> i32 {
        self.value + by
    }
}

fn main() {}
