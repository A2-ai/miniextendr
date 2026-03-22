//! Test fixtures for RCondition structured error adapter.

use miniextendr_api::condition::RCondition;
use miniextendr_api::miniextendr;

// region: Simple errors

/// @export
#[miniextendr]
pub fn test_condition_parse_int(s: &str) -> Result<i32, RCondition<std::num::ParseIntError>> {
    s.parse::<i32>().map_err(RCondition)
}

/// @export
#[miniextendr]
pub fn test_condition_ok() -> Result<i32, RCondition<std::num::ParseIntError>> {
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

/// @export
#[miniextendr]
pub fn test_condition_chained(s: &str) -> Result<i32, RCondition<ConfigError>> {
    let value = s.parse::<i32>().map_err(|e| {
        RCondition(ConfigError {
            msg: format!("failed to parse '{s}' as max_threads"),
            source: e,
        })
    })?;
    Ok(value)
}

// endregion
