//! Integration with the `uuid` crate.
//!
//! Provides conversions between R character vectors and `Uuid` types.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `character(1)` | `Uuid` | Single UUID string |
//! | `character` | `Vec<Uuid>` | Vector of UUID strings |
//! | `NA_character_` | `Option<Uuid>` | NA maps to None |
//!
//! # Features
//!
//! Enable this module with the `uuid` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["uuid"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use uuid::Uuid;
//!
//! #[miniextendr]
//! fn generate_uuid() -> Uuid {
//!     Uuid::new_v4()
//! }
//!
//! #[miniextendr]
//! fn parse_uuid(s: String) -> Option<Uuid> {
//!     Uuid::parse_str(&s).ok()
//! }
//! ```

pub use uuid::Uuid;

use crate::ffi::{SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

// =============================================================================
// Scalar conversions
// =============================================================================

impl TryFromSexp for Uuid {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let s: String = TryFromSexp::try_from_sexp(sexp)?;
        Uuid::parse_str(&s).map_err(|e| SexpError::InvalidValue(format!("invalid UUID: {}", e)))
    }
}

impl IntoR for Uuid {
    fn into_sexp(self) -> SEXP {
        self.to_string().into_sexp()
    }
}

// =============================================================================
// Option conversions (NA support)
// =============================================================================

impl TryFromSexp for Option<Uuid> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let opt: Option<String> = TryFromSexp::try_from_sexp(sexp)?;
        match opt {
            None => Ok(None),
            Some(s) => Uuid::parse_str(&s)
                .map(Some)
                .map_err(|e| SexpError::InvalidValue(format!("invalid UUID: {}", e))),
        }
    }
}

impl IntoR for Option<Uuid> {
    fn into_sexp(self) -> SEXP {
        self.map(|u| u.to_string()).into_sexp()
    }
}

// =============================================================================
// Vector conversions
// =============================================================================

impl TryFromSexp for Vec<Uuid> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        use crate::ffi::{Rf_translateCharUTF8, STRING_ELT};

        let actual = sexp.type_of();
        if actual != SEXPTYPE::STRSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::STRSXP,
                actual,
            }
            .into());
        }

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);

        for i in 0..len {
            let charsxp = unsafe { STRING_ELT(sexp, i as crate::ffi::R_xlen_t) };

            // Check for NA
            if charsxp == unsafe { crate::ffi::R_NaString } {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Uuid>",
                    i
                )));
            }

            let c_str = unsafe { Rf_translateCharUTF8(charsxp) };
            let s = if c_str.is_null() {
                String::new()
            } else {
                unsafe { std::ffi::CStr::from_ptr(c_str) }
                    .to_str()
                    .map_err(|_| SexpError::InvalidValue(format!("invalid UTF-8 at index {}", i)))?
                    .to_owned()
            };

            let uuid = Uuid::parse_str(&s).map_err(|e| {
                SexpError::InvalidValue(format!("invalid UUID at index {}: {}", i, e))
            })?;
            result.push(uuid);
        }

        Ok(result)
    }
}

impl IntoR for Vec<Uuid> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

// =============================================================================
// Vec<Option<Uuid>> conversions (NA-aware vectors)
// =============================================================================

impl TryFromSexp for Vec<Option<Uuid>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let strings: Vec<Option<String>> = TryFromSexp::try_from_sexp(sexp)?;
        strings
            .into_iter()
            .enumerate()
            .map(|(i, opt)| match opt {
                None => Ok(None),
                Some(s) => Uuid::parse_str(&s).map(Some).map_err(|e| {
                    SexpError::InvalidValue(format!("invalid UUID at index {}: {}", i, e))
                }),
            })
            .collect()
    }
}

impl IntoR for Vec<Option<Uuid>> {
    fn into_sexp(self) -> SEXP {
        self.into_iter()
            .map(|opt| opt.map(|u| u.to_string()))
            .collect::<Vec<_>>()
            .into_sexp()
    }
}

// =============================================================================
// RUuidOps adapter trait
// =============================================================================

/// Adapter trait for [`Uuid`] operations.
///
/// Provides UUID inspection and generation methods from R.
/// Automatically implemented for `uuid::Uuid`.
///
/// # Example
///
/// ```rust,ignore
/// use uuid::Uuid;
/// use miniextendr_api::uuid_impl::RUuidOps;
///
/// #[derive(ExternalPtr)]
/// struct MyUuid(Uuid);
///
/// #[miniextendr]
/// impl RUuidOps for MyUuid {}
///
/// miniextendr_module! {
///     mod mymodule;
///     impl RUuidOps for MyUuid;
/// }
/// ```
///
/// In R:
/// ```r
/// id <- MyUuid$new_v4()
/// id$version()       # 4
/// id$is_nil()        # FALSE
/// id$as_bytes()      # raw(16)
/// id$to_hyphenated() # "550e8400-e29b-41d4-a716-446655440000"
/// ```
pub trait RUuidOps {
    /// Get the UUID version number (1-8, or 0 for nil).
    fn version(&self) -> i32;

    /// Get the UUID variant.
    fn variant(&self) -> String;

    /// Check if this is a nil (all zeros) UUID.
    fn is_nil(&self) -> bool;

    /// Check if this is the max (all ones) UUID.
    fn is_max(&self) -> bool;

    /// Get the UUID as a 16-byte raw vector.
    fn as_bytes(&self) -> Vec<u8>;

    /// Get the UUID as hyphenated string (standard format).
    fn to_hyphenated(&self) -> String;

    /// Get the UUID as simple string (no hyphens).
    fn to_simple(&self) -> String;

    /// Get the UUID as URN format (urn:uuid:...).
    fn to_urn(&self) -> String;
}

impl RUuidOps for Uuid {
    fn version(&self) -> i32 {
        self.get_version_num() as i32
    }

    fn variant(&self) -> String {
        format!("{:?}", self.get_variant())
    }

    fn is_nil(&self) -> bool {
        Uuid::is_nil(self)
    }

    fn is_max(&self) -> bool {
        Uuid::is_max(self)
    }

    fn as_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn to_hyphenated(&self) -> String {
        self.hyphenated().to_string()
    }

    fn to_simple(&self) -> String {
        self.simple().to_string()
    }

    fn to_urn(&self) -> String {
        self.urn().to_string()
    }
}

/// Helper functions for UUID creation.
///
/// These are standalone functions rather than trait methods since they don't
/// take `&self`. Use them directly from R via exported functions.
pub mod uuid_helpers {
    use super::Uuid;

    /// Generate a new random (v4) UUID.
    pub fn new_v4() -> Uuid {
        Uuid::new_v4()
    }

    /// Get the nil UUID (all zeros).
    pub fn nil() -> Uuid {
        Uuid::nil()
    }

    /// Get the max UUID (all ones).
    pub fn max() -> Uuid {
        Uuid::max()
    }

    /// Parse a UUID from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Uuid, String> {
        if bytes.len() != 16 {
            return Err(format!("expected 16 bytes, got {}", bytes.len()));
        }
        let arr: [u8; 16] = bytes.try_into().unwrap();
        Ok(Uuid::from_bytes(arr))
    }

    /// Parse a UUID from a string (any format).
    pub fn parse_str(s: &str) -> Result<Uuid, String> {
        Uuid::parse_str(s).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_can_be_created() {
        let uuid = Uuid::new_v4();
        assert_eq!(uuid.to_string().len(), 36); // UUID format: 8-4-4-4-12
    }

    #[test]
    fn uuid_can_be_parsed() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(uuid.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn ruuidops_version() {
        let v4 = Uuid::new_v4();
        assert_eq!(RUuidOps::version(&v4), 4);

        let nil = Uuid::nil();
        // Nil UUID has no version
        assert_eq!(RUuidOps::version(&nil), 0);
    }

    #[test]
    fn ruuidops_nil_max() {
        let nil = Uuid::nil();
        assert!(RUuidOps::is_nil(&nil));
        assert!(!RUuidOps::is_max(&nil));

        let max = Uuid::max();
        assert!(!RUuidOps::is_nil(&max));
        assert!(RUuidOps::is_max(&max));

        let v4 = Uuid::new_v4();
        assert!(!RUuidOps::is_nil(&v4));
        assert!(!RUuidOps::is_max(&v4));
    }

    #[test]
    fn ruuidops_bytes() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = RUuidOps::as_bytes(&uuid);
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes[0], 0x55);
    }

    #[test]
    fn ruuidops_formats() {
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        assert_eq!(
            RUuidOps::to_hyphenated(&uuid),
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(
            RUuidOps::to_simple(&uuid),
            "550e8400e29b41d4a716446655440000"
        );
        assert_eq!(
            RUuidOps::to_urn(&uuid),
            "urn:uuid:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn uuid_helpers_work() {
        let v4 = uuid_helpers::new_v4();
        assert!(!v4.is_nil());

        let nil = uuid_helpers::nil();
        assert!(nil.is_nil());

        let max = uuid_helpers::max();
        assert!(max.is_max());

        let from_bytes = uuid_helpers::from_bytes(vec![0u8; 16]).unwrap();
        assert!(from_bytes.is_nil());

        let invalid_bytes = uuid_helpers::from_bytes(vec![0u8; 10]);
        assert!(invalid_bytes.is_err());

        let parsed = uuid_helpers::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(parsed.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }
}
