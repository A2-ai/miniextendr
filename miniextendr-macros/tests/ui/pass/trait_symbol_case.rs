//! #1778: case-distinct traits must coexist at declaration and impl sites.
#![allow(dead_code)]
#![deny(nonstandard_style)]
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Host;

#[miniextendr]
pub trait Counter { fn value(&self) -> i32; }
#[miniextendr]
#[allow(non_camel_case_types)]
pub trait counter { fn value(&self) -> i32; }

#[miniextendr(env)]
impl Counter for Host { fn value(&self) -> i32 { 1 } }
#[miniextendr(env)]
impl counter for Host { fn value(&self) -> i32 { 2 } }

// The generic-trait path builds its vtables at the impl site.
#[miniextendr]
pub trait ReadAs<T> { fn read(&self) -> T; }
#[miniextendr]
#[allow(non_camel_case_types)]
pub trait readAs<T> { fn read(&self) -> T; }
#[miniextendr(env)]
impl ReadAs<i32> for Host { fn read(&self) -> i32 { 3 } }
#[miniextendr(env)]
impl readAs<i32> for Host { fn read(&self) -> i32 { 4 } }

fn main() {
    let _: CounterVTable = __Counter_build_vtable::<Host>();
    let _: counterVTable = __counter_build_vtable::<Host>();
    assert_ne!(TAG_Counter, TAG_counter);
    assert_ne!(TAG_ReadAs, TAG_readAs);
}
