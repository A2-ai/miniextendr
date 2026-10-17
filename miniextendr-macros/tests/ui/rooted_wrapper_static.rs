//! #2038: wrappers must not hide a rooted argument's static lifetime.
use miniextendr_api::{Missing, RCow, StrVec, miniextendr};

#[miniextendr]
pub fn missing_view(value: Missing<StrVec<'static>>) {
    let _ = value;
}

#[miniextendr]
pub fn result_view(value: Result<RCow<'static, i32>, ()>) {
    let _ = value;
}

#[miniextendr]
pub fn nested_reference(value: Missing<Result<&'static [i32], ()>>) {
    let _ = value;
}

#[miniextendr]
pub trait RootedWrapper {
    fn read(&self, value: Missing<Result<StrVec<'static>, ()>>);
}

fn main() {}
