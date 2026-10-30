//! #2027: an `ExternalPtr<T>` argument doesn't need `T: Send`. Every wrapper
//! runs on R's main thread, so a handle never crosses threads: the four
//! argument forms, a trait's `&Self` parameters and a `#[r_data]` struct
//! (its `Sidecar` is `!Send`) all compile.
#![allow(dead_code)]
use miniextendr_api::externalptr::Sidecar;
use miniextendr_api::{ExternalPtr, miniextendr};
use std::rc::Rc;

#[derive(ExternalPtr)]
pub struct Local {
    value: Rc<i32>,
}

#[miniextendr]
pub fn make_local(value: i32) -> ExternalPtr<Local> {
    ExternalPtr::new(Local {
        value: Rc::new(value),
    })
}

#[miniextendr]
pub fn read_local(x: ExternalPtr<Local>) -> i32 {
    *x.borrow().value
}

#[miniextendr]
pub fn read_optional_local(x: Option<ExternalPtr<Local>>) -> i32 {
    x.map_or(0, |x| *x.borrow().value)
}

#[miniextendr]
pub fn sum_locals(x: Vec<ExternalPtr<Local>>) -> i32 {
    x.iter().map(|x| *x.borrow().value).sum()
}

#[miniextendr]
pub fn sum_optional_locals(x: Vec<Option<ExternalPtr<Local>>>) -> i32 {
    x.iter().flatten().map(|x| *x.borrow().value).sum()
}

#[miniextendr]
pub trait Compare {
    fn difference(&self, other: &Self) -> i32;
}

#[miniextendr(env)]
impl Compare for Local {
    fn difference(&self, other: &Self) -> i32 {
        *self.value - *other.value
    }
}

#[derive(ExternalPtr)]
pub struct WithSidecar {
    #[r_data]
    label: Sidecar<String>,
}

#[miniextendr]
pub fn take_with_sidecar(x: ExternalPtr<WithSidecar>) -> i32 {
    drop(x);
    1
}

fn main() {}
