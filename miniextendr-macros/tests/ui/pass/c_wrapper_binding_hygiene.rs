//! #1677: conversions must not shadow raw arguments or receiver bindings.
#![allow(dead_code)]
use miniextendr_api::dots::Dots;
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(Clone, ExternalPtr)]
pub struct Builder { value: i32 }

#[miniextendr(env)]
impl Builder {
    // The first conversion used to overwrite the second raw SEXP slot.
    pub fn numbered(&self, arg_1: i32, value: i32) -> i32 {
        self.value + arg_1 + value
    }
    pub fn receiver_names(&self, self_ref: i32, self_ptr: i32) -> i32 {
        self.value + self_ref + self_ptr
    }
    // All three return paths must return the original receiver SEXP.
    pub fn set(&mut self, self_sexp: i32) -> &mut Self {
        self.value = self_sexp;
        self
    }
    pub fn checked(&mut self, self_sexp: i32) -> Result<&mut Self, String> {
        self.value = self_sexp;
        Ok(self)
    }
    pub fn maybe(&mut self, self_sexp: i32) -> Option<&mut Self> {
        self.value = self_sexp;
        Some(self)
    }
    // Consuming paths retain a handle/slot while the arguments convert.
    pub fn finish(self, self_ptr: i32) -> Self {
        Self { value: self.value + self_ptr }
    }
    pub fn try_finish(self, self_slot: i32) -> Result<Self, String> {
        Ok(Self { value: self.value + self_slot })
    }
    pub fn terminal(self, __mx_taken: i32) -> i32 {
        self.value + __mx_taken
    }
}

// The generated Dots owner must not replace the scalar parameter.
#[miniextendr]
pub fn dots_owner(d_storage: i32, d: &Dots) -> i32 {
    let _ = d;
    d_storage
}

fn main() {}
