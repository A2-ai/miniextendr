//! Adapter traits for direct R serialization/deserialization.
//!
//! These traits provide the bridge between Rust types and R objects,
//! similar to how `RSerialize`/`RDeserialize` work for JSON.

use super::de::RDeserializer;
use super::error::RSerdeError;
use super::ser::RSerializer;
use crate::ffi::SEXP;

/// Adapter trait for direct R serialization (Rust -> R SEXP).
///
/// This trait provides methods to serialize Rust types directly to R objects
/// without going through an intermediate format like JSON.
///
/// # Type Mappings
///
/// | Rust Type | R Type |
/// |-----------|--------|
/// | `bool` | `logical(1)` |
/// | `i32` | `integer(1)` |
/// | `f64` | `numeric(1)` |
/// | `String` | `character(1)` |
/// | `Option<T>::None` | NA or NULL |
/// | `Vec<primitive>` | atomic vector |
/// | `Vec<struct>` | list of lists |
/// | `HashMap<String, T>` | named list |
/// | `struct` | named list |
///
/// # Registration
///
/// ```rust,ignore
/// use miniextendr_api::serde_r::RSerializeNative;
/// use serde::Serialize;
///
/// #[derive(Serialize, ExternalPtr)]
/// struct Config { name: String, value: i32 }
///
/// #[miniextendr]
/// impl RSerializeNative for Config {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RSerializeNative for Config;
/// }
/// ```
///
/// # R Usage
///
/// ```r
/// cfg <- Config$new("test", 42L)
/// data <- cfg$to_r()
/// # Returns: list(name = "test", value = 42L)
///
/// # Access fields directly
/// data$name   # "test"
/// data$value  # 42L
/// ```
pub trait RSerializeNative {
    /// Serialize this value to a native R object.
    ///
    /// # Returns
    ///
    /// - `Ok(SEXP)` - The serialized R object
    /// - `Err(String)` - Error message if serialization fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let point = Point { x: 1.0, y: 2.0 };
    /// let sexp = point.to_r()?;
    /// // sexp is now list(x = 1.0, y = 2.0)
    /// ```
    fn to_r(&self) -> Result<SEXP, String>;
}

/// Blanket implementation for all `Serialize` types.
impl<T: serde::Serialize> RSerializeNative for T {
    fn to_r(&self) -> Result<SEXP, String> {
        RSerializer::to_sexp(self).map_err(|e| e.to_string())
    }
}

/// Adapter trait for direct R deserialization (R SEXP -> Rust).
///
/// This trait provides methods to deserialize R objects directly to Rust types
/// without going through an intermediate format like JSON.
///
/// # Type Mappings
///
/// | R Type | Rust Type |
/// |--------|-----------|
/// | `logical(1)` | `bool` |
/// | `integer(1)` | `i32` |
/// | `numeric(1)` | `f64` |
/// | `character(1)` | `String` |
/// | NA values | `Option<T>::None` |
/// | atomic vectors | `Vec<primitive>` |
/// | lists | `Vec<T>` |
/// | named lists | struct or `HashMap<String, T>` |
/// | NULL | `()` or `Option<T>::None` |
///
/// # Registration
///
/// ```rust,ignore
/// use miniextendr_api::serde_r::RDeserializeNative;
/// use serde::Deserialize;
///
/// #[derive(Deserialize, ExternalPtr)]
/// struct Config { name: String, value: i32 }
///
/// #[miniextendr]
/// impl RDeserializeNative for Config {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RDeserializeNative for Config;
/// }
/// ```
///
/// # R Usage
///
/// ```r
/// # Create from R list
/// data <- list(name = "test", value = 42L)
/// cfg <- Config$from_r(data)
///
/// # Now cfg is a Config external pointer
/// cfg$name   # "test"
/// cfg$value  # 42L
/// ```
pub trait RDeserializeNative: Sized {
    /// Deserialize from an R object.
    ///
    /// # Arguments
    ///
    /// * `sexp` - The R object to deserialize from
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` - The deserialized Rust value
    /// - `Err(String)` - Error message if deserialization fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Given list(x = 1.0, y = 2.0) from R:
    /// let point = Point::from_r(sexp)?;
    /// assert_eq!(point.x, 1.0);
    /// assert_eq!(point.y, 2.0);
    /// ```
    fn from_r(sexp: SEXP) -> Result<Self, String>;
}

/// Blanket implementation for all `Deserialize` types.
impl<T: for<'de> serde::Deserialize<'de>> RDeserializeNative for T {
    fn from_r(sexp: SEXP) -> Result<Self, String> {
        RDeserializer::from_sexp_to(sexp).map_err(|e| e.to_string())
    }
}

/// Convenience function to serialize a value to R.
///
/// # Example
///
/// ```rust,ignore
/// use miniextendr_api::serde_r::to_r;
///
/// let point = Point { x: 1.0, y: 2.0 };
/// let sexp = to_r(&point)?;
/// ```
pub fn to_r<T: serde::Serialize>(value: &T) -> Result<SEXP, RSerdeError> {
    RSerializer::to_sexp(value)
}

/// Convenience function to deserialize from R.
///
/// # Example
///
/// ```rust,ignore
/// use miniextendr_api::serde_r::from_r;
///
/// let point: Point = from_r(sexp)?;
/// ```
pub fn from_r<T: for<'de> serde::Deserialize<'de>>(sexp: SEXP) -> Result<T, RSerdeError> {
    RDeserializer::from_sexp_to(sexp)
}
