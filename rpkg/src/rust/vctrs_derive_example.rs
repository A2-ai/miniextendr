//! Example: Using `#[derive(Vctrs)]` for simpler vctrs class creation.
//!
//! This module demonstrates the derive macro approach as an alternative to
//! the manual implementation in `vctrs_class_example.rs`.
//!
//! # Comparison
//!
//! Manual approach (vctrs_class_example.rs):
//! - More control over every method
//! - More code to write
//! - Full flexibility for custom coercion logic
//!
//! Derive approach (this file):
//! - Automatic `VctrsClass` and `IntoVctrs` trait implementations
//! - Less boilerplate
//! - Suitable for standard vctrs patterns
//!
//! # Usage
//!
//! ```rust
//! #[derive(Vctrs)]
//! #[vctrs(class = "percent", base = "double", abbr = "%")]
//! pub struct Percent {
//!     #[vctrs(data)]
//!     values: Vec<f64>,
//! }
//! ```

use miniextendr_api::vctrs::{IntoVctrs, VctrsClass, VctrsKind};
use miniextendr_api::{miniextendr, miniextendr_module, Vctrs};

// =============================================================================
// Simple vctr: Percent backed by doubles
// =============================================================================

/// A percentage type backed by doubles.
///
/// The derive macro generates:
/// - `VctrsClass` trait with class metadata
/// - `IntoVctrs` trait for conversion to R vctrs object
#[derive(Vctrs)]
#[vctrs(class = "derived_percent", base = "double", abbr = "%")]
pub struct DerivedPercent {
    /// The underlying percentage values (as proportions, e.g., 0.5 = 50%)
    #[vctrs(data)]
    values: Vec<f64>,
}

impl DerivedPercent {
    /// Create a new Percent from a vector of proportions.
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Create a new derived_percent vector using the derive macro.
///
/// This demonstrates the simpler derive-based approach.
///
/// @param x Numeric values (as proportions).
/// @return A derived_percent vector.
#[miniextendr]
pub fn new_derived_percent(x: Vec<f64>) -> Result<miniextendr_api::ffi::SEXP, String> {
    let percent = DerivedPercent::new(x);
    percent.into_vctrs().map_err(|e| e.to_string())
}

/// Verify VctrsClass trait constants.
#[miniextendr]
pub fn derived_percent_class_info() -> Vec<String> {
    vec![
        format!("CLASS_NAME: {}", DerivedPercent::CLASS_NAME),
        format!("KIND: {:?}", DerivedPercent::KIND),
        format!("INHERIT_BASE_TYPE: {}", DerivedPercent::INHERIT_BASE_TYPE),
        format!("ABBR: {:?}", DerivedPercent::ABBR),
    ]
}

// =============================================================================
// Record type: Rational numbers
// =============================================================================

/// A rational number type (numerator/denominator) as a vctrs record.
///
/// Record types store multiple parallel fields of equal length.
/// Each "element" is a row across all fields.
#[derive(Vctrs)]
#[vctrs(class = "derived_rational", base = "record")]
pub struct DerivedRational {
    /// Numerators
    #[vctrs(data)]
    n: Vec<i32>,
    /// Denominators
    d: Vec<i32>,
}

impl DerivedRational {
    /// Create a new Rational from parallel vectors.
    pub fn new(n: Vec<i32>, d: Vec<i32>) -> Result<Self, String> {
        if n.len() != d.len() {
            return Err("n and d must have the same length".to_string());
        }
        Ok(Self { n, d })
    }
}

/// Create a new derived_rational vector.
///
/// @param n Numerator values.
/// @param d Denominator values.
/// @return A derived_rational record vector.
#[miniextendr]
pub fn new_derived_rational(n: Vec<i32>, d: Vec<i32>) -> Result<miniextendr_api::ffi::SEXP, String> {
    let rational = DerivedRational::new(n, d)?;
    rational.into_vctrs().map_err(|e| e.to_string())
}

/// Verify VctrsClass trait constants for rational.
#[miniextendr]
pub fn derived_rational_class_info() -> Vec<String> {
    vec![
        format!("CLASS_NAME: {}", DerivedRational::CLASS_NAME),
        format!("KIND: {:?}", DerivedRational::KIND),
        format!("INHERIT_BASE_TYPE: {}", DerivedRational::INHERIT_BASE_TYPE),
    ]
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod vctrs_derive_example;
    fn new_derived_percent;
    fn derived_percent_class_info;
    fn new_derived_rational;
    fn derived_rational_class_info;
}
