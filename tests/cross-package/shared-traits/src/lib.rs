//! Shared trait definitions for cross-package testing.
//!
//! This crate defines traits that can be implemented by types in different
//! R packages and dispatched across package boundaries.
//!
//! # Cross-Package Trait Dispatch
//!
//! The `#[miniextendr]` macro on traits generates ABI-compatible infrastructure:
//! - `TAG_*` - Type tag constant (128-bit hash for ABI compatibility)
//! - `*VTable` - Vtable struct with function pointers
//! - `*View` - View struct for trait dispatch
//!
//! By depending on this crate, both producer and consumer packages use the
//! same generated items, ensuring ABI compatibility across package boundaries.

use miniextendr_api::miniextendr;

/// A counter trait for demonstrating cross-package trait dispatch.
///
/// This trait is implemented by:
/// - `SimpleCounter` in producer.pkg (increments by 1)
/// - `DoubleCounter` in consumer.pkg (increments by 2)
///
/// Both implementations can be used via generic functions in either package.
#[miniextendr]
pub trait Counter {
    /// Get the current counter value.
    fn value(&self) -> i32;

    /// Increment the counter by its default step.
    fn increment(&mut self);

    /// Add a specific value to the counter.
    fn add(&mut self, n: i32);
}
