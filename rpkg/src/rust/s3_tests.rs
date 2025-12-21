//! Tests for S3-style impl blocks (e.g., `#[miniextendr(s3)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates S3-style impl block support.
/// This gets exported as S3 methods: `new_s3counter()`, `value.S3Counter()`, etc.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S3Counter {
    value: i32,
}

/// @rdname S3Counter
/// @description S3 counter with `s3_value()`, `s3_inc()`, and `s3_add()` methods.
/// @aliases new_s3counter s3_value s3_inc s3_add s3counter_default_counter
/// @examples
/// x <- new_s3counter(1L)
/// s3_value(x)
/// s3_inc(x)
/// s3_add(x, 5L)
/// s3_value(s3counter_default_counter())
#[miniextendr(s3)]
impl S3Counter {
    /// Creates a new counter with the given initial value.
    pub fn new(initial: i32) -> Self {
        S3Counter { value: initial }
    }

    /// Returns the current value (S3-specific method name to avoid conflicts).
    pub fn s3_value(&self) -> i32 {
        self.value
    }

    /// Increments the counter by 1 and returns the new value.
    pub fn s3_inc(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    /// Adds the given amount to the counter and returns the new value.
    pub fn s3_add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        self.value
    }

    /// A static method that returns a default counter (value = 0).
    pub fn default_counter() -> Self {
        S3Counter { value: 0 }
    }
}

miniextendr_module! {
    mod s3_tests;

    impl S3Counter;
}
