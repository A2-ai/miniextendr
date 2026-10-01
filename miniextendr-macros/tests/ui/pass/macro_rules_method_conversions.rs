//! Mixed macro/caller spans must not hide the wrapper's call-context parameter.
#![allow(dead_code)]

use miniextendr_api::{ExternalPtr, MatchArg, miniextendr};

#[derive(Clone, Copy, MatchArg)]
pub enum Mode { Fast, Safe }

macro_rules! define_class {
    ($name:ident, { $($methods:tt)* }) => {
        #[derive(ExternalPtr, Default)]
        pub struct $name;

        #[miniextendr(env, internal, strict)]
        impl $name {
            pub fn new() -> Self { Self }
            // Macro-authored method and caller-authored methods share one impl.
            pub fn generated(&self, value: i32) -> i32 { value }
            $($methods)*
        }
    };
}

define_class!(HygieneClass, {
    pub fn ordinary(&self, present: bool) -> Option<i8> { present.then_some(42) }
    pub fn owned(&self, values: Vec<i32>) -> Vec<i32> { values }
    pub fn text(&self, text: &str) -> String { text.to_owned() }
    pub fn strict(&self, value: i64) -> i64 { value }

    #[miniextendr(coerce)]
    pub fn coerce_scalar(&self, value: u32) -> u32 { value }
    #[miniextendr(coerce)]
    pub fn coerce_vector(&self, values: Vec<u32>) -> Vec<u32> { values }

    #[miniextendr(match_arg_several_ok(values))]
    pub fn mutable_choices(&self, values: &mut [Mode]) -> i32 {
        values.reverse();
        i32::try_from(values.len()).unwrap()
    }
    #[miniextendr(env(worker))]
    pub fn worker_text(text: &str) -> String { text.to_owned() }
});

fn main() {}
