# Plan: #1125 — `DataFrame::group_by_multi` (composite keys)

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `feat/1125-group-by-multi`.

Sequencing: #1126 (grouped_df ingest,
`plans/2026-07-11-issue-1126-grouped-df-ingest.md`) builds on this —
dispatch #1126 only after this PR merges.

## Decisions baked in

- New method `group_by_multi(&self, cols: &[&str]) -> Result<GroupedDataFrame,
  DataFrameError>`; the existing single-column `group_by`
  (`miniextendr-api/src/dataframe/group.rs:212`) keeps its signature.
- New `GroupKey::Tuple(Vec<GroupKey>)` variant (`group.rs:68` — existing
  derives `Debug, Clone, PartialEq, Eq, Hash` all hold for `Vec<GroupKey>`).
  `label()`/`Display` join element labels with `"."` (R `interaction()`
  default separator). Nested tuples are impossible by construction (element
  keys come from scalar columns) — `debug_assert!` that.
- **Order contract**: non-NA groups match
  `split(df, interaction(df$a, df$b, drop = TRUE))` EXACTLY — write the
  testthat parity test FIRST against real R output and make Rust match it
  (do not guess which factor varies fastest; the R output is the spec).
- **NA contract** (extends the single-column convention, per the issue's
  note): any tuple containing at least one NA element forms its own
  per-tuple group, ordered AFTER all non-NA groups, in first-encounter row
  order. (`split()+interaction()` drops NA rows, so this is the documented
  divergence — document it on `group_by_multi`'s rustdoc and in the
  DataFrame docs page that documents `group_by`.)
- `cols` validation: empty slice → error; unknown column → the same error
  shape `group_by` uses for a missing column; single-element slice must
  produce results identical to `group_by(col)` except keys are 1-tuples —
  NO: bake instead that a 1-element slice delegates to the scalar path and
  returns scalar keys (identical to `group_by`), so downstream `match`es on
  `GroupKey` never see 1-tuples.

## Implementation

1. Refactor the per-type bucket helpers (`factor_groups:247`,
   `character_groups:278`, `integer_groups:299`, `logical_groups:323`) to
   share per-row key extractors: `fn column_keys(column: SEXP) ->
   Result<Vec<GroupKey>, DataFrameError>` dispatching on SEXPTYPE exactly as
   `group_by:217-222` does (factor → level string, STRSXP/INTSXP/LGLSXP,
   same unsupported-type error). Single-column path may keep its current
   direct bucketing OR rebuild on the extractor — pick whichever leaves
   `group_by`'s observable behavior byte-identical (existing tests are the
   proof).
2. `group_by_multi`: extract `column_keys` per column in one pass each, zip
   rows into `GroupKey::Tuple`, bucket into `Vec<(GroupKey, Vec<usize>)>`,
   order per the contracts above, wrap in `GroupedDataFrame` (its rooting
   comment at `group.rs:110-123` applies unchanged — source preserved,
   `!Send`).
3. rpkg fixtures — extend `rpkg/src/rust/dataframe_group_tests.rs` mirroring
   `group_by_keys:37` / `group_by_sizes:46` / `group_by_frames:62`:
   `group_by_multi_keys(df, cols: Vec<String>)`,
   `group_by_multi_sizes(...)`, `group_by_multi_frames(...)`. New exports →
   ×2 install.
4. testthat (the file that tests dataframe_group_tests fixtures — grep
   `group_by_keys` under `rpkg/tests/testthat/`): the parity test from the
   order contract; the NA-tuple trailing test; labels joined with `.`;
   1-element-slice ≡ `group_by`.
5. gc-stress: `dataframe_group_tests.rs`'s existing gc-stress arrangement is
   the model — if `gc_stress_fixtures.rs` has a group_by driver, add a
   multi-column arm; if not, `GroupedDataFrame`'s rooting is pre-existing
   and unchanged, and no new SEXP-storage path is introduced beyond it —
   state that in the PR body (#430 rule reasoning).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/1125-api.log
just configure && just rcmdinstall && just force-document && just rcmdinstall
just devtools-test 2>&1 > /tmp/1125-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1125-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- `group_by`'s observable behavior (keys, order, errors) — pinned by
  existing tests.
- `GroupedDataFrame`'s rooting mechanics; the `frames`/`extract` methods'
  contracts (they must work unchanged on Tuple-keyed groups — verify via the
  `group_by_multi_frames` fixture).
- The DataFrame/BuiltDataFrame split (#1128/#1247) — grouping reads views;
  no new constructors.

## Done criteria

- Multi-column grouping matches R `split(interaction())` parity for non-NA
  groups; NA-tuple trailing groups pinned; labels `.`-joined; suites + three
  clippy legs green; `Fixes #1125`.

## Escalation rule

If reality diverges from this plan — the parity test exposes an ordering R
convention this plan mischaracterized, `frames`/`extract` need changes for
Tuple keys — **stop, commit nothing further, and report back. Do not
improvise.**
