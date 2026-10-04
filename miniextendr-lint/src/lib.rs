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
pub mod crate_root;
pub mod diagnostic;
pub mod helpers;
pub mod lint_code;
mod misplaced_dots;
pub mod rules;

use std::env;
use std::path::{Path, PathBuf};

pub use crate_index::{CrateIndex, IndexError, LintItem, LintKind};
pub use crate_root::{CrateRoot, RootOrigin};
pub use diagnostic::{Diagnostic, Severity};
pub use lint_code::LintCode;

/// A `cargo::warning` directive for `message`, on one line.
fn warning_directive(message: &str) -> String {
    let message = message.replace(['\n', '\r'], " ");
    format!("cargo::warning={}", message.trim())
}

/// A `cargo::rerun-if-changed` directive for `path`.
fn rerun_directive(path: &Path) -> String {
    format!("cargo::rerun-if-changed={}", path.display())
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
            println!("{}", warning_directive(&message));
            return;
        }
    };

    if !enabled {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(err) => {
            println!(
                "{}",
                warning_directive(&format!("CARGO_MANIFEST_DIR: {err}"))
            );
            return;
        }
    };

    for directive in build_directives(&manifest_dir) {
        println!("{directive}");
    }

    // Note: generate_link_registrations() was removed. Wrapper generation now
    // loads the freshly linked package library. The staticlib build uses
    // codegen-units = 1 so the linker pulls all linkme entries via stub.c anchor.
}

/// The cargo directives [`build_script`] prints for the crate at `manifest_dir`
/// once the lint is enabled: what to watch, then the warnings.
///
/// Once a build script prints a `rerun-if-*` directive, cargo reruns it only when
/// one of them fires and replays its cached warnings in between, so every source
/// file the lint found is watched on the error path too: a "failed to parse"
/// warning must not outlive the fix (#1738). A `src/` directory holding the crate
/// root is also watched as a whole (cargo scans it recursively), which covers a
/// module file created after the `mod` line naming it. A crate root anywhere
/// else, such as the `lib.rs` next to `Cargo.toml` of every scaffolded R package,
/// is watched file by file only: a recursive watch of the manifest directory
/// would take in `target/`, `vendor/` and `.cargo/` and rerun the script after
/// every build.
///
/// `Cargo.toml` holds `[lib] path` (#1745) but is watched only while no crate
/// root file can be found or read. Then no source file is watched, and without
/// the manifest nothing would ever rerun the lint to clear its warning. A
/// healthy crate leaves it unwatched: every scaffolded `configure` touches
/// `Cargo.toml`, and watching it would rebuild the crate on every install (a
/// crate cargo builds but the lint cannot read, such as an inline
/// `lib = { path = ... }`, pays that cost until the spelling is rewritten).
/// Moving the root file away is still seen, since a watched file that
/// disappears reruns the script (#1752).
pub fn build_directives(manifest_dir: &Path) -> Vec<String> {
    let mut directives = Vec::new();

    let root = CrateRoot::resolve(manifest_dir);
    let manifest = manifest_dir.join("Cargo.toml");
    let root_found = matches!(&root, Ok(root) if root.file.is_file());
    if !root_found && manifest.is_file() {
        directives.push(rerun_directive(&manifest));
    }

    let src_dir = manifest_dir.join("src");
    if let Ok(root) = &root
        && root.file.starts_with(&src_dir)
    {
        directives.push(rerun_directive(&src_dir));
    }

    match run(manifest_dir) {
        Ok(report) => {
            directives.extend(report.files.iter().map(|path| rerun_directive(path)));
            if !report.diagnostics.is_empty() {
                directives.push(warning_directive("miniextendr-lint found issues"));
                directives.extend(
                    report
                        .diagnostics
                        .iter()
                        .map(|diag| warning_directive(&diag.to_string())),
                );
            }
        }
        Err(err) => {
            directives.extend(err.files.iter().map(|path| rerun_directive(path)));
            directives.push(warning_directive(&err.message));
        }
    }

    directives
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

/// Run the lint against the crate whose `Cargo.toml` sits in `manifest_dir`.
///
/// The module walk starts from `[lib] path` in `Cargo.toml`; without that key,
/// from `src/lib.rs` when `src/` exists, else from `lib.rs` ([`CrateRoot`]).
pub fn run(manifest_dir: impl AsRef<Path>) -> Result<LintReport, IndexError> {
    let index = CrateIndex::build(manifest_dir.as_ref())?;
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
