# `fast` / `no_preconditions` / `no_call_attribution` knobs never landed (#669)

**Date:** 2026-06-09
**Area:** `miniextendr-macros/src/miniextendr_fn.rs`, project memory, issue #669
**PR:** (this one)

## What was attempted

Issue #669 (`documentation`) asked to surface three `#[miniextendr]` knobs —
`fast`, `no_preconditions`, `no_call_attribution` — in `CLAUDE.md`,
`miniextendr-macros/CLAUDE.md`, `docs/`, and the proc-macro docstring. The
issue, the parent commit message (`e3279011`), and the project memory entry
*"Fast knobs shipped 2026-05-20"* all describe the feature as present and
merged. The task was framed as pure documentation polish (~30 min).

## What went wrong

The feature **does not exist anywhere in the repository.** Documenting it would
have told users to write `#[miniextendr(fast)]`, which today is a hard compile
error (`unknown #[miniextendr] option; expected one of: invisible, visible,
check_interrupt, worker, no_worker, coerce, no_coerce, rng, unwrap_in_r, strict,
no_strict, internal, noexport, export`). Writing the docs as requested would
have been a regression, not a fix.

## Root cause

The branch that introduced the knobs (commits `e3279011`, `ea4d22b4`) was
**never merged into `main`** — it was apparently superseded or discarded. Every
check points the same way:

- `git merge-base --is-ancestor e3279011 main` → **not an ancestor**;
  same for `ea4d22b4`. No ref (`git for-each-ref --contains`) holds either.
- `grep -rn "no_preconditions\|no_call_attribution\|with_fast_flags\|fast_flags"`
  over `miniextendr-macros/`, `miniextendr-api/`, `rpkg/` (excluding `target/`
  and `analysis/`) → **zero matches.**
- `FN_BOOL_FLAGS_HELP` in `miniextendr_fn.rs` and the `MiniextendrFnAttrs`
  flag parser list **none** of the three flags; the parser would reject them.
- No PR (open or closed) touches `fast_fixtures.rs`; GitHub code search for
  `no_call_attribution` in the repo returns nothing.
- `rpkg/src/rust/fast_fixtures.rs`, `rpkg/man/FastCounter*.Rd`, and the
  `analysis/scaffolding-*` evidence chain — all listed in `e3279011 --stat` —
  are **absent** from the working tree and from `main`.

The project-memory line *"Fast knobs shipped 2026-05-20 … e3279011 + ea4d22b4;
8.25× speedup"* is **incorrect**: those commits live only on an unmerged branch.
The follow-up issues #663–#670 (e.g. #667 *"should `internal` imply `fast`?"*)
also assume the feature exists, so they are blocked on the underlying PR landing
first.

## The numbers (from the unmerged commit message `e3279011`, for the record)

Apple M3 Max / R 4.6.0, were the feature to land:

| Workload | before | after | ratio |
|---|---|---|---|
| 1-arg standalone fn | 2870 ns | 369 ns | 7.78× |
| 3-arg standalone fn | 4551 ns | 533 ns | 8.54× |
| R6 `value()` | 2337 ns | 1066 ns | 2.19× |
| R6 `add(1L)` | 3854 ns | 1148 ns | 3.36× |

The issue's headline *"8.25×"* is not a single measured datapoint — it sits
between the 1-arg (7.78×) and 3-arg (8.54×) standalone-fn ratios. There is no
artifact in the tree to substantiate any of these figures; they are recorded
here purely as the design intent on the unmerged branch.

Intended semantics (per the same commit message, **not yet code**):

- `no_preconditions` drops the R-side `stopifnot(...)` block; `TryFromSexp`
  still raises on bad input, but the error message originates in Rust rather
  than R (loses the named-argument R-side diagnostic).
- `no_call_attribution` emits `.call = NULL` instead of `.call = match.call()`;
  the R-side raise helper falls back to `sys.call()`, so errors still carry a
  call but with positional rather than named arguments.
- `fast` is a bundle alias for both. The trade-off is purely *error-UX
  precision* for *wrapper speed* — neither knob weakens memory or type safety
  (`TryFromSexp` validation is unchanged).

## The stale `analysis/scaffolding-*` reference

The issue cites `analysis/scaffolding-perf-roadmap.md` (P11/P12) and an
`analysis/scaffolding-*` chain as the system of record. These files exist **only
in the unmerged `e3279011 --stat`** — `git log --all -- 'analysis/scaffolding-*'`
returns nothing, and none are in the working tree. They cannot be cited as live
references. The real (and only) source of record today is the unmerged commit
message itself.

## Fix

No user-facing documentation was added, because there is nothing in the codebase
to document. The correct resolution is **not a docs PR** — it is to re-land the
feature branch (or re-implement the knobs) first, then document. This review
records the discrepancy so the next person doesn't re-discover it. The project
memory entry should be corrected from *"shipped"* to *"never merged"*.

## Lesson

Project memory and issue framing are not ground truth — verify a feature exists
in the working tree (and that its commits are ancestors of `main`) before
documenting it. A squash-merge can leave the original feature commit
unreachable; a feature branch can be abandoned while its follow-up issues and
memory notes live on as if it shipped. `git merge-base --is-ancestor <sha> main`
plus a working-tree grep is the decisive check, not the commit's own `--stat`.
