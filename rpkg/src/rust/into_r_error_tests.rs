//! Test fixtures for into_r_error (IntoRError type).

use miniextendr_api::into_r_error::IntoRError;
use miniextendr_api::prelude::*;

/// Trigger a StringTooLong error and return its message.
#[miniextendr]
pub fn into_r_error_string_too_long() -> String {
    let err = IntoRError::StringTooLong { len: usize::MAX };
    format!("{}", err)
}

/// Trigger a LengthOverflow error and return its message.
#[miniextendr]
pub fn into_r_error_length_overflow() -> String {
    let err = IntoRError::LengthOverflow { len: usize::MAX };
    format!("{}", err)
}

/// Trigger an Inner error and return its message.
#[miniextendr]
pub fn into_r_error_inner() -> String {
    let err = IntoRError::Inner("custom error message".to_string());
    format!("{}", err)
}
