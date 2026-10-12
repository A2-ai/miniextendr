//! #1489: a wrapper's call slot and a conversion's error binding keep one
//! hygiene context when the wrapper is generated inside another macro's
//! expansion. That covers methods spliced into a `macro_rules!`-authored impl,
//! and every trait impl, which expands through its trait's metadata macro.
//! Plain and coercing conversions, `no_na` checks and an `Either` choice each
//! read the call slot or the error from a template spanned at the argument's
//! type.
#![allow(dead_code)]

use miniextendr_api::either_impl::Either;
use miniextendr_api::{ExternalPtr, miniextendr};

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
