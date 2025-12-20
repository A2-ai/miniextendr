//! Tests for S7-style impl blocks (e.g., `#[miniextendr(s7)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates S7-style impl block support.
/// This gets exported as an S7 class with new_generic methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7Counter {
    value: i32,
}

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
