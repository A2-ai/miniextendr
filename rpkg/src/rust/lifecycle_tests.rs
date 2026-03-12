//! Tests for lifecycle support integration.
//!
//! Tests `#[deprecated]` and `#[miniextendr(lifecycle = "...")]` attributes.

use miniextendr_api::miniextendr;

// =============================================================================
// Tests using #[deprecated] attribute
// =============================================================================

/// A deprecated function using Rust's built-in deprecated attribute.
/// This should generate lifecycle::deprecate_warn in the R wrapper.
/// @noRd
#[deprecated(since = "0.5.0", note = "Use new_api() instead")]
#[miniextendr]
pub fn old_deprecated_fn(x: i32) -> i32 {
    x * 2
}

/// A deprecated function without replacement info.
/// @noRd
#[deprecated]
#[miniextendr]
pub fn also_deprecated() -> String {
    "deprecated".to_string()
}

// =============================================================================
// Tests using #[miniextendr(lifecycle = "...")]
// =============================================================================

/// An experimental function that may change.
/// @noRd
#[miniextendr(lifecycle = "experimental")]
pub fn experimental_feature(x: f64) -> f64 {
    x.powf(2.0)
}

/// A superseded function with a better alternative.
/// @noRd
#[miniextendr(lifecycle = "superseded")]
pub fn superseded_fn(x: i32) -> i32 {
    x + 1
}

/// A soft-deprecated function (warning on first use only).
/// @noRd
#[miniextendr(lifecycle = "soft-deprecated")]
pub fn soft_deprecated_fn(x: i32) -> i32 {
    x - 1
}

// =============================================================================
// Tests using full lifecycle(...) syntax
// =============================================================================

/// A deprecated function with full lifecycle spec.
/// @noRd
#[miniextendr(lifecycle(stage = "deprecated", when = "1.0.0", with = "better_fn()"))]
pub fn fully_deprecated(x: i32) -> i32 {
    x
}

/// A defunct function that should error.
/// @noRd
#[miniextendr(lifecycle(
    stage = "defunct",
    when = "2.0.0",
    details = "This function has been removed"
))]
pub fn defunct_fn(_x: i32) -> i32 {
    panic!("This should never be called")
}

// =============================================================================
// Method-level lifecycle tests (R6)
// =============================================================================

/// An R6 class demonstrating lifecycle on methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct LifecycleDemo {
    value: i32,
}

/// @noRd
#[miniextendr(r6)]
impl LifecycleDemo {
    /// Creates a new LifecycleDemo.
    pub fn new(value: i32) -> Self {
        LifecycleDemo { value }
    }

    /// Returns the value.
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// A deprecated method using lifecycle attribute.
    #[miniextendr(lifecycle(
        stage = "deprecated",
        when = "0.3.0",
        with = "LifecycleDemo$get_value()"
    ))]
    pub fn old_value(&self) -> i32 {
        self.value
    }

    /// An experimental method.
    #[miniextendr(lifecycle = "experimental")]
    pub fn experimental_method(&self) -> i32 {
        self.value * 2
    }

    /// A method deprecated via Rust's #[deprecated] attribute.
    #[deprecated(since = "0.2.0", note = "Use get_value() instead")]
    pub fn legacy_value(&self) -> i32 {
        self.value
    }
}

// =============================================================================
// Module registration
// =============================================================================
