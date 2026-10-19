#![allow(dead_code)]
use miniextendr_api::{ExternalPtr, Unchecked, miniextendr};
type Text = str;
type Values = [i32];

#[miniextendr]
pub fn text(#[miniextendr(convert)] value: &Text) -> &Text { value }
#[miniextendr]
pub fn values(#[miniextendr(convert)] value: Option<&Values>) -> i32 {
    value.map_or(0, |v| v.iter().sum())
}
#[miniextendr]
pub fn mutable<'a>(#[miniextendr(convert)] value: &'a mut Values) { value.fill(0); }
// The marker peels first: `convert` sees the reference it wraps.
#[miniextendr]
pub fn marked(#[miniextendr(convert)] value: Unchecked<&str>) -> usize { value.0.len() }

#[derive(ExternalPtr)]
pub struct Host;
#[miniextendr(env)]
impl Host {
    pub fn new() -> Self { Self }
    #[miniextendr(convert(value))]
    pub fn read(&self, value: &Text) -> String { value.to_owned() }
    #[miniextendr(convert(value))]
    pub fn marked(&self, value: Unchecked<&str>) -> usize { value.0.len() }
}

#[miniextendr]
pub trait Read { fn total(&self, value: &Values) -> i32; }
#[miniextendr(env)]
impl Read for Host {
    #[miniextendr(convert(value))]
    fn total(&self, value: &Values) -> i32 { value.iter().sum() }
}
fn main() {}
