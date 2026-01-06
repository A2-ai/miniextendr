//! Example: Implementing a vctrs-compatible S3 class entirely in Rust.
//!
//! This module demonstrates how to use the `#[miniextendr(s3(...))]` attribute
//! to implement S3 methods for vctrs classes like `vec_proxy`, `vec_restore`,
//! `vec_ptype2`, `vec_cast`, and `vec_ptype_abbr`.
//!
//! The example implements a `percent` class that stores numeric values as percentages.
//!
//! # Note on Coercion Behavior
//!
//! This example uses "percent wins" coercion: `percent + double = percent`.
//! The vctrs documentation example uses "double wins" (`percent + double = double`).
//! Both are valid design choices - we chose to preserve the specialized type.

use miniextendr_api::ffi::{
    R_ClassSymbol, R_NilValue, Rf_allocVector, Rf_duplicate, Rf_getAttrib, Rf_setAttrib,
    SEXPTYPE, SEXP, SexpExt,
};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::vctrs::new_vctr;
use miniextendr_api::{miniextendr, miniextendr_module};

// =============================================================================
// Constructor
// =============================================================================

/// Create a new percent vector.
///
/// @param x Numeric values (as proportions, e.g., 0.5 for 50%).
/// @param ... Additional arguments (ignored, for vctrs compatibility).
/// @return A percent vector.
#[miniextendr]
pub fn new_percent(x: SEXP, _dots: ...) -> Result<SEXP, String> {
    // Validate input is numeric
    if !x.is_real() && !x.is_integer() {
        return Err("x must be numeric".to_string());
    }

    // Create the vctrs vctr with "percent" class
    new_vctr(x, &["percent"], &[], Some(false)).map_err(|e| e.to_string())
}

// =============================================================================
// S3 Methods for vctrs generics
// =============================================================================

/// Print abbreviation for percent vectors.
///
/// Returns "%" to display in tibble headers and other compact contexts.
#[miniextendr(s3(generic = "vec_ptype_abbr", class = "percent"))]
pub fn vec_ptype_abbr_percent(_x: SEXP, _dots: ...) -> &'static str {
    "%"
}

/// Print method for percent vectors.
///
/// Formats values as percentages (e.g., 0.5 -> "50%").
#[miniextendr(s3(generic = "format", class = "percent"))]
pub fn format_percent(x: SEXP, _dots: ...) -> Result<Vec<String>, String> {
    // Get the underlying numeric data
    if !x.is_real() {
        return Err("percent must contain numeric data".to_string());
    }

    let data: &[f64] = unsafe { x.as_slice::<f64>() };

    // Format as percentages
    let formatted: Vec<String> = data
        .iter()
        .map(|v| {
            if v.is_nan() {
                "NA%".to_string()
            } else {
                format!("{:.1}%", v * 100.0)
            }
        })
        .collect();

    Ok(formatted)
}

/// Get the proxy for subsetting operations.
///
/// For percent vectors, the proxy is just the underlying numeric data
/// without the class attribute.
#[miniextendr(s3(generic = "vec_proxy", class = "percent"))]
pub fn vec_proxy_percent(x: SEXP, _dots: ...) -> SEXP {
    // Return x without class attribute (strip vctrs class for operations)
    let class = unsafe { Rf_getAttrib(x, R_ClassSymbol) };
    if class != unsafe { R_NilValue } {
        // Duplicate to avoid modifying original, with GC protection
        let out = unsafe { OwnedProtect::new(Rf_duplicate(x)) };
        unsafe { Rf_setAttrib(out.get(), R_ClassSymbol, R_NilValue) };
        // OwnedProtect drops here, calling UNPROTECT(1). This is safe because
        // R captures the return value before any GC can run.
        out.get()
    } else {
        x
    }
}

/// Restore from proxy after subsetting.
///
/// Reconstructs a percent vector from the proxy data.
#[miniextendr(s3(generic = "vec_restore", class = "percent"))]
pub fn vec_restore_percent(x: SEXP, _to: SEXP, _dots: ...) -> Result<SEXP, String> {
    // Restore the percent class
    new_vctr(x, &["percent"], &[], Some(false)).map_err(|e| e.to_string())
}

/// Self-coercion prototype (percent + percent = percent).
///
/// Returns an empty percent prototype when combining two percent vectors.
#[miniextendr(s3(generic = "vec_ptype2", class = "percent.percent"))]
pub fn vec_ptype2_percent_percent(_x: SEXP, _y: SEXP, _dots: ...) -> Result<SEXP, String> {
    // Create empty prototype with GC protection
    let empty = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::REALSXP, 0)) };
    new_vctr(empty.get(), &["percent"], &[], Some(false)).map_err(|e| e.to_string())
}

/// Self-cast (percent -> percent is identity).
#[miniextendr(s3(generic = "vec_cast", class = "percent.percent"))]
pub fn vec_cast_percent_percent(x: SEXP, _to: SEXP, _dots: ...) -> SEXP {
    x
}

/// Cast from double to percent.
#[miniextendr(s3(generic = "vec_cast", class = "percent.double"))]
pub fn vec_cast_percent_double(x: SEXP, _to: SEXP, _dots: ...) -> Result<SEXP, String> {
    if !x.is_real() {
        return Err("expected numeric input".to_string());
    }
    new_vctr(x, &["percent"], &[], Some(false)).map_err(|e| e.to_string())
}

/// Coercion: double + percent = percent.
#[miniextendr(s3(generic = "vec_ptype2", class = "percent.double"))]
pub fn vec_ptype2_percent_double(_x: SEXP, _y: SEXP, _dots: ...) -> Result<SEXP, String> {
    let empty = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::REALSXP, 0)) };
    new_vctr(empty.get(), &["percent"], &[], Some(false)).map_err(|e| e.to_string())
}

/// Coercion: double + percent = percent (symmetric).
#[miniextendr(s3(generic = "vec_ptype2", class = "double.percent"))]
pub fn vec_ptype2_double_percent(_x: SEXP, _y: SEXP, _dots: ...) -> Result<SEXP, String> {
    let empty = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::REALSXP, 0)) };
    new_vctr(empty.get(), &["percent"], &[], Some(false)).map_err(|e| e.to_string())
}

/// Cast from percent to double.
#[miniextendr(s3(generic = "vec_cast", class = "double.percent"))]
pub fn vec_cast_double_percent(x: SEXP, _to: SEXP, _dots: ...) -> SEXP {
    // Strip the class to get raw numeric, with GC protection
    let out = unsafe { OwnedProtect::new(Rf_duplicate(x)) };
    unsafe { Rf_setAttrib(out.get(), R_ClassSymbol, R_NilValue) };
    // OwnedProtect drops here, unprotecting. Safe because R captures return value.
    out.get()
}

// =============================================================================
// Module registration
// =============================================================================

miniextendr_module! {
    mod vctrs_class_example;
    fn new_percent;
    fn vec_ptype_abbr_percent;
    fn format_percent;
    fn vec_proxy_percent;
    fn vec_restore_percent;
    fn vec_ptype2_percent_percent;
    fn vec_cast_percent_percent;
    fn vec_cast_percent_double;
    fn vec_ptype2_percent_double;
    fn vec_ptype2_double_percent;
    fn vec_cast_double_percent;
}
