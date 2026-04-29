//! Strip unnecessary directories and TOML sections from vendored crates

use crate::Verbosity;
use anyhow::{Context, Result};
use std::path::Path;

/// Configuration for what to strip from vendored crates
#[derive(Clone, Debug)]
pub struct StripConfig {
    pub tests: bool,
    pub benches: bool,
    pub examples: bool,
    pub bins: bool,
    /// Strip TOML sections only — leave source-related directories
    /// (`tests/`, `benches/`, `examples/`) on disk. Crates such as
    /// `zerocopy` reference files in those directories from regular
    /// library source via `include_str!()`; deleting them breaks
    /// `cargo check --offline` post-vendor. Always-safe base dirs
    /// (`.github`, `.circleci`, `ci`, `target`) are still removed.
    pub toml_only: bool,
}

impl StripConfig {
    /// Strip everything (directories and TOML sections)
    pub fn all() -> Self {
        Self {
            tests: true,
            benches: true,
            examples: true,
            bins: true,
            toml_only: false,
        }
    }

    /// Strip TOML sections for all source-target categories without
    /// deleting `tests/` / `benches/` / `examples/` directories. See #330.
    pub fn toml_only() -> Self {
        Self {
            tests: true,
            benches: true,
            examples: true,
            bins: true,
            toml_only: true,
        }
    }

    /// Whether any stripping is enabled
    pub fn any(&self) -> bool {
        self.tests || self.benches || self.examples || self.bins
    }

    /// Get directory names to strip
    fn dirs_to_strip(&self) -> Vec<&'static str> {
        let mut dirs = vec![".github", ".circleci", "ci", "target"];
        if self.toml_only {
            return dirs;
        }
        if self.tests {
            dirs.push("tests");
        }
        if self.benches {
            dirs.push("benches");
        }
        if self.examples {
            dirs.push("examples");
        }
        // bins: no standard dir to strip (binaries are in src/bin/)
        dirs
    }

    /// Get TOML section prefixes to strip
    fn toml_sections_to_strip(&self) -> Vec<&'static str> {
        let mut sections = Vec::new();
        if self.tests {
            sections.push("[[test]]");
            sections.push("[dev-dependencies");
        }
        if self.benches {
            sections.push("[[bench]]");
        }
        if self.examples {
            sections.push("[[example]]");
        }
        if self.bins {
            sections.push("[[bin]]");
        }
        sections
    }
}

/// Strip all vendored crates in a vendor directory.
/// Returns list of stripped items for reporting.
pub fn strip_vendor_dir(
    vendor_dir: &Path,
    config: &StripConfig,
    v: Verbosity,
) -> Result<Vec<String>> {
    let mut stripped = Vec::new();
    for entry in std::fs::read_dir(vendor_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let items = strip_crate_dir(&entry.path(), config, v)?;
            stripped.extend(items);
        }
    }
    Ok(stripped)
}

/// Strip a single vendored crate directory
fn strip_crate_dir(crate_dir: &Path, config: &StripConfig, v: Verbosity) -> Result<Vec<String>> {
    let crate_name = crate_dir.file_name().unwrap().to_string_lossy().to_string();
    let mut stripped = Vec::new();

    // Remove configured directories
    for dir_name in config.dirs_to_strip() {
        let dir_path = crate_dir.join(dir_name);
        if dir_path.exists() {
            std::fs::remove_dir_all(&dir_path).with_context(|| {
                format!("failed to remove {}/{}", crate_dir.display(), dir_name)
            })?;
            if v.debug() {
                eprintln!("  Stripped {}/{}", crate_name, dir_name);
            }
            stripped.push(format!("{}/{}", crate_name, dir_name));
        }
    }

    // Remove hidden files/dirs (except .cargo-checksum.json)
    for entry in std::fs::read_dir(crate_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') && name_str != ".cargo-checksum.json" {
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(entry.path())?;
            } else {
                std::fs::remove_file(entry.path())?;
            }
        }
    }

    // Clean TOML sections that reference stripped directories.
    // Before stripping, collect the dep names being removed from
    // [dev-dependencies] so we can prune dangling [features] refs.
    let cargo_toml = crate_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let sections = config.toml_sections_to_strip();
        if !sections.is_empty() {
            let removed_deps = if sections.iter().any(|s| s.starts_with("[dev-dependencies")) {
                collect_dep_names(&cargo_toml, "dev-dependencies")?
            } else {
                Vec::new()
            };
            strip_toml_sections(&cargo_toml, &sections)?;
            if !removed_deps.is_empty() {
                prune_dangling_features_inplace(&cargo_toml, &removed_deps)?;
            }
        }
    }

    Ok(stripped)
}

/// Read dep names from a specific dependency table in a Cargo.toml.
fn collect_dep_names(cargo_toml: &Path, section: &str) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(cargo_toml)?;
    let doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("failed to parse {}", cargo_toml.display()))?;
    let names = doc
        .get(section)
        .and_then(|v| v.as_table_like())
        .map(|tbl| tbl.iter().map(|(k, _)| k.to_string()).collect())
        .unwrap_or_default();
    Ok(names)
}

/// Prune [features] entries that reference removed deps, in-place.
fn prune_dangling_features_inplace(cargo_toml: &Path, removed_deps: &[String]) -> Result<()> {
    let content = std::fs::read_to_string(cargo_toml)?;
    let pruned = prune_dangling_feature_refs(&content, removed_deps);
    if pruned != content {
        std::fs::write(cargo_toml, &pruned)?;
    }
    Ok(())
}

/// Remove feature array entries that reference removed dependencies.
///
/// Cargo validates ALL [features] entries at parse time regardless of which
/// features are enabled. After dev-dependencies are stripped, any feature
/// referencing `"<dep>/..."`, `"<dep>?/..."`, or exactly `"<dep>"` for a
/// removed dep becomes a dangling reference that breaks every consumer.
///
/// If a feature's array becomes empty after pruning, it is kept as `[]`
/// (a valid, harmless feature flag). Non-referencing features are unchanged.
pub fn prune_dangling_feature_refs(content: &str, removed_deps: &[String]) -> String {
    if removed_deps.is_empty() {
        return content.to_string();
    }

    let mut doc: toml_edit::DocumentMut = match content.parse() {
        Ok(d) => d,
        Err(_) => return content.to_string(),
    };

    let Some(features) = doc.get_mut("features").and_then(|v| v.as_table_mut()) else {
        return content.to_string();
    };

    let mut changed = false;
    for (_feat_name, feat_val) in features.iter_mut() {
        let Some(arr) = feat_val.as_array_mut() else {
            continue;
        };
        let before = arr.len();
        arr.retain(|item| {
            let s = match item.as_str() {
                Some(s) => s,
                None => return true, // keep non-string items unchanged
            };
            !removed_deps.iter().any(|dep| {
                // Exact match: "dep"
                s == dep
                    // Dep-feature ref: "dep/feature" or "dep?/feature"
                    || s.starts_with(&format!("{}/", dep))
                    || s.starts_with(&format!("{}?/", dep))
            })
        });
        if arr.len() != before {
            changed = true;
        }
    }

    if changed {
        doc.to_string()
    } else {
        content.to_string()
    }
}

/// Remove specified TOML sections from Cargo.toml
fn strip_toml_sections(cargo_toml: &Path, sections_to_strip: &[&str]) -> Result<()> {
    let content = std::fs::read_to_string(cargo_toml)?;
    let mut output_lines = Vec::new();
    let mut in_stripped_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_stripped_section = sections_to_strip.iter().any(|s| trimmed.starts_with(s));

            if in_stripped_section {
                continue;
            }
        }

        if in_stripped_section {
            if trimmed.starts_with('[') && !sections_to_strip.iter().any(|s| trimmed.starts_with(s))
            {
                in_stripped_section = false;
                output_lines.push(line.to_string());
            }
        } else {
            output_lines.push(line.to_string());
        }
    }

    // Remove trailing blank lines
    while output_lines.last().is_some_and(|l| l.trim().is_empty()) {
        output_lines.pop();
    }
    output_lines.push(String::new());

    std::fs::write(cargo_toml, output_lines.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_toml(dir: &Path, content: &str) -> std::path::PathBuf {
        let path = dir.join("Cargo.toml");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn strip_toml_removes_bench_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[[bench]]\nname = \"my_bench\"\nharness = false\n\n[lib]\nname = \"foo\"\n",
        );
        strip_toml_sections(&path, &["[[bench]]"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[bench]]"));
        assert!(!result.contains("my_bench"));
        assert!(result.contains("[dependencies]"));
        assert!(result.contains("[lib]"));
    }

    #[test]
    fn strip_toml_removes_test_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[[test]]\nname = \"integration\"\n\n[dependencies]\nbar = \"1\"\n",
        );
        strip_toml_sections(&path, &["[[test]]"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[test]]"));
        assert!(result.contains("[dependencies]"));
    }

    #[test]
    fn strip_toml_removes_dev_dependencies() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n\n[features]\ndefault = []\n",
        );
        strip_toml_sections(&path, &["[dev-dependencies"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[dev-dependencies]"));
        assert!(!result.contains("criterion"));
        assert!(result.contains("[features]"));
        // No dangling refs to criterion exist in this fixture, default = [] survives
        assert!(result.contains("default = []"));
    }

    #[test]
    fn strip_toml_removes_example_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[[example]]\nname = \"demo\"\n\n[dependencies]\nbar = \"1\"\n",
        );
        strip_toml_sections(&path, &["[[example]]"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[example]]"));
        assert!(result.contains("[dependencies]"));
    }

    #[test]
    fn strip_toml_preserves_regular_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[build-dependencies]\ncc = \"1\"\n\n[features]\ndefault = []\n\n[profile.release]\nopt-level = 3\n",
        );
        strip_toml_sections(&path, &["[[test]]", "[[bench]]", "[dev-dependencies"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(result.contains("[package]"));
        assert!(result.contains("[dependencies]"));
        assert!(result.contains("[build-dependencies]"));
        assert!(result.contains("[features]"));
        assert!(result.contains("[profile.release]"));
    }

    #[test]
    fn strip_toml_handles_multiple_stripped_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[[test]]\nname = \"t1\"\n\n[[test]]\nname = \"t2\"\n\n[[bench]]\nname = \"b1\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n\n[features]\ndefault = []\n",
        );
        strip_toml_sections(&path, &["[[test]]", "[[bench]]", "[dev-dependencies"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[test]]"));
        assert!(!result.contains("[[bench]]"));
        assert!(!result.contains("[dev-dependencies]"));
        assert!(result.contains("[features]"));
    }

    // region: prune_dangling_feature_refs (#322)

    #[test]
    fn prune_dangling_feature_refs_removes_dep_slash_feature() {
        let toml = r#"[package]
name = "foo"
version = "0.1.0"

[dev-dependencies]
criterion = "0.5"

[features]
real_blackbox = ["criterion/real_blackbox"]
default = []
"#;
        let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
        // criterion/real_blackbox should be gone
        assert!(
            !result.contains("criterion/real_blackbox"),
            "dangling dep-feature ref should be pruned, got:\n{result}"
        );
        // default = [] should survive
        assert!(
            result.contains("default = []"),
            "default feature should be preserved, got:\n{result}"
        );
        // real_blackbox feature key still exists but as empty array
        assert!(
            result.contains("real_blackbox"),
            "feature key should still be present, got:\n{result}"
        );
    }

    #[test]
    fn prune_dangling_feature_refs_removes_optional_dep_feature() {
        // dep?/feature syntax (optional dep)
        let toml = r#"[features]
foo = ["criterion?/real_blackbox", "serde/derive"]
"#;
        let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
        assert!(!result.contains("criterion?/real_blackbox"));
        // serde/derive is untouched (serde was not removed)
        assert!(result.contains("serde/derive"));
    }

    #[test]
    fn prune_dangling_feature_refs_removes_exact_dep_name() {
        // Feature enabling a dep by exact name: `feature = ["criterion"]`
        let toml = r#"[features]
benchmarks = ["criterion"]
default = []
"#;
        let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
        assert!(!result.contains("\"criterion\""));
        assert!(result.contains("default = []"));
    }

    #[test]
    fn prune_dangling_feature_refs_keeps_non_removed_deps() {
        let toml = r#"[features]
full = ["serde/derive", "criterion/real_blackbox"]
default = []
"#;
        // Only criterion removed — serde/derive must survive
        let result = prune_dangling_feature_refs(toml, &["criterion".to_string()]);
        assert!(!result.contains("criterion/real_blackbox"));
        assert!(result.contains("serde/derive"));
    }

    #[test]
    fn prune_dangling_feature_refs_empty_removed_deps_is_noop() {
        let toml = r#"[features]
full = ["serde/derive"]
"#;
        let result = prune_dangling_feature_refs(toml, &[]);
        assert_eq!(result, toml);
    }

    #[test]
    fn strip_crate_dir_prunes_dangling_features_after_dev_dep_strip() {
        // End-to-end: strip_crate_dir with StripConfig { tests: true, ... }
        // should remove [dev-dependencies] AND prune features that reference them.
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("mycrate");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "mycrate"
version = "0.1.0"

[dependencies]
serde = "1"

[dev-dependencies]
criterion = "0.5"

[features]
real_blackbox = ["criterion/real_blackbox"]
default = []
"#,
        )
        .unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        let result = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert!(!result.contains("[dev-dependencies]"), "dev-deps should be removed");
        assert!(!result.contains("criterion"), "criterion ref should be gone");
        assert!(result.contains("serde"), "regular dep preserved");
        assert!(result.contains("default = []"), "default feature preserved");
        assert!(result.contains("real_blackbox"), "feature key still present but pruned");
    }

    // endregion

    // region: --strip-toml-sections (#330)

    #[test]
    fn toml_only_dirs_to_strip_excludes_source_dirs() {
        let cfg = StripConfig::toml_only();
        let dirs = cfg.dirs_to_strip();
        // Always-safe base dirs survive
        assert!(dirs.contains(&".github"));
        assert!(dirs.contains(&".circleci"));
        assert!(dirs.contains(&"ci"));
        assert!(dirs.contains(&"target"));
        // Source-related dirs are NOT stripped (zerocopy include_str! safety)
        assert!(!dirs.contains(&"tests"));
        assert!(!dirs.contains(&"benches"));
        assert!(!dirs.contains(&"examples"));
    }

    #[test]
    fn toml_only_still_strips_toml_sections() {
        let cfg = StripConfig::toml_only();
        let sections = cfg.toml_sections_to_strip();
        assert!(sections.contains(&"[[test]]"));
        assert!(sections.contains(&"[[bench]]"));
        assert!(sections.contains(&"[[example]]"));
        assert!(sections.contains(&"[[bin]]"));
        assert!(sections.contains(&"[dev-dependencies"));
    }

    #[test]
    fn strip_crate_dir_toml_only_preserves_source_dirs() {
        // Mirrors the zerocopy footgun: bench source files referenced by
        // `include_str!()` from regular lib code must survive stripping.
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("zerocopy_like");
        std::fs::create_dir_all(crate_dir.join("benches/formats")).unwrap();
        std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
        std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
        std::fs::create_dir_all(crate_dir.join(".github/workflows")).unwrap();
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("benches/formats/static_size.rs"), "// bench").unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "zerocopy_like"
version = "0.1.0"

[dependencies]
serde = "1"

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "static_size"
harness = false

[features]
real_blackbox = ["criterion/real_blackbox"]
default = []
"#,
        )
        .unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::toml_only(), Verbosity(0)).unwrap();

        // Source-related dirs preserved (the whole point of --strip-toml-sections)
        assert!(crate_dir.join("benches/formats/static_size.rs").exists(),
            "benches/ must survive — referenced by include_str! in some crates");
        assert!(crate_dir.join("tests").exists(), "tests/ must survive");
        assert!(crate_dir.join("examples").exists(), "examples/ must survive");
        // Always-safe base dirs still go
        assert!(!crate_dir.join(".github").exists(), ".github/ should still be stripped");
        // TOML surgery still happens
        let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert!(!manifest.contains("[dev-dependencies]"));
        assert!(!manifest.contains("[[bench]]"));
        assert!(!manifest.contains("criterion"));
        // Dangling [features] ref pruned
        assert!(!manifest.contains("criterion/real_blackbox"));
        // Regular content preserved
        assert!(manifest.contains("serde"));
        assert!(manifest.contains("default = []"));
    }

    // endregion

    #[test]
    fn strip_crate_dir_removes_directories() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("mycrate");
        std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
        std::fs::create_dir_all(crate_dir.join("benches")).unwrap();
        std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
        std::fs::create_dir_all(crate_dir.join(".github")).unwrap();
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("tests/test.rs"), "").unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        assert!(!crate_dir.join("tests").exists());
        assert!(!crate_dir.join("benches").exists());
        assert!(!crate_dir.join("examples").exists());
        assert!(!crate_dir.join(".github").exists());
        assert!(crate_dir.join("src").exists());
        assert!(crate_dir.join(".cargo-checksum.json").exists());
    }
}
