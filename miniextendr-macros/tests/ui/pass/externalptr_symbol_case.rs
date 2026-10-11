//! Case-distinct types and sidecar fields must coexist in one module.
#![allow(dead_code)]
#![deny(nonstandard_style)]

use miniextendr_api::ExternalPtr;
use miniextendr_api::abi::mx_erased;
use miniextendr_api::externalptr::RSidecar;

// S7 emits the property table as well as the shared sidecar tables.
#[derive(ExternalPtr)]
#[externalptr(s7, restored(refuse = "every_read"))]
#[allow(non_snake_case)]
pub struct Counter {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub value: i32,
    #[r_data]
    pub Value: i32,
}

#[derive(ExternalPtr)]
#[externalptr(s7, restored(refuse = "every_read"))]
#[allow(non_camel_case_types, non_snake_case)]
pub struct counter {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub value: i32,
    #[r_data]
    pub Value: i32,
}

// Raw syntax must not become part of an erased-wrapper helper's name.
#[derive(ExternalPtr)]
#[allow(non_camel_case_types)]
pub struct r#type;

fn main() {
    let _: fn(Counter) -> *mut mx_erased = __mx_wrap_Counter;
    let _: fn(counter) -> *mut mx_erased = __mx_wrap_counter;
    let _: fn(r#type) -> *mut mx_erased = __mx_wrap_type;
    assert_ne!(__MX_TAG_Counter, __MX_TAG_counter);
}
