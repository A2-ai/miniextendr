//! Tests for R6-style impl blocks (e.g., `#[miniextendr(r6)] impl Foo`).
//!
//! This module also tests:
//! - Standalone functions mixed with impl blocks
//! - Multiple impl blocks in a single module

use miniextendr_api::{miniextendr, miniextendr_module};

/// @noRd
#[miniextendr]
/// @name rpkg_r6_standalone
/// @noRd
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

/// @noRd
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

/// @noRd
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

/// @noRd
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

/// An R6 class demonstrating active bindings with both getters and setters.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Temperature {
    celsius: f64,
}

/// @noRd
#[miniextendr(r6)]
impl R6Temperature {
    /// Creates a new temperature in Celsius.
    pub fn new(celsius: f64) -> Self {
        R6Temperature { celsius }
    }

    /// Get the temperature in Celsius (active binding getter).
    #[miniextendr(r6(active))]
    pub fn celsius(&self) -> f64 {
        self.celsius
    }

    /// Set the temperature in Celsius (active binding setter).
    #[miniextendr(r6(setter, prop = "celsius"))]
    pub fn set_celsius(&mut self, value: f64) {
        self.celsius = value;
    }

    /// Get the temperature in Fahrenheit (active binding with getter+setter).
    #[miniextendr(r6(active, prop = "fahrenheit"))]
    pub fn fahrenheit(&self) -> f64 {
        self.celsius * 9.0 / 5.0 + 32.0
    }

    /// Set the temperature via Fahrenheit (active binding setter).
    #[miniextendr(r6(setter, prop = "fahrenheit"))]
    pub fn set_fahrenheit(&mut self, value: f64) {
        self.celsius = (value - 32.0) * 5.0 / 9.0;
    }
}

/// An R6 class demonstrating cloneable support.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Cloneable {
    value: i32,
}

/// @noRd
#[miniextendr(r6(cloneable, lock_class))]
impl R6Cloneable {
    /// Creates a new instance.
    pub fn new(value: i32) -> Self {
        R6Cloneable { value }
    }

    /// Returns the value.
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// Sets the value.
    pub fn set_value(&mut self, value: i32) {
        self.value = value;
    }
}

// === R6 Inheritance ===

/// Base animal class for R6 inheritance testing.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Animal {
    name: String,
    sound: String,
}

/// @noRd
#[miniextendr(r6)]
impl R6Animal {
    pub fn new(name: String, sound: String) -> Self {
        R6Animal { name, sound }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn speak(&self) -> String {
        format!("{} says {}", self.name, self.sound)
    }
}

/// Dog inherits from R6Animal.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6Dog {
    breed: String,
}

/// @noRd
#[miniextendr(r6(inherit = "R6Animal"))]
impl R6Dog {
    pub fn new(breed: String) -> Self {
        R6Dog { breed }
    }
    pub fn breed(&self) -> String {
        self.breed.clone()
    }
    pub fn fetch(&self) -> String {
        format!("{} fetches the ball!", self.breed)
    }
}

/// GoldenRetriever for 3-level chain: R6Animal -> R6Dog -> R6GoldenRetriever
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6GoldenRetriever {
    owner: String,
}

/// @noRd
#[miniextendr(r6(inherit = "R6Dog"))]
impl R6GoldenRetriever {
    pub fn new(owner: String) -> Self {
        R6GoldenRetriever { owner }
    }
    pub fn owner(&self) -> String {
        self.owner.clone()
    }
}

// === R6 Portable Flag ===

/// Non-portable R6 class for testing portable = FALSE flag.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6NonPortable {
    value: i32,
}

/// @noRd
#[miniextendr(r6(portable = false))]
impl R6NonPortable {
    pub fn new(value: i32) -> Self {
        R6NonPortable { value }
    }
    pub fn get_value(&self) -> i32 {
        self.value
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
    impl R6Temperature;
    impl R6Cloneable;

    // Inheritance tests
    impl R6Animal;
    impl R6Dog;
    impl R6GoldenRetriever;

    // Portable flag test
    impl R6NonPortable;
}
