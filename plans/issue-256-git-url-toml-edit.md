# Plan: structural toml_edit parsing for git URLs in `generate_cargo_config` (#256)

Replace the fragile line-regex in `vendor.rs:628–634` with structural
toml_edit traversal. Modest refactor; aligns cargo-revendor with upstream
cargo-vendor.

## Problem

The current code at `cargo-revendor/src/vendor.rs:628–634`:

```rust
for line in manifest_content.lines() {
    if let Some(start) = line.find("git = \"https://") {
        let url_start = start + 7; // skip `git = "`
        if let Some(end) = line[url_start..].find('"') {
            git_urls.insert(line[url_start..url_start + end].to_string());
        }
    }
}
```

Known miss cases:
- `git="https://..."` (no spaces around `=`) — `find("git = \"")` fails
- `git = "http://..."` or `git = "ssh://..."` — `https://` prefix filters them out
- `git = "...", rev = "..."` on one line — the closing `"` found is for the URL, that works, but brittle
- `[dependencies.foo]` table-header form where `git` is below the header — works today but only accidentally

## Files to change

- `cargo-revendor/src/vendor.rs` — rewrite the git-URL extraction block in
  `generate_cargo_config` (~lines 620–641).
- New helper module or inline: `collect_git_sources(manifest_path: &Path) -> Result<Vec<GitSource>>`.
- `cargo-revendor/tests/integration.rs` or new `git_sources.rs` — unit tests
  for the extractor over the 4 problematic shapes above.

## Implementation

1. **Add `GitSource` struct**:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GitSource {
    url: String,
    rev: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
}
```

2. **Add `collect_git_sources` that walks the manifest with toml_edit**:

```rust
use toml_edit::{DocumentMut, Item, Table, Value};

fn collect_git_sources(manifest_path: &Path) -> Result<Vec<GitSource>> {
    let content = std::fs::read_to_string(manifest_path)?;
    let doc: DocumentMut = content.parse()?;
    let mut sources = Vec::new();
    for tbl_name in &[
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
    ] {
        if let Some(Item::Table(tbl)) = doc.get(tbl_name) {
            collect_from_dep_table(tbl, &mut sources);
        }
    }
    // Also walk [target.*.dependencies], [target.*.build-dependencies]
    if let Some(Item::Table(target_tbl)) = doc.get("target") {
        for (_cfg, target_item) in target_tbl.iter() {
            if let Item::Table(cfg_tbl) = target_item {
                for tbl_name in &["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some(Item::Table(tbl)) = cfg_tbl.get(tbl_name) {
                        collect_from_dep_table(tbl, &mut sources);
                    }
                }
            }
        }
    }
    Ok(sources)
}

fn collect_from_dep_table(tbl: &Table, out: &mut Vec<GitSource>) {
    for (_name, item) in tbl.iter() {
        if let Item::Value(Value::InlineTable(inline)) = item {
            if let Some(url) = inline.get("git").and_then(|v| v.as_str()) {
                out.push(GitSource {
                    url: url.to_string(),
                    rev: inline.get("rev").and_then(|v| v.as_str()).map(String::from),
                    branch: inline.get("branch").and_then(|v| v.as_str()).map(String::from),
                    tag: inline.get("tag").and_then(|v| v.as_str()).map(String::from),
                });
            }
        } else if let Item::Table(subtbl) = item {
            // [dependencies.foo] table form
            if let Some(url) = subtbl.get("git").and_then(|i| i.as_str()) {
                out.push(GitSource {
                    url: url.to_string(),
                    rev: subtbl.get("rev").and_then(|i| i.as_str()).map(String::from),
                    branch: subtbl.get("branch").and_then(|i| i.as_str()).map(String::from),
                    tag: subtbl.get("tag").and_then(|i| i.as_str()).map(String::from),
                });
            }
        }
    }
}
```

3. **Wire into `generate_cargo_config`**:

```rust
// Replace lines 624–635 with:
let git_sources = collect_git_sources(manifest_path)?;
let git_urls: std::collections::BTreeSet<String> =
    git_sources.iter().map(|s| s.url.clone()).collect();
// existing emission of [source."git+..."] entries continues unchanged
```

4. **Unit tests** (`tests/git_sources.rs` or inline in vendor.rs if there's a
   test module already):
   - `git = "https://..."` with spaces
   - `git="https://..."` without spaces
   - `git = "ssh://..."`
   - `git = "https://...", rev = "..."` single-line inline table
   - `[dependencies.foo]\ngit = "..."` table form
   - `[target.'cfg(windows)'.dependencies]\nfoo = { git = "..." }` — target-gated
   - No `git =` field at all — return empty
   - URL scheme variants: https, http, ssh, git — all preserved verbatim

## Verification

```bash
just revendor-test
cd cargo-revendor && cargo test collect_git_sources
# manual: regenerate vendor/.cargo-config.toml for rpkg and diff against
# committed version; should be identical
```

## Out of scope

- Rewriting git deps in `freeze_manifest` to use vendor paths — that's #252
- Supporting non-`git`/`path`/registry sources (e.g. `paths = [...]`
  workspace-level overrides) — not used in miniextendr, scope creep

## Risk

Low. toml_edit is already a dep. The structural traversal is strictly more
correct than the regex. Main risk: missing a rare manifest shape — the
unit-test matrix above covers all known shapes; any exotic form found later
can be a follow-up fix.

## PR expectations

- Branch: `fix/issue-256-git-url-toml-edit`
- No merge — CR review
- Include a "before/after diff of vendor/.cargo-config.toml on a manifest
  with each of the 4 shapes" in the PR body to show the fix's effect
