//! Payload field types chosen at the condition boundary (#2207).
//!
//! `QuotaError` and `RuleError` stand in for error types in a crate that
//! can't depend on miniextendr: their `Serialize` is hand-written and uses
//! none of the `miniextendr_api::serde::as_*` field helpers, so an empty list
//! of names arrives as `list()` and a limit as an integer. The R-facing crate
//! gives the fields their R types instead:
//!
//! - `serde_error_shape!(QuotaError = QUOTA_SHAPE)` declares the shape once for
//!   the type, and every function returning it uses that shape without naming
//!   it. `JsonValue` (`serde_json::Value`) gets one too: the type may come from
//!   any crate.
//! - `serde_error(shape = RULE_SHAPE)` names a shape per function, and
//!   `serde_error(as_character(..), as_integer(..), ..)` sets options per
//!   function. Both apply after a declared shape and win for a field set by
//!   both.

use crate::serde::ser::{Serialize, SerializeStruct, Serializer};
use miniextendr_api::condition::{ConditionData, RConditionData, SerdeErrorShape};
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

// region: A shape declared for the error type

/// A quota check's failure, hand-serialized like `RuleError`. Its shape is
/// declared once, below; no function returning it names it.
#[derive(Debug)]
pub enum QuotaError {
    /// The quota on `what` was exceeded on `rows`.
    Exceeded {
        what: Vec<String>,
        n: u32,
        limit: u32,
        rows: Vec<u32>,
    },
}

impl Serialize for QuotaError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let QuotaError::Exceeded {
            what,
            n,
            limit,
            rows,
        } = self;
        let mut s = serializer.serialize_struct("QuotaError", 5)?;
        s.serialize_field("kind", "exceeded")?;
        s.serialize_field("what", what)?;
        s.serialize_field("n", n)?;
        s.serialize_field("limit", limit)?;
        s.serialize_field("rows", rows)?;
        s.end()
    }
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let QuotaError::Exceeded { n, limit, .. } = self;
        write!(f, "quota exceeded on {n} rows (limit {limit})")
    }
}

impl std::error::Error for QuotaError {}

fn quota_exceeded(what: Vec<String>, rows: Vec<i32>) -> QuotaError {
    let rows: Vec<u32> = rows
        .into_iter()
        .filter_map(|r| u32::try_from(r).ok())
        .collect();
    QuotaError::Exceeded {
        what,
        n: u32::try_from(rows.len()).unwrap_or(u32::MAX),
        limit: 10,
        rows,
    }
}

/// The R types of `QuotaError`'s fields.
pub const QUOTA_SHAPE: SerdeErrorShape = SerdeErrorShape::new()
    .prefix("quota")
    .as_character(&["what"])
    .as_integer(&["n", "rows"])
    .as_double(&["limit"]);

miniextendr_api::serde_error_shape!(QuotaError = QUOTA_SHAPE);

/// A shape one function names on top of the declared one.
pub const QUOTA_STRICT: SerdeErrorShape = SerdeErrorShape::new()
    .prefix("strict_quota")
    .rename(&[("n", "count")]);

/// No attribute: the declared shape.
/// @param what Quota names.
/// @param rows Rows over the quota.
#[miniextendr]
pub fn quota_err_a(what: Vec<String>, rows: Vec<i32>) -> Result<(), QuotaError> {
    Err(quota_exceeded(what, rows))
}

/// No attribute either: the same declared shape.
/// @param what Quota names.
/// @param rows Rows over the quota.
#[miniextendr]
pub fn quota_err_b(what: Vec<String>, rows: Vec<i32>) -> Result<(), QuotaError> {
    Err(quota_exceeded(what, rows))
}

/// The declared shape, with `n` overridden to a double.
/// @param what Quota names.
/// @param rows Rows over the quota.
#[miniextendr(serde_error(as_double("n")))]
pub fn quota_err_override(what: Vec<String>, rows: Vec<i32>) -> Result<(), QuotaError> {
    Err(quota_exceeded(what, rows))
}

/// All three, in order: the declared shape, `QUOTA_STRICT` (prefix, `n`
/// renamed to `count`), then the function's own `as_double("n")`.
/// @param what Quota names.
/// @param rows Rows over the quota.
#[miniextendr(serde_error(shape = QUOTA_STRICT, as_double("n")))]
pub fn quota_err_chain(what: Vec<String>, rows: Vec<i32>) -> Result<(), QuotaError> {
    Err(quota_exceeded(what, rows))
}

/// Raises `QuotaError` from a method without an attribute.
#[derive(Default, miniextendr_api::ExternalPtr)]
pub struct QuotaChecker;

#[miniextendr(s3)]
impl QuotaChecker {
    /// A quota checker.
    pub fn new() -> Self {
        QuotaChecker
    }

    /// Check the quota on `what` for `rows`, with the declared shape.
    /// @param what Quota names.
    /// @param rows Rows over the quota.
    pub fn quota_check(&self, what: Vec<String>, rows: Vec<i32>) -> Result<(), QuotaError> {
        Err(quota_exceeded(what, rows))
    }
}

// endregion

// region: A declared shape wins over the type's RConditionData

/// An error that classes itself through `RConditionData` and also has a
/// declared shape: the declaration wins.
#[derive(Debug)]
pub enum ClassedQuotaError {
    /// The quota was exceeded by `n`.
    Exceeded { n: u32 },
}

impl Serialize for ClassedQuotaError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let ClassedQuotaError::Exceeded { n } = self;
        let mut s = serializer.serialize_struct("ClassedQuotaError", 2)?;
        s.serialize_field("kind", "exceeded")?;
        s.serialize_field("n", n)?;
        s.end()
    }
}

impl std::fmt::Display for ClassedQuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ClassedQuotaError::Exceeded { n } = self;
        write!(f, "quota exceeded by {n}")
    }
}

impl std::error::Error for ClassedQuotaError {}

impl RConditionData for ClassedQuotaError {
    fn message(&self) -> String {
        self.to_string()
    }

    fn class(&self) -> Vec<String> {
        vec!["classed_by_impl".to_string()]
    }

    fn data(&self) -> Option<ConditionData> {
        let ClassedQuotaError::Exceeded { n } = self;
        Some(vec![(
            "impl_n".to_string(),
            i32::try_from(*n).unwrap_or(i32::MAX).into(),
        )])
    }
}

miniextendr_api::serde_error_shape!(
    ClassedQuotaError = SerdeErrorShape::new().prefix("declared").as_double(&["n"])
);

/// Raises a `ClassedQuotaError`: the declared shape classes it.
/// @param n By how much.
#[miniextendr]
pub fn classed_quota_err(n: i32) -> Result<(), ClassedQuotaError> {
    Err(ClassedQuotaError::Exceeded {
        n: u32::try_from(n).unwrap_or(0),
    })
}

// endregion

// region: A shape declared for a type from another crate

#[cfg(feature = "serde_json")]
mod json {
    use miniextendr_api::condition::SerdeErrorShape;
    use miniextendr_api::miniextendr;
    use miniextendr_api::serde_impl::{JsonValue, serde_json};

    // `serde_json::Value` is neither this crate's nor miniextendr's.
    miniextendr_api::serde_error_shape!(
        JsonValue = SerdeErrorShape::new()
            .prefix("json")
            .as_character(&["what"])
            .as_double(&["n"])
    );

    /// Raises a JSON object as the error: `{"kind": "invalid", "what": .., "n": ..}`.
    /// @param what Names.
    /// @param n A count.
    #[miniextendr]
    pub fn json_err(what: Vec<String>, n: i32) -> Result<(), JsonValue> {
        let mut map = serde_json::Map::new();
        map.insert("kind".to_string(), JsonValue::from("invalid"));
        map.insert("what".to_string(), JsonValue::from(what));
        map.insert("n".to_string(), JsonValue::from(n));
        Err(JsonValue::Object(map))
    }
}

// endregion
