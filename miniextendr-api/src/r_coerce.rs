//! Traits for R's `as.<class>()` coercion functions.
//!
//! This module provides traits that enable Rust types wrapped in `ExternalPtr<T>`
//! to define how they convert to R base types. When used with the `#[miniextendr(as = "...")]`
//! attribute, these generate proper S3 method wrappers for R's coercion generics.
//!
//! # Tradeoff
//!
//! These traits are about exposing user-controlled conversions to R callers
//! via S3 method dispatch — they're **not** the inbound/outbound conversion
//! path for `#[miniextendr]` arguments and return values. For that, see
//! [`crate::from_r::TryFromSexp`] / [`crate::into_r::IntoR`] (and
//! [`crate::coerce::Coerce`] / [`crate::strict`] for the lax/strict pairs).
//! Failure mode of confusing the two: an `RCoerceList` impl will not satisfy a
//! `fn foo(_: MyType)` argument coming from R — that needs `TryFromSexp`.
//!
//! # Supported Conversions
//!
//! | R Generic | Rust Trait | Method |
//! |-----------|------------|--------|
//! | `as.data.frame` | [`RCoerceDataFrame`] | `as_data_frame(&self)` |
//! | `as.list` | [`RCoerceList`] | `as_list(&self)` |
//! | `as.character` | [`RCoerceCharacter`] | `as_character(&self)` |
//! | `as.numeric` / `as.double` | [`RCoerceNumeric`] | `as_numeric(&self)` |
//! | `as.integer` | [`RCoerceInteger`] | `as_integer(&self)` |
//! | `as.logical` | [`RCoerceLogical`] | `as_logical(&self)` |
//! | `as.matrix` | [`RCoerceMatrix`] | `as_matrix(&self)` |
//! | `as.vector` | [`RCoerceVector`] | `as_vector(&self)` |
//! | `as.factor` | [`RCoerceFactor`] | `as_factor(&self)` |
//! | `as.Date` | [`RCoerceDate`] | `as_date(&self)` |
//! | `as.POSIXct` | [`RCoercePOSIXct`] | `as_posixct(&self)` |
//! | `as.complex` | [`RCoerceComplex`] | `as_complex(&self)` |
//! | `as.raw` | [`RCoerceRaw`] | `as_raw(&self)` |
//! | `as.environment` | [`RCoerceEnvironment`] | `as_environment(&self)` |
//! | `as.function` | [`RCoerceFunction`] | `as_function(&self)` |
//!
//! # Usage with `#[miniextendr]`
//!
//! Use `#[miniextendr(as = "...")]` on impl methods to generate S3 method wrappers:
//!
//! ```ignore
//! use miniextendr_api::{miniextendr, ExternalPtr, List};
//! use miniextendr_api::r_coerce::RCoerceError;
//!
//! pub struct MyData {
//!     names: Vec<String>,
//!     values: Vec<f64>,
//! }
//!
//! #[miniextendr(s3)]
//! impl MyData {
//!     pub fn new(names: Vec<String>, values: Vec<f64>) -> Self {
//!         Self { names, values }
//!     }
//!
//!     /// Convert to data.frame
//!     #[miniextendr(as = "data.frame")]
//!     pub fn as_data_frame(&self) -> Result<List, RCoerceError> {
//!         if self.names.len() != self.values.len() {
//!             return Err(RCoerceError::InvalidData {
//!                 message: "names and values must have same length".into(),
//!             });
//!         }
//!         Ok(List::from_pairs(vec![
//!             ("name", self.names.clone()),
//!             ("value", self.values.clone()),
//!         ])
//!         .set_class_str(&["data.frame"])
//!         .set_row_names_int(self.names.len()))
//!     }
//!
//!     /// Convert to character representation
//!     #[miniextendr(as = "character")]
//!     pub fn as_character(&self) -> Result<String, RCoerceError> {
//!         Ok(format!("MyData({} items)", self.values.len()))
//!     }
//! }
//! ```
//!
//! This generates R S3 methods:
//!
//! ```r
//! # Generated automatically:
//! as.data.frame.MyData <- function(x, ...) {
//!     .Call(C_MyData__as_data_frame, .call = match.call(), x)
//! }
//!
//! as.character.MyData <- function(x, ...) {
//!     .Call(C_MyData__as_character, .call = match.call(), x)
//! }
//! ```

use std::fmt;

// region: Error Types

/// Error type for `as.<class>()` coercion failures.
///
/// This error type provides structured information about why a coercion failed,
/// allowing for meaningful error messages in R.
#[derive(Debug, Clone)]
pub enum RCoerceError {
    /// The conversion is not supported for this type combination.
    ///
    /// Use this when a type fundamentally cannot be converted to the target type
    /// (e.g., trying to convert a non-numeric type to numeric).
    NotSupported {
        /// The source type name
        from: &'static str,
        /// The target type name
        to: &'static str,
    },

    /// The conversion failed due to invalid or malformed data.
    ///
    /// Use this when the data itself prevents conversion (e.g., mismatched
    /// lengths for data.frame columns, invalid format strings).
    InvalidData {
        /// Description of what's invalid
        message: String,
    },

    /// The conversion would result in unacceptable precision loss.
    ///
    /// Use this when numeric conversion would truncate or lose significant
    /// digits beyond acceptable limits.
    PrecisionLoss {
        /// Description of the precision loss
        message: String,
    },

    /// A custom error message.
    ///
    /// Use this for domain-specific errors that don't fit the other categories.
    Custom(String),
}

impl fmt::Display for RCoerceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSupported { from, to } => {
                write!(f, "cannot coerce {} to {}", from, to)
            }
            Self::InvalidData { message } => {
                write!(f, "invalid data: {}", message)
            }
            Self::PrecisionLoss { message } => {
                write!(f, "precision loss: {}", message)
            }
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for RCoerceError {}

// Implement From<String> for convenient error creation
impl From<String> for RCoerceError {
    fn from(s: String) -> Self {
        RCoerceError::Custom(s)
    }
}

impl From<&str> for RCoerceError {
    fn from(s: &str) -> Self {
        RCoerceError::Custom(s.to_string())
    }
}
// endregion

// region: Coercion Traits

/// Trait for types that can be coerced to `data.frame` via `as.data.frame()`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::r_coerce::{RCoerceDataFrame, RCoerceError};
/// use miniextendr_api::List;
///
/// impl RCoerceDataFrame for MyStruct {
///     fn as_data_frame(&self) -> Result<List, RCoerceError> {
///         Ok(List::from_pairs(vec![
///             ("col1", self.field1.clone()),
///             ("col2", self.field2.clone()),
///         ])
///         .set_class_str(&["data.frame"])
///         .set_row_names_int(self.field1.len()))
///     }
/// }
/// ```
pub trait RCoerceDataFrame {
    /// Convert to an R data.frame.
    ///
    /// The returned List should have:
    /// - Named columns of equal length
    /// - Class attribute set to "data.frame"
    /// - row.names attribute set appropriately
    fn as_data_frame(&self) -> Result<crate::List, RCoerceError>;
}

/// Trait for types that can be coerced to `list` via `as.list()`.
///
/// # Example
///
/// ```ignore
/// impl RCoerceList for MyStruct {
///     fn as_list(&self) -> Result<List, RCoerceError> {
///         Ok(List::from_pairs(vec![
///             ("field1", self.field1.clone()),
///             ("field2", self.field2.clone()),
///         ]))
///     }
/// }
/// ```
pub trait RCoerceList {
    /// Convert to an R list.
    fn as_list(&self) -> Result<crate::List, RCoerceError>;
}

/// Trait for types that can be coerced to `character` via `as.character()`.
///
/// This typically produces a string representation of the object.
/// For single values, return a single-element vector; for collections,
/// return a vector with one element per item.
pub trait RCoerceCharacter {
    /// Convert to an R character vector.
    fn as_character(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `numeric`/`double` via `as.numeric()`.
///
/// The result should be an R numeric vector (REALSXP).
pub trait RCoerceNumeric {
    /// Convert to an R numeric vector.
    fn as_numeric(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `integer` via `as.integer()`.
///
/// The result should be an R integer vector (INTSXP).
pub trait RCoerceInteger {
    /// Convert to an R integer vector.
    fn as_integer(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `logical` via `as.logical()`.
///
/// The result should be an R logical vector (LGLSXP).
pub trait RCoerceLogical {
    /// Convert to an R logical vector.
    fn as_logical(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `matrix` via `as.matrix()`.
///
/// The result should be an R matrix with appropriate dimensions.
pub trait RCoerceMatrix {
    /// Convert to an R matrix.
    fn as_matrix(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to a generic `vector` via `as.vector()`.
///
/// This is the most general vector coercion, typically stripping attributes.
pub trait RCoerceVector {
    /// Convert to an R vector.
    fn as_vector(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `factor` via `as.factor()`.
///
/// The result should be an R factor (integer vector with levels attribute).
pub trait RCoerceFactor {
    /// Convert to an R factor.
    fn as_factor(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `Date` via `as.Date()`.
///
/// The result should be an R Date object (numeric with "Date" class).
pub trait RCoerceDate {
    /// Convert to an R Date.
    fn as_date(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `POSIXct` via `as.POSIXct()`.
///
/// The result should be an R POSIXct object (numeric with "POSIXct", "POSIXt" class).
pub trait RCoercePOSIXct {
    /// Convert to an R POSIXct.
    fn as_posixct(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `complex` via `as.complex()`.
///
/// The result should be an R complex vector (CPLXSXP).
pub trait RCoerceComplex {
    /// Convert to an R complex vector.
    fn as_complex(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `raw` via `as.raw()`.
///
/// The result should be an R raw vector (RAWSXP).
pub trait RCoerceRaw {
    /// Convert to an R raw vector.
    fn as_raw(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `environment` via `as.environment()`.
///
/// The result should be an R environment.
pub trait RCoerceEnvironment {
    /// Convert to an R environment.
    fn as_environment(&self) -> Result<crate::SEXP, RCoerceError>;
}

/// Trait for types that can be coerced to `function` via `as.function()`.
///
/// The result should be an R function (closure).
pub trait RCoerceFunction {
    /// Convert to an R function.
    fn as_function(&self) -> Result<crate::SEXP, RCoerceError>;
}
// endregion

// region: Helper Functions

/// Maps an R generic name to the corresponding trait method name.
///
/// This is used by the proc-macro to validate `#[miniextendr(as = "...")]` attributes.
///
/// # Returns
///
/// The Rust method name that corresponds to the R generic, or `None` if the
/// generic is not supported.
pub const fn r_generic_to_method(generic: &str) -> Option<&'static str> {
    // Use a match on string slices. We can't use HashMap in const fn.
    // This is a compile-time lookup table.
    Some(match generic.as_bytes() {
        b"data.frame" => "as_data_frame",
        b"list" => "as_list",
        b"character" => "as_character",
        b"numeric" | b"double" => "as_numeric",
        b"integer" => "as_integer",
        b"logical" => "as_logical",
        b"matrix" => "as_matrix",
        b"vector" => "as_vector",
        b"factor" => "as_factor",
        b"Date" => "as_date",
        b"POSIXct" => "as_posixct",
        b"complex" => "as_complex",
        b"raw" => "as_raw",
        b"environment" => "as_environment",
        b"function" => "as_function",
        _ => return None,
    })
}

/// All supported R coercion generics.
///
/// This list can be used to validate user input or generate documentation.
pub const SUPPORTED_AS_GENERICS: &[&str] = &[
    "data.frame",
    "list",
    "character",
    "numeric",
    "double",
    "integer",
    "logical",
    "matrix",
    "vector",
    "factor",
    "Date",
    "POSIXct",
    "complex",
    "raw",
    "environment",
    "function",
];

/// Check if a generic name is a supported `as.<class>()` generic.
#[inline]
pub fn is_supported_as_generic(generic: &str) -> bool {
    SUPPORTED_AS_GENERICS.contains(&generic)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RCoerceError::NotSupported {
            from: "MyType",
            to: "data.frame",
        };
        assert_eq!(err.to_string(), "cannot coerce MyType to data.frame");

        let err = RCoerceError::InvalidData {
            message: "column lengths differ".to_string(),
        };
        assert_eq!(err.to_string(), "invalid data: column lengths differ");

        let err = RCoerceError::Custom("something went wrong".to_string());
        assert_eq!(err.to_string(), "something went wrong");
    }

    #[test]
    fn test_supported_generics() {
        assert!(is_supported_as_generic("data.frame"));
        assert!(is_supported_as_generic("list"));
        assert!(is_supported_as_generic("character"));
        assert!(is_supported_as_generic("numeric"));
        assert!(is_supported_as_generic("double"));
        assert!(!is_supported_as_generic("foo"));
        assert!(!is_supported_as_generic(""));
    }

    #[test]
    fn test_generic_to_method() {
        assert_eq!(r_generic_to_method("data.frame"), Some("as_data_frame"));
        assert_eq!(r_generic_to_method("list"), Some("as_list"));
        assert_eq!(r_generic_to_method("numeric"), Some("as_numeric"));
        assert_eq!(r_generic_to_method("double"), Some("as_numeric"));
        assert_eq!(r_generic_to_method("foo"), None);
    }
}
// endregion
