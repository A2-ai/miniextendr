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
//! Enable this module with the `serde_json` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["serde_json"] }
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

// =============================================================================
// JSON conversion options
// =============================================================================

/// How to handle NA values when converting R to JSON.
#[derive(Debug, Clone, Default)]
pub enum NaHandling {
    /// Convert NA to JSON null (default).
    #[default]
    Null,
    /// Return an error when NA is encountered.
    Error,
    /// Convert NA to a custom string value.
    String(String),
}

/// How to handle special float values (NaN, Inf) when converting R to JSON.
#[derive(Debug, Clone, Default)]
pub enum SpecialFloatHandling {
    /// Return an error (default) - JSON has no representation for these.
    #[default]
    Error,
    /// Convert to JSON null.
    Null,
    /// Convert to a string representation ("NaN", "Infinity", "-Infinity").
    String,
}

/// How to serialize R factors to JSON.
#[derive(Debug, Clone, Default)]
pub enum FactorHandling {
    /// Use the factor level label as a string (default).
    #[default]
    Label,
    /// Use the factor level index as an integer (1-based, matching R).
    Index,
}

/// Options for converting R objects to JSON.
///
/// # Example
///
/// ```rust,ignore
/// use miniextendr_api::serde_impl::{JsonOptions, NaHandling, SpecialFloatHandling};
///
/// let opts = JsonOptions::default()
///     .na(NaHandling::String("NA".into()))
///     .nan(SpecialFloatHandling::Null)
///     .inf(SpecialFloatHandling::String);
///
/// let json = json_from_sexp_with(sexp, &opts)?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct JsonOptions {
    /// How to handle NA values.
    pub na: NaHandling,
    /// How to handle NaN values.
    pub nan: SpecialFloatHandling,
    /// How to handle Inf/-Inf values.
    pub inf: SpecialFloatHandling,
    /// How to serialize factors.
    pub factor: FactorHandling,
}

impl JsonOptions {
    /// Create new options with defaults (NA→null, NaN/Inf→error, factors→labels).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create strict options (all special values cause errors).
    pub fn strict() -> Self {
        Self {
            na: NaHandling::Error,
            nan: SpecialFloatHandling::Error,
            inf: SpecialFloatHandling::Error,
            factor: FactorHandling::Label,
        }
    }

    /// Create permissive options (all special values become null).
    pub fn permissive() -> Self {
        Self {
            na: NaHandling::Null,
            nan: SpecialFloatHandling::Null,
            inf: SpecialFloatHandling::Null,
            factor: FactorHandling::Label,
        }
    }

    /// Set NA handling.
    pub fn na(mut self, handling: NaHandling) -> Self {
        self.na = handling;
        self
    }

    /// Set NaN handling.
    pub fn nan(mut self, handling: SpecialFloatHandling) -> Self {
        self.nan = handling;
        self
    }

    /// Set Inf handling.
    pub fn inf(mut self, handling: SpecialFloatHandling) -> Self {
        self.inf = handling;
        self
    }

    /// Set factor handling.
    pub fn factor(mut self, handling: FactorHandling) -> Self {
        self.factor = handling;
        self
    }
}
use crate::ffi::{
    INTEGER_ELT, LOGICAL_ELT, REAL_ELT, Rboolean, Rf_allocVector, Rf_getAttrib, Rf_isFactor,
    Rf_setAttrib, Rf_xlength, SET_INTEGER_ELT, SET_LOGICAL_ELT, SET_REAL_ELT, SET_STRING_ELT,
    SET_VECTOR_ELT, SEXP, SEXPTYPE, STRING_ELT, SexpExt,
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

/// Convert an R object to a JSON value with custom options.
///
/// # Example
///
/// ```rust,ignore
/// let opts = JsonOptions::default()
///     .na(NaHandling::String("NA".into()))
///     .nan(SpecialFloatHandling::Null);
/// let json = json_from_sexp_with(sexp, &opts)?;
/// ```
pub fn json_from_sexp_with(sexp: SEXP, opts: &JsonOptions) -> Result<JsonValue, SexpError> {
    sexp_to_json_value(sexp, opts)
}

/// Convert an R object to a JSON value with default options.
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
    json_from_sexp_with(sexp, &JsonOptions::default())
}

/// Convert R to JSON with strict NA handling (errors on NA).
pub fn json_from_sexp_strict(sexp: SEXP) -> Result<JsonValue, SexpError> {
    json_from_sexp_with(sexp, &JsonOptions::strict())
}

/// Convert R to JSON with permissive handling (NA/NaN/Inf -> Null).
pub fn json_from_sexp_permissive(sexp: SEXP) -> Result<JsonValue, SexpError> {
    json_from_sexp_with(sexp, &JsonOptions::permissive())
}

/// Helper to handle NA values according to options.
fn handle_na(opts: &JsonOptions, context: Option<usize>) -> Result<JsonValue, SexpError> {
    match &opts.na {
        NaHandling::Null => Ok(JsonValue::Null),
        NaHandling::Error => {
            let msg = match context {
                Some(i) => format!("NA at index {} not allowed", i),
                None => "NA not allowed".into(),
            };
            Err(SexpError::InvalidValue(msg))
        }
        NaHandling::String(s) => Ok(JsonValue::String(s.clone())),
    }
}

fn sexp_to_json_value(sexp: SEXP, opts: &JsonOptions) -> Result<JsonValue, SexpError> {
    let sexp_type = sexp.type_of();

    // Handle NULL
    if sexp_type == SEXPTYPE::NILSXP {
        return Ok(JsonValue::Null);
    }

    // Handle factors first (convert to string)
    if unsafe { Rf_isFactor(sexp) } != Rboolean::FALSE {
        return factor_to_json(sexp, opts);
    }

    let len = unsafe { Rf_xlength(sexp) } as usize;

    match sexp_type {
        SEXPTYPE::LGLSXP => {
            if len == 1 {
                let val = unsafe { LOGICAL_ELT(sexp, 0) };
                if val == NA_LOGICAL {
                    return handle_na(opts, None);
                }
                Ok(JsonValue::Bool(val != 0))
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let val = unsafe { LOGICAL_ELT(sexp, i as isize) };
                        if val == NA_LOGICAL {
                            handle_na(opts, Some(i))
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
                    return handle_na(opts, None);
                }
                Ok(JsonValue::Number(serde_json::Number::from(val)))
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let val = unsafe { INTEGER_ELT(sexp, i as isize) };
                        if val == NA_INTEGER {
                            handle_na(opts, Some(i))
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
                real_to_json(val, opts, None)
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let val = unsafe { REAL_ELT(sexp, i as isize) };
                        real_to_json(val, opts, Some(i))
                    })
                    .collect();
                Ok(JsonValue::Array(arr?))
            }
        }
        SEXPTYPE::STRSXP => {
            if len == 1 {
                let charsxp = unsafe { STRING_ELT(sexp, 0) };
                if charsxp == unsafe { crate::ffi::R_NaString } {
                    return handle_na(opts, None);
                }
                let s = unsafe { charsxp_to_str(charsxp) };
                Ok(JsonValue::String(s.to_string()))
            } else {
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let charsxp = unsafe { STRING_ELT(sexp, i as isize) };
                        if charsxp == unsafe { crate::ffi::R_NaString } {
                            handle_na(opts, Some(i))
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
                    let val = sexp_to_json_value(elem, opts)?;
                    map.insert(key, val);
                }
                Ok(JsonValue::Object(map))
            } else {
                // Convert to array
                let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
                    .map(|i| {
                        let elem = unsafe { crate::ffi::VECTOR_ELT(sexp, i as isize) };
                        sexp_to_json_value(elem, opts)
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

fn real_to_json(
    val: f64,
    opts: &JsonOptions,
    context: Option<usize>,
) -> Result<JsonValue, SexpError> {
    // Check for NA (NA_REAL is a specific NaN bit pattern)
    if val.to_bits() == NA_REAL.to_bits() {
        return handle_na(opts, context);
    }

    // Check for NaN
    if val.is_nan() {
        return match &opts.nan {
            SpecialFloatHandling::Error => {
                let msg = match context {
                    Some(i) => format!("NaN at index {} cannot be represented in JSON", i),
                    None => "NaN cannot be represented in JSON".into(),
                };
                Err(SexpError::InvalidValue(msg))
            }
            SpecialFloatHandling::Null => Ok(JsonValue::Null),
            SpecialFloatHandling::String => Ok(JsonValue::String("NaN".into())),
        };
    }

    // Check for Inf
    if val.is_infinite() {
        return match &opts.inf {
            SpecialFloatHandling::Error => {
                let msg = match context {
                    Some(i) => format!("Infinity at index {} cannot be represented in JSON", i),
                    None => "Infinity cannot be represented in JSON".into(),
                };
                Err(SexpError::InvalidValue(msg))
            }
            SpecialFloatHandling::Null => Ok(JsonValue::Null),
            SpecialFloatHandling::String => {
                if val.is_sign_positive() {
                    Ok(JsonValue::String("Infinity".into()))
                } else {
                    Ok(JsonValue::String("-Infinity".into()))
                }
            }
        };
    }

    serde_json::Number::from_f64(val)
        .map(JsonValue::Number)
        .ok_or_else(|| SexpError::InvalidValue("cannot convert f64 to JSON number".into()))
}

fn factor_to_json(sexp: SEXP, opts: &JsonOptions) -> Result<JsonValue, SexpError> {
    let len = unsafe { Rf_xlength(sexp) } as usize;
    let levels = unsafe { Rf_getAttrib(sexp, crate::ffi::R_LevelsSymbol) };

    // Helper to convert factor index to JSON based on FactorHandling
    let index_to_json = |idx: i32| -> JsonValue {
        match opts.factor {
            FactorHandling::Label => {
                // Factor indices are 1-based
                let charsxp = unsafe { STRING_ELT(levels, (idx - 1) as isize) };
                let s = unsafe { charsxp_to_str(charsxp) };
                JsonValue::String(s.to_string())
            }
            FactorHandling::Index => JsonValue::Number(serde_json::Number::from(idx)),
        }
    };

    if len == 1 {
        let idx = unsafe { INTEGER_ELT(sexp, 0) };
        if idx == NA_INTEGER {
            return handle_na(opts, None);
        }
        Ok(index_to_json(idx))
    } else {
        let arr: Result<Vec<JsonValue>, SexpError> = (0..len)
            .map(|i| {
                let idx = unsafe { INTEGER_ELT(sexp, i as isize) };
                if idx == NA_INTEGER {
                    handle_na(opts, Some(i))
                } else {
                    Ok(index_to_json(idx))
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
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        json_value_to_sexp(&self)
    }
}

impl IntoR for Option<JsonValue> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        match self {
            Some(value) => json_value_to_sexp(&value),
            None => unsafe { crate::ffi::R_NilValue },
        }
    }
}

impl IntoR for Vec<JsonValue> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
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
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
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

/// Check if a JSON number fits in an R integer (i32, excluding NA_integer_).
fn json_number_fits_i32(n: &serde_json::Number) -> bool {
    if let Some(i) = n.as_i64() {
        // i32::MIN is NA_integer_ in R, so exclude it from integer range
        i > i32::MIN as i64 && i <= i32::MAX as i64
    } else {
        false
    }
}

/// Discriminant for JSON value type, used for homogeneous array detection.
fn json_discriminant(v: &JsonValue) -> u8 {
    match v {
        JsonValue::Null => 0,
        JsonValue::Bool(_) => 1,
        JsonValue::Number(_) => 2,
        JsonValue::String(_) => 3,
        JsonValue::Array(_) => 4,
        JsonValue::Object(_) => 5,
    }
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
            if json_number_fits_i32(n) {
                let sexp = unsafe { Rf_allocVector(SEXPTYPE::INTSXP, 1) };
                unsafe { SET_INTEGER_ELT(sexp, 0, n.as_i64().unwrap() as i32) };
                return sexp;
            }
            // Fall back to f64
            let f = n.as_f64().unwrap_or(f64::NAN);
            let sexp = unsafe { Rf_allocVector(SEXPTYPE::REALSXP, 1) };
            unsafe { SET_REAL_ELT(sexp, 0, f) };
            sexp
        }
        JsonValue::String(s) => {
            let sexp = unsafe { Rf_allocVector(SEXPTYPE::STRSXP, 1) };
            let charsxp = unsafe { crate::altrep_impl::checked_mkchar(s) };
            unsafe { SET_STRING_ELT(sexp, 0, charsxp) };
            sexp
        }
        JsonValue::Array(arr) => json_array_to_sexp(arr),
        JsonValue::Object(map) => {
            let len = map.len();
            // Protect both sexp and names before recursive calls that may trigger GC
            let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, len as isize)) };
            let names =
                unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, len as isize)) };

            for (i, (key, val)) in map.iter().enumerate() {
                let charsxp = unsafe { crate::altrep_impl::checked_mkchar(key) };
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

/// Convert a JSON array to an R object, using homogeneous native vectors when possible.
///
/// If all elements share the same JSON type, produces a native R vector:
/// - All bools -> LGLSXP (logical vector)
/// - All integers (fitting i32) -> INTSXP (integer vector)
/// - All numbers -> REALSXP (numeric vector)
/// - All strings -> STRSXP (character vector)
///
/// Mixed-type or nested arrays fall back to VECSXP (R list).
fn json_array_to_sexp(arr: &[JsonValue]) -> SEXP {
    if arr.is_empty() {
        return unsafe { Rf_allocVector(SEXPTYPE::VECSXP, 0) };
    }

    let first_disc = json_discriminant(&arr[0]);
    let all_same = arr.iter().all(|v| json_discriminant(v) == first_disc);

    if all_same {
        match &arr[0] {
            JsonValue::Bool(_) => {
                let sexp = unsafe { Rf_allocVector(SEXPTYPE::LGLSXP, arr.len() as isize) };
                for (i, v) in arr.iter().enumerate() {
                    if let JsonValue::Bool(b) = v {
                        unsafe { SET_LOGICAL_ELT(sexp, i as isize, if *b { 1 } else { 0 }) };
                    }
                }
                return sexp;
            }
            JsonValue::Number(_) => {
                // Check if all fit in i32 (excluding NA_integer_)
                let all_i32 = arr.iter().all(|v| {
                    if let JsonValue::Number(n) = v {
                        json_number_fits_i32(n)
                    } else {
                        false
                    }
                });

                if all_i32 {
                    let sexp = unsafe { Rf_allocVector(SEXPTYPE::INTSXP, arr.len() as isize) };
                    for (i, v) in arr.iter().enumerate() {
                        if let JsonValue::Number(n) = v {
                            unsafe {
                                SET_INTEGER_ELT(sexp, i as isize, n.as_i64().unwrap() as i32)
                            };
                        }
                    }
                    return sexp;
                }

                // Fall back to f64
                let sexp = unsafe { Rf_allocVector(SEXPTYPE::REALSXP, arr.len() as isize) };
                for (i, v) in arr.iter().enumerate() {
                    if let JsonValue::Number(n) = v {
                        unsafe { SET_REAL_ELT(sexp, i as isize, n.as_f64().unwrap_or(f64::NAN)) };
                    }
                }
                return sexp;
            }
            JsonValue::String(_) => {
                // Protect sexp before Rf_mkCharLenCE calls which can trigger GC
                let sexp = unsafe {
                    OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, arr.len() as isize))
                };
                for (i, v) in arr.iter().enumerate() {
                    if let JsonValue::String(s) = v {
                        unsafe {
                            let charsxp = crate::altrep_impl::checked_mkchar(s);
                            SET_STRING_ELT(sexp.get(), i as isize, charsxp);
                        }
                    }
                }
                return sexp.get();
            }
            _ => {
                // Null, Array, Object - fall through to generic list
            }
        }
    }

    // Heterogeneous or complex types -> list
    let sexp = unsafe { OwnedProtect::new(Rf_allocVector(SEXPTYPE::VECSXP, arr.len() as isize)) };
    for (i, elem) in arr.iter().enumerate() {
        unsafe { SET_VECTOR_ELT(sexp.get(), i as isize, json_value_to_sexp(elem)) };
    }
    sexp.get()
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

// =============================================================================
// RJsonBridge: Direct struct <-> R list via serde_json::Value
// =============================================================================

/// Bridge trait for direct Rust struct to R list conversion via `serde_json::Value`.
///
/// This converts Rust types to/from native R lists without going through
/// a JSON string intermediate. The path is: `Rust struct -> serde_json::Value -> R SEXP`
/// (and vice versa), which avoids the overhead of serializing to and parsing
/// from a JSON string.
///
/// # Example
///
/// ```rust,ignore
/// use serde::{Serialize, Deserialize};
/// use miniextendr_api::serde_impl::RJsonBridge;
///
/// #[derive(Serialize, Deserialize)]
/// struct Config {
///     name: String,
///     value: i32,
///     tags: Vec<String>,
/// }
///
/// // The blanket impl provides to_r_list() and from_r_list() automatically.
/// fn export_config(cfg: &Config) -> SEXP {
///     cfg.to_r_list()
/// }
///
/// fn import_config(sexp: SEXP) -> Result<Config, String> {
///     Config::from_r_list(sexp)
/// }
/// ```
///
/// In R, the result is a native named list:
/// ```r
/// cfg <- export_config(cfg_ptr)
/// cfg$name   # "my-app"
/// cfg$value  # 42L
/// cfg$tags   # c("alpha", "beta")
/// ```
pub trait RJsonBridge: Serialize + for<'de> Deserialize<'de> {
    /// Convert this value to a native R list/vector via `serde_json::Value`.
    ///
    /// The struct is first serialized to a `serde_json::Value` (no string
    /// intermediate), then that value is converted to the appropriate R type.
    fn to_r_list(&self) -> SEXP;

    /// Create a value of this type from an R object via `serde_json::Value`.
    ///
    /// The R object is first converted to a `serde_json::Value`, then
    /// deserialized into the target type.
    fn from_r_list(sexp: SEXP) -> Result<Self, String>;
}

impl<T: Serialize + for<'de> Deserialize<'de>> RJsonBridge for T {
    fn to_r_list(&self) -> SEXP {
        // Serialize to serde_json::Value (in-memory, no string intermediate)
        let value = serde_json::to_value(self).expect("serde_json::to_value failed");
        json_value_to_sexp(&value)
    }

    fn from_r_list(sexp: SEXP) -> Result<Self, String> {
        // Convert R SEXP to serde_json::Value
        let value = json_from_sexp(sexp).map_err(|e| format!("{}", e))?;
        // Deserialize from the Value (no string intermediate)
        serde_json::from_value(value).map_err(|e| e.to_string())
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
        let opts = JsonOptions::default();
        let result = real_to_json(std::f64::consts::PI, &opts, None);
        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(RJsonValueOps::is_number(&val));
    }

    #[test]
    fn real_to_json_nan_strict() {
        let opts = JsonOptions {
            nan: SpecialFloatHandling::Error,
            ..Default::default()
        };
        let result = real_to_json(f64::NAN, &opts, None);
        assert!(result.is_err());
    }

    #[test]
    fn real_to_json_nan_permissive() {
        let opts = JsonOptions {
            nan: SpecialFloatHandling::Null,
            ..Default::default()
        };
        let result = real_to_json(f64::NAN, &opts, None);
        assert!(result.is_ok());
        assert!(RJsonValueOps::is_null(&result.unwrap()));
    }

    #[test]
    fn real_to_json_inf_permissive() {
        let opts = JsonOptions {
            inf: SpecialFloatHandling::Null,
            ..Default::default()
        };
        let result = real_to_json(f64::INFINITY, &opts, None);
        assert!(result.is_ok());
        assert!(RJsonValueOps::is_null(&result.unwrap()));
    }

    // =========================================================================
    // NA_integer_ exclusion
    // =========================================================================

    #[test]
    fn json_number_fits_i32_excludes_na_integer() {
        // i32::MIN is NA_integer_ in R - must not be treated as integer
        let n = serde_json::Number::from(i32::MIN as i64);
        assert!(!json_number_fits_i32(&n));

        // i32::MIN + 1 should still work
        let n = serde_json::Number::from(i32::MIN as i64 + 1);
        assert!(json_number_fits_i32(&n));

        // i32::MAX should work
        let n = serde_json::Number::from(i32::MAX as i64);
        assert!(json_number_fits_i32(&n));

        // i32::MAX + 1 should not fit
        let n = serde_json::Number::from(i32::MAX as i64 + 1);
        assert!(!json_number_fits_i32(&n));
    }

    // =========================================================================
    // json_discriminant
    // =========================================================================

    #[test]
    fn discriminant_coverage() {
        assert_eq!(json_discriminant(&JsonValue::Null), 0);
        assert_eq!(json_discriminant(&JsonValue::Bool(true)), 1);
        assert_eq!(json_discriminant(&serde_json::json!(42)), 2);
        assert_eq!(json_discriminant(&serde_json::json!("hi")), 3);
        assert_eq!(json_discriminant(&serde_json::json!([1])), 4);
        assert_eq!(json_discriminant(&serde_json::json!({"a": 1})), 5);
    }

    // =========================================================================
    // RJsonBridge roundtrip tests (pure Rust, no R runtime)
    // =========================================================================

    #[test]
    fn bridge_to_value_roundtrip() {
        // Test that struct -> Value -> struct roundtrips correctly
        let original = TestStruct {
            name: "bridge".to_string(),
            value: 42,
            enabled: true,
        };

        // Serialize to serde_json::Value
        let value = serde_json::to_value(&original).unwrap();
        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "bridge");
        assert_eq!(obj.get("value").unwrap().as_i64().unwrap(), 42);
        assert!(obj.get("enabled").unwrap().as_bool().unwrap());

        // Deserialize back
        let parsed: TestStruct = serde_json::from_value(value).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn bridge_nested_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Outer {
            label: String,
            inner: Inner,
            items: Vec<i32>,
        }
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Inner {
            x: f64,
            y: f64,
        }

        let original = Outer {
            label: "point".into(),
            inner: Inner { x: 1.5, y: 2.5 },
            items: vec![10, 20, 30],
        };

        let value = serde_json::to_value(&original).unwrap();
        let parsed: Outer = serde_json::from_value(value).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn bridge_with_options() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Config {
            name: String,
            debug: Option<bool>,
            max_retries: Option<u32>,
        }

        let cfg = Config {
            name: "app".into(),
            debug: Some(true),
            max_retries: None,
        };

        let value = serde_json::to_value(&cfg).unwrap();
        let obj = value.as_object().unwrap();
        assert_eq!(obj.get("debug").unwrap(), &serde_json::json!(true));
        assert_eq!(obj.get("max_retries").unwrap(), &JsonValue::Null);

        let parsed: Config = serde_json::from_value(value).unwrap();
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn bridge_enum_variants() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(tag = "type")]
        enum Shape {
            Circle { radius: f64 },
            Rect { width: f64, height: f64 },
        }

        let circle = Shape::Circle { radius: 5.0 };
        let value = serde_json::to_value(&circle).unwrap();
        let parsed: Shape = serde_json::from_value(value).unwrap();
        assert_eq!(circle, parsed);

        let rect = Shape::Rect {
            width: 3.0,
            height: 4.0,
        };
        let value = serde_json::to_value(&rect).unwrap();
        let parsed: Shape = serde_json::from_value(value).unwrap();
        assert_eq!(rect, parsed);
    }

    #[test]
    fn bridge_vec_of_structs() {
        let items = vec![
            TestStruct {
                name: "a".into(),
                value: 1,
                enabled: true,
            },
            TestStruct {
                name: "b".into(),
                value: 2,
                enabled: false,
            },
        ];

        let value = serde_json::to_value(&items).unwrap();
        assert!(value.is_array());
        let arr = value.as_array().unwrap();
        assert_eq!(arr.len(), 2);

        let parsed: Vec<TestStruct> = serde_json::from_value(value).unwrap();
        assert_eq!(items, parsed);
    }
}
