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

    // Some crates reference files in `tests/`/`benches/`/`examples/` from
    // regular library source via `include_str!()`/`include_bytes!()`/`include!()`.
    // Stripping those dirs breaks `cargo build --offline` post-vendor (winnow
    // ships `include_str!("../examples/css/parser.rs")` in `src/lib.rs`).
    // Scan the crate up front and skip stripping any top-level dir that's
    // referenced this way.
    let referenced_top_dirs = scan_referenced_top_dirs(crate_dir);

    // Remove configured directories
    for dir_name in config.dirs_to_strip() {
        if referenced_top_dirs.iter().any(|d| d == dir_name) {
            if v.debug() {
                eprintln!(
                    "  Preserving {}/{} — referenced by include_str!/include_bytes!/include!",
                    crate_name, dir_name
                );
            }
            continue;
        }
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
    // [dev-dependencies] so we can prune dangling [features] refs — but
    // subtract names that also live in [dependencies] / [build-dependencies],
    // because those entries survive the strip and the feature ref points at
    // them, not at the dev-only copy. (The `time` crate ships `time-macros`
    // in both tables and `[features] formatting = ["time-macros?/formatting"]`
    // would otherwise be silently dropped.)
    let cargo_toml = crate_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let sections = config.toml_sections_to_strip();
        if !sections.is_empty() {
            let removed_deps = if sections.iter().any(|s| s.starts_with("[dev-dependencies")) {
                let dev = collect_dep_names(&cargo_toml, "dev-dependencies")?;
                let kept_deps = collect_dep_names(&cargo_toml, "dependencies")?;
                let kept_build = collect_dep_names(&cargo_toml, "build-dependencies")?;
                dev.into_iter()
                    .filter(|d| !kept_deps.contains(d) && !kept_build.contains(d))
                    .collect()
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
/// Exact-match items (`"<dep>"`) are only pruned when no `[features]` entry
/// of the same name exists in this crate. Otherwise the string refers to the
/// crate's own feature, not the dep — e.g. toml ships
/// `default = ["std", "serde", "parse", "display"]` where `"serde"` is the
/// feature key, not the dev-dep.
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

    // Snapshot defined feature names so we don't strip exact-match items that
    // refer to a feature rather than the removed dep.
    let feature_keys: std::collections::BTreeSet<String> = doc
        .get("features")
        .and_then(|v| v.as_table_like())
        .map(|tbl| tbl.iter().map(|(k, _)| k.to_string()).collect())
        .unwrap_or_default();

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
                // Exact match: "dep" — only when not also a feature name.
                (s == dep && !feature_keys.contains(s))
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

// region: include macro scanner

/// Stripable top-level directories whose names we recognize in path literals
/// even when the file isn't (yet) on disk — needed when the include path is
/// composed via `concat!()` (e.g. zerocopy:
/// `include_str!(concat!("../benches/formats/", $format, ".rs"))`).
const STRIPABLE_TOP_DIRS: &[&str] = &["tests", "benches", "examples", "bin"];

/// Walk every `.rs` file under `crate_dir` and collect the top-level directory
/// names referenced by `include_str!`/`include_bytes!`/`include!` macros.
/// Two layers of detection:
///   1. literal-path macros — resolve relative to the source file (matching
///      rustc) and read the first component under the crate root.
///   2. composed-path macros — scan all string literals in the macro's
///      argument span (handles `concat!(...)`) for `<...>tests/`, `benches/`,
///      `examples/`, `bin/` substrings and preserve those dirs.
/// Paths escaping the crate root or referencing files outside it are ignored.
fn scan_referenced_top_dirs(crate_dir: &Path) -> Vec<String> {
    let crate_root = match crate_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    let mut top_dirs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for entry in walkdir::WalkDir::new(crate_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let parent = match path.parent() {
            Some(p) => p,
            None => continue,
        };
        for literal in extract_include_arg_literals(&content) {
            // (1) Literal-path resolution: works for `include_str!("...")`.
            let resolved = parent.join(&literal);
            if let Ok(canon) = resolved.canonicalize() {
                if let Ok(rel) = canon.strip_prefix(&crate_root) {
                    if let Some(first) = rel.components().next() {
                        if let Some(s) = first.as_os_str().to_str() {
                            top_dirs.insert(s.to_string());
                            continue;
                        }
                    }
                }
            }
            // (2) Substring sniff: catches `concat!("../benches/...", x, ".rs")`
            //     where the literal alone doesn't resolve to a real file.
            for dir in STRIPABLE_TOP_DIRS {
                let needle = format!("/{}/", dir);
                if literal.contains(&needle) || literal.starts_with(&format!("{}/", dir)) {
                    top_dirs.insert((*dir).to_string());
                }
            }
        }
    }
    top_dirs.into_iter().collect()
}

/// Pull every string literal that appears inside the argument list of an
/// `include_str!`, `include_bytes!`, or `include!` macro invocation. Argument
/// spans are matched with balanced-paren tracking so nested macro calls
/// (e.g. `concat!(...)`) are scanned through, not skipped.
fn extract_include_arg_literals(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    for macro_name in ["include_str!", "include_bytes!", "include!"] {
        let needle = macro_name.as_bytes();
        let mut search = 0usize;
        while search + needle.len() <= bytes.len() {
            let Some(idx) = find_subslice(&bytes[search..], needle) else {
                break;
            };
            let mut i = search + idx + needle.len();
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] != b'(' {
                search += idx + 1;
                continue;
            }
            // Find matching close paren, tracking nested parens and strings.
            let arg_start = i + 1;
            let arg_end = match find_matching_close_paren(bytes, i) {
                Some(end) => end,
                None => break,
            };
            // Extract every string literal in [arg_start, arg_end).
            extract_string_literals(&content[arg_start..arg_end], &mut out);
            search = arg_end + 1;
        }
    }
    out
}

/// Given byte index `open` pointing at `b'('`, return the index of the
/// matching `b')'`, or None if unbalanced. Skips parens inside `"..."`
/// string literals and `'.'` char literals.
fn find_matching_close_paren(bytes: &[u8], open: usize) -> Option<usize> {
    debug_assert_eq!(bytes[open], b'(');
    let mut depth = 1i32;
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                i += 1;
            }
            b'\'' => {
                // Skip char literal: '\'', '\\', or 'X'. Lifetime markers
                // ('a) lack a closing quote — bail out of skipping if we
                // don't see one within a few chars.
                let scan_end = std::cmp::min(bytes.len(), i + 5);
                let close = bytes[i + 1..scan_end].iter().position(|&b| b == b'\'');
                match close {
                    Some(off) => i += 2 + off,
                    None => i += 1,
                }
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Append every `"..."` string literal in `s` to `out`, decoding common
/// escapes. Raw strings (`r"..."`, `r#"..."#`) and char literals are skipped.
fn extract_string_literals(s: &str, out: &mut Vec<String>) {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            let lit_start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i >= bytes.len() {
                break;
            }
            out.push(s[lit_start..i].replace("\\\"", "\"").replace("\\\\", "\\"));
            i += 1;
        } else {
            i += 1;
        }
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

// endregion

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

    #[test]
    fn strip_keeps_default_feature_ref_when_name_collides_with_dev_dep() {
        // Mirrors the `toml` crate footgun: `serde` is BOTH a [dev-dependencies]
        // entry AND a [features] key. `default = ["std", "serde", ...]` refers
        // to the feature, not the dev-dep, so the prune must not drop it.
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("toml_like");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "toml_like"
version = "0.1.0"

[dependencies]
serde_core = { version = "1", optional = true }

[dev-dependencies]
serde = { version = "1", features = ["derive"] }

[features]
default = ["std", "serde", "parse"]
std = []
parse = []
serde = ["dep:serde_core"]
"#,
        )
        .unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        let result = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert!(!result.contains("[dev-dependencies]"));
        // 'serde' in default list is the FEATURE, not the dev-dep — keep it.
        assert!(
            result.contains("\"serde\""),
            "default's 'serde' (feature ref) must survive: {result}"
        );
        // The serde feature definition itself is intact.
        assert!(result.contains("dep:serde_core"));
    }

    #[test]
    fn strip_keeps_feature_refs_for_deps_in_both_dev_and_regular() {
        // Mirrors the `time` crate: `time-macros` appears in BOTH
        // `[dev-dependencies]` and `[dependencies]` (as optional). Stripping
        // dev-dependencies must not prune `time-macros?/formatting` from
        // `[features]`, because the optional dep entry is still there.
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("time_like");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "time_like"
version = "0.1.0"

[dependencies]
time-macros = { version = "0.2", optional = true }

[dev-dependencies]
time-macros = "0.2"

[features]
formatting = ["time-macros?/formatting"]
parsing    = ["time-macros?/parsing"]
macros     = ["dep:time-macros"]
"#,
        )
        .unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        let result = std::fs::read_to_string(crate_dir.join("Cargo.toml")).unwrap();
        assert!(!result.contains("[dev-dependencies]"));
        assert!(
            result.contains("time-macros?/formatting"),
            "feature ref to optional dep must survive: {result}"
        );
        assert!(result.contains("time-macros?/parsing"));
        assert!(result.contains("dep:time-macros"));
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

    // region: include macro scanner tests

    #[test]
    fn extract_include_arg_literals_basic() {
        let src = r#"
            const A: &str = include_str!("../examples/css/parser.rs");
            const B: &[u8] = include_bytes!( "data/blob.bin" );
            include!("generated.rs");
        "#;
        let paths = extract_include_arg_literals(src);
        assert!(paths.iter().any(|p| p == "../examples/css/parser.rs"));
        assert!(paths.iter().any(|p| p == "data/blob.bin"));
        assert!(paths.iter().any(|p| p == "generated.rs"));
    }

    #[test]
    fn extract_include_arg_literals_through_concat() {
        // zerocopy pattern: include_str!(concat!("../benches/formats/", x, ".rs"))
        let src = r#"
            #[doc = include_str!(concat!("../benches/formats/", $f, ".rs"))]
            #[doc = include_str!(concat!("../benches/", $b))]
        "#;
        let paths = extract_include_arg_literals(src);
        assert!(
            paths.iter().any(|p| p == "../benches/formats/"),
            "expected '../benches/formats/' literal: {:?}",
            paths
        );
        assert!(paths.iter().any(|p| p == "../benches/"));
        assert!(paths.iter().any(|p| p == ".rs"));
    }

    #[test]
    fn strip_all_preserves_dirs_referenced_by_include_str() {
        // Mirrors winnow's footgun: src/lib.rs has include_str!("../examples/...")
        // The strip must not delete examples/ even with --strip-all.
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("winnow_like");
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::create_dir_all(crate_dir.join("examples/css")).unwrap();
        std::fs::create_dir_all(crate_dir.join("benches")).unwrap();
        std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
        std::fs::write(crate_dir.join("examples/css/parser.rs"), "// example").unwrap();
        std::fs::write(crate_dir.join("benches/bench.rs"), "// bench").unwrap();
        std::fs::write(crate_dir.join("tests/test.rs"), "// test").unwrap();
        std::fs::write(
            crate_dir.join("src/lib.rs"),
            r#"#![doc = include_str!("../examples/css/parser.rs")]"#,
        )
        .unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"winnow_like\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        // examples/ preserved because include_str! references it
        assert!(
            crate_dir.join("examples/css/parser.rs").exists(),
            "examples/ must survive — referenced by include_str!"
        );
        // tests/ and benches/ have no incoming references → still stripped
        assert!(!crate_dir.join("tests").exists());
        assert!(!crate_dir.join("benches").exists());
    }

    #[test]
    fn strip_all_preserves_dirs_referenced_via_concat() {
        // Mirrors zerocopy's footgun:
        //   include_str!(concat!("../benches/formats/", x, ".rs"))
        // The literal alone doesn't resolve to a file, so substring detection
        // on '/benches/' has to kick in for the dir to survive.
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("zerocopy_like");
        std::fs::create_dir_all(crate_dir.join("src/util")).unwrap();
        std::fs::create_dir_all(crate_dir.join("benches/formats")).unwrap();
        std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
        std::fs::write(crate_dir.join("benches/formats/coco.rs"), "// bench").unwrap();
        std::fs::write(crate_dir.join("examples/ex.rs"), "// example").unwrap();
        std::fs::write(
            crate_dir.join("src/util/macros.rs"),
            r#"
                #[doc = include_str!(concat!("../benches/formats/", $format, ".rs"))]
                fn _hint() {}
            "#,
        )
        .unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "pub mod util;").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"zerocopy_like\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        // benches/ preserved via concat-substring detection
        assert!(
            crate_dir.join("benches/formats/coco.rs").exists(),
            "benches/ must survive — referenced via concat!() in include_str!"
        );
        // examples/ has no reference → still stripped
        assert!(!crate_dir.join("examples").exists());
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
