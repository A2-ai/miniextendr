//! Where the module walk starts: the library target's root file.
//!
//! Cargo takes it from `[lib] path` in `Cargo.toml`. Without that key the lint
//! keeps its two guesses: `src/lib.rs` when the crate has a `src/` directory,
//! else `lib.rs` next to `Cargo.toml` (every scaffolded R package sets
//! `[lib] path = "lib.rs"`, so the guess only matters for manifests without it).
//!
//! The key is read with a small line-based scanner, so the lint gains no TOML
//! dependency. Mirror of the reader in `miniextendr-macros/src/crate_config.rs`
//! (`normalize_key`, `parse_string_value`): the current table header is tracked
//! and every `key = value` line resolves to a dotted path, so `[lib]` followed by
//! `path = "..."` and a top-level `lib.path = "..."` are the same key, and a
//! `path` under any other table (`[[bin]]`, `[package]`, `[dependencies.lib]`) is
//! a different one.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Dotted path of the key that names the library target's root file.
const LIB_PATH_KEY: &str = "lib.path";

/// Where [`CrateRoot::file`] came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootOrigin {
    /// `[lib] path` in `Cargo.toml`.
    Manifest,
    /// No `[lib] path`: `src/lib.rs` when `src/` exists, else `lib.rs`.
    Default,
}

/// The library target's root file, resolved for a manifest directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrateRoot {
    /// The root file. It may not exist; [`crate::CrateIndex::build`] reports that.
    pub file: PathBuf,
    /// Whether the path came from `[lib] path` or from the default guess.
    pub origin: RootOrigin,
}

impl CrateRoot {
    /// Resolve the root file of the crate whose `Cargo.toml` sits in `manifest_dir`.
    ///
    /// A missing or unreadable `Cargo.toml` resolves to the default guess. Only a
    /// `[lib] path` the scanner cannot read is an error, so the lint never walks
    /// a guessed tree while the manifest names another one.
    pub fn resolve(manifest_dir: &Path) -> Result<Self, String> {
        let manifest = manifest_dir.join("Cargo.toml");
        let lib_path = match fs::read_to_string(&manifest) {
            Ok(text) => lib_path_from_manifest(&text).map_err(|message| {
                format!("miniextendr-lint: {}: {message}", manifest.display())
            })?,
            Err(_) => None,
        };

        Ok(match lib_path {
            Some(path) => Self {
                file: normalize(&manifest_dir.join(path)),
                origin: RootOrigin::Manifest,
            },
            None => {
                let src_dir = manifest_dir.join("src");
                let dir = if src_dir.is_dir() {
                    src_dir
                } else {
                    manifest_dir.to_path_buf()
                };
                Self {
                    file: dir.join("lib.rs"),
                    origin: RootOrigin::Default,
                }
            }
        })
    }
}

impl fmt::Display for RootOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Manifest => "[lib] path in Cargo.toml",
            Self::Default => "no [lib] path in Cargo.toml",
        })
    }
}

/// Whether the crate sets `roxygen_prose_links = "keep"` under
/// `[package.metadata.miniextendr]`: leading doc prose then keeps its links on
/// the way to roxygen2. Same line scanner as [`lib_path_from_manifest`]; a
/// missing, unreadable or malformed value reads as the default (`"strip"`).
pub(crate) fn prose_links_keep(manifest_dir: &Path) -> bool {
    const KEY: &str = "package.metadata.miniextendr.roxygen_prose_links";
    let Ok(text) = fs::read_to_string(manifest_dir.join("Cargo.toml")) else {
        return false;
    };
    let mut table = String::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            let header = header.strip_prefix('[').unwrap_or(header);
            if let Some(end) = header.find(']') {
                table = normalize_key(&header[..end]);
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = normalize_key(key);
        let full = if table.is_empty() {
            key
        } else {
            format!("{table}.{key}")
        };
        if full == KEY {
            return parse_string_value(value.trim()).is_some_and(|v| v == "keep");
        }
    }
    false
}

/// `manifest_dir.join(path)` without `.` components, so `path = "./lib.rs"`
/// names the same file as `path = "lib.rs"`.
fn normalize(path: &Path) -> PathBuf {
    path.components().collect()
}

/// The `[lib] path` value of manifest text, `None` when the key is absent.
///
/// Errors when the key is present but its value is not a single-line string
/// without escapes, or when the `[lib]` table is written inline with a `path`.
fn lib_path_from_manifest(text: &str) -> Result<Option<String>, String> {
    let mut table = String::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            // `[[array.of.tables]]` and `[table]`: keys below belong to it. A
            // line with no `]` (a nested array spread over lines) opens no
            // table; the lint skips it where the macro crate's copy errors, so
            // an unrelated value never costs the crate its lint.
            let header = header.strip_prefix('[').unwrap_or(header);
            if let Some(end) = header.find(']') {
                table = normalize_key(&header[..end]);
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = normalize_key(key);
        let full = if table.is_empty() {
            key
        } else {
            format!("{table}.{key}")
        };
        let value = value.trim();
        if full == "lib" && value.starts_with('{') && inline_table_has_path(value) {
            return Err(
                "an inline `lib = { path = ... }` table is not read; write a `[lib]` table with \
                 `path = \"...\"` on its own line"
                    .to_string(),
            );
        }
        if full != LIB_PATH_KEY {
            continue;
        }
        return parse_string_value(value).map(Some).ok_or_else(|| {
            format!(
                "cannot read [lib] path `{value}`; write it as a one-line string without escapes"
            )
        });
    }

    Ok(None)
}

/// Whether a one-line inline table names a `path` key. Splitting on `,` can cut
/// an array or a string apart, but every key still starts a piece of its own.
fn inline_table_has_path(value: &str) -> bool {
    value
        .trim_start_matches('{')
        .split(',')
        .filter_map(|piece| piece.split_once('='))
        .any(|(key, _)| normalize_key(key) == "path")
}

/// Collapse a dotted key or table header to `a.b.c`: trims each segment and
/// strips one layer of quotes so `[ "lib" ]` compares equal to `[lib]`.
fn normalize_key(key: &str) -> String {
    key.split('.')
        .map(|segment| {
            let segment = segment.trim();
            segment
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .or_else(|| {
                    segment
                        .strip_prefix('\'')
                        .and_then(|s| s.strip_suffix('\''))
                })
                .unwrap_or(segment)
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// Parse a single-line TOML basic (`"..."`) or literal (`'...'`) string,
/// allowing a trailing `# comment`. Escape sequences in a basic string,
/// multi-line strings and non-string values return `None`. Unlike the macro
/// crate's copy, a literal string keeps its backslashes (TOML has no escapes
/// there), so `path = 'rust\lib.rs'` reads as written.
fn parse_string_value(value: &str) -> Option<String> {
    let (quote, rest) = match value.chars().next()? {
        c @ ('"' | '\'') => (c, &value[1..]),
        _ => return None,
    };
    if rest.starts_with(quote) && rest[1..].starts_with(quote) {
        // `"""` / `'''`: multi-line strings are not supported.
        return None;
    }
    let end = rest.find(quote)?;
    let body = &rest[..end];
    if body.contains('\\') && quote == '"' {
        return None;
    }
    let trailing = rest[end + 1..].trim_start();
    if !(trailing.is_empty() || trailing.starts_with('#')) {
        return None;
    }
    Some(body.to_string())
}
