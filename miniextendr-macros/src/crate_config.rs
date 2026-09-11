//! Crate-level `#[miniextendr]` defaults read from the user crate's manifest.
//!
//! Proc-macro invocations share no state, so a default meant for every
//! `#[miniextendr]` item in a crate cannot be declared on one macro call:
//! `miniextendr_init!` expands with no knowledge of the functions and vice
//! versa. The one place every expansion can see is the manifest. Cargo sets
//! `CARGO_MANIFEST_DIR` for the crate being compiled and the proc macro runs
//! inside that `rustc` process, so it can read `Cargo.toml` at expansion time.
//! Cargo refingerprints a package when its manifest changes, so an edit takes
//! effect on the next build with no stale expansions.
//!
//! Supported spelling (the table header; dotted keys under `[package]` or
//! `[package.metadata]` are accepted too, inline tables are rejected):
//!
//! ```toml
//! [package.metadata.miniextendr]
//! noexport_postfix = "_impl"
//! ```
//!
//! The reader is a deliberately small line-based scanner rather than a TOML
//! dependency: the macro crate ships in every downstream build, and the only
//! value it needs is a single string under one well-known table.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Dotted path of the only key the macro reads today.
const NOEXPORT_POSTFIX_KEY: &str = "package.metadata.miniextendr.noexport_postfix";
/// Dotted path of the table itself, used to reject the inline-table spelling.
const TABLE_KEY: &str = "package.metadata.miniextendr";

/// Crate-wide defaults declared under `[package.metadata.miniextendr]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CrateConfig {
    /// `noexport_postfix = "..."`: appended to the Rust name of every
    /// `noexport` / `internal` free function that sets neither `r_name` nor
    /// its own `postfix`. Exported functions keep their Rust names.
    pub(crate) noexport_postfix: Option<String>,
}

/// A malformed `[package.metadata.miniextendr]` entry, reported as a compile
/// error on the `#[miniextendr]` item that tried to use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CrateConfigError {
    manifest: PathBuf,
    message: String,
}

impl std::fmt::Display for CrateConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: [package.metadata.miniextendr] {}",
            self.manifest.display(),
            self.message
        )
    }
}

/// The crate config for the crate currently being compiled.
///
/// Returns the defaults when `CARGO_MANIFEST_DIR` is unset or its `Cargo.toml`
/// cannot be read; only a present-but-malformed entry is an error. Results are
/// cached per manifest directory, so a single proc-macro server process serving
/// several crates (rust-analyzer) never leaks one crate's default into another.
pub(crate) fn crate_config() -> Result<CrateConfig, CrateConfigError> {
    let Some(dir) = std::env::var_os("CARGO_MANIFEST_DIR") else {
        return Ok(CrateConfig::default());
    };
    crate_config_for_dir(Path::new(&dir))
}

/// [`crate_config`] for an explicit manifest directory (the cached entry point).
pub(crate) fn crate_config_for_dir(dir: &Path) -> Result<CrateConfig, CrateConfigError> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Result<CrateConfig, CrateConfigError>>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .entry(dir.to_path_buf())
        .or_insert_with(|| read_crate_config(&dir.join("Cargo.toml")))
        .clone()
}

/// Read and parse one manifest. Missing or unreadable files yield the defaults.
fn read_crate_config(manifest: &Path) -> Result<CrateConfig, CrateConfigError> {
    match std::fs::read_to_string(manifest) {
        Ok(text) => parse_crate_config(&text).map_err(|message| CrateConfigError {
            manifest: manifest.to_path_buf(),
            message,
        }),
        Err(_) => Ok(CrateConfig::default()),
    }
}

/// Parse the `[package.metadata.miniextendr]` table out of manifest text.
///
/// Tracks the current table header and resolves every `key = value` line to a
/// dotted path, so the header form, `miniextendr.noexport_postfix = "..."`
/// under `[package.metadata]`, and `metadata.miniextendr.noexport_postfix`
/// under `[package]` all resolve to the same key. Everything else in the
/// manifest is skipped without interpretation.
pub(crate) fn parse_crate_config(text: &str) -> Result<CrateConfig, String> {
    let mut config = CrateConfig::default();
    let mut table = String::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            // `[[array.of.tables]]` and `[table]`: keys below belong to it.
            let header = header.strip_prefix('[').unwrap_or(header);
            let end = header
                .find(']')
                .ok_or_else(|| format!("unterminated table header `{line}`"))?;
            table = normalize_key(&header[..end]);
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
        if full == TABLE_KEY && value.trim_start().starts_with('{') {
            return Err(
                "inline tables are not supported here; write a `[package.metadata.miniextendr]` \
                 table with `noexport_postfix = \"...\"` on its own line"
                    .to_string(),
            );
        }
        if full != NOEXPORT_POSTFIX_KEY {
            continue;
        }
        if config.noexport_postfix.is_some() {
            return Err("`noexport_postfix` is set more than once".to_string());
        }
        let value = parse_string_value(value.trim()).ok_or_else(|| {
            format!(
                "`noexport_postfix` must be a string, found `{}`",
                value.trim()
            )
        })?;
        validate_noexport_postfix(&value)?;
        config.noexport_postfix = Some(value);
    }

    Ok(config)
}

/// Collapse a dotted key or table header to `a.b.c`: trims each segment and
/// strips one layer of quotes so `[ "package" . metadata ]` compares equal to
/// `[package.metadata]`.
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

/// Parse a single-line TOML basic (`"..."`) or literal (`'...'`) string.
///
/// Escape sequences are not accepted: the value must be an R identifier
/// fragment, which never needs one. Multi-line strings and non-string values
/// return `None`.
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
    if body.contains('\\') {
        return None;
    }
    let trailing = rest[end + 1..].trim_start();
    if !(trailing.is_empty() || trailing.starts_with('#')) {
        return None;
    }
    Some(body.to_string())
}

/// Mirror of `miniextendr_fn::validate_postfix` for the manifest value.
fn validate_noexport_postfix(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("`noexport_postfix` must not be empty".to_string());
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(
            "`noexport_postfix` must be a valid R identifier fragment (letters, digits, `_`, `.`)"
                .to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn postfix(text: &str) -> Result<Option<String>, String> {
        parse_crate_config(text).map(|c| c.noexport_postfix)
    }

    #[test]
    fn header_form_is_read() {
        let text = r#"
[package]
name = "demo"

[package.metadata.miniextendr]
noexport_postfix = "_impl" # trailing comment

[dependencies]
miniextendr-api = "0.1"
"#;
        assert_eq!(postfix(text), Ok(Some("_impl".to_string())));
    }

    #[test]
    fn dotted_forms_are_read() {
        let under_metadata = "[package.metadata]\nminiextendr.noexport_postfix = '_impl'\n";
        assert_eq!(postfix(under_metadata), Ok(Some("_impl".to_string())));

        let under_package = "[package]\nmetadata.miniextendr.noexport_postfix = \"_rs\"\n";
        assert_eq!(postfix(under_package), Ok(Some("_rs".to_string())));

        let quoted_header =
            "[ \"package\" . metadata . 'miniextendr' ]\nnoexport_postfix = \"_impl\"\n";
        assert_eq!(postfix(quoted_header), Ok(Some("_impl".to_string())));
    }

    #[test]
    fn unrelated_tables_and_keys_are_ignored() {
        let text = r#"
[package.metadata.docs.rs]
noexport_postfix = "not-ours"

[package.metadata.miniextendr]
other_key = "ignored"

[[bin]]
name = "x"
noexport_postfix = "also not ours"
"#;
        assert_eq!(postfix(text), Ok(None));
        assert_eq!(postfix(""), Ok(None));
        assert_eq!(postfix("[package]\nname = \"demo\"\n"), Ok(None));
    }

    #[test]
    fn malformed_values_are_errors() {
        let empty = "[package.metadata.miniextendr]\nnoexport_postfix = \"\"\n";
        assert!(postfix(empty).unwrap_err().contains("must not be empty"));

        let bad_chars = "[package.metadata.miniextendr]\nnoexport_postfix = \"-impl\"\n";
        assert!(
            postfix(bad_chars)
                .unwrap_err()
                .contains("identifier fragment")
        );

        let not_a_string = "[package.metadata.miniextendr]\nnoexport_postfix = 1\n";
        assert!(
            postfix(not_a_string)
                .unwrap_err()
                .contains("must be a string")
        );

        let escaped = "[package.metadata.miniextendr]\nnoexport_postfix = \"a\\tb\"\n";
        assert!(postfix(escaped).unwrap_err().contains("must be a string"));

        let multiline = "[package.metadata.miniextendr]\nnoexport_postfix = \"\"\"_impl\"\"\"\n";
        assert!(postfix(multiline).unwrap_err().contains("must be a string"));

        let twice = "[package.metadata.miniextendr]\nnoexport_postfix = \"_a\"\nnoexport_postfix = \"_b\"\n";
        assert!(postfix(twice).unwrap_err().contains("more than once"));

        let inline = "[package.metadata]\nminiextendr = { noexport_postfix = \"_impl\" }\n";
        assert!(postfix(inline).unwrap_err().contains("inline tables"));
    }

    #[test]
    fn manifest_dir_lookup_reads_and_caches() {
        let dir = std::env::temp_dir().join(format!(
            "miniextendr-crate-config-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\n\n[package.metadata.miniextendr]\nnoexport_postfix = \"_impl\"\n",
        )
        .unwrap();

        let first = crate_config_for_dir(&dir).unwrap();
        assert_eq!(first.noexport_postfix.as_deref(), Some("_impl"));

        // The cache answers subsequent lookups; a later edit is not observed
        // within one process (Cargo starts a new rustc per manifest change).
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"demo\"\n").unwrap();
        let second = crate_config_for_dir(&dir).unwrap();
        assert_eq!(second, first);

        let missing = crate_config_for_dir(&dir.join("does-not-exist")).unwrap();
        assert_eq!(missing, CrateConfig::default());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
