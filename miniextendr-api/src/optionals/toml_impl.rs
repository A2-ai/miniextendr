//! Integration with the `toml` crate for TOML value conversions.
//!
//! Provides conversions between TOML values and R types.
//!
//! # Features
//!
//! Enable this module with the `toml` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["toml"] }
//! ```
//!
//! # Overview
//!
//! Two conversion paths are available:
//!
//! 1. **String parsing**: Parse TOML text to `TomlValue`, serialize back to text
//! 2. **Direct conversion**: Convert `TomlValue` to R lists
//!
//! # TOML Constraints
//!
//! TOML has stricter requirements than R:
//! - **No null**: NULL values cause an error
//! - **No NA**: NA values cause an error (use explicit empty string if needed)
//! - **Homogeneous arrays**: TOML arrays must contain same-type elements
//! - **String keys only**: Table keys must be strings
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::toml_impl::{toml_from_str, toml_to_string};
//!
//! #[miniextendr]
//! fn parse_config(text: &str) -> Result<TomlValue, String> {
//!     toml_from_str(text).map_err(|e| e.to_string())
//! }
//!
//! #[miniextendr]
//! fn config_to_text(value: TomlValue) -> String {
//!     toml_to_string(&value)
//! }
//! ```
//!
//! # Type Mapping
//!
//! | TOML Type | R Type |
//! |-----------|--------|
//! | String | character(1) |
//! | Integer | integer(1) or numeric(1) if out of i32 range |
//! | Float | numeric(1) |
//! | Boolean | logical(1) |
//! | Array | vector (homogeneous) or list (heterogeneous) |
//! | Table | named list |
//! | Datetime | character(1) (ISO 8601 format) |

pub use toml::Value as TomlValue;

use crate::ffi::{Rf_allocVector, Rf_xlength, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp, charsxp_to_str};
use crate::gc_protect::OwnedProtect;
use crate::impl_option_try_from_sexp;
use crate::into_r::IntoR;

// region: Helper functions

/// Parse a TOML document string into a `TomlValue`.
///
/// # Errors
///
/// Returns an error if the string is not valid TOML.
///
/// # Example
///
/// ```ignore
/// let value = toml_from_str("[package]\nname = \"my-pkg\"")?;
/// ```
pub fn toml_from_str(s: &str) -> Result<TomlValue, SexpError> {
    // In toml 0.9+, Value::from_str expects a single value literal, not a document.
    // To parse a document with key-value pairs, we parse into Table and convert.
    let table: toml::Table = s
        .parse()
        .map_err(|e: toml::de::Error| SexpError::InvalidValue(format!("invalid TOML: {}", e)))?;
    Ok(TomlValue::Table(table))
}

/// Serialize a `TomlValue` to a TOML string.
///
/// # Example
///
/// ```ignore
/// let text = toml_to_string(&value);
/// ```
pub fn toml_to_string(v: &TomlValue) -> String {
    // Note: toml::to_string can fail for certain edge cases, but for well-formed
    // TomlValue it should always succeed
    toml::to_string(v).unwrap_or_else(|e| format!("# Error serializing: {}", e))
}

/// Serialize a `TomlValue` to a pretty-printed TOML string.
pub fn toml_to_string_pretty(v: &TomlValue) -> String {
    toml::to_string_pretty(v).unwrap_or_else(|e| format!("# Error serializing: {}", e))
}
// endregion

// region: TryFromSexp for TomlValue

impl TryFromSexp for TomlValue {
    type Error = SexpError;

    /// Parse a TOML value from an R character scalar.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input is not a character vector (STRSXP)
    /// - Input is not length 1
    /// - Input is NA
    /// - TOML parsing fails
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpError::Type(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }));
        }

        let len = unsafe { Rf_xlength(sexp) } as usize;
        if len != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected character(1), got length {}",
                len
            )));
        }

        let charsxp = sexp.string_elt(0);
        if charsxp == unsafe { crate::ffi::R_NaString } {
            return Err(SexpError::InvalidValue(
                "NA not allowed for TOML parsing".to_string(),
            ));
        }

        let s = unsafe { charsxp_to_str(charsxp) };
        toml_from_str(s)
    }
}
// endregion

// region: Option / Vec conversions

// Use macro for Option<TomlValue>
impl_option_try_from_sexp!(TomlValue);

// Vec conversions have custom logic (parse from Vec<String>, not VECSXP)
impl TryFromSexp for Vec<TomlValue> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let strings: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        strings
            .into_iter()
            .enumerate()
            .map(|(i, opt)| {
                let s = opt.ok_or_else(|| {
                    SexpError::InvalidValue(format!(
                        "NA at index {} not allowed for TOML parsing",
                        i
                    ))
                })?;
                toml_from_str(&s).map_err(|e| {
                    SexpError::InvalidValue(format!("invalid TOML at index {}: {}", i, e))
                })
            })
            .collect()
    }
}

impl TryFromSexp for Vec<Option<TomlValue>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let strings: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        strings
            .into_iter()
            .enumerate()
            .map(|(i, opt)| match opt {
                None => Ok(None),
                Some(s) => toml_from_str(&s).map(Some).map_err(|e| {
                    SexpError::InvalidValue(format!("invalid TOML at index {}: {}", i, e))
                }),
            })
            .collect()
    }
}
// endregion

// region: IntoR for TomlValue

impl IntoR for TomlValue {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    /// Convert a TOML value to an R object.
    ///
    /// - String -> character(1)
    /// - Integer -> integer(1)
    /// - Float -> numeric(1)
    /// - Boolean -> logical(1)
    /// - Array -> vector or list
    /// - Table -> named list
    /// - Datetime -> character(1)
    fn into_sexp(self) -> SEXP {
        toml_value_to_sexp(&self)
    }
}

impl IntoR for Option<TomlValue> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        match self {
            Some(value) => toml_value_to_sexp(&value),
            None => crate::ffi::SEXP::null(),
        }
    }
}

impl IntoR for Vec<TomlValue> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let len = self.len();
        let sexp = unsafe {
            OwnedProtect::new(Rf_allocVector(
                SEXPTYPE::VECSXP,
                isize::try_from(len).expect("length exceeds R_xlen_t"),
            ))
        };
        for (i, value) in self.iter().enumerate() {
            unsafe {
                sexp.get().set_vector_elt(
                    isize::try_from(i).expect("index overflow"),
                    toml_value_to_sexp(value),
                )
            };
        }
        // Return the SEXP - guard drops and unprotects
        sexp.get()
    }
}

impl IntoR for Vec<Option<TomlValue>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<crate::ffi::SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<crate::ffi::SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        let len = self.len();
        let sexp = unsafe {
            OwnedProtect::new(Rf_allocVector(
                SEXPTYPE::VECSXP,
                isize::try_from(len).expect("length exceeds R_xlen_t"),
            ))
        };
        for (i, value) in self.iter().enumerate() {
            let elem = match value {
                Some(v) => toml_value_to_sexp(v),
                None => crate::ffi::SEXP::null(),
            };
            unsafe {
                sexp.get()
                    .set_vector_elt(isize::try_from(i).expect("index overflow"), elem)
            };
        }
        // Return the SEXP - guard drops and unprotects
        sexp.get()
    }
}

fn toml_value_to_sexp(v: &TomlValue) -> SEXP {
    match v {
        TomlValue::String(s) => string_to_sexp(s),
        TomlValue::Integer(i) => int_to_sexp(*i),
        TomlValue::Float(f) => float_to_sexp(*f),
        TomlValue::Boolean(b) => bool_to_sexp(*b),
        TomlValue::Array(arr) => array_to_sexp(arr),
        TomlValue::Table(table) => table_to_sexp(table),
        TomlValue::Datetime(dt) => string_to_sexp(&dt.to_string()),
    }
}

fn string_to_sexp(s: &str) -> SEXP {
    unsafe {
        // Protect sexp before checked_mkchar which can trigger GC
        let sexp = OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, 1));
        let charsxp = crate::altrep_impl::checked_mkchar(s);
        sexp.get().set_string_elt(0, charsxp);
        // Return the SEXP - guard drops and unprotects
        sexp.get()
    }
}

/// Check if an i64 fits in an R integer (i32, excluding i32::MIN which is NA_integer_).
fn i64_fits_r_int(i: i64) -> bool {
    i > i64::from(i32::MIN) && i <= i64::from(i32::MAX)
}

fn int_to_sexp(i: i64) -> SEXP {
    unsafe {
        if i64_fits_r_int(i) {
            let sexp = Rf_allocVector(SEXPTYPE::INTSXP, 1);

            sexp.set_integer_elt(0, i32::try_from(i).expect("validated by i64_fits_r_int"));
            sexp
        } else {
            // Value out of R integer range — store as double
            let sexp = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            #[allow(clippy::cast_precision_loss)]
            sexp.set_real_elt(0, i as f64);
            sexp
        }
    }
}

fn float_to_sexp(f: f64) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::REALSXP, 1);
        sexp.set_real_elt(0, f);
        sexp
    }
}

fn bool_to_sexp(b: bool) -> SEXP {
    unsafe {
        let sexp = Rf_allocVector(SEXPTYPE::LGLSXP, 1);
        sexp.set_logical_elt(0, if b { 1 } else { 0 });
        sexp
    }
}

fn array_to_sexp(arr: &[TomlValue]) -> SEXP {
    if arr.is_empty() {
        // Empty array -> empty list
        return unsafe { Rf_allocVector(SEXPTYPE::VECSXP, 0) };
    }

    // Check if all elements are the same type for potential vector conversion
    let first_type = discriminant(&arr[0]);
    let all_same = arr.iter().all(|v| discriminant(v) == first_type);

    if all_same {
        // Try to create a homogeneous vector
        match &arr[0] {
            TomlValue::String(_) => {
                // Protect sexp before Rf_mkCharLenCE calls which can trigger GC
                let sexp = unsafe {
                    OwnedProtect::new(Rf_allocVector(
                        SEXPTYPE::STRSXP,
                        isize::try_from(arr.len()).expect("length exceeds R_xlen_t"),
                    ))
                };
                for (i, v) in arr.iter().enumerate() {
                    if let TomlValue::String(s) = v {
                        unsafe {
                            let charsxp = crate::altrep_impl::checked_mkchar(s);

                            sexp.get().set_string_elt(
                                isize::try_from(i).expect("index overflow"),
                                charsxp,
                            );
                        }
                    }
                }
                // Return the SEXP - guard drops and unprotects
                return sexp.get();
            }
            TomlValue::Integer(_) => {
                // Check if ALL elements fit in R integer range
                let all_fit = arr.iter().all(|v| {
                    if let TomlValue::Integer(n) = v {
                        i64_fits_r_int(*n)
                    } else {
                        false
                    }
                });

                if all_fit {
                    let sexp = unsafe {
                        Rf_allocVector(
                            SEXPTYPE::INTSXP,
                            isize::try_from(arr.len()).expect("length exceeds R_xlen_t"),
                        )
                    };
                    for (i, v) in arr.iter().enumerate() {
                        if let TomlValue::Integer(n) = v {
                            unsafe {
                                sexp.set_integer_elt(
                                    isize::try_from(i).expect("index overflow"),
                                    i32::try_from(*n).expect("validated by i64_fits_r_int"),
                                )
                            };
                        }
                    }
                    return sexp;
                }
                // Fall back to REALSXP if any value is out of range
                let sexp = unsafe {
                    Rf_allocVector(
                        SEXPTYPE::REALSXP,
                        isize::try_from(arr.len()).expect("length exceeds R_xlen_t"),
                    )
                };
                for (i, v) in arr.iter().enumerate() {
                    if let TomlValue::Integer(n) = v {
                        #[allow(clippy::cast_precision_loss)]
                        unsafe {
                            sexp.set_real_elt(
                                isize::try_from(i).expect("index overflow"),
                                *n as f64,
                            )
                        };
                    }
                }
                return sexp;
            }
            TomlValue::Float(_) => {
                let sexp = unsafe {
                    Rf_allocVector(
                        SEXPTYPE::REALSXP,
                        isize::try_from(arr.len()).expect("length exceeds R_xlen_t"),
                    )
                };
                for (i, v) in arr.iter().enumerate() {
                    if let TomlValue::Float(f) = v {
                        unsafe {
                            sexp.set_real_elt(isize::try_from(i).expect("index overflow"), *f)
                        };
                    }
                }
                return sexp;
            }
            TomlValue::Boolean(_) => {
                let sexp = unsafe {
                    Rf_allocVector(
                        SEXPTYPE::LGLSXP,
                        isize::try_from(arr.len()).expect("length exceeds R_xlen_t"),
                    )
                };
                for (i, v) in arr.iter().enumerate() {
                    if let TomlValue::Boolean(b) = v {
                        unsafe {
                            sexp.set_logical_elt(
                                isize::try_from(i).expect("index overflow"),
                                if *b { 1 } else { 0 },
                            )
                        };
                    }
                }
                return sexp;
            }
            _ => {
                // Tables, arrays, datetimes - fall through to list
            }
        }
    }

    // Heterogeneous or complex types -> list
    // Protect sexp before recursive calls that may trigger GC
    let sexp = unsafe {
        OwnedProtect::new(Rf_allocVector(
            SEXPTYPE::VECSXP,
            isize::try_from(arr.len()).expect("length exceeds R_xlen_t"),
        ))
    };
    for (i, v) in arr.iter().enumerate() {
        unsafe {
            sexp.get().set_vector_elt(
                isize::try_from(i).expect("index overflow"),
                toml_value_to_sexp(v),
            )
        };
    }
    // Return the SEXP - guard drops and unprotects
    sexp.get()
}

fn table_to_sexp(table: &toml::map::Map<String, TomlValue>) -> SEXP {
    let len = table.len();
    // Protect both sexp and names before recursive calls that may trigger GC
    let sexp = unsafe {
        OwnedProtect::new(Rf_allocVector(
            SEXPTYPE::VECSXP,
            isize::try_from(len).expect("length exceeds R_xlen_t"),
        ))
    };
    let names = unsafe {
        OwnedProtect::new(Rf_allocVector(
            SEXPTYPE::STRSXP,
            isize::try_from(len).expect("length exceeds R_xlen_t"),
        ))
    };

    for (i, (key, value)) in table.iter().enumerate() {
        unsafe {
            let charsxp = crate::altrep_impl::checked_mkchar(key);

            names
                .get()
                .set_string_elt(isize::try_from(i).expect("index overflow"), charsxp);

            sexp.get().set_vector_elt(
                isize::try_from(i).expect("index overflow"),
                toml_value_to_sexp(value),
            );
        }
    }

    unsafe {
        sexp.get().set_names(names.get());
    }
    // Return the SEXP - guards drop and unprotect
    sexp.get()
}

// Helper to get a discriminant for type comparison
fn discriminant(v: &TomlValue) -> u8 {
    match v {
        TomlValue::String(_) => 0,
        TomlValue::Integer(_) => 1,
        TomlValue::Float(_) => 2,
        TomlValue::Boolean(_) => 3,
        TomlValue::Datetime(_) => 4,
        TomlValue::Array(_) => 5,
        TomlValue::Table(_) => 6,
    }
}
// endregion

// region: Adapter trait

/// Adapter trait for TOML value inspection.
///
/// # Registration
///
/// Registration is automatic when you annotate `impl RTomlOps for TomlValue`
/// with `#[miniextendr]`.
pub trait RTomlOps {
    /// Check if this is a string value.
    fn is_string(&self) -> bool;

    /// Check if this is an integer value.
    fn is_integer(&self) -> bool;

    /// Check if this is a float value.
    fn is_float(&self) -> bool;

    /// Check if this is a boolean value.
    fn is_boolean(&self) -> bool;

    /// Check if this is a datetime value.
    fn is_datetime(&self) -> bool;

    /// Check if this is an array.
    fn is_array(&self) -> bool;

    /// Check if this is a table.
    fn is_table(&self) -> bool;

    /// Get the type name as a string.
    fn type_name(&self) -> String;

    /// Serialize to TOML string.
    fn to_toml_string(&self) -> String;

    /// Serialize to pretty TOML string.
    fn to_toml_string_pretty(&self) -> String;

    /// Get as string if this is a string value.
    fn as_str(&self) -> Option<String>;

    /// Get as integer if this is an integer value.
    fn as_integer(&self) -> Option<i64>;

    /// Get as float if this is a float value.
    fn as_float(&self) -> Option<f64>;

    /// Get as boolean if this is a boolean value.
    fn as_bool(&self) -> Option<bool>;

    /// Get array length if this is an array.
    fn array_len(&self) -> Option<i32>;

    /// Get table keys if this is a table.
    fn table_keys(&self) -> Vec<String>;
}

impl RTomlOps for TomlValue {
    fn is_string(&self) -> bool {
        TomlValue::is_str(self)
    }

    fn is_integer(&self) -> bool {
        TomlValue::is_integer(self)
    }

    fn is_float(&self) -> bool {
        TomlValue::is_float(self)
    }

    fn is_boolean(&self) -> bool {
        TomlValue::is_bool(self)
    }

    fn is_datetime(&self) -> bool {
        TomlValue::is_datetime(self)
    }

    fn is_array(&self) -> bool {
        TomlValue::is_array(self)
    }

    fn is_table(&self) -> bool {
        TomlValue::is_table(self)
    }

    fn type_name(&self) -> String {
        match self {
            TomlValue::String(_) => "string".to_string(),
            TomlValue::Integer(_) => "integer".to_string(),
            TomlValue::Float(_) => "float".to_string(),
            TomlValue::Boolean(_) => "boolean".to_string(),
            TomlValue::Datetime(_) => "datetime".to_string(),
            TomlValue::Array(_) => "array".to_string(),
            TomlValue::Table(_) => "table".to_string(),
        }
    }

    fn to_toml_string(&self) -> String {
        toml_to_string(self)
    }

    fn to_toml_string_pretty(&self) -> String {
        toml_to_string_pretty(self)
    }

    fn as_str(&self) -> Option<String> {
        TomlValue::as_str(self).map(|s| s.to_string())
    }

    fn as_integer(&self) -> Option<i64> {
        TomlValue::as_integer(self)
    }

    fn as_float(&self) -> Option<f64> {
        TomlValue::as_float(self)
    }

    fn as_bool(&self) -> Option<bool> {
        TomlValue::as_bool(self)
    }

    fn array_len(&self) -> Option<i32> {
        TomlValue::as_array(self)
            .map(|arr| i32::try_from(arr.len()).expect("array length exceeds i32"))
    }

    fn table_keys(&self) -> Vec<String> {
        TomlValue::as_table(self)
            .map(|table| table.keys().cloned().collect())
            .unwrap_or_default()
    }
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_from_str_simple() {
        let toml_str = "name = \"test\"\nvalue = 42";
        let value = toml_from_str(toml_str).unwrap();
        assert!(value.is_table());
    }

    #[test]
    fn test_toml_from_str_invalid() {
        let result = toml_from_str("invalid toml [");
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_to_string() {
        // TOML requires a table at the top level, so test with a table
        let toml_str = r#"name = "hello""#;
        let value = toml_from_str(toml_str).unwrap();
        let s = toml_to_string(&value);
        assert!(s.contains("hello"));
        assert!(s.contains("name"));
    }

    #[test]
    fn test_discriminant() {
        assert_eq!(discriminant(&TomlValue::String("a".into())), 0);
        assert_eq!(discriminant(&TomlValue::Integer(1)), 1);
        assert_eq!(discriminant(&TomlValue::Float(1.0)), 2);
        assert_eq!(discriminant(&TomlValue::Boolean(true)), 3);
    }

    #[test]
    fn test_adapter_trait_types() {
        let s = TomlValue::String("test".into());
        assert!(RTomlOps::is_string(&s));
        assert!(!RTomlOps::is_integer(&s));
        assert_eq!(RTomlOps::type_name(&s), "string");

        let i = TomlValue::Integer(42);
        assert!(RTomlOps::is_integer(&i));
        assert_eq!(RTomlOps::as_integer(&i), Some(42));

        let f = TomlValue::Float(std::f64::consts::PI);
        assert!(RTomlOps::is_float(&f));
        assert_eq!(RTomlOps::as_float(&f), Some(std::f64::consts::PI));

        let b = TomlValue::Boolean(true);
        assert!(RTomlOps::is_boolean(&b));
        assert_eq!(RTomlOps::as_bool(&b), Some(true));
    }

    #[test]
    fn test_adapter_trait_array() {
        let arr = TomlValue::Array(vec![
            TomlValue::Integer(1),
            TomlValue::Integer(2),
            TomlValue::Integer(3),
        ]);
        assert!(RTomlOps::is_array(&arr));
        assert_eq!(RTomlOps::array_len(&arr), Some(3));
    }

    #[test]
    fn test_adapter_trait_table() {
        let toml_str = "alpha = 1\nbeta = 2";
        let value = toml_from_str(toml_str).unwrap();
        assert!(RTomlOps::is_table(&value));
        let keys = RTomlOps::table_keys(&value);
        assert!(keys.contains(&"alpha".to_string()));
        assert!(keys.contains(&"beta".to_string()));
    }
}
// endregion
