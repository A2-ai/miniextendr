//! Tests for `Missing<T>` -- optional R argument handling.

use miniextendr_api::{Missing, miniextendr};

/// Test Missing<f64> -- returns "absent" or the value as string.
///
/// @param x An optional numeric value.
/// @export
#[miniextendr]
pub fn missing_test_f64(x: Missing<f64>) -> String {
    match x {
        Missing::Present(v) => format!("{v}"),
        Missing::Absent => "absent".to_string(),
    }
}

/// Test Missing<String> with the helper methods.
///
/// @param x An optional string value.
/// @export
#[miniextendr]
pub fn missing_test_string(x: Missing<String>) -> String {
    x.unwrap_or_else(|| "default_value".to_string())
}

/// Test Missing<i32> with is_present / is_absent.
///
/// @param x An optional integer value.
/// @export
#[miniextendr]
pub fn missing_test_present(x: Missing<i32>) -> bool {
    x.is_present()
}

/// Test Missing<Option<f64>> -- distinguishes missing, NULL, and present.
///
/// @param x An optional nullable numeric.
/// @export
#[miniextendr]
pub fn missing_test_option(x: Missing<Option<f64>>) -> String {
    match x {
        Missing::Absent => "missing".to_string(),
        Missing::Present(None) => "null".to_string(),
        Missing::Present(Some(v)) => format!("{v}"),
    }
}
