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

use crate::into_r::into_r_infallible;

// region: TryFromSexp / IntoR conversions

crate::try_from_sexp_via_str_parse!(Uuid, "UUID", |s| Uuid::parse_str(s));

into_r_infallible!(Uuid, |this| this.to_string().into_sexp());
into_r_infallible!(Option<Uuid>, |this| this.map(|u| u.to_string()).into_sexp());
into_r_infallible!(Vec<Uuid>, |this| this
    .into_iter()
    .map(|u| u.to_string())
    .collect::<Vec<_>>()
    .into_sexp());
into_r_infallible!(Vec<Option<Uuid>>, |this| this
    .into_iter()
    .map(|opt| opt.map(|u| u.to_string()))
    .collect::<Vec<_>>()
    .into_sexp());
// endregion

// region: RUuidOps adapter trait

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
        // The UUID version is a 4-bit field (0..=15), so it always fits in i32.
        i32::try_from(self.get_version_num()).expect("UUID version fits in i32")
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
        let arr: [u8; 16] = bytes.try_into().expect("length checked == 16");
        Ok(Uuid::from_bytes(arr))
    }

    /// Parse a UUID from a string (any format).
    pub fn parse_str(s: &str) -> Result<Uuid, String> {
        Uuid::parse_str(s).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests;
// endregion
