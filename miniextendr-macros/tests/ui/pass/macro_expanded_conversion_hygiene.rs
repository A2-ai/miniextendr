//! #1489: a wrapper's call slot and a conversion's error binding keep one
//! hygiene context when the wrapper is generated inside another macro's
//! expansion. That covers methods spliced into a `macro_rules!`-authored impl,
//! and every trait impl, which expands through its trait's metadata macro.
//! Plain and coercing conversions, `no_na` checks and an `Either` choice each
//! read the call slot or the error from a template spanned at the argument's
//! type. So do vectors, strings, strict integers, a mutable `match_arg` slice
//! and an `Either` arm whose failure asks the type what it declares.
#![allow(dead_code)]

use miniextendr_api::either_impl::Either;
use miniextendr_api::{AsNumericVec, DataFrame, ExternalPtr, MatchArg, miniextendr};

#[derive(Clone, Copy, MatchArg)]
pub enum Mode {
    Fast,
    Safe,
}

// An `Either` arm the macro can't word: the failure path asks the type what it
// declares, which reads the error binding.
#[derive(miniextendr_api::TryFromSexp)]
pub struct OpaqueNumbers(pub AsNumericVec);

macro_rules! define_class {
    ($name:ident, { $($extra:tt)* }) => {
        #[derive(ExternalPtr)]
        pub struct $name;

        #[miniextendr(env, internal, strict)]
        impl $name {
            pub fn new() -> Self {
                Self
            }
            $($extra)*
        }
    };
}

define_class!(SplicedHost, {
    pub fn optional_i8(&self, present: bool) -> Option<i8> {
        present.then_some(42)
    }
    #[miniextendr(coerce)]
    pub fn narrowed(&self, x: u16) -> i32 {
        i32::from(x)
    }
    #[miniextendr(no_na(value))]
    pub fn doubled(&self, value: f64) -> f64 {
        value * 2.0
    }
    #[miniextendr(choices(level = "low, high"))]
    pub fn level(&self, level: Either<String, f64>) -> String {
        match level {
            Either::Left(name) => name,
            Either::Right(n) => n.to_string(),
        }
    }
    pub fn owned(&self, values: Vec<i32>) -> Vec<i32> {
        values
    }
    pub fn text(&self, text: &str) -> String {
        text.to_owned()
    }
    pub fn strict(&self, value: i64) -> i64 {
        value
    }
    #[miniextendr(coerce)]
    pub fn coerce_vector(&self, values: Vec<u32>) -> Vec<u32> {
        values
    }
    #[miniextendr(match_arg_several_ok(values))]
    pub fn mutable_choices(&self, values: &mut [Mode]) -> i32 {
        values.reverse();
        i32::try_from(values.len()).unwrap()
    }
    pub fn opaque_or_frame(&self, x: Either<OpaqueNumbers, DataFrame>) -> bool {
        x.is_left()
    }
});

#[miniextendr]
pub trait Levels {
    fn level(&self, level: Either<String, f64>) -> String;
    fn scale(&self, factor: f64) -> f64;
    fn count(&self, n: i32) -> Option<i32>;
    fn narrowed(&self, x: u16) -> i32;
}

#[derive(ExternalPtr)]
pub struct TraitHost;

#[miniextendr(env)]
impl Levels for TraitHost {
    #[miniextendr(choices(level = "low, high"))]
    fn level(&self, level: Either<String, f64>) -> String {
        match level {
            Either::Left(name) => name,
            Either::Right(n) => n.to_string(),
        }
    }
    #[miniextendr(no_na(factor))]
    fn scale(&self, factor: f64) -> f64 {
        factor
    }
    fn count(&self, n: i32) -> Option<i32> {
        (n > 0).then_some(n)
    }
    #[miniextendr(coerce)]
    fn narrowed(&self, x: u16) -> i32 {
        i32::from(x)
    }
}

fn main() {}
