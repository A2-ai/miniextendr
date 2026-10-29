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
    /// (`tests/`, `benches/`, `examples/`) on disk, for a crate that reaches
    /// into one in a way `scan_referenced_top_dirs` does not detect (the
    /// directory pass already keeps those named by include macros or
    /// `#[path]`). Always-safe base dirs (`.github`, `.circleci`, `ci`,
    /// `target`) are still removed.
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
    // regular library source via `include_str!()`/`include_bytes!()`/`include!()`
    // or a `#[path = "..."]` module attribute. Stripping those dirs breaks
    // `cargo build --offline` post-vendor (winnow ships
    // `include_str!("../examples/css/parser.rs")` in `src/lib.rs`). Scan the
    // crate up front and skip stripping any top-level dir that's referenced
    // this way.
    let referenced_top_dirs = scan_referenced_top_dirs(crate_dir);

    // Remove configured directories
    for dir_name in config.dirs_to_strip() {
        if referenced_top_dirs.iter().any(|d| d == dir_name) {
            if v.debug() {
                eprintln!(
                    "  Preserving {}/{} — referenced by include_str!/include_bytes!/include!/#[path]",
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

/// Prune `[features]` entries that reference removed deps, in-place.
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
/// Cargo validates ALL `[features]` entries at parse time regardless of which
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
/// names referenced by `include_str!`/`include_bytes!`/`include!` macros and
/// by `#[path = "..."]` module attributes.
///
/// Two layers of detection:
///
/// 1. literal paths — resolve relative to the source file (matching rustc
///    for the include macros, and for `#[path]` outside inline modules) and
///    read the first component under the crate root.
/// 2. composed-path macros — scan all string literals in the macro's
///    argument span (handles `concat!(...)`) for `<...>tests/`, `benches/`,
///    `examples/`, `bin/` substrings and preserve those dirs. The same
///    substring sniff covers a `#[path]` inside an inline `mod { }`, whose
///    base directory differs from the file's.
///
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
        let literals = extract_include_arg_literals(&content)
            .into_iter()
            .chain(extract_path_attr_literals(&content));
        for literal in literals {
            // (1) Literal-path resolution: works for `include_str!("...")`.
            let resolved = parent.join(&literal);
            if let Ok(canon) = resolved.canonicalize()
                && let Ok(rel) = canon.strip_prefix(&crate_root)
                && let Some(first) = rel.components().next()
                && let Some(s) = first.as_os_str().to_str()
            {
                top_dirs.insert(s.to_string());
                continue;
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

/// Collect the absolute, canonicalized paths of every `.md` file under
/// `crate_dir` that is pulled into Rust source via `include_str!` /
/// `include_bytes!` / `include!`.
///
/// Such files are *source inputs*, not documentation: blanking them turns
/// `format!(include_str!("tpl.md"), name = …)` into `format!("", name = …)`,
/// which is a hard compile error (see #828). The caller uses the returned set
/// to skip those files during `--blank-md`.
///
/// Each macro-argument string literal is resolved relative to the referencing
/// `.rs` file (matching rustc's `include_str!` semantics) and canonicalized;
/// only paths that resolve to a real file under the crate root with a `.md`
/// extension are returned. Composed paths (`concat!(...)`) that don't resolve
/// to a literal file are ignored — they don't name a single `.md` to protect.
pub(crate) fn referenced_md_files(
    crate_dir: &Path,
) -> std::collections::BTreeSet<std::path::PathBuf> {
    let mut protected: std::collections::BTreeSet<std::path::PathBuf> =
        std::collections::BTreeSet::new();
    let crate_root = match crate_dir.canonicalize() {
        Ok(p) => p,
        Err(_) => return protected,
    };

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
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let Some(parent) = path.parent() else {
            continue;
        };
        for literal in extract_include_arg_literals(&content) {
            if !literal.ends_with(".md") {
                continue;
            }
            // Resolve relative to the referencing .rs file (rustc semantics),
            // canonicalize, and confirm it stays within the crate root.
            let resolved = parent.join(&literal);
            if let Ok(canon) = resolved.canonicalize()
                && canon.starts_with(&crate_root)
            {
                protected.insert(canon);
            }
        }
    }
    protected
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

/// Pull the string literal out of every `#[path = "..."]` attribute, the
/// module-file override rustc honours on `mod name;` declarations. A crate
/// can point one at a file under `tests/` / `benches/` / `examples/` from
/// regular library source, which stripping that directory would break.
fn extract_path_attr_literals(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let needle = b"#[path";
    let skip_ws = |mut i: usize| {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        i
    };
    let mut search = 0usize;
    while let Some(idx) = find_subslice(&bytes[search..], needle) {
        let start = search + idx + needle.len();
        search = start;
        // `#[path = "..."]`, not `#[path_like]` or `#[path(...)]`.
        let i = skip_ws(start);
        if i >= bytes.len() || bytes[i] != b'=' {
            continue;
        }
        let i = skip_ws(i + 1);
        if i >= bytes.len() || bytes[i] != b'"' {
            continue;
        }
        let Some(len) = bytes[i + 1..].iter().position(|&b| b == b'"') else {
            break;
        };
        out.push(content[i + 1..i + 1 + len].to_string());
        search = i + 1 + len;
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
mod tests;
