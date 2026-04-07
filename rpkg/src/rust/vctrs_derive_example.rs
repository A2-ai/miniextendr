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

use miniextendr_api::vctrs::{IntoVctrs, VctrsClass};
use miniextendr_api::{Vctrs, miniextendr};

// region: Simple vctr: Percent backed by doubles

/// A percentage type backed by doubles.
///
/// The derive macro generates:
/// - `VctrsClass` trait with class metadata
/// - `IntoVctrs` trait for conversion to R vctrs object
/// - R S3 methods for vctrs compatibility (format, vec_proxy, vec_restore, etc.)
/// - Coercion methods for double type (vec_ptype2, vec_cast)
#[derive(Vctrs)]
#[vctrs(
    class = "derived_percent",
    base = "double",
    abbr = "%",
    coerce = "double"
)]
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
///
/// @export
#[miniextendr]
pub fn derived_percent_class_info() -> Vec<String> {
    vec![
        format!("CLASS_NAME: {}", DerivedPercent::CLASS_NAME),
        format!("KIND: {:?}", DerivedPercent::KIND),
        format!("INHERIT_BASE_TYPE: {}", DerivedPercent::INHERIT_BASE_TYPE),
        format!("ABBR: {:?}", DerivedPercent::ABBR),
    ]
}
// endregion

// region: Record type: Rational numbers

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
pub fn new_derived_rational(
    n: Vec<i32>,
    d: Vec<i32>,
) -> Result<miniextendr_api::ffi::SEXP, String> {
    let rational = DerivedRational::new(n, d)?;
    rational.into_vctrs().map_err(|e| e.to_string())
}

/// Verify VctrsClass trait constants for rational.
///
/// @export
#[miniextendr]
pub fn derived_rational_class_info() -> Vec<String> {
    vec![
        format!("CLASS_NAME: {}", DerivedRational::CLASS_NAME),
        format!("KIND: {:?}", DerivedRational::KIND),
        format!("INHERIT_BASE_TYPE: {}", DerivedRational::INHERIT_BASE_TYPE),
    ]
}
// endregion

// region: List-of type: IntegerLists (list of integer vectors)

/// A list of integer vectors as a vctrs list_of type.
///
/// list_of types ensure all elements have a consistent prototype.
#[derive(Vctrs)]
#[vctrs(class = "derived_int_lists", base = "list", ptype = "integer()")]
pub struct DerivedIntLists {
    #[vctrs(data)]
    lists: Vec<Vec<i32>>,
}

impl DerivedIntLists {
    pub fn new(lists: Vec<Vec<i32>>) -> Self {
        Self { lists }
    }
}

/// Create a new derived_int_lists vector (list_of<integer>).
///
/// @param x A list of integer vectors.
/// @return A derived_int_lists list_of vector.
#[miniextendr]
pub fn new_derived_int_lists(
    x: miniextendr_api::ffi::SEXP,
) -> Result<miniextendr_api::ffi::SEXP, String> {
    use miniextendr_api::from_r::TryFromSexp;
    let lists: Vec<Vec<i32>> = TryFromSexp::try_from_sexp(x).map_err(|e| format!("{:?}", e))?;
    let int_lists = DerivedIntLists::new(lists);
    int_lists.into_vctrs().map_err(|e| e.to_string())
}
// endregion

// region: Type with proxy methods: ComparablePoint

/// A 2D point with custom equality and comparison behavior.
///
/// Demonstrates `proxy_equal`, `proxy_compare`, and `proxy_order`.
#[derive(Vctrs)]
#[vctrs(
    class = "derived_point",
    base = "record",
    proxy_equal,
    proxy_compare,
    proxy_order
)]
pub struct DerivedPoint {
    #[vctrs(data)]
    x: Vec<f64>,
    y: Vec<f64>,
}

impl DerivedPoint {
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Result<Self, String> {
        if x.len() != y.len() {
            return Err("x and y must have the same length".to_string());
        }
        Ok(Self { x, y })
    }
}

/// Create a new derived_point record vector.
///
/// @param x X coordinates.
/// @param y Y coordinates.
/// @return A derived_point record vector with proxy methods.
#[miniextendr]
pub fn new_derived_point(x: Vec<f64>, y: Vec<f64>) -> Result<miniextendr_api::ffi::SEXP, String> {
    let point = DerivedPoint::new(x, y)?;
    point.into_vctrs().map_err(|e| e.to_string())
}
// endregion

// region: Type with arithmetic: Temperature

/// A temperature type with arithmetic and math support.
///
/// Demonstrates `arith` and `math` attributes for numeric vctrs.
#[derive(Vctrs)]
#[vctrs(class = "derived_temp", base = "double", abbr = "deg", arith, math)]
pub struct DerivedTemp {
    #[vctrs(data)]
    values: Vec<f64>,
}

impl DerivedTemp {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }
}

/// Create a new derived_temp vector with arithmetic/math support.
///
/// @param x Temperature values.
/// @return A derived_temp vector.
#[miniextendr]
pub fn new_derived_temp(x: Vec<f64>) -> Result<miniextendr_api::ffi::SEXP, String> {
    let temp = DerivedTemp::new(x);
    temp.into_vctrs().map_err(|e| e.to_string())
}
// endregion

// region: Vctrs impl block with protocol method override: Currency

/// A currency type demonstrating Rust-backed vctrs protocol methods.
#[derive(miniextendr_api::ExternalPtr)]
pub struct DerivedCurrency {
    symbol: String,
    amounts: Vec<f64>,
}

/// Vctrs currency class demonstrating Rust-backed protocol method overrides.
/// @param symbol Character currency symbol.
/// @param amounts Numeric vector of currency amounts.
#[miniextendr(vctrs(kind = "vctr", base = "double", abbr = "$"))]
impl DerivedCurrency {
    /// Creates a new currency value.
    pub fn new(symbol: String, amounts: Vec<f64>) -> Self {
        DerivedCurrency { symbol, amounts }
    }

    /// Returns the currency symbol.
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    /// Rust-backed format method override.
    #[miniextendr(vctrs(format))]
    pub fn format_currency(&self) -> Vec<String> {
        self.amounts
            .iter()
            .map(|a| format!("{}{:.2}", self.symbol, a))
            .collect()
    }
}
// endregion

// region: Module registration
// endregion
