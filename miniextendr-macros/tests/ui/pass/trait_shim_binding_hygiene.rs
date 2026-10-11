//! #1677: user parameters may share names with trait-shim internals.
#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Counter { value: i32 }

#[miniextendr]
pub trait Inspect {
    fn probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32;
    fn transfer(&mut self, data: &mut Self, argc: i32, argv: i32,
        self_ref: i32, self_ptr: i32, __extptr_data: i32) -> i32;
}

#[miniextendr(env)]
impl Inspect for Counter {
    fn probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32 {
        self.value + data + argc + argv + self_ref
    }
    fn transfer(&mut self, data: &mut Self, argc: i32, argv: i32,
        self_ref: i32, self_ptr: i32, __extptr_data: i32) -> i32 {
        data.value += self.value;
        data.value + argc + argv + self_ref + self_ptr + __extptr_data
    }
}

// Generic trait implementations emit the concrete shim at the impl site.
#[derive(ExternalPtr)]
pub struct GenericCounter { value: i32 }

#[miniextendr]
pub trait InspectWith<T> {
    fn probe(&self, data: T, argc: T, argv: T, self_ref: T) -> i32;
    fn transfer(&mut self, data: &mut Self, argc: T, argv: T,
        self_ref: T, self_ptr: T, __extptr_data: T) -> i32;
}

#[miniextendr(env)]
impl InspectWith<i32> for GenericCounter {
    fn probe(&self, data: i32, argc: i32, argv: i32, self_ref: i32) -> i32 {
        self.value + data + argc + argv + self_ref
    }
    fn transfer(&mut self, data: &mut Self, argc: i32, argv: i32,
        self_ref: i32, self_ptr: i32, __extptr_data: i32) -> i32 {
        data.value += self.value;
        data.value + argc + argv + self_ref + self_ptr + __extptr_data
    }
}

fn main() {}
