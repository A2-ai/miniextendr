//! Tests for R6-style impl blocks (e.g., `#[miniextendr(r6)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates R6-style impl block support.
/// This gets exported as an R6Class with `$new()`, `$value()`, `$inc()`, `$add()` methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Counter {
    value: i32,
}

#[miniextendr(r6)]
impl R6Counter {
    /// Creates a new counter with the given initial value.
    pub fn new(initial: i32) -> Self {
        R6Counter { value: initial }
    }

    /// Returns the current value.
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Increments the counter by 1 and returns the new value.
    pub fn inc(&mut self) -> i32 {
        self.value += 1;
        self.value
    }

    /// Adds the given amount to the counter and returns the new value.
    pub fn add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        self.value
    }

    /// A static method that returns a default counter (value = 0).
    pub fn default_counter() -> Self {
        R6Counter { value: 0 }
    }
}

miniextendr_module! {
    mod r6_tests;

    impl R6Counter;
}
