//! Test default parameters in R6 methods.

use miniextendr_api::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
pub struct Calculator {
    value: f64,
}

/// @noRd
#[miniextendr(r6)]
impl Calculator {
    /// Creates a new calculator.
    /// @param initial Starting value (defaults to 0.0).
    #[miniextendr(defaults(initial = "0.0"))]
    pub fn new(initial: f64) -> Self {
        Calculator { value: initial }
    }

    /// Returns the current value.
    pub fn get(&self) -> f64 {
        self.value
    }

    /// Adds to the current value.
    /// @param amount Amount to add (defaults to 1.0).
    #[miniextendr(defaults(amount = "1.0"))]
    pub fn add(&mut self, amount: f64) -> f64 {
        self.value += amount;
        self.value
    }

    /// Sets a new value.
    /// @param new_value New value to set (defaults to 0.0).
    #[miniextendr(defaults(new_value = "0.0"))]
    pub fn set(&mut self, new_value: f64) {
        self.value = new_value;
    }
}
