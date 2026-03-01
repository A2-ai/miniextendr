use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Miniextendr workspace crate names — the crates that get vendored/synced.
pub const MINIEXTENDR_CRATES: &[&str] = &[
    "miniextendr-api",
    "miniextendr-macros",
    "miniextendr-macros-core",
    "miniextendr-lint",
    "miniextendr-engine",
];

/// Find the workspace root containing `start`.
///
/// Tries `git rev-parse --show-toplevel` first (fast, accurate when in a git repo),
/// then falls back to walking up to 3 parent directories looking for a `Cargo.toml`
/// with `[workspace]`.
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    // Try git first — fast and handles deeply nested paths
    if let Some(root) = find_workspace_root_via_git(start) {
        return Some(root);
    }
    // Fallback: walk up to 3 levels
    find_workspace_root_via_walk(start)
}

fn find_workspace_root_via_git(start: &Path) -> Option<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let git_root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    let toml = git_root.join("Cargo.toml");
    if toml.is_file() {
        let content = std::fs::read_to_string(&toml).ok()?;
        if content.contains("[workspace]") {
            return std::fs::canonicalize(&git_root).ok();
        }
    }
    None
}

fn find_workspace_root_via_walk(start: &Path) -> Option<PathBuf> {
    let dirs = [start.to_path_buf(), start.join(".."), start.join("../..")];
    for dir in &dirs {
        let toml = dir.join("Cargo.toml");
        if toml.is_file()
            && let Ok(content) = std::fs::read_to_string(&toml)
            && content.contains("[workspace]")
        {
            return std::fs::canonicalize(dir).ok();
        }
    }
    None
}

/// Discovered project paths.
#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// The project root (where DESCRIPTION or Cargo.toml lives).
    pub root: PathBuf,
    /// `src/rust/Cargo.toml` if this is an R package with Rust.
    pub cargo_manifest: Option<PathBuf>,
    /// `DESCRIPTION` file if this is an R package.
    pub description: Option<PathBuf>,
    /// `configure.ac` if autoconf is set up.
    pub configure_ac: Option<PathBuf>,
    /// `configure` script.
    pub configure: Option<PathBuf>,
}

impl ProjectContext {
    /// Discover project structure starting from `path`.
    pub fn discover(path: &Path) -> Result<Self> {
        let root = std::fs::canonicalize(path)
            .with_context(|| format!("path does not exist: {}", path.display()))?;

        let cargo_manifest = {
            let p = root.join("src/rust/Cargo.toml");
            if p.is_file() { Some(p) } else { None }
        };

        let description = {
            let p = root.join("DESCRIPTION");
            if p.is_file() { Some(p) } else { None }
        };

        let configure_ac = {
            let p = root.join("configure.ac");
            if p.is_file() { Some(p) } else { None }
        };

        let configure = {
            let p = root.join("configure");
            if p.is_file() { Some(p) } else { None }
        };

        Ok(Self {
            root,
            cargo_manifest,
            description,
            configure_ac,
            configure,
        })
    }

    /// Returns the cargo manifest path, or an error with guidance.
    pub fn require_cargo_manifest(&self) -> Result<&Path> {
        self.cargo_manifest.as_deref().context(
            "No src/rust/Cargo.toml found. Is this a miniextendr R package?\n\
             Run `miniextendr init use` to add miniextendr to an existing package, or\n\
             Run `miniextendr init package` to create a new package.",
        )
    }

    /// Returns the DESCRIPTION path, or an error with guidance.
    #[allow(dead_code)]
    pub fn require_description(&self) -> Result<&Path> {
        self.description.as_deref().context(
            "No DESCRIPTION file found. Is this an R package directory?\n\
             Run `miniextendr init package` to create a new package.",
        )
    }

    /// Returns the configure.ac path, or an error with guidance.
    pub fn require_configure_ac(&self) -> Result<&Path> {
        self.configure_ac.as_deref().context(
            "No configure.ac found. Run `miniextendr workflow autoconf` first, or\n\
             `miniextendr init use` to set up miniextendr scaffolding.",
        )
    }

    /// Check if this looks like a miniextendr project.
    pub fn has_miniextendr(&self) -> bool {
        self.cargo_manifest.is_some() && self.description.is_some()
    }
}
