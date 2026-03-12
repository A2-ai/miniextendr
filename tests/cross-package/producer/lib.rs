// Producer package: Creates Counter objects and exposes them to R
//
// This demonstrates cross-package trait dispatch:
// - Defines the Counter trait with #[miniextendr]
// - Implements Counter for SimpleCounter
// - Objects can be passed to consumer package

use miniextendr_api::{miniextendr, ExternalPtr};

// ============================================================================
// Shared trait definition
// ============================================================================

#[miniextendr]
pub trait Counter {
    fn value(&self) -> i32;
    fn increment(&mut self);
    fn add(&mut self, n: i32);
}

// ============================================================================
// SimpleCounter implementation
// ============================================================================

#[derive(ExternalPtr)]
pub struct SimpleCounter {
    value: i32,
}

#[miniextendr]
impl SimpleCounter {
    /// Create a new counter
    fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    /// Get current value (inherent method)
    fn get_value(&self) -> i32 {
        self.value
    }
}

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
