# Plan: detect duplicate git-source for same crate name+version (#255)

Align cargo-revendor with upstream cargo's hard-error on conflicting git
sources. Small defensive addition.

## Problem

Upstream `/Users/elea/Documents/GitHub/cargo/src/cargo/ops/vendor.rs` errors
when two different git sources resolve to the same `(name, version)` pair
(rare but possible — two repos publishing the same crate name). cargo-revendor
has no equivalent check; it silently last-write-wins during extraction, so
the vendored crate contents depend on dep-graph iteration order.

## Files to change

- `cargo-revendor/src/vendor.rs` — add duplicate-source collection + check
  before the extraction loop.
- `cargo-revendor/tests/git_deps.rs` — add regression test.

## Implementation

1. **Study upstream's logic**. Read
   `/Users/elea/Documents/GitHub/cargo/src/cargo/ops/vendor.rs` around the
   `SourceId`/`duplicate` handling. Upstream collects into a BTreeMap keyed
   by `(name, version)` → `(SourceId, path)`; on second insert at the same
   key, errors if the SourceIds differ.

2. **Mirror the check in cargo-revendor's extraction path**. Before iterating
   to extract, collect:

```rust
use std::collections::BTreeMap;

#[derive(Debug)]
struct ResolvedSource {
    source_url: String, // e.g. git URL or "registry+https://..."
    commit: Option<String>, // for git deps, the resolved rev
}

fn check_duplicate_sources(
    packages: &[cargo_metadata::Package],
) -> Result<()> {
    let mut seen: BTreeMap<(String, String), ResolvedSource> = BTreeMap::new();
    for pkg in packages {
        let key = (pkg.name.clone(), pkg.version.to_string());
        // Source::to_string in cargo-metadata gives us a URL-ish identifier.
        // For git deps the commit is in the Source URL; for registry deps
        // the URL is the registry itself (same for all registry-deps, never
        // a conflict).
        let source = pkg
            .source
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let current = ResolvedSource {
            source_url: source.clone(),
            commit: None, // pkg.source already encodes commit for git sources
        };
        if let Some(prev) = seen.get(&key) {
            if prev.source_url != current.source_url {
                bail!(
                    "duplicate crate `{} v{}` from different sources:\n  - {}\n  - {}",
                    pkg.name,
                    pkg.version,
                    prev.source_url,
                    current.source_url,
                );
            }
        } else {
            seen.insert(key, current);
        }
    }
    Ok(())
}
```

3. Invoke at the top of the extraction flow (inside the main vendor function,
   after `cargo_metadata` is available, before any extraction loop).

4. **Test**: `tests/git_deps.rs` — add `G6_duplicate_git_sources_errors`.
   Fixture: a workspace with two deps that both name `foo v1.2.3` from
   different git URLs. Assert `cargo revendor` exits non-zero with stderr
   containing "duplicate crate".

   Constructing the fixture is the tricky part — cargo-metadata will only
   resolve if the graph is valid. Easiest is to mock-fixture two test
   registries or use two local-git-dep repos serving the same name+version.
   See `tests/common/mod.rs` for `LocalGitRepo` helpers.

5. Edge case: `[patch.crates-io]` legitimately replaces a registry crate with
   a git source. Walk the `pkg.source` field carefully — a patched dep should
   NOT be flagged as duplicate of the registry version. Upstream handles this
   by only comparing sources that made it into the final resolved graph.
   Mirror that behavior.

## Verification

```bash
just revendor-test
cd cargo-revendor && cargo test --test git_deps duplicate
# regression check: existing G1–G5 still pass (normal git deps not flagged)
```

## Out of scope

- Detecting duplicate path-dep sources — not in upstream either, and the
  workspace semantic of "one path, one crate" makes this moot
- Improving the error message with suggested remediation — basic error is
  enough; can be polished in a follow-up

## Risk

Low. Pure addition. The only behavior change is adding an error case that
previously silently last-wrote. If any user currently relies on the silent
behavior (extremely unlikely), they'll see the new error and must pick one
source.

## PR expectations

- Branch: `fix/issue-255-duplicate-git-sources`
- No merge — CR review
- Cross-reference upstream cargo line numbers in the PR body for audit
