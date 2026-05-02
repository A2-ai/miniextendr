//! Demo fixtures for the condition macro system.
//!
//! These functions exercise `error!()`, `warning!()`, `message!()`, and
//! `condition!()` macros including the optional `class = "..."` form.
//! Tests live in `rpkg/tests/testthat/test-conditions.R`.

use miniextendr_api::{condition, error, message, miniextendr, warning};

// region: error! fixtures

/// Raise a rust_error with the standard class layering.
///
/// @export
#[miniextendr]
pub fn demo_error(msg: &str) {
    error!("{msg}");
}

/// Raise a rust_error with a custom class prepended.
///
/// @export
#[miniextendr]
pub fn demo_error_custom_class(class: &str, msg: &str) {
    // Can't use a runtime string as the `class =` argument in the macro because
    // the macro takes a literal. Use the enum directly for the variable-class case.
    std::panic::panic_any(miniextendr_api::condition::RCondition::Error {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

// endregion

// region: warning! fixtures

/// Raise a rust_warning.
///
/// @export
#[miniextendr]
pub fn demo_warning(msg: &str) {
    warning!("{msg}");
}

/// Raise a rust_warning with a custom class prepended.
///
/// @export
#[miniextendr]
pub fn demo_warning_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(miniextendr_api::condition::RCondition::Warning {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

// endregion

// region: message! fixtures

/// Emit a rust_message.
///
/// @export
#[miniextendr]
pub fn demo_message(msg: &str) {
    message!("{msg}");
}

// endregion

// region: condition! fixtures

/// Signal a generic rust_condition (no-op if unhandled).
///
/// @export
#[miniextendr]
pub fn demo_condition(msg: &str) {
    condition!("{msg}");
}

/// Signal a rust_condition with a custom class.
///
/// @export
#[miniextendr]
pub fn demo_condition_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(miniextendr_api::condition::RCondition::Condition {
        message: msg.to_string(),
        class: Some(class.to_string()),
    });
}

// endregion
