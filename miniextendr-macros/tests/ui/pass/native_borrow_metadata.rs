//! Borrow preflight must query the selected conversion, preserving custom paths.
#![allow(dead_code, private_interfaces, non_camel_case_types)]
use miniextendr_api::dots::Dots;
use miniextendr_api::{miniextendr, FromRNewtype, SEXP, TryFromSexp};

#[miniextendr(no_worker)]
pub fn named_lifetimes<'a>(x: &'a i32, y: &'a str) -> i32 { *x + i32::try_from(y.len()).unwrap() }

#[miniextendr(no_worker)]
pub fn ignored_args(x: &mut [i32], _ignored: (), dots: &Dots) -> i32 { let _ = (_ignored, dots); i32::try_from(x.len()).unwrap() }

#[miniextendr(match_arg)]
#[derive(Copy, Clone)]
pub enum Mode { Fast, Slow }

#[miniextendr(no_worker)]
pub fn special_conversions(x: &mut [i32],
    #[miniextendr(match_arg)] optional: Option<Mode>,
    #[miniextendr(match_arg, several_ok)] vec: Vec<Mode>,
    #[miniextendr(match_arg, several_ok)] boxed: Box<[Mode]>,
    #[miniextendr(match_arg, several_ok)] array: [Mode; 2],
    #[miniextendr(match_arg, several_ok)] slice: &[Mode],
) -> i32 { let _ = (optional, vec, boxed, array, slice); i32::try_from(x.len()).unwrap() }

struct RLogical;
impl TryFromSexp for &RLogical {
    type Error = std::convert::Infallible;
    fn try_from_sexp(_: SEXP) -> Result<Self, Self::Error> { Ok(&RLogical) }
}
#[miniextendr(no_worker)]
pub fn custom_reference(x: &RLogical, y: &mut [i32]) -> i32 { let _ = x; i32::try_from(y.len()).unwrap() }

struct User;
impl FromRNewtype for &User {
    type Inner = i32;
    fn from_inner(_: i32) -> Self { &User }
}
// Vec<&User> converts via Vec<i32>; &User itself has no TryFromSexp impl.
#[miniextendr(no_worker)]
pub fn custom_container(x: Vec<&User>, y: &mut [i32]) -> i32 { i32::try_from(x.len() + y.len()).unwrap() }

struct u16;
impl miniextendr_api::TryCoerce<u16> for i32 {
    type Error = std::convert::Infallible;
    fn try_coerce(self) -> Result<u16, Self::Error> { Ok(u16) }
}
#[miniextendr(no_worker)]
pub fn coerce_only(#[miniextendr(coerce)] x: u16, y: &mut [i32]) -> i32 { let _ = x; i32::try_from(y.len()).unwrap() }

#[derive(TryFromSexp)]
struct Borrowed(&'static mut i32);
#[miniextendr(no_worker)]
pub fn newtype_borrow(x: Box<[Option<Borrowed>]>, y: &i32) -> i32 { i32::try_from(x.len()).unwrap() + *y }
fn main() {}
