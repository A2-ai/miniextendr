//! #2020: consumer blanket iterator methods must not affect generated Rust.
#![allow(dead_code)]

// A distinct return type also catches silent interception, not just E0034.
struct ConsumerIteratorCalled;

trait ConsumerIterators: Sized {
    fn into_iter(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn into_par_iter(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn next(&mut self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn map<F>(self, _f: F) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn filter<F>(self, _f: F) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn filter_map<F>(self, _f: F) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn collect<T>(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn enumerate(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn unzip<A, B, C, D>(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn flatten(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn zip<T>(self, _other: T) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn for_each<F>(self, _f: F) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn position<F>(&mut self, _f: F) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn chain<T>(self, _other: T) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn copied(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
    fn max(self) -> ConsumerIteratorCalled { ConsumerIteratorCalled }
}
impl<T> ConsumerIterators for T {}

include!("common/every_macro.rs");

fn main() {}
