//! The parameter forms that borrow an external value for the call (#2106):
//! `&T` / `&mut T`, the views, `Option` of either, `&Self` in a method, view
//! receivers, and the unborrowed `ExternalPtr<T>` handle.
#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, ExternalPtrMut, ExternalPtrRef, miniextendr};

#[derive(ExternalPtr)]
pub struct Counter {
    value: i32,
}

#[miniextendr]
pub fn shared(x: &Counter) -> i32 {
    x.value
}

#[miniextendr]
pub fn exclusive(x: &mut Counter, y: &Counter) -> i32 {
    x.value += y.value;
    x.value
}

#[miniextendr]
pub fn optional(x: Option<&Counter>, y: Option<&mut Counter>) -> i32 {
    let add = x.map_or(0, |x| x.value);
    y.map_or(0, |y| {
        y.value += add;
        y.value
    })
}

fn helper(x: &mut Counter) {
    x.value += 1;
}

#[miniextendr]
pub fn views(x: ExternalPtrRef<'_, Counter>, mut y: ExternalPtrMut<'_, Counter>) -> i32 {
    helper(&mut y);
    let again = y.rb_mut();
    again.value + x.value
}

#[miniextendr]
pub fn optional_view(x: Option<ExternalPtrMut<'_, Counter>>) -> Option<i32> {
    x.map(|x| x.value)
}

#[miniextendr]
pub fn handle(x: ExternalPtr<Counter>) -> i32 {
    ExternalPtr::into_inner(x).value
}

#[miniextendr]
pub fn handles(x: Vec<ExternalPtr<Counter>>) -> i32 {
    x.iter().map(|x| x.borrow().value).sum()
}

#[miniextendr(env)]
impl Counter {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn absorb(&mut self, other: &Self) {
        self.value += other.value;
    }

    pub fn shared_view(self: ExternalPtrRef<'_, Self>, other: &Counter) -> i32 {
        self.value + other.value
    }

    pub fn exclusive_view(mut self: ExternalPtrMut<'_, Self>) -> i32 {
        self.value += 1;
        self.value
    }

    pub fn by_handle(self: ExternalPtr<Self>) -> i32 {
        self.borrow().value
    }
}

fn main() {}
