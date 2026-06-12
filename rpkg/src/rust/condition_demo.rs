//! Demo fixtures for the condition macro system.
//!
//! These functions exercise `error!()`, `warning!()`, `message!()`, and
//! `condition!()` macros including the optional `class = "..."` form.
//! Tests live in `rpkg/tests/testthat/test-conditions.R`.

use miniextendr_api::miniextendr;

// Type alias avoids ambiguous-associated-type errors when using enum variants
// (RCondition impls TryFrom/IntoR which have `Error`/`Condition` assoc types).
type RCondition = miniextendr_api::condition::RCondition;

// region: error! fixtures

/// Raise a rust_error with the standard class layering.
///
/// @export
#[miniextendr]
pub fn demo_error(msg: &str) {
    miniextendr_api::error!("{msg}");
}

/// Raise a rust_error with a custom class prepended.
///
/// @export
#[miniextendr]
pub fn demo_error_custom_class(class: &str, msg: &str) {
    // Can't use a runtime string as the `class =` argument in the macro because
    // the macro takes a literal. Use the enum directly for the variable-class case.
    std::panic::panic_any(RCondition::Error {
        message: msg.to_string(),
        class: Some(class.to_string()),
        data: None,
    });
}

// endregion

// region: warning! fixtures

/// Raise a rust_warning.
///
/// @export
#[miniextendr]
pub fn demo_warning(msg: &str) {
    miniextendr_api::warning!("{msg}");
}

/// Raise a rust_warning with a custom class prepended.
///
/// @export
#[miniextendr]
pub fn demo_warning_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(RCondition::Warning {
        message: msg.to_string(),
        class: Some(class.to_string()),
        data: None,
    });
}

// endregion

// region: message! fixtures

/// Emit a rust_message.
///
/// @export
#[miniextendr]
pub fn demo_message(msg: &str) {
    miniextendr_api::message!("{msg}");
}

// endregion

// region: condition! fixtures

/// Signal a generic rust_condition (no-op if unhandled).
///
/// @export
#[miniextendr]
pub fn demo_condition(msg: &str) {
    miniextendr_api::condition!("{msg}");
}

/// Signal a rust_condition with a custom class.
///
/// @export
#[miniextendr]
pub fn demo_condition_custom_class(class: &str, msg: &str) {
    std::panic::panic_any(RCondition::Condition {
        message: msg.to_string(),
        class: Some(class.to_string()),
        data: None,
    });
}

// endregion

// region: data = ... payload fixtures (issue #346)

/// Raise a classed error carrying a single structured field (`value`).
///
/// Handlers can read `e$value` instead of parsing the message string.
///
/// @export
#[miniextendr]
pub fn demo_error_data_scalar(value: i32) {
    miniextendr_api::error!(
        class = "range_error",
        data = ("value", value),
        "value {value} out of range"
    );
}

/// Raise a classed error carrying several heterogeneous scalar fields.
///
/// Demonstrates the bracketed `data = [(..), (..)]` form with mixed value
/// types: integer, double, logical, and character.
///
/// @export
#[miniextendr]
pub fn demo_error_data_multi(value: f64, code: i32, label: &str) {
    miniextendr_api::error!(
        class = "validation_error",
        data = [
            ("value", value),
            ("code", code),
            ("label", label),
            ("fatal", true)
        ],
        "validation failed for {label}"
    );
}

/// Raise a classed error whose data field is a vector.
///
/// @export
#[miniextendr]
pub fn demo_error_data_vector(values: Vec<i32>) {
    miniextendr_api::error!(
        class = "batch_error",
        data = ("offending", values),
        "batch contained out-of-range values"
    );
}

/// Raise a classed warning carrying structured data.
///
/// @export
#[miniextendr]
pub fn demo_warning_data(count: i32) {
    miniextendr_api::warning!(
        class = "truncation_warning",
        data = ("dropped", count),
        "dropped {count} rows"
    );
}

/// Emit a message carrying structured data.
///
/// @export
#[miniextendr]
pub fn demo_message_data(step: i32) {
    miniextendr_api::message!(data = ("step", step), "progress at step {step}");
}

/// Signal a classed condition carrying structured data.
///
/// @export
#[miniextendr]
pub fn demo_condition_data(n: i32) {
    miniextendr_api::condition!(
        class = "progress",
        data = ("processed", n),
        "processed {n} items"
    );
}

// endregion
