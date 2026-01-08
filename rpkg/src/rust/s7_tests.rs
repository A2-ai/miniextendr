//! Tests for S7-style impl blocks (e.g., `#[miniextendr(s7)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates S7-style impl block support.
/// This gets exported as an S7 class with new_generic methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7Counter {
    value: i32,
}

/// @rdname S7Counter
/// @description S7 counter with `s7_value()`, `s7_inc()`, and `s7_add()` methods.
/// @aliases S7Counter s7_value s7_inc s7_add S7Counter_default_counter
/// @examples
/// x <- S7Counter(1L)
/// s7_value(x)
/// s7_inc(x)
/// s7_add(x, 2L)
/// s7_value(S7Counter_default_counter())
/// @param initial Initial value for the counter.
/// @param .ptr External pointer (used internally by S7).
#[miniextendr(s7)]
impl S7Counter {
    /// Creates a new counter with the given initial value.
    pub fn new(initial: i32) -> Self {
        S7Counter { value: initial }
    }

    /// Returns the current value (S7-specific method name to avoid conflicts).
    pub fn s7_value(&self) -> i32 {
        self.value
    }

    /// Increments the counter by 1 and returns the new value.
    pub fn s7_inc(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    /// Adds the given amount to the counter and returns the new value.
    pub fn s7_add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        self.value
    }

    /// A static method that returns a default counter (value = 0).
    pub fn default_counter() -> Self {
        S7Counter { value: 0 }
    }
}

miniextendr_module! {
    mod s7_tests;

    impl S7Counter;
}
