//! Test fixtures for AsRError structured error adapter.

use miniextendr_api::condition::{AsRError, ConditionCall, RCondition, catch_condition};
use miniextendr_api::{RValue, miniextendr};

// region: Simple errors

#[miniextendr(noexport)]
pub fn test_condition_parse_int(s: &str) -> Result<i32, AsRError<std::num::ParseIntError>> {
    s.parse::<i32>().map_err(AsRError)
}

#[miniextendr(noexport)]
pub fn test_condition_ok() -> Result<i32, AsRError<std::num::ParseIntError>> {
    Ok(42)
}

// endregion

// region: Chained errors (custom error with source)

#[derive(Debug)]
pub struct ConfigError {
    msg: String,
    source: std::num::ParseIntError,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "config error: {}", self.msg)
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[miniextendr(noexport)]
pub fn test_condition_chained(s: &str) -> Result<i32, AsRError<ConfigError>> {
    let value = s.parse::<i32>().map_err(|e| {
        AsRError(ConfigError {
            msg: format!("failed to parse '{s}' as max_threads"),
            source: e,
        })
    })?;
    Ok(value)
}

// endregion

// region: Cause chains (`RCondition::with_parent`, #2146)

fn parse_setting(line: &str) -> i32 {
    line.trim().parse().unwrap_or_else(|_| {
        miniextendr_api::error!(
            class = "pkg_error_parse",
            data = ("line", line),
            "not a number: {line}"
        )
    })
}

/// The `with_parent` example in `docs/CONDITIONS.md`.
#[miniextendr(noexport)]
pub fn test_condition_with_parent(line: &str) -> i32 {
    match catch_condition(|| parse_setting(line)) {
        Ok(value) => value,
        Err(cause) => RCondition::Error {
            message: "could not load the setting".into(),
            class: vec!["pkg_error_load".into()],
            data: None,
            call: ConditionCall::Inherit,
        }
        .with_parent(cause)
        .raise(),
    }
}

/// An error whose cause is a warning without a call, whose own cause is a
/// message. The error's `parent` data field and its first cause are both
/// replaced by the second `with_parent`.
#[miniextendr(noexport)]
pub fn test_condition_with_parent_chain() -> i32 {
    let root = RCondition::Message {
        message: "root".into(),
        class: vec!["pkg_root".into()],
        data: None,
        call: ConditionCall::Inherit,
    };
    let middle = RCondition::Warning {
        message: "middle".into(),
        class: vec!["pkg_middle".into()],
        data: Some(vec![("step".into(), RValue::Integer(vec![Some(2)]))]),
        call: ConditionCall::None,
    }
    .with_parent(root);
    let replaced = RCondition::Error {
        message: "replaced".into(),
        class: vec!["pkg_replaced".into()],
        data: None,
        call: ConditionCall::Inherit,
    };
    RCondition::Error {
        message: "top".into(),
        class: vec!["pkg_top".into()],
        data: Some(vec![(
            "parent".into(),
            RValue::Character(vec![Some("a field".into())]),
        )]),
        call: ConditionCall::Inherit,
    }
    .with_parent(replaced)
    .with_parent(middle)
    .raise()
}

// endregion
