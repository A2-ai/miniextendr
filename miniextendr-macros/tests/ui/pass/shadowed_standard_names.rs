//! #1991: generated Rust must resolve its own standard-library names.
#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, miniextendr};

// Occupy both type and value namespaces. Bare generated variants/constructors
// would resolve to these zero-field structs instead of the standard library.
struct Some;
struct None;
struct Ok;
struct Err;
struct Option;
struct Result;
struct Box;
fn drop(_: u8) {}

#[miniextendr]
pub fn optional(x: i32) -> ::core::option::Option<i32> {
    ::core::option::Option::Some(x)
}
#[miniextendr]
pub fn fallible(x: i32) -> ::core::result::Result<i32, String> {
    ::core::result::Result::Ok(x)
}
// rpkg's shadowed_std_names_tests.rs covers the worker wrapper and the Vctrs
// derive, which need features this crate doesn't have.

#[derive(ExternalPtr)]
pub struct Counter {
    #[r_data]
    pub value: i32,
    #[r_data]
    pub label: String,
}
#[miniextendr(env)]
impl Counter {
    pub fn read(&self) -> ::core::option::Option<i32> {
        ::core::option::Option::Some(self.value)
    }
    pub fn write(&mut self, value: i32) -> ::core::result::Result<(), String> {
        self.value = value;
        ::core::result::Result::Ok(())
    }
}
#[miniextendr]
pub trait ReadValue { fn read_value(&self) -> i32; }
#[miniextendr(env)]
impl ReadValue for Counter { fn read_value(&self) -> i32 { self.value } }

#[derive(miniextendr_api::IntoList, miniextendr_api::TryFromList)]
pub struct Record { pub value: i32 }
#[derive(Clone, Copy, Debug, miniextendr_api::MatchArg)]
pub enum Mode { Fast, Slow }
#[derive(Clone, Copy, Debug, miniextendr_api::RFactor)]
pub enum Level { Low, High }
#[derive(Clone, miniextendr_api::DataFrameRow)]
pub struct Row { pub value: i32 }
#[derive(Clone, miniextendr_api::DataFrameRow)]
pub enum Event { Value { value: i32 }, Empty }
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(len = "len", elt = "value")]
pub struct Repeated { len: usize, value: i32 }
#[derive(miniextendr_api::AltrepString)]
#[altrep(len = "len", elt = "value")]
pub struct RepeatedString { len: usize, value: String }

fn main() {}
