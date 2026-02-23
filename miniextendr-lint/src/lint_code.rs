//! Stable lint rule identifiers.
//!
//! Each rule has a code like `MXL101` that is grep-able and CI-friendly.
//! Codes in the `MXL0##` range cover checks that existed before the numbered
//! scheme; `MXL1##` are P0 (high impact), `MXL2##` are P1 (important).

use std::fmt;

/// Stable lint rule identifier.
///
/// Display format is `MXL###`, derived directly from the variant name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LintCode {
    // ── Existing checks (retroactively numbered) ────────────────────────
    /// Multiple `miniextendr_module!` macros in one file.
    MXL001,
    /// `#[miniextendr]` items found but no `miniextendr_module!` in file.
    MXL002,
    /// `miniextendr_module!` present but no `#[miniextendr]` items in file.
    MXL003,
    /// `#[miniextendr]` item not listed in `miniextendr_module!`.
    MXL004,
    /// Item in `miniextendr_module!` has no matching `#[miniextendr]` attribute.
    MXL005,
    /// Child module with `miniextendr_module!` missing `use child;` in parent.
    MXL006,
    /// `impl Type;` entry requires `ExternalPtr` derive or `TypedExternal` impl.
    MXL007,
    /// Trait impl class system incompatible with inherent impl class system.
    MXL008,
    /// Multiple impl blocks for one type without labels.
    MXL009,
    /// Duplicate labels on impl blocks for one type.
    MXL010,

    // ── P0: High Impact ─────────────────────────────────────────────────
    /// Duplicate module entrypoint symbol risk (two `mod <name>;` with same name).
    MXL100,
    /// Duplicate entries in one `miniextendr_module!`.
    MXL101,
    /// Trait impl registration missing `TypedExternal` contract.
    MXL102,
    /// Generic concrete type in trait-ABI registration.
    MXL103,
    /// `#[cfg(...)]` parity mismatch between item and module entry.
    MXL104,
    /// Unreachable module file with `miniextendr_module!`.
    MXL105,
    /// Registered top-level function is not `pub`.
    MXL106,
    /// Missing `#[miniextendr] impl Trait for Type` for registered trait entry.
    MXL107,
    /// Missing module trait entry for `#[miniextendr] impl Trait for Type`.
    MXL108,
    /// `#[cfg(...)]` parity mismatch between `mod` declaration and `use` entry.
    MXL109,

    // ── P1: Important ───────────────────────────────────────────────────
    /// Trait tag collision preflight.
    MXL200,
    /// Impl label mismatch quality diagnostic.
    MXL201,
    /// Orphan `use child;` with no child module macro.
    MXL202,
    /// `internal` + `noexport` redundancy.
    MXL203,
    /// Multiple root-level module macros in crate graph.
    MXL204,
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
            // Existing checks are errors (CI-blocking).
            Self::MXL001
            | Self::MXL002
            | Self::MXL003
            | Self::MXL004
            | Self::MXL005
            | Self::MXL006
            | Self::MXL007
            | Self::MXL008
            | Self::MXL009
            | Self::MXL010 => Severity::Error,

            // New P0 rules start as warnings.
            Self::MXL100
            | Self::MXL101
            | Self::MXL102
            | Self::MXL103
            | Self::MXL104
            | Self::MXL105
            | Self::MXL106
            | Self::MXL107
            | Self::MXL108
            | Self::MXL109 => Severity::Warning,

            // P1 rules are warnings.
            Self::MXL200 | Self::MXL201 | Self::MXL202 | Self::MXL203 | Self::MXL204 => {
                Severity::Warning
            }
        }
    }
}
