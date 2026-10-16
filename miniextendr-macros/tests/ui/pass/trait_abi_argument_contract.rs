#![allow(dead_code)]
use miniextendr_api::dots::Dots;
use miniextendr_api::{ExternalPtr, List, miniextendr};

#[derive(ExternalPtr)]
pub struct Other(i32);
#[derive(ExternalPtr)]
pub struct Host(i32);

#[miniextendr]
pub trait Read {
    fn read(&self, values: &[i32], other: &Self) -> i32;
    fn handles(&self, other: ExternalPtr<Other>, args: List) -> i32;
    fn static_read(other: &Other, args: &Dots) -> i32;
    #[miniextendr(skip)]
    fn rust_only(&self, other: &mut Other, args: &Dots) { let _ = (other, args); }
}
#[miniextendr(env)]
impl Read for Host {
    fn read(&self, values: &[i32], other: &Self) -> i32 {
        self.0 + other.0 + values.iter().sum::<i32>()
    }
    fn handles(&self, other: ExternalPtr<Other>, args: List) -> i32 {
        other.borrow().0 + i32::try_from(args.len()).unwrap()
    }
    fn static_read(other: &Other, args: &Dots) -> i32 {
        other.0 + i32::try_from(args.len()).unwrap()
    }
}

type Values = [i32];
#[miniextendr]
pub trait GenericRead<T: ?Sized> {
    fn total(&self, #[miniextendr(convert)] values: &T) -> i32;
    fn change(&self, #[miniextendr(convert)] values: &mut T);
}
#[miniextendr(env)]
impl GenericRead<Values> for Host {
    fn total(&self, renamed: &Values) -> i32 { renamed.iter().sum() }
    fn change(&self, renamed: &mut Values) { renamed.fill(self.0); }
}
fn main() {}
