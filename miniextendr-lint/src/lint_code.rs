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
    /// vctrs constructor returns `Self` / named type, or impl has an instance-method receiver.
    ///
    /// Mirror: `miniextendr-macros/src/miniextendr_impl.rs` (proc-macro hard error).
    /// Both checks must fire on the same source; keep them in sync.
    MXL120,
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
    /// `into_sexp()` call inside a `vec!`/array literal — unprotected SEXP across allocations (UAF).
    MXL302,
    /// Two `#[miniextendr]` trait impls collapse to the same vtable symbol
    /// (`__VTABLE_{TRAIT}_FOR_{TYPE}`) after the macro's case-folding.
    MXL303,
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

            // Runtime-breaking: vctrs constructors returning Self produce EXTPTRSXP
            // which vctrs::new_vctr() rejects; instance-method receivers panic at runtime.
            Self::MXL120 => Severity::Error,

            // Build-breaking: colliding trait impls emit duplicate `#[no_mangle]`
            // vtable statics → cryptic linker error divorced from the source.
            Self::MXL303 => Severity::Error,

            // Everything else is a warning.
            Self::MXL106
            | Self::MXL111
            | Self::MXL203
            | Self::MXL300
            | Self::MXL301
            | Self::MXL302 => Severity::Warning,
        }
    }
}
