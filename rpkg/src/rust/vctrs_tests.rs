//! Test functions for vctrs API support.
//!
//! These functions are exposed to R for testing the vctrs integration.

use miniextendr_api::ffi::SEXP;
use miniextendr_api::list::List;
use miniextendr_api::vctrs::{
    R_len_t, VctrsBuildError, VctrsError, is_vctrs_initialized, new_list_of, new_rcrd, new_vctr,
    obj_is_vector, short_vec_recycle, short_vec_size,
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

// =============================================================================
// Construction helper tests
// =============================================================================

/// Test new_vctr with default inherit_base_type.
///
/// Creates a vctr with the given class name(s) from data.
/// Uses default inherit_base_type (true for lists, false otherwise).
#[miniextendr]
fn test_new_vctr(data: SEXP, class: Vec<String>) -> Result<SEXP, String> {
    let class_refs: Vec<&str> = class.iter().map(|s| s.as_str()).collect();
    new_vctr(data, &class_refs, &[], None).map_err(|e| e.to_string())
}

/// Test new_vctr with explicit inherit_base_type.
#[miniextendr]
fn test_new_vctr_inherit(
    data: SEXP,
    class: Vec<String>,
    inherit_base_type: bool,
) -> Result<SEXP, String> {
    let class_refs: Vec<&str> = class.iter().map(|s| s.as_str()).collect();
    new_vctr(data, &class_refs, &[], Some(inherit_base_type)).map_err(|e| e.to_string())
}

/// Test new_rcrd with the given field list and class name(s).
#[miniextendr]
fn test_new_rcrd(fields: SEXP, class: Vec<String>) -> Result<SEXP, String> {
    let fields = unsafe { List::from_raw(fields) };
    let class_refs: Vec<&str> = class.iter().map(|s| s.as_str()).collect();
    new_rcrd(fields, &class_refs, &[]).map_err(|e| e.to_string())
}

/// Test new_list_of with a ptype.
#[miniextendr]
fn test_new_list_of_ptype(x: SEXP, ptype: SEXP, class: Vec<String>) -> Result<SEXP, String> {
    let x = unsafe { List::from_raw(x) };
    let class_refs: Vec<&str> = class.iter().map(|s| s.as_str()).collect();
    new_list_of(x, Some(ptype), None, &class_refs, &[]).map_err(|e| e.to_string())
}

/// Test new_list_of with a size.
#[miniextendr]
fn test_new_list_of_size(x: SEXP, size: i32, class: Vec<String>) -> Result<SEXP, String> {
    let x = unsafe { List::from_raw(x) };
    let class_refs: Vec<&str> = class.iter().map(|s| s.as_str()).collect();
    new_list_of(x, None, Some(size), &class_refs, &[]).map_err(|e| e.to_string())
}

/// Get the VctrsBuildError message for a specific error type.
#[miniextendr]
fn test_vctrs_build_error_message(error_type: &str) -> String {
    let err: VctrsBuildError = match error_type {
        "not_initialized" => VctrsBuildError::VctrsNotInitialized,
        "not_a_vector" => VctrsBuildError::NotAVector,
        "list_requires_inherit" => VctrsBuildError::ListRequiresInheritBaseType,
        "field_length_mismatch" => VctrsBuildError::FieldLengthMismatch {
            field: "x".to_string(),
            expected: 3,
            actual: 5,
        },
        "empty_record" => VctrsBuildError::EmptyRecord,
        "duplicate_field" => VctrsBuildError::DuplicateFieldName {
            name: "x".to_string(),
        },
        "unnamed_fields" => VctrsBuildError::UnnamedFields,
        "missing_ptype_or_size" => VctrsBuildError::MissingPtypeOrSize,
        "invalid_size" => VctrsBuildError::InvalidSize { size: -1 },
        "empty_class" => VctrsBuildError::EmptyClass,
        _ => return "unknown error type".to_string(),
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
    // Construction helper tests
    fn test_new_vctr;
    fn test_new_vctr_inherit;
    fn test_new_rcrd;
    fn test_new_list_of_ptype;
    fn test_new_list_of_size;
    fn test_vctrs_build_error_message;
}
