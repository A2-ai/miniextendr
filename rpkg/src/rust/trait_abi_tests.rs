//! Trait ABI tests for cross-package trait dispatch.
//!
//! This module tests `#[miniextendr]` on traits and trait implementations,
//! plus `` wiring.

use miniextendr_api::miniextendr;

// region: Define a trait with #[miniextendr]

/// Counter trait for testing cross-type trait ABI dispatch.
#[miniextendr]
pub trait Counter {
    /// Associated constant: Maximum value for this counter type.
    const MAX_VALUE: i32;

    /// Get the current value.
    fn value(&self) -> i32;

    /// Increment the counter by 1.
    fn increment(&mut self);

    /// Add n to the counter (checked - may panic on overflow).
    fn checked_add(&mut self, n: i32);

    /// Static method: Returns the default initial value for this counter type.
    fn default_initial() -> i32;
}
// endregion

// region: Implement the trait for a concrete type

/// A simple counter implementation.
#[derive(miniextendr_api::ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

impl SimpleCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for SimpleCounter (default dispatch).
#[miniextendr]
impl Counter for SimpleCounter {
    const MAX_VALUE: i32 = i32::MAX;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        0 // SimpleCounter defaults to 0
    }
}
// endregion

// region: R-exposed functions for testing (using impl block pattern)

#[miniextendr]
impl SimpleCounter {
    /// Create a new counter with given initial value.
    fn new_counter(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get the counter's value.
    fn get_value(&self) -> i32 {
        self.value
    }

    /// Add to the counter using the trait method.
    fn trait_add(&mut self, n: i32) {
        // Call the trait method
        Counter::checked_add(self, n);
    }
}
// endregion

// region: Test panic handling in shims - separate struct

/// A counter that panics when you try to decrement below zero.
#[derive(miniextendr_api::ExternalPtr)]
pub struct PanickyCounter {
    value: i32,
}

impl PanickyCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

#[miniextendr]
impl Counter for PanickyCounter {
    const MAX_VALUE: i32 = 1000; // PanickyCounter has a lower max

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn checked_add(&mut self, n: i32) {
        if self.value + n < 0 {
            panic!(
                "PanickyCounter: cannot go below zero! (value={}, n={})",
                self.value, n
            );
        }
        self.value += n;
    }

    fn default_initial() -> i32 {
        100 // PanickyCounter defaults to 100 (safe margin above 0)
    }
}

#[miniextendr]
impl PanickyCounter {
    /// Create a new panicky counter.
    fn new_panicky(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value.
    fn get_value(&self) -> i32 {
        self.value
    }

    /// Add (may panic).
    fn try_add(&mut self, n: i32) {
        Counter::checked_add(self, n);
    }
}
// endregion

// region: S3 trait impl test - uses S3 dispatch for trait methods

/// A counter that uses S3 dispatch for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S3TraitCounter {
    value: i32,
}

impl S3TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for S3TraitCounter (S3 dispatch).
#[miniextendr(s3)]
impl Counter for S3TraitCounter {
    const MAX_VALUE: i32 = 500;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        50 // S3TraitCounter defaults to 50
    }
}

#[miniextendr(s3)]
impl S3TraitCounter {
    /// Create a new S3 trait counter.
    fn new_s3trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: S4 trait impl test - uses S4 dispatch for trait methods

/// A counter that uses S4 dispatch for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S4TraitCounter {
    value: i32,
}

impl S4TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for S4TraitCounter (S4 dispatch).
#[miniextendr(s4, internal)]
impl Counter for S4TraitCounter {
    const MAX_VALUE: i32 = 400;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        40 // S4TraitCounter defaults to 40
    }
}

/// S4TraitCounter inherent methods (constructor and getters).
#[miniextendr(s4, internal)]
impl S4TraitCounter {
    /// Create a new S4 trait counter.
    fn new_s4trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: S7 trait impl test - uses S7 dispatch for trait methods

/// A counter that uses S7 dispatch for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct S7TraitCounter {
    value: i32,
}

impl S7TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for S7TraitCounter (S7 dispatch).
#[miniextendr(s7, internal)]
impl Counter for S7TraitCounter {
    const MAX_VALUE: i32 = 300;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        30 // S7TraitCounter defaults to 30
    }
}

/// S7TraitCounter inherent methods (constructor and getters).
#[miniextendr(s7, internal)]
impl S7TraitCounter {
    /// Create a new S7 trait counter.
    fn new_s7trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: R6 trait impl test - uses R6 dispatch for trait methods

/// A counter that uses R6 dispatch (standalone functions) for its trait methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct R6TraitCounter {
    value: i32,
}

impl R6TraitCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Counter trait implementation for R6TraitCounter (R6 dispatch).
#[miniextendr(r6)]
impl Counter for R6TraitCounter {
    const MAX_VALUE: i32 = 200;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    /// @param n Amount to add.
    fn checked_add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        20 // R6TraitCounter defaults to 20
    }
}

/// R6TraitCounter inherent methods (constructor and getters).
#[miniextendr(r6)]
impl R6TraitCounter {
    /// Create a new R6 trait counter.
    pub fn new_r6trait(initial: i32) -> Self {
        Self::new(initial)
    }

    /// Get value via inherent method.
    pub fn get_value(&self) -> i32 {
        self.value
    }
}
// endregion

// region: Module registration
// endregion
