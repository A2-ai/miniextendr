//! Tests for S7-style impl blocks (e.g., `#[miniextendr(s7)] impl Foo`).

use miniextendr_api::{miniextendr, miniextendr_module};

/// A simple counter that demonstrates S7-style impl block support.
/// This gets exported as an S7 class with new_generic methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7Counter {
    value: i32,
}

/// @noRd
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

/// Demonstrates S7 computed and dynamic properties.
///
/// This shows how to use `#[s7(getter)]` for computed properties (read-only)
/// and `#[s7(getter)]` + `#[s7(setter)]` for dynamic properties (read-write).
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7Range {
    start: f64,
    end: f64,
}

/// @noRd
#[miniextendr(s7)]
impl S7Range {
    /// Creates a new range with the given start and end values.
    pub fn new(start: f64, end: f64) -> Self {
        S7Range { start, end }
    }

    /// Computed property: read-only length of the range.
    ///
    /// This becomes an S7 property accessed as `obj@length` in R.
    /// Since there's no setter, it's a computed (read-only) property.
    #[miniextendr(s7(getter))]
    pub fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Dynamic property getter: read the midpoint.
    ///
    /// Combined with set_midpoint, this creates a dynamic property
    /// that can be both read and written.
    #[miniextendr(s7(getter, prop = "midpoint"))]
    pub fn get_midpoint(&self) -> f64 {
        (self.start + self.end) / 2.0
    }

    /// Dynamic property setter: set the midpoint (adjusts start and end).
    ///
    /// When the midpoint is set, both start and end are adjusted
    /// to maintain the current length while centering on the new midpoint.
    #[miniextendr(s7(setter, prop = "midpoint"))]
    pub fn set_midpoint(&mut self, value: f64) {
        let half_length = (self.end - self.start) / 2.0;
        self.start = value - half_length;
        self.end = value + half_length;
    }

    /// Regular method: returns the start value.
    pub fn s7_start(&self) -> f64 {
        self.start
    }

    /// Regular method: returns the end value.
    pub fn s7_end(&self) -> f64 {
        self.end
    }
}

miniextendr_module! {
    mod s7_tests;

    impl S7Counter;
    impl S7Range;
}
