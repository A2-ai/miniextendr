# Disk-full disguised as three different e2e test corruption bugs

**Date**: 2026-06-11
**Context**: implementing #914 (version-consistency testthat test); running
`just minirextendr-test` in an agent worktree during an 8-agent parallel sprint.

## What was attempted

Full `just minirextendr-test` runs to validate a new (pure-R, no-compile) test
file before opening a PR.

## What went wrong

Three consecutive runs failed in `test-templates.R` e2e build tests
(lines 419/506/565) with *different* errors each time:

1. `internal error 1 in R_decompress1 with libdeflate` /
   `lazy-load database '.../mxroundtrip.rdb' is corrupt`
2. `error[E0786]: found invalid metadata files for crate miniextendr_macros`
   followed by a cascade of `unresolved imports crate::sys::*_unchecked`
   (the proc-macro that generates the `_unchecked` variants failed to load)
3. Finally the honest one: `rustc-LLVM ERROR: IO failure on output stream:
   No space left on device`

Failure set was non-deterministic across runs (1 failure, then 3, then 1),
which initially looked like parallel-agent races on shared caches.

## Root cause

The disk was at 100% (246 MiB free; 884 Gi used). Truncated writes during
`R CMD INSTALL` and cargo builds produced corrupt `.rdb` lazy-load databases
and dylibs with invalid metadata. Errors 1 and 2 are *downstream corruption
symptoms* — only the third run surfaced the actual ENOSPC. The multi-GB
`target/` dirs in 4+ parallel agent worktrees plus temp-dir e2e builds
(~1.2 GB in `$TMPDIR`) tipped it over.

## Fix

None in code — environmental. The new `test-version-consistency.R` is pure R
(reads two text files) and unaffected; 526 suite tests pass apart from the
disk-induced e2e flakes.

## Lessons

- `R_decompress1`/`lazy-load database is corrupt` and rustc
  `invalid metadata files for crate` during *fresh* builds: check `df -h`
  before chasing toolchain/race theories.
- An `E0786 invalid metadata` on a proc-macro crate cascades into bogus
  "unresolved import" errors for everything the macro generates — the first
  error is the only meaningful one.
- Multi-agent worktree sprints carry 2–3 GB of `target/` per agent
  (CLAUDE.md already warns); budget disk before dispatching a fleet.
