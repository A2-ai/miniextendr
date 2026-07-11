# Plan: #1097 — batch all element errors in `IntoRAs` vector conversions

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `fix/1097-intoras-batched-errors`.

Decision baked in (ledger): **option 2** — change the error contract to an
aggregate (pre-release, no backcompat), implemented directly on the vector
`IntoRAs` impls (the blanket `TryCoerce` route is E0119-infeasible —
`audit/coerce.md` issue #4). Reuse #1192's grammar. No file overlap with the
#1217 PRs; land any order.

## Design

1. New variant on `StorageCoerceError` (`miniextendr-api/src/into_r_as.rs:59`;
   derives `Debug, Clone, PartialEq, Eq` — the new variant must keep those):
   ```rust
   /// Aggregated per-element failures from a vector conversion (#1097).
   /// `listed` holds at most the first BATCHED_ERROR_CAP element errors
   /// (each with `index` set); `total` counts all failures.
   Batched {
       container: &'static str,   // e.g. "Vec<i64>"
       listed: Vec<StorageCoerceError>,
       total: usize,
   },
   ```
2. Cap sharing: `BATCHED_ERROR_CAP` in `miniextendr-api/src/from_r.rs:1989`
   is file-private — widen to `pub(crate)` and import it here so the cap
   can't drift from the #1192 paths.
3. `Display` for `Batched`:
   `"{container} conversion failed: {e1}; {e2}; ...; and {total-listed} more"`
   (elements' own Display already carries `index`; the `and N more` tail only
   when `total > listed.len()` — mirror `BatchedErrors::into_error`,
   `from_r.rs:2037-2046`). Update the `at_index` helper (`:180`) with a
   `Batched` arm that is a no-op (indices live on the elements).
4. Convert every SHORT-CIRCUITING vector impl in `into_r_as.rs` to
   accumulate: the authoritative inventory is
   `grep -n 'at_index' miniextendr-api/src/into_r_as.rs` — currently the
   macro-generated impls (`:592-610`, two `.map_err(...)?` sites covering
   many `Vec<$from>` impls) plus any direct `Vec<...>` impls that propagate
   an element error (walk `:619-930`; impls that are infallible per element
   stay untouched). Pattern per site: enumerate; push successes; on failure,
   if `listed.len() < BATCHED_ERROR_CAP` push `e.at_index(i)`; count total;
   after the walk return `Err(StorageCoerceError::Batched { ... })` when
   `total > 0`. Container labels: use the same `from`/`to` naming the impl's
   scalar errors already use, formatted as `Vec<{from}>` (a `&'static str`
   per impl — hardcode per impl/macro-arm; do not invent a new naming
   scheme).
5. Scalar impls and the scalar error variants are UNCHANGED — a scalar
   conversion still returns the plain variant (no single-element `Batched`).
6. Callers: `grep -rn 'into_r_as' miniextendr-api miniextendr-macros rpkg/src/rust`
   — any caller that pattern-matches `StorageCoerceError` variants must gain
   a `Batched` arm (rustc will find them: the enum is exhaustively matched or
   Display-routed; fix what the compiler flags, changing no behavior beyond
   message text).

## Tests

7. Unit tests beside the existing into_r_as tests (find the test module via
   `grep -n 'mod tests' miniextendr-api/src/into_r_as.rs` or the
   `tests/` dir): `Vec<i64>` → i32 storage with failures at indices 3, 17, 42
   lists all three in one Display string; a 15-failure input shows 10 + `and
   5 more`; scalar failure Display unchanged.
8. R-side: grep `rpkg/tests/testthat/` for pins on storage-coerce messages
   (`conversion failed`/`out of range` shapes tied to `into_r_as` fixtures)
   and update to the batched grammar where a vector path is pinned.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/1097-api.log   # Read it
just configure && just rcmdinstall && just force-document
# (no new exports expected — single install)
just test 2>&1 > /tmp/1097-rust.log
just devtools-test 2>&1 > /tmp/1097-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/1097-devtools.log  # devtools::test always exits 0
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

No snapshot churn expected. If trybuild/insta move, stop and report (never
`TRYBUILD=overwrite` locally — #1239).

## Must NOT touch

- Which elements pass/fail (semantics identical; only aggregation changes).
- `BatchedErrors`/`SexpError` (#1192 machinery) beyond the `pub(crate)` cap.
- NA handling (#1217 item 4 decision pending).
- `coerce.rs` blankets (E0119 — documented infeasible).

## Done criteria

- All fallible `IntoRAs` vector impls report every failing index (cap 10 +
  `and N more`) in one error; scalar contract untouched; unit tests pin the
  shapes; suites + three clippy legs green; `Fixes #1097`.

## Escalation rule

If reality diverges from this plan — an impl's error flow doesn't match the
`at_index` inventory, a caller relies on receiving the FIRST error only, the
enum's `Eq` derive breaks — **stop, commit nothing further, and report back.
Do not improvise.**
