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

// The alias check of an `Either` argument (#1504) binds its prepared state
// by position, and its borrows, arguments and selection: none may capture
// a parameter, in the declaration's shim or a generic impl's.
use miniextendr_api::either_impl::Either;

#[derive(miniextendr_api::TryFromSexp)]
pub struct Slot(pub &'static mut i32);

impl miniextendr_api::IntoR for Slot {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<miniextendr_api::SEXP, Self::Error> {
        Ok(miniextendr_api::IntoR::into_sexp(*self.0))
    }
}

fn pick(a: Either<i32, Slot>) -> i32 {
    match a {
        Either::Left(n) => n,
        Either::Right(slot) => *slot.0,
    }
}

#[miniextendr]
pub trait Pick {
    fn pick(&self, __mx_prepared_1: i32, a: Either<i32, Slot>,
        __miniextendr_native_borrows: i32, __miniextendr_native_args: i32,
        __miniextendr_select: i32) -> i32;
}

#[miniextendr(env)]
impl Pick for Counter {
    fn pick(&self, __mx_prepared_1: i32, a: Either<i32, Slot>,
        __miniextendr_native_borrows: i32, __miniextendr_native_args: i32,
        __miniextendr_select: i32) -> i32 {
        pick(a) + __mx_prepared_1 + __miniextendr_native_borrows
            + __miniextendr_native_args + __miniextendr_select
    }
}

#[miniextendr]
pub trait PickWith<T> {
    fn pick_with(&self, __mx_prepared_1: T, a: Either<i32, Slot>,
        __miniextendr_native_borrows: T, __miniextendr_native_args: T,
        __miniextendr_select: T) -> i32;
}

#[miniextendr(env)]
impl PickWith<i32> for GenericCounter {
    fn pick_with(&self, __mx_prepared_1: i32, a: Either<i32, Slot>,
        __miniextendr_native_borrows: i32, __miniextendr_native_args: i32,
        __miniextendr_select: i32) -> i32 {
        pick(a) + __mx_prepared_1 + __miniextendr_native_borrows
            + __miniextendr_native_args + __miniextendr_select
    }
}

fn main() {}
