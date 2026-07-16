use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Miniextendr workspace crate names — the crates that get vendored/synced.
pub const MINIEXTENDR_CRATES: &[&str] = &[
    "miniextendr-api",
    "miniextendr-macros",
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

    /// Read a field's value from `DESCRIPTION`, if present.
    ///
    /// DCF (Debian Control File, the format `DESCRIPTION` uses) allows a
    /// field's value to continue onto following lines: a continuation line
    /// starts with whitespace and is joined onto the value of the field it
    /// extends, separated by a single space.
    pub fn description_field(&self, field: &str) -> Option<String> {
        let content = std::fs::read_to_string(self.root.join("DESCRIPTION")).ok()?;
        parse_description_field(&content, field)
    }

    /// The `Package` field from `DESCRIPTION`, if present.
    pub fn package_name(&self) -> Option<String> {
        self.description_field("Package")
    }
}

/// Parse a single field's value out of DCF-formatted `content` (the format
/// used by `DESCRIPTION` files), joining continuation lines onto the field
/// they extend.
pub fn parse_description_field(content: &str, field: &str) -> Option<String> {
    let prefix = format!("{field}:");
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        let Some(value) = line.strip_prefix(&prefix) else {
            continue;
        };
        let mut value = value.trim().to_string();
        while let Some(next_line) = lines.peek() {
            if !next_line.starts_with(|c: char| c.is_whitespace()) {
                break;
            }
            let cont = lines.next().unwrap().trim();
            if !cont.is_empty() {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(cont);
            }
        }
        return Some(value);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_description_field;

    #[test]
    fn simple_field() {
        let content = "Package: mypkg\nVersion: 0.1.0\n";
        assert_eq!(
            parse_description_field(content, "Package"),
            Some("mypkg".to_string())
        );
    }

    #[test]
    fn continuation_line_field() {
        let content = "Package: mypkg\n\
             Description: This is a\n    long description that\n    wraps.\n\
             Version: 0.1.0\n";
        assert_eq!(
            parse_description_field(content, "Description"),
            Some("This is a long description that wraps.".to_string())
        );
    }

    #[test]
    fn missing_field() {
        let content = "Package: mypkg\nVersion: 0.1.0\n";
        assert_eq!(parse_description_field(content, "License"), None);
    }

    #[test]
    fn field_name_prefix_of_another() {
        // "Packaged:" must not be mistaken for a match on "Package".
        let content = "Packaged: 2024-01-01\nPackage: mypkg\n";
        assert_eq!(
            parse_description_field(content, "Package"),
            Some("mypkg".to_string())
        );
        // And querying the longer name should not pick up the shorter one.
        let content = "Package: mypkg\nPackaged: 2024-01-01\n";
        assert_eq!(
            parse_description_field(content, "Packaged"),
            Some("2024-01-01".to_string())
        );
    }
}
