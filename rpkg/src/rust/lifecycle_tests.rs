//! Tests for lifecycle support integration.
//!
//! Tests `#[deprecated]` and `#[miniextendr(lifecycle = "...")]` attributes.

use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Tests using #[deprecated] attribute
// =============================================================================

/// A deprecated function using Rust's built-in deprecated attribute.
/// This should generate lifecycle::deprecate_warn in the R wrapper.
#[deprecated(since = "0.5.0", note = "Use new_api() instead")]
#[miniextendr]
pub fn old_deprecated_fn(x: i32) -> i32 {
    x * 2
}

/// A deprecated function without replacement info.
#[deprecated]
#[miniextendr]
pub fn also_deprecated() -> String {
    "deprecated".to_string()
}

// =============================================================================
// Tests using #[miniextendr(lifecycle = "...")]
// =============================================================================

/// An experimental function that may change.
#[miniextendr(lifecycle = "experimental")]
pub fn experimental_feature(x: f64) -> f64 {
    x.powf(2.0)
}

/// A superseded function with a better alternative.
#[miniextendr(lifecycle = "superseded")]
pub fn superseded_fn(x: i32) -> i32 {
    x + 1
}

/// A soft-deprecated function (warning on first use only).
#[miniextendr(lifecycle = "soft-deprecated")]
pub fn soft_deprecated_fn(x: i32) -> i32 {
    x - 1
}

// =============================================================================
// Tests using full lifecycle(...) syntax
// =============================================================================

/// A deprecated function with full lifecycle spec.
#[miniextendr(lifecycle(stage = "deprecated", when = "1.0.0", with = "better_fn()"))]
pub fn fully_deprecated(x: i32) -> i32 {
    x
}

/// A defunct function that should error.
#[miniextendr(lifecycle(stage = "defunct", when = "2.0.0", details = "This function has been removed"))]
pub fn defunct_fn(_x: i32) -> i32 {
    panic!("This should never be called")
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod lifecycle_tests;
    fn old_deprecated_fn;
    fn also_deprecated;
    fn experimental_feature;
    fn superseded_fn;
    fn soft_deprecated_fn;
    fn fully_deprecated;
    fn defunct_fn;
}
