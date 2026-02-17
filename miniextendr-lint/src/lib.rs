//! miniextendr-lint: internal build-time lint helpers for the workspace.
//!
//! This crate scans Rust sources for miniextendr macro usage and emits
//! cargo warnings with actionable diagnostics. It is intended for local
//! development and CI, not as a public API.
//!
//! ## Usage in build.rs
//!
//! ```ignore
//! fn main() {
//!     miniextendr_lint::build_script();
//! }
//! ```
//!
//! ## Configuration
//! - Controlled by the `MINIEXTENDR_LINT` env var (enabled by default).
//! - Set it to `0`, `false`, `no`, or `off` to disable.
//!
//! ## Lint Codes
//!
//! Each diagnostic carries a stable `MXL###` code. See [`LintCode`] for the full catalog.

pub mod crate_index;
pub mod diagnostic;
pub mod helpers;
pub mod lint_code;
pub mod rules;

use std::env;
use std::path::{Path, PathBuf};

pub use crate_index::{CrateIndex, LintItem, LintKind};
pub use diagnostic::{Diagnostic, Severity};
pub use lint_code::LintCode;

/// Emits a single cargo warning line after normalizing whitespace.
fn cargo_warning(message: &str) {
    let message = message.replace(['\n', '\r'], " ");
    println!("cargo::warning={}", message.trim());
}

/// Entry point for build.rs. Runs the lint and prints cargo directives.
///
/// Controlled by `MINIEXTENDR_LINT` env var (enabled by default).
/// Set to `0`, `false`, `no`, or `off` to disable.
pub fn build_script() {
    println!("cargo::rerun-if-env-changed=MINIEXTENDR_LINT");

    let enabled = match lint_enabled("MINIEXTENDR_LINT") {
        Ok(enabled) => enabled,
        Err(message) => {
            cargo_warning(&message);
            return;
        }
    };

    if !enabled {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(err) => {
            cargo_warning(&format!("CARGO_MANIFEST_DIR: {err}"));
            return;
        }
    };

    let report = match run(&manifest_dir) {
        Ok(report) => report,
        Err(message) => {
            cargo_warning(&message);
            return;
        }
    };

    for path in &report.files {
        println!("cargo::rerun-if-changed={}", path.display());
    }

    if !report.diagnostics.is_empty() {
        cargo_warning("miniextendr-lint found issues");
        for diag in &report.diagnostics {
            cargo_warning(&diag.to_string());
        }
    }
}

#[derive(Debug, Default)]
/// Result of running the lint over a crate source tree.
pub struct LintReport {
    /// Rust source files that were scanned.
    pub files: Vec<PathBuf>,
    /// Structured diagnostics from all rules.
    pub diagnostics: Vec<Diagnostic>,
    /// Legacy string errors (derived from diagnostics, for backward compatibility).
    pub errors: Vec<String>,
}

/// Returns whether the lint should run based on the given env var.
///
/// Defaults to `true` when the var is unset. Set to 0/false/no/off to disable.
pub fn lint_enabled(env_var: &str) -> Result<bool, String> {
    match env::var(env_var) {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "0" | "false" | "no" | "off" | "" => Ok(false),
                "1" | "true" | "yes" | "on" => Ok(true),
                _ => Err(format!(
                    "{env_var} has invalid value '{value}'; use 1/0, true/false, yes/no, on/off"
                )),
            }
        }
        Err(env::VarError::NotPresent) => Ok(true),
        Err(err) => Err(format!("{env_var}: {err}")),
    }
}

/// Run the lint against the crate rooted at `root`.
///
/// If `root/src` exists, that directory is scanned. Otherwise `root` is scanned.
pub fn run(root: impl AsRef<Path>) -> Result<LintReport, String> {
    let root = root.as_ref();

    let index = CrateIndex::build(root)?;
    let diagnostics = rules::run_all_rules(&index);

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.to_legacy_string())
        .collect();

    Ok(LintReport {
        files: index.files,
        diagnostics,
        errors,
    })
}
