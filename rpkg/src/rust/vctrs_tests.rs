//! Test functions for vctrs API support.
//!
//! These functions are exposed to R for testing the vctrs integration.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::vctrs::{
    R_len_t, VctrsError, is_vctrs_initialized, obj_is_vector, short_vec_recycle, short_vec_size,
};
use miniextendr_api::{miniextendr, miniextendr_module};

/// Check if vctrs support has been initialized.
#[miniextendr]
fn test_vctrs_is_initialized() -> bool {
    is_vctrs_initialized()
}

/// Test obj_is_vector on an R object.
///
/// Returns true if the object is a vector according to vctrs.
/// Returns an error message if vctrs is not initialized.
#[miniextendr]
fn test_vctrs_obj_is_vector(x: SEXP) -> Result<bool, String> {
    obj_is_vector(x).map_err(|e| e.to_string())
}

/// Test short_vec_size on an R object.
///
/// Returns the size of the vector according to vctrs.
/// Returns an error message if vctrs is not initialized or if x is not a vector.
#[miniextendr]
fn test_vctrs_short_vec_size(x: SEXP) -> Result<i32, String> {
    short_vec_size(x)
        .map(|n| n as i32)
        .map_err(|e| e.to_string())
}

/// Test short_vec_recycle on an R object.
///
/// Recycles the vector to the specified size.
/// Returns an error message if vctrs is not initialized or if recycling fails.
#[miniextendr]
fn test_vctrs_short_vec_recycle(x: SEXP, size: i32) -> Result<SEXP, String> {
    short_vec_recycle(x, size as R_len_t).map_err(|e| e.to_string())
}

/// Get the vctrs error message for a specific error code.
#[miniextendr]
fn test_vctrs_error_message(code: i32) -> String {
    let err = match code {
        0 => VctrsError::NotInitialized,
        1 => VctrsError::NotAvailable { name: "test" },
        2 => VctrsError::AlreadyInitialized,
        3 => VctrsError::NotMainThread,
        _ => return "unknown error code".to_string(),
    };
    err.to_string()
}

miniextendr_module! {
    mod vctrs_tests;
    fn test_vctrs_is_initialized;
    fn test_vctrs_obj_is_vector;
    fn test_vctrs_short_vec_size;
    fn test_vctrs_short_vec_recycle;
    fn test_vctrs_error_message;
}
