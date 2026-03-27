//! Error types for fallible [`IntoR`](crate::into_r::IntoR) conversions.
//!
//! [`IntoRError`] is returned by `try_into_sexp` when conversion fails
//! (e.g., string exceeds R's `i32` length limit).

/// Error returned by [`IntoR::try_into_sexp`](crate::into_r::IntoR::try_into_sexp)
/// for types whose conversion to R can fail.
///
/// # Variants
///
/// - `StringTooLong` — a Rust string exceeds R's `i32` length limit (~2 GB)
/// - `LengthOverflow` — a collection length exceeds R's `R_xlen_t` capacity
/// - `Inner` — a sub-conversion failed (wraps the inner error message)
#[derive(Debug, Clone)]
pub enum IntoRError {
    /// A string's byte length exceeds `i32::MAX`.
    StringTooLong {
        /// Actual byte length of the string.
        len: usize,
    },
    /// A collection's element count exceeds the target R vector capacity.
    LengthOverflow {
        /// Actual element count.
        len: usize,
    },
    /// A nested conversion failed.
    Inner(String),
}

impl std::fmt::Display for IntoRError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntoRError::StringTooLong { len } => {
                write!(
                    f,
                    "string byte length {} exceeds R's i32 limit ({})",
                    len,
                    i32::MAX
                )
            }
            IntoRError::LengthOverflow { len } => {
                write!(f, "collection length {} overflows R vector capacity", len)
            }
            IntoRError::Inner(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for IntoRError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_string_too_long() {
        let e = IntoRError::StringTooLong { len: 3_000_000_000 };
        assert!(e.to_string().contains("3000000000"));
        assert!(e.to_string().contains("i32 limit"));
    }

    #[test]
    fn display_length_overflow() {
        let e = IntoRError::LengthOverflow { len: 42 };
        assert!(e.to_string().contains("42"));
    }

    #[test]
    fn display_inner() {
        let e = IntoRError::Inner("nested failure".to_string());
        assert_eq!(e.to_string(), "nested failure");
    }
}
