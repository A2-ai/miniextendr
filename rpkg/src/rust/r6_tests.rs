//! Tests for R6-style impl blocks (e.g., `#[miniextendr(r6)] impl Foo`).
//!
//! This module also tests:
//! - Standalone functions mixed with impl blocks
//! - Multiple impl blocks in a single module

use miniextendr_api::{miniextendr, miniextendr_module};

/// R6 Standalone Function
///
/// Standalone helper in the R6 test module.
///
/// A standalone function in an impl-block module.
/// Tests that standalone fns work alongside impl blocks.
#[miniextendr]
/// @name rpkg_r6_standalone
/// @keywords internal
/// @examples
/// r6_standalone_add(1L, 2L)
/// @aliases r6_standalone_add
pub fn r6_standalone_add(a: i32, b: i32) -> i32 {
    a + b
}

/// A simple counter that demonstrates R6-style impl block support.
/// This gets exported as an R6Class with `$new()`, `$value()`, `$inc()`, `$add()` methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Counter {
    value: i32,
}

/// @title R6 Counter Class
/// @name R6Counter
/// @rdname R6Counter
/// @description R6 counter class that stores a single integer value.
/// @aliases R6Counter
/// @param initial The initial counter value (integer).
/// @param amount The amount to add to the counter (integer).
/// @details
/// **Methods:**
/// - `$new(initial)`: Creates a new counter with the given initial value.
/// - `$value()`: Returns the current value.
/// - `$inc()`: Increments the counter by 1 and returns the new value.
/// - `$add(amount)`: Adds the given amount to the counter and returns the new value.
/// @examples
/// c <- R6Counter$new(1L)
/// c$value()
/// c$inc()
/// c$add(10L)
/// R6Counter$default_counter()$value()
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

/// A second R6 class to test multiple impl blocks in one module.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Accumulator {
    total: f64,
    count: usize,
}

/// @title R6 Accumulator Class
/// @name R6Accumulator
/// @rdname R6Accumulator
/// @description R6 accumulator with running total and count.
/// @aliases R6Accumulator
/// @param value The value to accumulate (numeric).
/// @details
/// **Methods:**
/// - `$new()`: Creates a new accumulator starting at zero.
/// - `$accumulate(value)`: Adds a value and returns the new total.
/// - `$total()`: Returns the current total.
/// - `$count()`: Returns the count of accumulated values.
/// - `$average()`: Returns the average, or NA if no values accumulated.
/// @examples
/// acc <- R6Accumulator$new()
/// acc$accumulate(1.5)
/// acc$total()
/// acc$count()
/// acc$average()
#[miniextendr(r6)]
impl R6Accumulator {
    /// Creates a new accumulator starting at zero.
    pub fn new() -> Self {
        R6Accumulator {
            total: 0.0,
            count: 0,
        }
    }

    /// Adds a value and returns the new total.
    pub fn accumulate(&mut self, value: f64) -> f64 {
        self.total += value;
        self.count += 1;
        self.total
    }

    /// Returns the current total.
    pub fn total(&self) -> f64 {
        self.total
    }

    /// Returns the count of accumulated values.
    pub fn count(&self) -> i32 {
        self.count as i32
    }

    /// Returns the average, or NA if no values accumulated.
    pub fn average(&self) -> f64 {
        if self.count == 0 {
            f64::NAN
        } else {
            self.total / self.count as f64
        }
    }
}

/// An R6 class demonstrating active bindings for property-like access.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Rectangle {
    width: f64,
    height: f64,
}

/// @title R6 Rectangle Class with Active Bindings
/// @name R6Rectangle
/// @rdname R6Rectangle
/// @description R6 rectangle class demonstrating active bindings.
/// Active bindings provide property-like access (obj$area instead of obj$area()).
/// @aliases R6Rectangle
/// @param width The width of the rectangle (numeric).
/// @param height The height of the rectangle (numeric).
/// @details
/// **Methods:**
/// - `$new(width, height)`: Creates a new rectangle with given dimensions.
/// - `$get_width()`: Returns the width (regular method).
/// - `$get_height()`: Returns the height (regular method).
///
/// **Active Bindings (properties):**
/// - `$area`: Returns the area (width * height).
/// - `$perimeter`: Returns the perimeter (2 * (width + height)).
/// @examples
/// r <- R6Rectangle$new(3, 4)
/// r$area        # Property access (not r$area())
/// r$perimeter   # Property access
/// r$get_width() # Regular method
#[miniextendr(r6)]
impl R6Rectangle {
    /// Creates a new rectangle with given dimensions.
    pub fn new(width: f64, height: f64) -> Self {
        R6Rectangle { width, height }
    }

    /// Returns the width (regular method).
    pub fn get_width(&self) -> f64 {
        self.width
    }

    /// Returns the height (regular method).
    pub fn get_height(&self) -> f64 {
        self.height
    }

    /// Returns the area (active binding - property access).
    #[miniextendr(r6(active))]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// Returns the perimeter (active binding - property access).
    #[miniextendr(r6(active))]
    pub fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

miniextendr_module! {
    mod r6_tests;

    // Standalone function
    fn r6_standalone_add;

    // Multiple impl blocks
    impl R6Counter;
    impl R6Accumulator;
    impl R6Rectangle;
}
