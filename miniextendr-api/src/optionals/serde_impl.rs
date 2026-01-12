//! Integration with the `serde` crate for JSON serialization.
//!
//! Provides adapter traits for serializing Rust types to JSON and deserializing
//! from JSON strings in R.
//!
//! | Rust Trait | Adapter Trait | Use Case |
//! |------------|---------------|----------|
//! | `serde::Serialize` | `RSerialize` | Convert Rust → JSON string |
//! | `serde::Deserialize` | `RDeserialize` | Parse JSON string → Rust |
//!
//! # Features
//!
//! Enable this module with the `serde` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["serde"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use serde::{Serialize, Deserialize};
//! use miniextendr_api::serde_impl::{RSerialize, RDeserialize};
//!
//! #[derive(Serialize, Deserialize, Clone, ExternalPtr)]
//! struct Config {
//!     name: String,
//!     value: i32,
//!     enabled: bool,
//! }
//!
//! #[miniextendr]
//! impl RSerialize for Config {}
//!
//! #[miniextendr]
//! impl RDeserialize for Config {}
//! ```
//!
//! In R:
//! ```r
//! cfg <- Config$new("test", 42L, TRUE)
//! json <- cfg$r_to_json()
//! # '{"name":"test","value":42,"enabled":true}'
//!
//! cfg2 <- Config$from_json(json)
//! cfg2$name  # "test"
//!
//! # Pretty-printed JSON
//! cfg$r_to_json_pretty()
//! # '{
//! #   "name": "test",
//! #   "value": 42,
//! #   "enabled": true
//! # }'
//! ```
//!
//! # Error Handling
//!
//! - `r_to_json()` returns `Result<String, String>` - errors include serialization failures
//! - `from_json()` returns `Option<Self>` - returns None on parse failure
//! - For more detailed error information, use `from_json_result()`

pub use serde::{Deserialize, Serialize};
pub use serde_json;
pub use serde_json::Value as JsonValue;

use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::ffi::{
    INTEGER_ELT, LOGICAL_ELT, REAL_ELT, Rboolean, Rf_allocVector, Rf_getAttrib, Rf_isFactor,
    Rf_mkCharLenCE, Rf_setAttrib, Rf_xlength, SET_INTEGER_ELT, SET_LOGICAL_ELT, SET_REAL_ELT,
    SET_STRING_ELT, SET_VECTOR_ELT, SEXP, SEXPTYPE, STRING_ELT, SexpExt, cetype_t,
};
use crate::from_r::{SexpError, TryFromSexp, charsxp_to_str};
use crate::gc_protect::OwnedProtect;
use crate::into_r::IntoR;
use crate::{
    impl_option_try_from_sexp, impl_vec_option_try_from_sexp_list, impl_vec_try_from_sexp_list,
};

/// Adapter trait for [`serde::Serialize`].
///
/// Provides JSON serialization for R, allowing Rust types to be converted
/// to JSON strings for storage, transmission, or interop with other tools.
///
/// # Methods
///
/// - `r_to_json()` - Serialize to compact JSON string
/// - `r_to_json_pretty()` - Serialize to pretty-printed JSON
/// - `r_to_json_value()` - Serialize to a JSON value (for inspection)
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Serialize, ExternalPtr)]
/// struct Point { x: f64, y: f64 }
///
/// #[miniextendr]
/// impl RSerialize for Point {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RSerialize for Point;
/// }
/// ```
pub trait RSerialize {
    /// Serialize to a compact JSON string.
    ///
    /// Returns `Ok(json_string)` on success, `Err(error_message)` on failure.
    fn to_json(&self) -> Result<String, String>;

    /// Serialize to a pretty-printed JSON string with indentation.
    ///
    /// Returns `Ok(json_string)` on success, `Err(error_message)` on failure.
    fn to_json_pretty(&self) -> Result<String, String>;
}

impl<T: Serialize> RSerialize for T {
    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    fn to_json_pretty(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}

/// Adapter trait for [`serde::Deserialize`].
///
/// Provides JSON deserialization for R, allowing JSON strings to be parsed
/// into Rust types.
///
/// # Methods
///
/// - `from_json(s)` - Parse JSON string, returning None on failure
/// - `from_json_result(s)` - Parse JSON string with detailed error
///
/// # Example
///
/// ```rust,ignore
/// #[derive(Deserialize, ExternalPtr)]
/// struct Config { name: String, value: i32 }
///
/// #[miniextendr]
/// impl RDeserialize for Config {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDeserialize for Config;
/// }
/// ```
///
/// In R:
/// ```r
/// cfg <- Config$from_json('{"name":"test","value":42}')
/// ```
pub trait RDeserialize: Sized {
    /// Parse a JSON string into this type.
    ///
    /// Returns `Some(value)` on success, `None` on parse failure.
    /// The None case maps to NULL in R.
    fn from_json(s: &str) -> Option<Self>;

    /// Parse a JSON string with detailed error information.
    ///
    /// Returns `Ok(value)` on success, `Err(error_message)` on failure.
    fn from_json_result(s: &str) -> Result<Self, String>;
}

impl<T: for<'de> Deserialize<'de>> RDeserialize for T {
    fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    fn from_json_result(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
}

// =============================================================================
// serde_json::Value <-> R Bridge
// =============================================================================

/// Convert an R object to a JSON value.
///
/// Mapping rules (R -> JSON):
/// - `NULL` -> `Null`
/// - Scalar `LGLSXP` -> `Bool` (NA -> `Null`)
/// - Scalar `INTSXP` -> `Number` (NA -> `Null`)
/// - Scalar `REALSXP` -> `Number` (NA/NaN/Inf -> error)
/// - Scalar `STRSXP` -> `String` (NA -> `Null`)
/// - Vector of length > 1 -> `Array`
/// - Named `VECSXP` -> `Object`
/// - Unnamed `VECSXP` -> `Array`
/// - Factor -> `String` (via levels)
///
/// # Errors
///
/// Returns an error for:
/// - `NaN` or `Inf` values (JSON has no representation)
/// - Unsupported R types (e.g., CLOSXP, ENVSXP)
pub fn json_from_sexp(sexp: SEXP) -> Result<JsonValue, SexpError> {
    sexp_to_json_value(sexp, false, false)
}

/// Convert R to JSON with strict NA handling (errors on NA).
pub fn json_from_sexp_strict(sexp: SEXP) -> Result<JsonValue, SexpError> {
    sexp_to_json_value(sexp, true, false)
}

/// Convert R to JSON with permissive handling (NA/NaN/Inf -> Null).
pub fn json_from_sexp_permissive(sexp: SEXP) -> Result<JsonValue, SexpError> {
    sexp_to_json_value(sexp, false, true)
}

fn sexp_to_json_value(sexp: SEXP, strict: bool, permissive: bool) -> Result<JsonValue, SexpError> {
    let sexp_type = sexp.type_of();

    // Handle NULL
    if sexp_type == SEXPTYPE::NILSXP {
        return Ok(JsonValue::Null);
    }

    // Handle factors first (convert to string)
    if unsafe { Rf_isFactor(sexp) } != Rboolean::FALSE {
        return factor_to_json(sexp, strict, permissive);
    }

    let len = unsafe { Rf_xlength(sexp) } as usize;

    match sexp_type {
        SEXPTYPE::LGLSXP => {
            if len == 1 {
                let val = unsafe { LOGICAL_ELT(sexp, 0) };
                if val == NA_LOGICAL {
                    if strict {
                        return Err(SexpError::InvalidValue(
                            "NA not allowed in strict mode".into(),
                        ));
                    }
                    return Ok(JsonValue::Null);
                }
                Ok(JsonValue::Bool(val != 0))
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let val = unsafe { LOGICAL_ELT(sexp, i as isize) };
                        if val == NA_LOGICAL {
                            if strict {
                                return Err(SexpError::InvalidValue(format!(
                                    "NA at index {} not allowed in strict mode",
                                    i
                                )));
                            }
                            Ok(JsonValue::Null)
                        } else {
                            Ok(JsonValue::Bool(val != 0))
                        }
                    })
                    .collect();
                Ok(JsonValue::Array(arr?))
            }
        }
        SEXPTYPE::INTSXP => {
            if len == 1 {
                let val = unsafe { INTEGER_ELT(sexp, 0) };
                if val == NA_INTEGER {
                    if strict {
                        return Err(SexpError::InvalidValue(
                            "NA not allowed in strict mode".into(),
                        ));
                    }
                    return Ok(JsonValue::Null);
                }
                Ok(JsonValue::Number(serde_json::Number::from(val)))
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let val = unsafe { INTEGER_ELT(sexp, i as isize) };
                        if val == NA_INTEGER {
                            if strict {
                                return Err(SexpError::InvalidValue(format!(
                                    "NA at index {} not allowed in strict mode",
                                    i
                                )));
                            }
                            Ok(JsonValue::Null)
                        } else {
                            Ok(JsonValue::Number(serde_json::Number::from(val)))
                        }
                    })
                    .collect();
                Ok(JsonValue::Array(arr?))
            }
        }
        SEXPTYPE::REALSXP => {
            if len == 1 {
                let val = unsafe { REAL_ELT(sexp, 0) };
                real_to_json(val, strict, permissive)
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let val = unsafe { REAL_ELT(sexp, i as isize) };
                        real_to_json(val, strict, permissive)
                    })
                    .collect();
                Ok(JsonValue::Array(arr?))
            }
        }
        SEXPTYPE::STRSXP => {
            if len == 1 {
                let charsxp = unsafe { STRING_ELT(sexp, 0) };
                if charsxp == unsafe { crate::ffi::R_NaString } {
                    if strict {
                        return Err(SexpError::InvalidValue(
                            "NA not allowed in strict mode".into(),
                        ));
                    }
                    return Ok(JsonValue::Null);
                }
                let s = unsafe { charsxp_to_str(charsxp) };
                Ok(JsonValue::String(s.to_string()))
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let charsxp = unsafe { STRING_ELT(sexp, i as isize) };
                        if charsxp == unsafe { crate::ffi::R_NaString } {
                            if strict {
                                return Err(SexpError::InvalidValue(format!(
                                    "NA at index {} not allowed in strict mode",
                                    i
                                )));
                            }
                            Ok(JsonValue::Null)
                        } else {
                            let s = unsafe { charsxp_to_str(charsxp) };
                            Ok(JsonValue::String(s.to_string()))
                        }
                    })
                    .collect();
                Ok(JsonValue::Array(arr?))
            }
        }
        SEXPTYPE::VECSXP => {
            // Check for names
            let names = unsafe { Rf_getAttrib(sexp, crate::ffi::R_NamesSymbol) };
            let has_names = !names.is_null() && names.type_of() == SEXPTYPE::STRSXP;

            if has_names {
                // Convert to object
                let mut map = serde_json::Map::new();
                for i in 0..len {
                    let charsxp = unsafe { STRING_ELT(names, i as isize) };
                    let key = if charsxp == unsafe { crate::ffi::R_NaString } {
                        format!("V{}", i + 1) // Auto-name for NA keys
                    } else {
                        unsafe { charsxp_to_str(charsxp) }.to_string()
                    };
                    let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as isize) };
                    let val = sexp_to_json_value(elem, strict, permissive)?;
                    map.insert(key, val);
                }
                Ok(JsonValue::Object(map))
            } else {
                // Convert to array
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as isize) };
                        sexp_to_json_value(elem, strict, permissive)
                    })
                    .collect();
                Ok(JsonValue::Array(arr?))
            }
        }
        _ => Err(SexpError::InvalidValue(format!(
            "unsupported R type for JSON conversion: {:?}",
            sexp_type
        ))),
    }
}

fn real_to_json(val: f64, strict: bool, permissive: bool) -> Result<JsonValue, SexpError> {
    // Check for NA (NA_REAL is a specific NaN bit pattern)
    if val.to_bits() == NA_REAL.to_bits() {
        if strict {
            return Err(SexpError::InvalidValue(
                "NA not allowed in strict mode".into(),
            ));
        }
        return Ok(JsonValue::Null);
    }

    // Check for NaN/Inf
    if val.is_nan() {
        if permissive {
            return Ok(JsonValue::Null);
        }
        return Err(SexpError::InvalidValue(
            "NaN cannot be represented in JSON".into(),
        ));
    }
    if val.is_infinite() {
        if permissive {
            return Ok(JsonValue::Null);
        }
        return Err(SexpError::InvalidValue(
            "Infinity cannot be represented in JSON".into(),
        ));
    }

    serde_json::Number::from_f64(val)
        .map(JsonValue::Number)
        .ok_or_else(|| SexpError::InvalidValue("cannot convert f64 to JSON number".into()))
}

fn factor_to_json(sexp: SEXP, strict: bool, _permissive: bool) -> Result<JsonValue, SexpError> {
    let len = unsafe { Rf_xlength(sexp) } as usize;
    let levels = unsafe { Rf_getAttrib(sexp, crate::ffi::R_LevelsSymbol) };

    if len == 1 {
        let idx = unsafe { INTEGER_ELT(sexp, 0) };
        if idx == NA_INTEGER {
            if strict {
                return Err(SexpError::InvalidValue(
                    "NA factor not allowed in strict mode".into(),
                ));
            }
            return Ok(JsonValue::Null);
        }
        // Factor indices are 1-based
        let charsxp = unsafe { STRING_ELT(levels, (idx - 1) as isize) };
        let s = unsafe { charsxp_to_str(charsxp) };
        Ok(JsonValue::String(s.to_string()))
    } else {
        let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
            .map(|i| {
                let idx = unsafe { INTEGER_ELT(sexp, i as isize) };
                if idx == NA_INTEGER {
                    if strict {
                        return Err(SexpError::InvalidValue(format!(
                            "NA factor at index {} not allowed in strict mode",
                            i
                        )));
                    }
                    Ok(JsonValue::Null)
                } else {
                    let charsxp = unsafe { STRING_ELT(levels, (idx - 1) as isize) };
                    let s = unsafe { charsxp_to_str(charsxp) };
                    Ok(JsonValue::String(s.to_string()))
                }
            })
            .collect();
        Ok(JsonValue::Array(arr?))
    }
}

// =============================================================================
// TryFromSexp for JsonValue
// =============================================================================

impl TryFromSexp for JsonValue {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        json_from_sexp(sexp)
    }
}

// =============================================================================
// Option / Vec conversions
// =============================================================================

// Use macros to implement Option/Vec conversions
impl_option_try_from_sexp!(JsonValue);
impl_vec_try_from_sexp_list!(JsonValue);
impl_vec_option_try_from_sexp_list!(JsonValue);

// =============================================================================
// IntoR for JsonValue
// =============================================================================

impl IntoR for JsonValue {
    fn into_sexp(self) -> SEXP {
        json_value_to_sexp(&self)
    }
}

impl IntoR for Option<JsonValue> {
    fn into_sexp(self) -> SEXP {
        match self {
            Some(value) => json_value_to_sexp(&value),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

impl IntoR for Vec<JsonValue> {
    fn into_sexp(self) -> SEXP {
        let len = self.len();
        let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, len as isize)) };
        for (i, value) in self.iter().enumerate() {
            unsafe { SET_VECTOR_ELT(sexp.get(), i as isize, json_value_to_sexp(value)) };
        }
        // Return the SEXP - guard drops and unprotects
        sexp.get()
    }
}

impl IntoR for Vec<Option<JsonValue>> {
    fn into_sexp(self) -> SEXP {
        let len = self.len();
        let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, len as isize)) };
        for (i, value) in self.iter().enumerate() {
            let elem = match value {
                Some(v) => json_value_to_sexp(v),
                None => unsafe { crate::ffi::R_NilValue },
            };
            unsafe { SET_VECTOR_ELT(sexp.get(), i as isize, elem) };
        }
        // Return the SEXP - guard drops and unprotects
        sexp.get()
    }
}

/// Convert a JSON value to an R object.
pub fn json_into_sexp(value: &JsonValue) -> SEXP {
    json_value_to_sexp(value)
}

fn json_value_to_sexp(value: &JsonValue) -> SEXP {
    match value {
        JsonValue::Null => unsafe { crate::ffi::R_NilValue },
        JsonValue::Bool(b) => {
            let sexp = unsafe { Rf_allocVector(SEXPTYPE::LGLSXP, 1) };
            unsafe { SET_LOGICAL_ELT(sexp, 0, if *b { 1 } else { 0 }) };
            sexp
        }
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    let sexp = unsafe { Rf_allocVector(SEXPTYPE::INTSXP, 1) };
                    unsafe { SET_INTEGER_ELT(sexp, 0, i as i32) };
                    return sexp;
                }
            }
            // Fall back to f64
            let f = n.as_f64().unwrap_or(f64::NAN);
            let sexp = unsafe { Rf_allocVector(SEXPTYPE::REALSXP, 1) };
            unsafe { SET_REAL_ELT(sexp, 0, f) };
            sexp
        }
        JsonValue::String(s) => {
            let sexp = unsafe { Rf_allocVector(SEXPTYPE::STRSXP, 1) };
            let charsxp =
                unsafe { Rf_mkCharLenCE(s.as_ptr().cast(), s.len() as i32, cetype_t::CE_UTF8) };
            unsafe { SET_STRING_ELT(sexp, 0, charsxp) };
            sexp
        }
        JsonValue::Array(arr) => {
            let len = arr.len();
            // Protect sexp before recursive calls that may trigger GC
            let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, len as isize)) };
            for (i, elem) in arr.iter().enumerate() {
                unsafe { SET_VECTOR_ELT(sexp.get(), i as isize, json_value_to_sexp(elem)) };
            }
            // Return the SEXP - guard drops and unprotects
            sexp.get()
        }
        JsonValue::Object(map) => {
            let len = map.len();
            // Protect both sexp and names before recursive calls that may trigger GC
            let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, len as isize)) };
            let names =
                unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, len as isize)) };

            for (i, (key, val)) in map.iter().enumerate() {
                let charsxp = unsafe {
                    Rf_mkCharLenCE(key.as_ptr().cast(), key.len() as i32, cetype_t::CE_UTF8)
                };
                unsafe {
                    SET_STRING_ELT(names.get(), i as isize, charsxp);
                    SET_VECTOR_ELT(sexp.get(), i as isize, json_value_to_sexp(val));
                }
            }

            unsafe { Rf_setAttrib(sexp.get(), crate::ffi::R_NamesSymbol, names.get()) };
            // Return the SEXP - guards drop and unprotect
            sexp.get()
        }
    }
}

/// Adapter trait for JSON value inspection.
///
/// # Registration
///
/// ```ignore
/// use miniextendr_api::serde_impl::{JsonValue, RJsonValueOps};
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RJsonValueOps for JsonValue;
/// }
/// ```
pub trait RJsonValueOps {
    /// Check if this is a null value.
    fn is_null(&self) -> bool;
    /// Check if this is a boolean.
    fn is_boolean(&self) -> bool;
    /// Check if this is a number.
    fn is_number(&self) -> bool;
    /// Check if this is a string.
    fn is_string(&self) -> bool;
    /// Check if this is an array.
    fn is_array(&self) -> bool;
    /// Check if this is an object.
    fn is_object(&self) -> bool;
    /// Get the type name.
    fn type_name(&self) -> String;
    /// Serialize to compact JSON string.
    fn to_json_string(&self) -> String;
    /// Serialize to pretty JSON string.
    fn to_json_string_pretty(&self) -> String;
    /// Get as boolean if this is a boolean.
    fn as_bool(&self) -> Option<bool>;
    /// Get as integer if this is an integer.
    fn as_i64(&self) -> Option<i64>;
    /// Get as float if this is a number.
    fn as_f64(&self) -> Option<f64>;
    /// Get as string if this is a string.
    fn as_str(&self) -> Option<String>;
    /// Get array length if this is an array.
    fn array_len(&self) -> Option<i32>;
    /// Get object keys if this is an object.
    fn object_keys(&self) -> Vec<String>;
}

impl RJsonValueOps for JsonValue {
    fn is_null(&self) -> bool {
        JsonValue::is_null(self)
    }
    fn is_boolean(&self) -> bool {
        JsonValue::is_boolean(self)
    }
    fn is_number(&self) -> bool {
        JsonValue::is_number(self)
    }
    fn is_string(&self) -> bool {
        JsonValue::is_string(self)
    }
    fn is_array(&self) -> bool {
        JsonValue::is_array(self)
    }
    fn is_object(&self) -> bool {
        JsonValue::is_object(self)
    }
    fn type_name(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(_) => "boolean".to_string(),
            JsonValue::Number(_) => "number".to_string(),
            JsonValue::String(_) => "string".to_string(),
            JsonValue::Array(_) => "array".to_string(),
            JsonValue::Object(_) => "object".to_string(),
        }
    }
    fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
    fn to_json_string_pretty(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
    fn as_bool(&self) -> Option<bool> {
        JsonValue::as_bool(self)
    }
    fn as_i64(&self) -> Option<i64> {
        JsonValue::as_i64(self)
    }
    fn as_f64(&self) -> Option<f64> {
        JsonValue::as_f64(self)
    }
    fn as_str(&self) -> Option<String> {
        JsonValue::as_str(self).map(|s| s.to_string())
    }
    fn array_len(&self) -> Option<i32> {
        JsonValue::as_array(self).map(|arr| arr.len() as i32)
    }
    fn object_keys(&self) -> Vec<String> {
        JsonValue::as_object(self)
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestStruct {
        name: String,
        value: i32,
        enabled: bool,
    }

    #[test]
    fn rserialize_to_json() {
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };
        let json = data.to_json().unwrap();
        assert_eq!(json, r#"{"name":"test","value":42,"enabled":true}"#);
    }

    #[test]
    fn rserialize_to_json_pretty() {
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };
        let json = data.to_json_pretty().unwrap();
        assert!(json.contains('\n'));
        assert!(json.contains("  ")); // Indentation
        assert!(json.contains("\"name\": \"test\""));
    }

    #[test]
    fn rdeserialize_from_json() {
        let json = r#"{"name":"hello","value":123,"enabled":false}"#;
        let data: Option<TestStruct> = RDeserialize::from_json(json);
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.name, "hello");
        assert_eq!(data.value, 123);
        assert!(!data.enabled);
    }

    #[test]
    fn rdeserialize_from_json_result() {
        let json = r#"{"name":"world","value":456,"enabled":true}"#;
        let result: Result<TestStruct, String> = RDeserialize::from_json_result(json);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "world");
    }

    #[test]
    fn rdeserialize_invalid_json() {
        let invalid = "not valid json";
        let data: Option<TestStruct> = RDeserialize::from_json(invalid);
        assert!(data.is_none());

        let result: Result<TestStruct, String> = RDeserialize::from_json_result(invalid);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected"));
    }

    #[test]
    fn rdeserialize_missing_field() {
        let json = r#"{"name":"test"}"#; // Missing value and enabled
        let data: Option<TestStruct> = RDeserialize::from_json(json);
        assert!(data.is_none());
    }

    #[test]
    fn roundtrip() {
        let original = TestStruct {
            name: "roundtrip".to_string(),
            value: 999,
            enabled: true,
        };
        let json = original.to_json().unwrap();
        let parsed: TestStruct = RDeserialize::from_json(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn serialize_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let json = data.to_json().unwrap();
        assert_eq!(json, "[1,2,3,4,5]");
    }

    #[test]
    fn deserialize_vec() {
        let json = "[10,20,30]";
        let data: Option<Vec<i32>> = RDeserialize::from_json(json);
        assert_eq!(data, Some(vec![10, 20, 30]));
    }

    #[test]
    fn serialize_option() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(some_val.to_json().unwrap(), "42");
        assert_eq!(none_val.to_json().unwrap(), "null");
    }

    #[test]
    fn deserialize_option() {
        let data: Option<Option<i32>> = RDeserialize::from_json("42");
        assert_eq!(data, Some(Some(42)));

        let data: Option<Option<i32>> = RDeserialize::from_json("null");
        assert_eq!(data, Some(None));
    }

    #[test]
    fn serialize_nested() {
        #[derive(Serialize)]
        struct Outer {
            inner: Inner,
        }
        #[derive(Serialize)]
        struct Inner {
            x: i32,
        }

        let data = Outer {
            inner: Inner { x: 100 },
        };
        let json = data.to_json().unwrap();
        assert_eq!(json, r#"{"inner":{"x":100}}"#);
    }

    #[test]
    fn serialize_with_special_chars() {
        let data = TestStruct {
            name: "hello\nworld\t\"quoted\"".to_string(),
            value: 0,
            enabled: false,
        };
        let json = data.to_json().unwrap();
        // Special chars should be escaped
        assert!(json.contains("\\n"));
        assert!(json.contains("\\t"));
        assert!(json.contains("\\\""));
    }

    // =========================================================================
    // JsonValue tests
    // =========================================================================

    #[test]
    fn json_value_adapter_null() {
        let v = JsonValue::Null;
        assert!(RJsonValueOps::is_null(&v));
        assert!(!RJsonValueOps::is_boolean(&v));
        assert_eq!(RJsonValueOps::type_name(&v), "null");
    }

    #[test]
    fn json_value_adapter_bool() {
        let v = JsonValue::Bool(true);
        assert!(RJsonValueOps::is_boolean(&v));
        assert_eq!(RJsonValueOps::as_bool(&v), Some(true));
    }

    #[test]
    fn json_value_adapter_number() {
        let v = serde_json::json!(42);
        assert!(RJsonValueOps::is_number(&v));
        assert_eq!(RJsonValueOps::as_i64(&v), Some(42));
        assert_eq!(RJsonValueOps::as_f64(&v), Some(42.0));

        let v = serde_json::json!(std::f64::consts::PI);
        assert!(RJsonValueOps::is_number(&v));
        assert_eq!(RJsonValueOps::as_f64(&v), Some(std::f64::consts::PI));
    }

    #[test]
    fn json_value_adapter_string() {
        let v = serde_json::json!("hello");
        assert!(RJsonValueOps::is_string(&v));
        assert_eq!(RJsonValueOps::as_str(&v), Some("hello".to_string()));
    }

    #[test]
    fn json_value_adapter_array() {
        let v = serde_json::json!([1, 2, 3]);
        assert!(RJsonValueOps::is_array(&v));
        assert_eq!(RJsonValueOps::array_len(&v), Some(3));
    }

    #[test]
    fn json_value_adapter_object() {
        let v = serde_json::json!({"a": 1, "b": 2});
        assert!(RJsonValueOps::is_object(&v));
        let keys = RJsonValueOps::object_keys(&v);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }

    #[test]
    fn json_value_to_string() {
        let v = serde_json::json!({"x": 1});
        let s = RJsonValueOps::to_json_string(&v);
        assert!(s.contains("\"x\""));
        assert!(s.contains("1"));
    }

    #[test]
    fn real_to_json_normal() {
        let result = real_to_json(std::f64::consts::PI, false, false);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(RJsonValueOps::is_number(&val));
    }

    #[test]
    fn real_to_json_nan_strict() {
        let result = real_to_json(f64::NAN, true, false);
        assert!(result.is_err());
    }

    #[test]
    fn real_to_json_nan_permissive() {
        let result = real_to_json(f64::NAN, false, true);
        assert!(result.is_ok());
        assert!(RJsonValueOps::is_null(&result.unwrap()));
    }

    #[test]
    fn real_to_json_inf_permissive() {
        let result = real_to_json(f64::INFINITY, false, true);
        assert!(result.is_ok());
        assert!(RJsonValueOps::is_null(&result.unwrap()));
    }
}
