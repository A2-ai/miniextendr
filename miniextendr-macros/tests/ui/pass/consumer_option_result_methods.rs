//! #2020: blanket consumer methods must not intercept Option/Result codegen.
#![allow(dead_code)]

struct ConsumerOptionResultCalled;

// Borrowing methods need qualification. Consuming inherent methods already
// win for an exact Option/Result receiver; keep those cases covered too.
trait ConsumerOptionResult: Sized {
    fn is_some(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn is_none(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn is_ok(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn is_err(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn as_ref(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn as_mut(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn as_deref(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn as_deref_mut(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn map<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn map_err<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn map_or<T, F>(self, _value: T, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn map_or_else<F, G>(self, _default: F, _f: G) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn and_then<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn or_else<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn ok_or<T>(self, _error: T) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn ok_or_else<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn unwrap(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn unwrap_or<T>(self, _value: T) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn unwrap_or_else<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn unwrap_or_default(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn expect(self, _message: &str) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn ok(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn err(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn transpose(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn flatten(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn take(self) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn replace<T>(self, _value: T) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn get_or_insert<T>(self, _value: T) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
    fn get_or_insert_with<F>(self, _f: F) -> ConsumerOptionResultCalled { ConsumerOptionResultCalled }
}
impl<T> ConsumerOptionResult for T {}

include!("common/every_macro.rs");

#[miniextendr]
pub fn optional_unit(present: bool) -> ::core::option::Option<()> {
    present.then_some(())
}

fn main() {}
