//! Tests for S4-style impl blocks (e.g., `#[miniextendr(s4)] impl Foo`).

use miniextendr_api::miniextendr;

/// A simple counter that demonstrates S4-style impl block support.
/// This gets exported as an S4 class with setMethod dispatches.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S4Counter {
    value: i32,
}

/// S4 counter class with setMethod dispatches for constructor, accessor, and mutation.
/// @aliases s4_add,S4Counter-method s4_current,S4Counter-method s4_get_value,S4Counter-method s4_inc,S4Counter-method s4_type,S4Counter-method s4_value,S4Counter-method
#[miniextendr(s4, internal)]
impl S4Counter {
    /// Creates a new counter with the given initial value.
    /// @param initial Initial value for the counter.
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
    /// @param amount Integer amount to add.
    pub fn add(&mut self, amount: i32) -> i32 {
        self.value += amount;
        self.value
    }

    // #1677: the default S4 generic follows the R method name.
    /// Returns the current value through the generic `s4_current`: the
    /// default S4 generic follows the R method name.
    #[miniextendr(r_name = "current")]
    pub fn peek(&self) -> i32 {
        self.value
    }

    /// Returns `"counter"` through the generic `s4_type`: a raw identifier
    /// names the generic without its `r#`.
    pub fn r#type(&self) -> String {
        "counter".to_string()
    }

    /// A static method that returns a default counter (value = 0).
    pub fn default_counter() -> Self {
        S4Counter { value: 0 }
    }
}
