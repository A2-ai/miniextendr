//! Method parameter gates precede wrapper and trait-ABI generation (#2138).
#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Host;

#[miniextendr(env)]
impl Host {
    pub fn new(#[cfg(any())] _hidden: MissingType) -> Self { Self }

    #[cfg_attr(any(), miniextendr(no_na(hidden)))]
    pub fn read(
        &self,
        #[cfg(any())] #[miniextendr(no_na)] hidden: MissingType,
        #[cfg_attr(all(), miniextendr(no_na))] value: i32,
        #[cfg_attr(all(), cfg(any()))] _nested: MissingType,
        #[cfg_attr(any(), cfg(any()))] kept: i32,
    ) -> i32 { value + kept }
}

#[miniextendr]
pub trait Reads {
    fn required(
        &self,
        #[cfg(any())] _hidden: MissingType,
        #[cfg(all())] value: i32,
    ) -> i32;

    fn inherited(
        &self,
        #[cfg_attr(all(), cfg_attr(all(), cfg(any())))] _hidden: MissingType,
        value: i32,
    ) -> i32 { value }
}

#[miniextendr(env)]
impl Reads for Host {
    fn required(
        &self,
        #[cfg(any())] _hidden: MissingType,
        #[cfg(all())] value: i32,
    ) -> i32 { value }
}

#[miniextendr]
pub trait GenericReads<T> {
    fn required(&self, #[cfg(any())] _hidden: MissingType, value: T) -> T;
    fn inherited(&self, #[cfg(any())] _hidden: MissingType, value: T) -> T { value }
}

#[miniextendr(env)]
impl GenericReads<i32> for Host {
    fn required(&self, #[cfg(any())] _hidden: MissingType, value: i32) -> i32 { value }
}

#[miniextendr]
pub trait Defaults {
    fn value(&self, #[cfg(any())] _hidden: MissingType, value: i32) -> i32 { value }
}

impl Defaults for Host {}

#[miniextendr(env)]
impl Defaults for Host {}

fn main() {}
