//! Test: the receivers `self: &ExternalPtr<Self>` / `self: &mut ExternalPtr<Self>`
//! are refused with a pointer to the views the wrapper borrows (#2106).

use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Counter {
    value: i32,
}

#[miniextendr(env)]
impl Counter {
    pub fn read(self: &ExternalPtr<Self>) -> i32 {
        0
    }
}

#[derive(ExternalPtr)]
pub struct Gauge {
    value: i32,
}

#[miniextendr(env)]
impl Gauge {
    pub fn write(self: &mut ExternalPtr<Self>, value: i32) {}
}

fn main() {}
