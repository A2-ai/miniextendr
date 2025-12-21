//! Tests for S4-style impl blocks (e.g., `#[miniextendr(s4)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates S4-style impl block support.
/// This gets exported as an S4 class with setMethod dispatches.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S4Counter {
    value: i32,
}

/// @rdname S4Counter
/// @description S4 counter with `s4_value()`, `s4_inc()`, and `s4_add()` methods.
/// @aliases S4Counter s4_value s4_inc s4_add S4Counter_default_counter
/// @examples
/// x <- S4Counter(1L)
/// s4_value(x)
/// s4_inc(x)
/// s4_add(x, 3L)
/// s4_value(S4Counter_default_counter())
#[miniextendr(s4)]
impl S4Counter {
    /// Creates a new counter with the given initial value.
    pub fn new(initial: i32) -> Self {
        S4Counter { value: initial }
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
        S4Counter { value: 0 }
    }
}

miniextendr_module! {
    mod s4_tests;

    impl S4Counter;
}
