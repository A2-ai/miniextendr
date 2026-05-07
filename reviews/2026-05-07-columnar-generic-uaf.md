# 2026-05-07 — `ColumnBuffer::Generic` SEXP use-after-free (issue #307, PR #424)

## What was attempted

Add new test fixtures for #307 covering `ColumnarDataFrame::from_rows` behaviour
on `Option<T>` columns where every row is `None`. Two of the new fixtures —
`test_columnar_bytes_with_values` and `test_columnar_bytes_and_opt_none` —
introduced rows with a `Vec<u8>` field, which routes through
`ColumnBuffer::Generic` (R list column), pushing real R-allocated SEXPs into a
`Vec<Option<SEXP>>` for later assembly into a parent VECSXP.

## What went wrong

CI's `R CMD check / Linux release` job (R 4.6) aborted partway through the full
suite with `malloc(): unsorted double linked list corrupted` followed by
`Aborted (core dumped)`. Linux devel and oldrel-1 both passed the same suite;
local macOS `devtools::test()` passed at 4999 PASS / 0 FAIL.

Initial assumption was flake (the crash signature pointed at glibc heap
consistency check, the failing tests were unrelated `LazyIntSeq` ALTREP cases).
Re-running the failed CI didn't help. User pushed back: heap corruption is
deterministic — the bug is real, just hidden behind tolerant allocators.

## Root cause

`ColumnBuffer::Generic(Vec<Option<SEXP>>)` stored raw R-managed pointers across
the fill→assemble boundary without rooting them in R's protect stack. The fill
phase calls `RSerializer::serialize`, gets a freshly-allocated SEXP, and pushes
it into the buffer. Each subsequent row's serialize call, plus
`Rf_allocVector(VECSXP, …)` inside `column_to_sexp`, is a GC opportunity that
can free those buffered SEXPs before `set_vector_elt` reads them in phase 4.

Pre-existing bug — present before #307. The new `Vec<u8>` fixtures were the
first tests in the project to exercise this path with real (non-nil)
SEXPs across multiple rows, so the latent UAF only surfaced now.

`gctorture(TRUE)` deterministically reproduced the crash on
`test_columnar_bytes_with_values` after `library(miniextendr)`. This was the
turning point — a 30-second local repro instead of a 12-minute CI gamble.

## Fix

`miniextendr-api/src/serde/columnar.rs` — opened a `ProtectScope` in `from_rows`
between buffer allocation and the fill loop, threaded `&'a ProtectScope` through
`ColumnFiller`, and modified `ColumnBuffer::push_value`'s Generic branch to
`scope.protect_raw(sexp)` before storing. Scope drops after `assemble_dataframe`
returns, by which point each Generic SEXP has been written into the parent
VECSXP via `set_vector_elt` and is rooted by R's normal write-barrier mechanism.

Stack discipline holds — assemble's internal `Rf_protect`/`Rf_unprotect(N)` push
and pop on top of the scope's N entries; the scope's batch unprotect happens
after assemble has balanced its own.

Verification: 37/37 columnar test functions survive `gctorture(TRUE)`
post-patch; `devtools::test()` still 4999 PASS / 0 FAIL.

## Lessons

- **Heap-checker output on one CI runner is not flake.** glibc's
  `malloc(): unsorted double linked list corrupted` is deterministic; other
  allocators just accept the corrupted write silently. If one runner aborts and
  others pass, suspect strict-allocator-vs-tolerant-allocator before flake.
- **`ColumnBuffer::Generic` is a UAF trap.** Any `Vec<SEXP>` held across
  allocations needs explicit rooting. Future generic-list code should protect
  at push time, not hope set-time write barriers cover the gap.
- **gctorture pays for itself.** A 30-second local gctorture run found the bug
  after a 12-minute CI roundtrip surfaced only the symptom. See
  `docs/GCTORTURE_TESTING.md` for the harness pattern.
- **Plan files lie about scope.** The original plan said "no other risks"; the
  agent's commit shipped a real GC bug that survived 4999 testthat assertions
  and three local clippy variants. Rule: any new path that holds an SEXP across
  an allocation gets a gctorture pass before commit.
