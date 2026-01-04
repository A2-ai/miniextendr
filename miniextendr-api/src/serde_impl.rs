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
//! cfg2 <- Config$r_from_json(json)
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
//! - `r_from_json()` returns `Option<Self>` - returns None on parse failure
//! - For more detailed error information, use `r_from_json_result()`

pub use serde::{Deserialize, Serialize};
pub use serde_json;

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
    fn r_to_json(&self) -> Result<String, String>;

    /// Serialize to a pretty-printed JSON string with indentation.
    ///
    /// Returns `Ok(json_string)` on success, `Err(error_message)` on failure.
    fn r_to_json_pretty(&self) -> Result<String, String>;
}

impl<T: Serialize> RSerialize for T {
    fn r_to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    fn r_to_json_pretty(&self) -> Result<String, String> {
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
/// - `r_from_json(s)` - Parse JSON string, returning None on failure
/// - `r_from_json_result(s)` - Parse JSON string with detailed error
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
/// cfg <- Config$r_from_json('{"name":"test","value":42}')
/// ```
pub trait RDeserialize: Sized {
    /// Parse a JSON string into this type.
    ///
    /// Returns `Some(value)` on success, `None` on parse failure.
    /// The None case maps to NULL in R.
    fn r_from_json(s: &str) -> Option<Self>;

    /// Parse a JSON string with detailed error information.
    ///
    /// Returns `Ok(value)` on success, `Err(error_message)` on failure.
    fn r_from_json_result(s: &str) -> Result<Self, String>;
}

impl<T: for<'de> Deserialize<'de>> RDeserialize for T {
    fn r_from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }

    fn r_from_json_result(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
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
        let json = data.r_to_json().unwrap();
        assert_eq!(json, r#"{"name":"test","value":42,"enabled":true}"#);
    }

    #[test]
    fn rserialize_to_json_pretty() {
        let data = TestStruct {
            name: "test".to_string(),
            value: 42,
            enabled: true,
        };
        let json = data.r_to_json_pretty().unwrap();
        assert!(json.contains('\n'));
        assert!(json.contains("  ")); // Indentation
        assert!(json.contains("\"name\": \"test\""));
    }

    #[test]
    fn rdeserialize_from_json() {
        let json = r#"{"name":"hello","value":123,"enabled":false}"#;
        let data: Option<TestStruct> = RDeserialize::r_from_json(json);
        assert!(data.is_some());
        let data = data.unwrap();
        assert_eq!(data.name, "hello");
        assert_eq!(data.value, 123);
        assert!(!data.enabled);
    }

    #[test]
    fn rdeserialize_from_json_result() {
        let json = r#"{"name":"world","value":456,"enabled":true}"#;
        let result: Result<TestStruct, String> = RDeserialize::r_from_json_result(json);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "world");
    }

    #[test]
    fn rdeserialize_invalid_json() {
        let invalid = "not valid json";
        let data: Option<TestStruct> = RDeserialize::r_from_json(invalid);
        assert!(data.is_none());

        let result: Result<TestStruct, String> = RDeserialize::r_from_json_result(invalid);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected"));
    }

    #[test]
    fn rdeserialize_missing_field() {
        let json = r#"{"name":"test"}"#; // Missing value and enabled
        let data: Option<TestStruct> = RDeserialize::r_from_json(json);
        assert!(data.is_none());
    }

    #[test]
    fn roundtrip() {
        let original = TestStruct {
            name: "roundtrip".to_string(),
            value: 999,
            enabled: true,
        };
        let json = original.r_to_json().unwrap();
        let parsed: TestStruct = RDeserialize::r_from_json(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn serialize_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let json = data.r_to_json().unwrap();
        assert_eq!(json, "[1,2,3,4,5]");
    }

    #[test]
    fn deserialize_vec() {
        let json = "[10,20,30]";
        let data: Option<Vec<i32>> = RDeserialize::r_from_json(json);
        assert_eq!(data, Some(vec![10, 20, 30]));
    }

    #[test]
    fn serialize_option() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert_eq!(some_val.r_to_json().unwrap(), "42");
        assert_eq!(none_val.r_to_json().unwrap(), "null");
    }

    #[test]
    fn deserialize_option() {
        let data: Option<Option<i32>> = RDeserialize::r_from_json("42");
        assert_eq!(data, Some(Some(42)));

        let data: Option<Option<i32>> = RDeserialize::r_from_json("null");
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
        let json = data.r_to_json().unwrap();
        assert_eq!(json, r#"{"inner":{"x":100}}"#);
    }

    #[test]
    fn serialize_with_special_chars() {
        let data = TestStruct {
            name: "hello\nworld\t\"quoted\"".to_string(),
            value: 0,
            enabled: false,
        };
        let json = data.r_to_json().unwrap();
        // Special chars should be escaped
        assert!(json.contains("\\n"));
        assert!(json.contains("\\t"));
        assert!(json.contains("\\\""));
    }
}
