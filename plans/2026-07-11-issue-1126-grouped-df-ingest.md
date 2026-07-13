# Plan: #1126 — ingest dplyr `grouped_df` metadata (`attr(df, "groups")`)

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/1126-grouped-df-ingest`.

**Blocked on #1125** (`plans/2026-07-11-issue-1125-group-by-multi-column.md`
— needs `GroupKey::Tuple` for multi-key groupings). Dispatch only after that
PR merges; start by merging `origin/main` into this seeded branch.

## Decisions baked in

- Method name: `DataFrame::group_by_metadata()` → `Result<GroupedDataFrame,
  DataFrameError>` (the issue left the name open; this one says what it
  reads). Errors — all as `DataFrameError` with messages naming the defect:
  no `groups` attribute / not a data.frame → "not a grouped_df"; missing
  `.rows` list-column; `.rows` element not INTSXP/REALSXP-integerish; any
  index `< 1` or `> nrow` → out-of-range error naming group index and value.
- Keys: the groups frame's non-`.rows` columns become the group keys —
  single key column → scalar `GroupKey` (via the #1125 `column_keys`
  extractor); multiple → `GroupKey::Tuple`, labels `.`-joined (consistent
  with #1125).
- 1-based → 0-based conversion for every `.rows` element.
- `.drop = FALSE` empty groups (zero-length `.rows`): KEEP them as groups
  with empty index vectors — the existing empty-factor-level convention
  (`factor_groups` retains empty levels; read `group.rs:247-277` and mirror
  whatever it does for empty levels; if it does NOT retain them, keep empty
  metadata groups anyway and document the divergence on the method rustdoc —
  the caller's dplyr grouping is authoritative).
- NO recompute fallback: a plain (non-grouped) data.frame errors; callers
  who want recomputation use `group_by`/`group_by_multi`. Document on the
  method.
- Ordering: preserve the groups-frame row order verbatim (dplyr's order is
  authoritative; no re-sorting, no NA reordering).

## Implementation

1. `group.rs`: read `attr(df, "groups")` via the crate's attribute-access
   idiom (grep `Rf_getAttrib` usage in `dataframe.rs`/`group.rs` and copy
   it, checked-FFI conventions included). Validate per the decisions; build
   `Vec<(GroupKey, Vec<usize>)>` and wrap in `GroupedDataFrame` (existing
   rooting applies — the SOURCE frame is preserved; the groups attribute
   frame is only read during construction, inside the call, so it needs no
   separate root — state this in a comment).
2. rpkg fixtures in `rpkg/src/rust/dataframe_group_tests.rs` (mirror
   existing style): `group_metadata_keys(df)`, `group_metadata_sizes(df)`,
   `group_metadata_frames(df)`. New exports → ×2 install.
3. testthat: build a real grouped_df in the test
   (`dplyr::group_by(df, a, b)` if dplyr is available —
   `skip_if_not_installed("dplyr")`; ALSO hand-construct the `groups`
   attribute with `structure()`/`list()` in a second test so coverage does
   not depend on dplyr) covering: single- and two-column keys; `.drop=FALSE`
   empty group retained; out-of-range index errors; non-grouped frame
   errors; 1-based→0-based correctness (row values, not just sizes).
4. Docs: the DataFrame grouping docs section gains a `grouped_df` paragraph
   (respecting caller grouping vs recomputing).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/1126-api.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1126-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1126-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Check whether `dplyr` is in `rproject.toml`'s dev dependency set before
using it in tests; if absent, rely on the hand-constructed-attribute test
only and note it (do NOT add dplyr to rproject.toml — dependency additions
are the maintainer's call; the `skip_if_not_installed` guard covers both
worlds).

## Must NOT touch

- `group_by`/`group_by_multi` behavior; `GroupedDataFrame` internals beyond
  the new constructor path; no aggregation/summarise surface (#501).

## Done criteria

- A dplyr-grouped frame's grouping is honored exactly (keys, order, empty
  groups) without recomputation; validation errors pinned; suites + three
  clippy legs green; `Fixes #1126`.

## Escalation rule

If reality diverges from this plan — the groups-attribute layout differs
from the description, `GroupedDataFrame` cannot represent an
empty-index group, #1125's extractor isn't reusable — **stop, commit
nothing further, and report back. Do not improvise.**
