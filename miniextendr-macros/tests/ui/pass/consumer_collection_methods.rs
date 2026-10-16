//! #2020: consumer traits must not intercept generated collection/string calls.
#![allow(dead_code)]

struct ConsumerCollectionCalled;

// A by-value receiver can win over an inherent method that borrows a Vec.
trait ConsumerCollections: Sized {
    fn len(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn push<T>(self, _value: T) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn iter(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn iter_mut(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn as_str(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn as_bytes(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn is_empty(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn contains<T>(self, _value: T) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn starts_with<T>(self, _value: T) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn strip_prefix<T>(self, _value: T) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn join<T>(self, _sep: T) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
    fn into_boxed_slice(self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
}
impl<T> ConsumerCollections for T {}

trait ConsumerFormatting {
    fn to_string(&self) -> ConsumerCollectionCalled { ConsumerCollectionCalled }
}
impl<T: ?Sized> ConsumerFormatting for T {}

include!("common/every_macro.rs");

fn literal_condition_field() -> ! {
    miniextendr_api::rust_error!(data = ("value", 1), "refused");
}

fn identifier_condition_field() -> ! {
    miniextendr_api::rust_error!(data = { value = 1 }, "refused");
}

fn computed_condition_field(name: &str) -> ! {
    miniextendr_api::rust_error!(data = (name, 1), "refused");
}

fn main() {}
