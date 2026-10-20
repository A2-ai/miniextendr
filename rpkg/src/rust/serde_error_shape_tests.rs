//! Payload field types chosen at the condition boundary (#2207).
//!
//! `RuleError` stands in for an error type in a crate that can't depend on
//! miniextendr: its `Serialize` is hand-written and uses none of the
//! `miniextendr_api::serde::as_*` field helpers, so an empty `rule` arrives as
//! `list()` and `limit` as an integer. The R-facing functions give the fields
//! their R types instead: `serde_error(as_character(..), as_integer(..), ..)`
//! per function, or `serde_error(shape = RULE_SHAPE)` naming one shared
//! `SerdeErrorShape` const. A function's own options apply after the shape's
//! and win for a field set by both.

use crate::serde::ser::{Serialize, SerializeStruct, Serializer};
use miniextendr_api::condition::SerdeErrorShape;
use miniextendr_api::miniextendr;

// region: A hand-serialized error type

/// A rule check's failure, internally tagged by hand (`kind`).
#[derive(Debug)]
pub enum RuleError {
    /// The rule failed on `rows`.
    Failed {
        rule: Vec<String>,
        n: u32,
        limit: u32,
        rows: Vec<u32>,
    },
    /// The rule is not known; carries none of `n`, `limit`, `rows`.
    Unknown { rule: Vec<String> },
}

impl Serialize for RuleError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            RuleError::Failed {
                rule,
                n,
                limit,
                rows,
            } => {
                let mut s = serializer.serialize_struct("RuleError", 5)?;
                s.serialize_field("kind", "failed")?;
                s.serialize_field("rule", rule)?;
                s.serialize_field("n", n)?;
                s.serialize_field("limit", limit)?;
                s.serialize_field("rows", rows)?;
                s.end()
            }
            RuleError::Unknown { rule } => {
                let mut s = serializer.serialize_struct("RuleError", 2)?;
                s.serialize_field("kind", "unknown")?;
                s.serialize_field("rule", rule)?;
                s.end()
            }
        }
    }
}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleError::Failed { n, limit, .. } => {
                write!(f, "rule failed on {n} rows (limit {limit})")
            }
            RuleError::Unknown { .. } => f.write_str("unknown rule"),
        }
    }
}

impl std::error::Error for RuleError {}

fn rule_failed(rule: Vec<String>, rows: Vec<i32>) -> RuleError {
    let rows: Vec<u32> = rows
        .into_iter()
        .filter_map(|r| u32::try_from(r).ok())
        .collect();
    RuleError::Failed {
        rule,
        n: u32::try_from(rows.len()).unwrap_or(u32::MAX),
        limit: 10,
        rows,
    }
}

// endregion

// region: The shared shape

/// The R types of `RuleError`'s fields, for every function returning one.
pub const RULE_SHAPE: SerdeErrorShape = SerdeErrorShape::new()
    .prefix("rule")
    .as_character(&["rule"])
    .as_integer(&["n", "rows"])
    .as_double(&["limit"]);

// endregion

// region: Functions

/// No options: the fields' usual serialization.
/// @param rule Rule names.
/// @param rows Failing rows.
#[miniextendr]
pub fn rule_err_plain(rule: Vec<String>, rows: Vec<i32>) -> Result<(), RuleError> {
    Err(rule_failed(rule, rows))
}

/// The shared shape.
/// @param rule Rule names.
/// @param rows Failing rows.
#[miniextendr(serde_error(shape = RULE_SHAPE))]
pub fn rule_err_shaped(rule: Vec<String>, rows: Vec<i32>) -> Result<(), RuleError> {
    Err(rule_failed(rule, rows))
}

/// The shared shape, with `n` overridden to a double.
/// @param rule Rule names.
/// @param rows Failing rows.
#[miniextendr(serde_error(shape = RULE_SHAPE, as_double("n")))]
pub fn rule_err_override(rule: Vec<String>, rows: Vec<i32>) -> Result<(), RuleError> {
    Err(rule_failed(rule, rows))
}

/// The shared shape on a variant without `n`, `limit` or `rows`.
/// @param rule Rule names.
#[miniextendr(serde_error(shape = RULE_SHAPE))]
pub fn rule_err_unknown(rule: Vec<String>) -> Result<(), RuleError> {
    Err(RuleError::Unknown { rule })
}

/// The function's own options only.
/// @param rule Rule names.
/// @param rows Failing rows.
#[miniextendr(serde_error(
    prefix = "own_rule",
    as_character("rule"),
    as_integer("rows"),
    as_double("limit")
))]
pub fn rule_err_own(rule: Vec<String>, rows: Vec<i32>) -> Result<(), RuleError> {
    Err(rule_failed(rule, rows))
}

/// `as_integer` on a field of strings: the field doesn't fit.
/// @param rule Rule names.
/// @param rows Failing rows.
#[miniextendr(serde_error(shape = RULE_SHAPE, as_integer("rule")))]
pub fn rule_err_misfit(rule: Vec<String>, rows: Vec<i32>) -> Result<(), RuleError> {
    Err(rule_failed(rule, rows))
}

// endregion

// region: S3 method (impl-block codegen arm)

/// Raises `RuleError` from a method.
#[derive(Default, miniextendr_api::ExternalPtr)]
pub struct RuleChecker;

impl RuleChecker {
    /// The shape as an associated const, named `Self::SHAPE` by the method.
    const SHAPE: SerdeErrorShape = RULE_SHAPE;
}

#[miniextendr(s3)]
impl RuleChecker {
    /// A rule checker.
    pub fn new() -> Self {
        RuleChecker
    }

    /// Check `rule` on `rows` with the shared shape, named through `Self`.
    /// @param rule Rule names.
    /// @param rows Failing rows.
    #[miniextendr(serde_error(shape = Self::SHAPE))]
    pub fn rule_check(&self, rule: Vec<String>, rows: Vec<i32>) -> Result<(), RuleError> {
        Err(rule_failed(rule, rows))
    }
}

// endregion
