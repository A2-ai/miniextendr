# Concurrent installs in one worktree corrupt generated wrappers.R — and perfectly mimic a GC bug

**Date**: 2026-06-11
**Context**: rebasing PR #965 (`feat/764-struct-map-reader`) onto main in
`.claude/worktrees/pr965-rebase`, with a background agent dispatched to do the
same rebase still alive in the same worktree.

## What was attempted

`just rcmdinstall` in the worktree, repeated after failures, then "bisect"
runs against intermediate main commits and plain `origin/main` to localise a
suspected regression.

## What went wrong

Installs intermittently failed during lazy-loading with parse errors in the
generated `rpkg/R/miniextendr-wrappers.R`, at a *different* position each run:

```
116:   rounding `return(...)`
              ^ unexpected symbol
```

The corrupted lines were mid-word splices of other parts of the same generated
file (`rounding `return(...)`` is the tail of "the wrapper's su**rrounding
`return(...)`** propagates" from the file header; another run produced
`.valrounding ...`). One "bisect" run failed with a different error entirely
(`namespaceExport` mismatch).

## The misdiagnosis

The splice signature — fragments of one string appearing inside another, at
non-deterministic positions, with identical sources — pattern-matched exactly
to unprotected-CHARSXP GC corruption (the prior `column_to_sexp` STRSXP bug,
#672/#674). Runs against plain `origin/main` also failed, "confirming" a main
regression. Several hours went into reading the writer
(`miniextendr-api/src/registry.rs::write_r_wrappers_to_file`) for a protect
bug that does not exist: the writer assembles a pure Rust `String` and cannot
be GC-corrupted.

## Root cause

**Every failing run — including the "plain main" bisect runs — executed inside
the same worktree as the still-running background agent**, sharing
`rpkg/R/miniextendr-wrappers.R`, the cargo target dir, and the rv library.
Two `R CMD INSTALL` pipelines interleave like this:

- the Makevars wrapper-gen step truncates + rewrites `wrappers.R`
  (`std::fs::write`, non-atomic) while
- the other install's lazy-loading step *parses the same path*, and
- a second writer can truncate the file under the first writer's open fd,
  producing offset-shifted splices — exactly the observed corruption.

cargo's artifact-dir lock serializes *cargo* invocations only. The
`Blocking waiting for file lock on artifact directory` line in the failing log
was the racer announcing itself, misread as background noise. The
`namespaceExport` failure was the same race hitting a different window
(installed wrappers vs NAMESPACE from different generations).

## Proof

- The wrapper writer looped 40× in isolation against a freshly built cdylib
  (fresh output path each iteration, same Rscript invocation as Makevars):
  **40/40 byte-identical** outputs. The writer is deterministic and clean.
- After the agent terminated, the very same worktree went green first try:
  `just rcmdinstall`, `just force-document` (zero artifact drift), and the
  PR's testthat file (118 PASS / 0 FAIL).

## Fix / lessons

1. **One builder per worktree.** Never run `just rcmdinstall` / `R CMD
   INSTALL` in a worktree an agent (or any other process) may still be
   building in. `ps aux | grep -E "R CMD|Rscript|cargo"` first.
2. **`Blocking waiting for file lock on artifact directory` is a red flag**,
   not noise: it proves a concurrent cargo build shares your target dir, and
   the non-cargo build steps (wrapper-gen Rscript, R's lazy-load parse) have
   *no* lock at all.
3. **Bisect data taken while a racer is alive is garbage.** Non-deterministic
   pass/fail on identical commits means an environmental cause; stop bisecting
   and look for the racer.
4. Corruption-shaped text in a *generated file* does not implicate the
   generator's memory safety when the generator's output is built as a plain
   Rust `String` — reproduce the generator in isolation before reading it for
   GC bugs.

Hardening follow-up: #967 — write generated artifacts (`wrappers.R`,
`wasm_registry.rs`) to a temp file + atomic rename so concurrent readers can
never observe a torn write.
