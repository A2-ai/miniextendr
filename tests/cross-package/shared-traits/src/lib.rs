//! Shared trait definitions for cross-package trait ABI tests.
//!
//! This crate defines traits that are implemented by types in different test
//! packages and dispatched across package boundaries.
//!
//! The `#[miniextendr]` macro on traits generates ABI-compatible items:
//! - `TAG_*` - type tag constants (stable hash for ABI compatibility)
//! - `*VTable` - vtable structs with function pointers
//! - `*View` - view types for dynamic dispatch
//!
//! Both producer and consumer test packages depend on this crate so they share
//! the same generated tags and vtables, ensuring ABI compatibility.

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
