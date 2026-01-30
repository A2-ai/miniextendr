//! Error types for R serialization/deserialization.

use std::fmt;

/// Error type for R serialization/deserialization.
///
/// This error type implements both `serde::ser::Error` and `serde::de::Error`,
/// allowing it to be used in both serialization and deserialization contexts.
#[derive(Debug, Clone)]
pub enum RSerdeError {
    /// Generic message error (from serde's `Error::custom`).
    Message(String),

    /// Type mismatch during deserialization.
    TypeMismatch {
        /// The expected Rust type.
        expected: &'static str,
        /// The actual R type encountered.
        actual: String,
    },

    /// Missing field in struct deserialization.
    MissingField(String),

    /// Invalid enum variant during deserialization.
    InvalidVariant {
        /// The variant name that was found.
        variant: String,
        /// The expected variant names.
        expected: Vec<&'static str>,
    },

    /// Length mismatch (e.g., tuple deserialization).
    LengthMismatch {
        /// Expected length.
        expected: usize,
        /// Actual length.
        actual: usize,
    },

    /// NA value encountered where not allowed.
    UnexpectedNa,

    /// Value overflow during numeric conversion.
    Overflow {
        /// The source type name.
        from: &'static str,
        /// The target type name.
        to: &'static str,
    },

    /// Invalid UTF-8 in R string.
    InvalidUtf8,

    /// Key was not a string (required for R named lists).
    NonStringKey,

    /// Unsupported R type for deserialization.
    UnsupportedType {
        /// The R SEXPTYPE code.
        sexptype: i32,
    },
}

impl serde::ser::Error for RSerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        RSerdeError::Message(msg.to_string())
    }
}

impl serde::de::Error for RSerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        RSerdeError::Message(msg.to_string())
    }
}

impl fmt::Display for RSerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RSerdeError::Message(msg) => write!(f, "{}", msg),
            RSerdeError::TypeMismatch { expected, actual } => {
                write!(f, "type mismatch: expected {}, got {}", expected, actual)
            }
            RSerdeError::MissingField(field) => {
                write!(f, "missing field: {}", field)
            }
            RSerdeError::InvalidVariant { variant, expected } => {
                write!(
                    f,
                    "invalid variant '{}', expected one of: {}",
                    variant,
                    expected.join(", ")
                )
            }
            RSerdeError::LengthMismatch { expected, actual } => {
                write!(f, "length mismatch: expected {}, got {}", expected, actual)
            }
            RSerdeError::UnexpectedNa => {
                write!(f, "unexpected NA value")
            }
            RSerdeError::Overflow { from, to } => {
                write!(f, "overflow converting {} to {}", from, to)
            }
            RSerdeError::InvalidUtf8 => {
                write!(f, "invalid UTF-8 in R string")
            }
            RSerdeError::NonStringKey => {
                write!(f, "map keys must be strings for R named lists")
            }
            RSerdeError::UnsupportedType { sexptype } => {
                write!(f, "unsupported R type: SEXPTYPE {}", sexptype)
            }
        }
    }
}

impl std::error::Error for RSerdeError {}
