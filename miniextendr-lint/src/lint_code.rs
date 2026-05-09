//! Stable lint rule identifiers.
//!
//! Each rule has a code like `MXL008` that is grep-able and CI-friendly.

use std::fmt;

/// Stable lint rule identifier.
///
/// Display format is `MXL###`, derived directly from the variant name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LintCode {
    // region: Source-side validation
    /// Trait impl class system incompatible with inherent impl class system.
    MXL008,
    /// Multiple impl blocks for one type without labels.
    MXL009,
    /// Duplicate labels on impl blocks for one type.
    MXL010,
    // endregion

    // region: P0: High Impact
    /// Registered top-level function is not `pub`.
    MXL106,
    /// Parameter name is an R reserved word; codegen will produce invalid R syntax.
    MXL110,
    /// `s4_*` method name on `#[miniextendr(s4)]` impl — codegen auto-prepends `s4_`.
    MXL111,
    /// Explicit lifetime parameter on `#[miniextendr]` fn or impl — use owned types instead.
    MXL112,
    // endregion

    // region: P1: Important
    /// `internal` + `noexport` redundancy.
    MXL203,
    // endregion

    // region: P2: Safety
    /// Direct `Rf_error`/`Rf_errorcall` call in user code.
    MXL300,
    /// `_unchecked` FFI call outside guard context.
    MXL301,
    // endregion
}

impl fmt::Display for LintCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Variant names are already `MXL###`, so Debug output works.
        fmt::Debug::fmt(self, f)
    }
}

impl LintCode {
    /// Default severity for this rule.
    pub fn default_severity(self) -> super::diagnostic::Severity {
        use super::diagnostic::Severity;
        match self {
            // Source-side checks are errors (CI-blocking).
            Self::MXL008 | Self::MXL009 | Self::MXL010 => Severity::Error,

            // Codegen-breaking: reserved words produce syntactically invalid R wrappers.
            Self::MXL110 => Severity::Error,

            // Everything else is a warning.
            Self::MXL106
            | Self::MXL111
            | Self::MXL112
            | Self::MXL203
            | Self::MXL300
            | Self::MXL301 => Severity::Warning,
        }
    }
}
