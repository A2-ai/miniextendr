#![allow(dead_code, unused_imports)]
use miniextendr_api::dots::Dots;
use miniextendr_api::{ExternalPtr, ExternalPtrMut, ExternalPtrRef, miniextendr};

#[derive(ExternalPtr)]
pub struct Other(i32);

#[miniextendr]
pub trait Shared {
    fn read(&self, other: &Other) -> i32;
}

#[miniextendr]
pub trait GenericDefault<T> {
    fn read(&self, other: &mut Other) -> i32 { other.0 }
}

#[miniextendr]
pub trait Optional {
    fn read(&self, other: Option<&Other>) -> i32;
}

#[miniextendr]
pub trait BorrowedView {
    fn read(&self, other: ExternalPtrRef<'_, Other>) -> i32;
}

#[miniextendr]
pub trait OptionalView {
    fn read(&self, other: Option<ExternalPtrMut<'_, Other>>) -> i32;
}

#[miniextendr]
pub trait ForcedDots {
    fn read(&self, rest: &Dots) -> i32;
}

#[miniextendr]
pub trait GenericDots<T> {
    fn read(&self, _: &miniextendr_api::dots::Dots) -> i32 { 0 }
}

#[miniextendr]
pub trait OptionalDots {
    fn read(&self, rest: Option<&Dots>) -> i32;
}

fn main() {}
