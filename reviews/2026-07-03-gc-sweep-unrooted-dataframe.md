# The gc_stress dynamic sweep's first catch: unrooted `DataFrame` across allocations

Date: 2026-07-03. Context: plan 06 (`plans/2026-07-01-gc-stress-sweep-r-backfill.md`,
audit A8) added a self-registering testthat sweep that runs every exported
no-arg `gc_stress_*` fixture under `gctorture(TRUE)`. 26 of the 56 fixtures had
never had a committed runner. The sweep's first full pass found two live bugs.

## What was attempted

`just devtools-test` with the new sweep in `test-gc-stress-fixtures.R`
(5 iterations per fixture under gctorture).

## What went wrong

1. **Segfault** (address 0x10, "invalid permissions", `*** recursive gc
   invocation` spam) in `gc_stress_dispatch_to_dataframes` — R aborts, whole
   suite dies.
2. **Intermittent R error** in `gc_stress_reader_nested_flatten`:
   `"DataFrame always carries a names attribute"` (the `.expect` in
   `DataFrame::named_list`, dataframe.rs) on ~2 of 5 iterations.
3. `gc_stress_with_r_thread_stop` "failed" every iteration — false positive:
   that fixture *raises by design* (its point is the raw `Rf_error` longjmp
   path; `test-worker-longjmp.R` asserts the error). The sweep now excludes it
   with a comment.

## Root cause (bugs 1 and 2 are the same class)

`DataFrame` (miniextendr-api/src/dataframe.rs) is a `#[derive(Clone, Copy)]`
wrapper over a **bare, unrooted SEXP**. `SerdeRowBuilder::finish()` /
`into_dataframe()` assemble the frame inside a `ProtectScope`, drop the scope,
and return the unrooted wrapper. Any R allocation while Rust still holds that
wrapper lets the GC reclaim the frame:

- `dispatch_to_dataframes` held `ok_df` unrooted across `err_builder.finish()`
  (which allocates the entire err frame) → freed node → segfault when the
  named-list builder touched it.
- The derive reader path (`FromDataFrame for Vec<T>` →
  `rows_from_dataframe`) read from a Rust-built frame with no GC root;
  reader-internal allocations reclaimed it mid-read → the names attribute
  vanished → expect fired. Intermittent because a freed node's payload stays
  intact until the allocator reuses it — **the sibling reader fixtures that
  "passed" were reading freed-but-intact memory.**

This is the same bug class as
`reviews/2026-05-29-serde-deserialize-fixture-gctorture-input-protect.md`;
serde's `dataframe_to_vec` already carries the input-rooting guard, which is
why the serde-path fixtures were genuinely safe while the derive-reader path
was not.

## Fix

- `dispatch_to_dataframes` (serde/columnar.rs): `OwnedProtect` both finished
  frames until the output list is assembled (guards drop LIFO).
- Blanket `FromDataFrame for Vec<T>` (dataframe.rs), both `from_dataframe`
  and `from_dataframe_par`: root the input frame for the duration of the
  read, mirroring `dataframe_to_vec`.
- Sweep: documented exclusion for the one raises-by-design fixture.

## Lessons

- A fixture without a runner is dead weight — both bugs were in *fixtures'
  production paths* that had existed for weeks; the fixtures were exported but
  never called by any committed test (audit A8's exact complaint).
- "Passed under gctorture" is necessary, not sufficient: freed-but-intact
  reads pass silently. When one variant of a family fails intermittently,
  audit the whole family's protection story, not just the failing member.
- `DataFrame`'s unrooted-`Copy`-wrapper design makes every
  hold-across-allocation site a latent UAF; the targeted guards fix the known
  compositions, the design-level fix is tracked in a follow-up issue
  (see PR body).
