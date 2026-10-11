//! #1990: preserve mutable Self arguments and raw names in both shim paths.
#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, miniextendr};

#[derive(ExternalPtr)]
pub struct Counter { value: i32 }

#[miniextendr]
pub trait Transfer {
    fn copy_to(&self, other: &mut Self);
    fn swap_with(&mut self, r#type: &mut Self);
    fn difference(&self, r#type: &Self) -> i32;
    fn exchange(left: &mut Self, right: &mut Self);
}

#[miniextendr(env)]
impl Transfer for Counter {
    fn copy_to(&self, other: &mut Self) { other.value = self.value; }
    fn swap_with(&mut self, r#type: &mut Self) { std::mem::swap(self, r#type); }
    fn difference(&self, r#type: &Self) -> i32 { self.value - r#type.value }
    fn exchange(left: &mut Self, right: &mut Self) { std::mem::swap(left, right); }
}

#[derive(ExternalPtr)]
pub struct GenericCounter { value: i32 }

#[miniextendr]
pub trait TransferBy<T> {
    fn add_to(&self, r#type: &mut Self, amount: T);
    fn difference(&self, r#type: &Self) -> i32;
}

#[miniextendr(env)]
impl TransferBy<i32> for GenericCounter {
    fn add_to(&self, r#type: &mut Self, amount: i32) {
        r#type.value = self.value + amount;
    }
    fn difference(&self, r#type: &Self) -> i32 { self.value - r#type.value }
}

fn main() {}
