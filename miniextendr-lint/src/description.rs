//! The R package's `DESCRIPTION`: its name and the packages it declares.
//!
//! Found from the Cargo manifest directory in the standard layout, where the
//! crate lives at `<pkg>/src/rust` and `DESCRIPTION` at `<pkg>/DESCRIPTION`.
//! Anywhere else (a plain Rust crate, a custom layout) there is none, and the
//! rules that need it skip.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The `DESCRIPTION` fields that declare a package dependency.
const DEPENDENCY_FIELDS: &[&str] = &["Depends", "Imports", "Suggests", "Enhances", "LinkingTo"];

/// Packages shipped with R (base and recommended), which every R install has.
const R_DISTRIBUTED_PACKAGES: &[&str] = &[
    "base",
    "compiler",
    "datasets",
    "graphics",
    "grDevices",
    "grid",
    "methods",
    "parallel",
    "splines",
    "stats",
    "stats4",
    "tcltk",
    "tools",
    "utils",
    "boot",
    "class",
    "cluster",
    "codetools",
    "foreign",
    "KernSmooth",
    "lattice",
    "MASS",
    "Matrix",
    "mgcv",
    "nlme",
    "nnet",
    "rpart",
    "spatial",
    "survival",
];

/// What the lint reads from a `DESCRIPTION`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Description {
    /// The `Package:` field.
    pub package: String,
    /// Every package in `Depends`, `Imports`, `Suggests`, `Enhances` and
    /// `LinkingTo`, without version constraints.
    pub declared: HashSet<String>,
}

impl Description {
    /// Whether an R link into `pkg` can resolve: the package itself, a declared
    /// dependency, or a package that ships with R.
    pub fn resolves(&self, pkg: &str) -> bool {
        pkg == self.package || self.declared.contains(pkg) || R_DISTRIBUTED_PACKAGES.contains(&pkg)
    }

    /// Parse `DESCRIPTION` text (Debian control format). `None` without a
    /// `Package:` field, so a file that is not an R `DESCRIPTION` reads as none.
    pub fn parse(text: &str) -> Option<Self> {
        let mut package = None;
        let mut declared = HashSet::new();
        for (field, value) in fields(text) {
            if field == "Package" {
                package = Some(value.trim().to_string());
            } else if DEPENDENCY_FIELDS.contains(&field.as_str()) {
                declared.extend(
                    value
                        .split(',')
                        .map(|dep| dep.split('(').next().unwrap_or("").trim())
                        .filter(|dep| !dep.is_empty())
                        .map(str::to_string),
                );
            }
        }
        Some(Self {
            package: package.filter(|p| !p.is_empty())?,
            declared,
        })
    }
}

/// The `DESCRIPTION` file of the R package whose Rust crate sits in
/// `manifest_dir`, when there is one.
pub fn locate(manifest_dir: &Path) -> Option<PathBuf> {
    let path = manifest_dir.join("../../DESCRIPTION");
    path.is_file().then_some(path)
}

/// Read and parse the `DESCRIPTION` of the package around `manifest_dir`.
pub fn read(manifest_dir: &Path) -> Option<Description> {
    Description::parse(&fs::read_to_string(locate(manifest_dir)?).ok()?)
}

/// `(field, value)` pairs; a line starting with whitespace continues the value.
fn fields(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if line.starts_with([' ', '\t']) {
            if let Some((_, value)) = out.last_mut() {
                value.push(' ');
                value.push_str(line.trim());
            }
        } else if let Some((field, value)) = line.split_once(':') {
            out.push((field.trim().to_string(), value.trim().to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_package_and_dependencies_across_continuation_lines() {
        let d = Description::parse(
            "Package: mypkg\nImports: cli (>= 3.0),\n    rlang,\n    vctrs\nSuggests: testthat\nLinkingTo: cpp11\nTitle: x, y\n",
        )
        .unwrap();
        assert_eq!(d.package, "mypkg");
        for dep in ["cli", "rlang", "vctrs", "testthat", "cpp11"] {
            assert!(d.declared.contains(dep), "{dep}");
        }
        assert!(!d.declared.contains("x"));
        assert!(d.resolves("mypkg") && d.resolves("stats") && d.resolves("MASS"));
        assert!(!d.resolves("Sources") && !d.resolves("dplyr"));
    }

    #[test]
    fn a_file_without_a_package_field_is_none() {
        assert_eq!(Description::parse("Title: x\n"), None);
    }
}
