//! #2020: type-qualified trait functions still see consumer blanket traits.
#![allow(dead_code)]

struct ConsumerAssociatedCalled;

trait ConsumerAssociated: Sized {
    fn from<T>(_value: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn try_from<T>(_value: T) -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
    fn default() -> ConsumerAssociatedCalled { ConsumerAssociatedCalled }
}
impl<T> ConsumerAssociated for T {}

include!("common/every_macro.rs");

fn condition_single_pair() -> ! {
    miniextendr_api::rust_error!(data = ("value", 1), "refused");
}

fn condition_computed_pair(name: &str) -> ! {
    miniextendr_api::rust_error!(data = (name, 1), "refused");
}

fn condition_pair_list(name: &str) -> ! {
    miniextendr_api::rust_error!(data = [("value", 1), (name, 2)], "refused");
}

fn condition_keyed() -> ! {
    miniextendr_api::rust_error!(data = { value = 1 }, "refused");
}

fn main() {}
