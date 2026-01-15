//! Tests for env-style impl blocks (e.g., `#[miniextendr(env)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates env-style impl block support.
/// This gets exported as an R class with `$new()`, `$value()`, `$inc()`, `$add()` methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct ReceiverCounter {
    value: i32,
}

/// @noRd
#[miniextendr(env)]
impl ReceiverCounter {
    /// Creates a new counter with the given initial value.
    pub fn new(initial: i32) -> Self {
        ReceiverCounter { value: initial }
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
        ReceiverCounter { value: 0 }
    }
}

miniextendr_module! {
    mod receiver_tests;

    impl ReceiverCounter;
}
