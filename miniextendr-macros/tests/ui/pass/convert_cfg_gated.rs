#![allow(dead_code, unexpected_cfgs)]
use miniextendr_api::{ExternalPtr, miniextendr};
type Text = str;

// A gated parameter resolves before `convert` is read (#2107).
#[miniextendr]
pub fn gated(
    #[cfg(feature = "nope")]
    #[miniextendr(convert)]
    off: &Text,
    #[cfg(not(feature = "nope"))]
    #[miniextendr(convert)]
    on: &Text,
) -> usize {
    #[cfg(feature = "nope")]
    return off.len();
    #[cfg(not(feature = "nope"))]
    on.len()
}

#[miniextendr]
pub fn gated_attr(#[cfg_attr(not(feature = "nope"), miniextendr(convert))] value: &Text) -> usize {
    value.len()
}

#[derive(ExternalPtr)]
pub struct Host;
#[miniextendr(env)]
impl Host {
    pub fn new() -> Self { Self }
    #[cfg(not(feature = "nope"))]
    #[miniextendr(convert(value))]
    pub fn read(&self, value: &Text) -> usize { value.len() }
}
fn main() {}
