//! Integration with the `borsh` crate for binary serialization.
//!
//! Provides the [`Borsh`] newtype wrapper for converting between Rust types
//! and R raw vectors (RAWSXP) via borsh binary format.
//!
//! # Features
//!
//! Enable this module with the `borsh` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["borsh"] }
//! ```
//!
//! # Standalone helpers
//!
//! | Function | Input | Output |
//! |----------|-------|--------|
//! | `borsh_to_raw` | `&T` | R raw vector (SEXP) |
//! | `borsh_from_raw` | R raw vector (SEXP) | `Result<T, SexpError>` |

use crate::ffi::SEXP;
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;

// region: Borsh<T> wrapper type

/// Wrapper for borsh-serializable types.
///
/// Converts between R raw vectors (RAWSXP) and borsh binary format.
/// Use `Borsh(value)` to wrap a value for conversion.
pub struct Borsh<T>(pub T);
// endregion

// region: IntoR: Borsh<T> -> RAWSXP

/// Convert `Borsh<T>` to R raw vector (RAWSXP) via borsh serialization.
impl<T: borsh::BorshSerialize> IntoR for Borsh<T> {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        let bytes = borsh::to_vec(&self.0).expect("borsh serialization failed");
        Ok(bytes.into_sexp())
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        let bytes = borsh::to_vec(&self.0).expect("borsh serialization failed");
        Ok(unsafe { bytes.into_sexp_unchecked() })
    }
}
// endregion

// region: TryFromSexp: RAWSXP -> Borsh<T>

/// Convert R raw vector (RAWSXP) to `Borsh<T>` via borsh deserialization.
impl<T: borsh::BorshDeserialize> TryFromSexp for Borsh<T> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let bytes: Vec<u8> = TryFromSexp::try_from_sexp(sexp)?;
        let value = borsh::from_slice(&bytes)
            .map_err(|e| SexpError::InvalidValue(format!("borsh deserialization failed: {e}")))?;
        Ok(Borsh(value))
    }

    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        let bytes: Vec<u8> = unsafe { TryFromSexp::try_from_sexp_unchecked(sexp)? };
        let value = borsh::from_slice(&bytes)
            .map_err(|e| SexpError::InvalidValue(format!("borsh deserialization failed: {e}")))?;
        Ok(Borsh(value))
    }
}
// endregion

// region: RBorshOps adapter trait

/// Adapter trait for borsh serialization from R.
///
/// Provides method-style access to borsh serialization operations.
/// This trait has a blanket implementation for all types that implement
/// both `BorshSerialize` and `BorshDeserialize`.
pub trait RBorshOps: borsh::BorshSerialize + borsh::BorshDeserialize + Sized {
    /// Serialize to raw bytes.
    fn borsh_serialize(&self) -> Vec<u8>;
    /// Deserialize from raw bytes.
    fn borsh_deserialize(bytes: &[u8]) -> Result<Self, String>;
    /// Size of serialized form in bytes.
    fn borsh_size(&self) -> usize;
}

impl<T: borsh::BorshSerialize + borsh::BorshDeserialize> RBorshOps for T {
    #[inline]
    fn borsh_serialize(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("borsh serialization failed")
    }

    #[inline]
    fn borsh_deserialize(bytes: &[u8]) -> Result<Self, String> {
        borsh::from_slice(bytes).map_err(|e| format!("borsh deserialization failed: {e}"))
    }

    #[inline]
    fn borsh_size(&self) -> usize {
        borsh::to_vec(self)
            .expect("borsh serialization failed")
            .len()
    }
}
// endregion

// region: Standalone helper functions

/// Serialize a borsh value to R raw vector.
#[inline]
pub fn borsh_to_raw<T: borsh::BorshSerialize>(value: &T) -> SEXP {
    let bytes = borsh::to_vec(value).expect("borsh serialization failed");
    bytes.into_sexp()
}

/// Deserialize from R raw vector.
#[inline]
pub fn borsh_from_raw<T: borsh::BorshDeserialize>(sexp: SEXP) -> Result<T, SexpError> {
    let bytes: Vec<u8> = TryFromSexp::try_from_sexp(sexp)?;
    borsh::from_slice(&bytes)
        .map_err(|e| SexpError::InvalidValue(format!("borsh deserialization failed: {e}")))
}
// endregion

// region: Unit tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borsh_serialize_roundtrip_vec() {
        let original: Vec<u8> = vec![1, 2, 3, 4, 5];
        let bytes = original.borsh_serialize();
        let recovered: Vec<u8> = RBorshOps::borsh_deserialize(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_borsh_serialize_roundtrip_tuple() {
        let original: (f64, f64, f64) = (1.5, 2.5, 3.5);
        let bytes = original.borsh_serialize();
        let recovered: (f64, f64, f64) = RBorshOps::borsh_deserialize(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_borsh_serialize_roundtrip_string() {
        let original = String::from("hello borsh");
        let bytes = original.borsh_serialize();
        let recovered: String = RBorshOps::borsh_deserialize(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_borsh_serialize_empty_vec() {
        let original: Vec<i32> = vec![];
        let bytes = original.borsh_serialize();
        let recovered: Vec<i32> = RBorshOps::borsh_deserialize(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_borsh_size() {
        let value: u32 = 42;
        let size = value.borsh_size();
        assert_eq!(size, 4);
    }

    #[test]
    fn test_borsh_size_string() {
        let value = String::from("hi");
        let size = value.borsh_size();
        // borsh encodes strings as: 4-byte length prefix + utf8 bytes
        assert_eq!(size, 4 + 2);
    }

    #[test]
    fn test_borsh_deserialize_invalid_data() {
        let bad_bytes: &[u8] = &[0xff, 0xff];
        let result: Result<String, _> = RBorshOps::borsh_deserialize(bad_bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("borsh deserialization failed"));
    }

    #[test]
    fn test_borsh_roundtrip_nested() {
        let original: Vec<(String, Vec<u8>)> = vec![
            ("first".to_string(), vec![1, 2, 3]),
            ("second".to_string(), vec![4, 5]),
        ];
        let bytes = original.borsh_serialize();
        let recovered: Vec<(String, Vec<u8>)> = RBorshOps::borsh_deserialize(&bytes).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_borsh_roundtrip_option() {
        let some_val: Option<u64> = Some(42);
        let none_val: Option<u64> = None;

        let some_bytes = some_val.borsh_serialize();
        let none_bytes = none_val.borsh_serialize();

        let recovered_some: Option<u64> = RBorshOps::borsh_deserialize(&some_bytes).unwrap();
        let recovered_none: Option<u64> = RBorshOps::borsh_deserialize(&none_bytes).unwrap();

        assert_eq!(some_val, recovered_some);
        assert_eq!(none_val, recovered_none);
    }
}
// endregion
