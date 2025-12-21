//! Test default parameters in R6 methods.

use miniextendr_api::{miniextendr, miniextendr_module};

#[derive(miniextendr_api::ExternalPtr)]
pub struct Calculator {
    value: f64,
}

/// @title Calculator R6 Class with Default Parameters
/// @description Calculator that demonstrates default parameter values in R6 methods.
/// @examples
/// calc <- Calculator$new()      # Uses default initial = 0.0
/// calc$add()                     # Uses default amount = 1.0
/// calc$get()
#[miniextendr(r6)]
impl Calculator {
    /// Creates a new calculator.
    #[miniextendr(defaults(initial = "0.0"))]
    pub fn new(initial: f64) -> Self {
        Calculator { value: initial }
    }

    /// Returns the current value.
    pub fn get(&self) -> f64 {
        self.value
    }

    /// Adds to the current value.
    #[miniextendr(defaults(amount = "1.0"))]
    pub fn add(&mut self, amount: f64) -> f64 {
        self.value += amount;
        self.value
    }

    /// Sets a new value.
    #[miniextendr(defaults(new_value = "0.0"))]
    pub fn set(&mut self, new_value: f64) {
        self.value = new_value;
    }
}

miniextendr_module! {
    mod r6_default_tests;
    impl Calculator;
}
