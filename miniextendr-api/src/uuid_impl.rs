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
}
