//! Trait ABI tests for cross-package trait dispatch.
//!
//! This module tests `#[miniextendr]` on traits and trait implementations,
//! plus `miniextendr_module! { impl Trait for Type; }` wiring.

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
    /// Associated constant: Maximum value for this counter type.
    const MAX_VALUE: i32;

    /// Get the current value.
    fn value(&self) -> i32;

    /// Increment the counter by 1.
    fn increment(&mut self);

    /// Add n to the counter.
    fn add(&mut self, n: i32);

    /// Static method: Returns the default initial value for this counter type.
    fn default_initial() -> i32;
}

// =============================================================================
// Implement the trait for a concrete type
// =============================================================================

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

/// Implement Counter for SimpleCounter.
///
/// The `#[miniextendr]` on trait impl generates:
/// - `__VTABLE_COUNTER_FOR_SIMPLECOUNTER`: Static vtable constant
#[miniextendr]
impl Counter for SimpleCounter {
    const MAX_VALUE: i32 = i32::MAX;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        0 // SimpleCounter defaults to 0
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

    fn add(&mut self, n: i32) {
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
        Counter::add(self, n);
    }
}

// =============================================================================
// S3 trait impl test - uses S3 dispatch for trait methods
// =============================================================================

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

/// S3 trait implementation - generates S3 generics + methods.
///
/// This generates:
/// - `value(x)` S3 generic (if not exists)
/// - `value.S3TraitCounter` S3 method
/// - etc. for each instance method
#[miniextendr(s3)]
impl Counter for S3TraitCounter {
    const MAX_VALUE: i32 = 500;

    fn value(&self) -> i32 {
        self.value
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn add(&mut self, n: i32) {
        self.value += n;
    }

    fn default_initial() -> i32 {
        50 // S3TraitCounter defaults to 50
    }
}

#[miniextendr]
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

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod trait_abi_tests;

    impl SimpleCounter;
    impl PanickyCounter;
    impl S3TraitCounter;

    // Register trait implementations for cross-package dispatch
    // The class system is determined by the #[miniextendr(s3)] on the impl block
    impl Counter for SimpleCounter;
    impl Counter for PanickyCounter;
    impl Counter for S3TraitCounter;
}
