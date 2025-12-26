//! Trait ABI tests for cross-package trait dispatch.
//!
//! This module tests the `#[miniextendr]` on traits, on trait implementations,
//! and `#[derive(ExternalPtr)]` with `traits = [...]` attribute.

use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Define a trait with #[miniextendr]
// =============================================================================

/// A simple counter trait for testing trait ABI generation.
///
/// When `#[miniextendr]` is applied, this generates:
/// - `TAG_COUNTER`: 128-bit type tag
/// - `CounterVTable`: Vtable struct with function pointers
/// - `CounterView`: Runtime view (data + vtable)
/// - `__counter_build_vtable::<T>()`: Vtable builder
/// - Method shims for each method
#[miniextendr]
pub trait Counter {
    /// Get the current value.
    fn value(&self) -> i32;

    /// Increment the counter by 1.
    fn increment(&mut self);

    /// Add n to the counter.
    fn add(&mut self, n: i32);
}

// =============================================================================
// Implement the trait for a concrete type
// =============================================================================

/// A simple counter implementation.
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(traits = [Counter])]
pub struct SimpleCounter {
    value: i32,
}

impl SimpleCounter {
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }
}

/// Implement Counter for SimpleCounter.
///
/// The `#[miniextendr]` on trait impl generates:
/// - `__VTABLE_COUNTER_FOR_SIMPLECOUNTER`: Static vtable constant
#[miniextendr]
impl Counter for SimpleCounter {
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }
}

// =============================================================================
// R-exposed functions for testing (using impl block pattern)
// =============================================================================

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
        Counter::add(self, n);
    }
}

// =============================================================================
// Test panic handling in shims - separate struct
// =============================================================================

/// A counter that panics when you try to decrement below zero.
#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(traits = [Counter])]
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
    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        if self.value + n < 0 {
            panic!(
                "PanickyCounter: cannot go below zero! (value={}, n={})",
                self.value, n
            );
        }
        self.value += n;
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
        Counter::add(self, n);
    }
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod trait_abi_tests;

    impl SimpleCounter;
    impl PanickyCounter;
}
