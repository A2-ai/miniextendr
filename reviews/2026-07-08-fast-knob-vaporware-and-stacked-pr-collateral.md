# The `fast`/`no_preconditions`/`no_call_attribution` knobs — two abandoned
# landings, an issue closed against its own recommendation, and stale docs
# that still describe them as shipped

**Date:** 2026-07-08
**Area:** `miniextendr-macros` fn/impl attribute parsing, issue tracker hygiene,
`analysis/scaffolding-*` evidence chain
**Trigger:** benchmark-audit investigation (asked to go deeper on the finding
that `analysis/scaffolding-fast-bench.R` references nonexistent functions)

## TL;DR

The `fast` knob (`no_preconditions` / `no_call_attribution` / `fast` /
`no_fast` on `#[miniextendr]` fns and impl blocks) has been implemented
**twice** and is on `main` **neither** time. It is not dead because the idea
is bad — the measured wins (7.78x/8.54x/2.19x/3.36x) were never disputed. It
died both times as **collateral damage of branch-stacking**: both attempts
were stacked underneath a *different*, separately-controversial feature
(`error_direct`), and closing the base closed the child with it. The
`analysis/scaffolding-*` docs and `bench-class-dispatch.R`'s scaffolding
predecessor still read as if the feature shipped; nobody has corrected them
since the second abandonment three weeks ago. The code for attempt #2 is not
lost — it is sitting on `origin/feat/g6-fast-default-666`, 86 commits behind
`main`.

## Timeline

1. **2026-05-20** — `Mossa` (the maintainer) writes `e3279011` (fn/impl-level
   `fast` knob codegen) + `ea4d22b4` (M2/M4 attribution benches) directly on
   top of main-at-the-time. **Never opened as a PR** (or opened and not
   merged) — both commits are dangling objects today, unreachable from any
   ref (`git branch -a --contains e3279011` → empty), only recoverable via
   their raw SHAs because they're still in the local object DB.
2. **2026-06-09** — An automated Claude Code triage loop picks up issue #669
   ("docs: surface the fast/no_preconditions/no_call_attribution knobs"),
   discovers the feature doesn't exist on `main` (`FN_BOOL_FLAGS_HELP` has no
   such flags; `e3279011`/`ea4d22b4` are dangling), writes
   `reviews/2026-06-09-fast-knobs-never-merged.md` (PR #947), and comments on
   #669 with the finding — **explicitly recommending the issue stay open** as
   the blocked tracking item ("Suggest keeping this issue open... blocked on
   the feature landing"). `Mossa` closes #669 as **COMPLETED** the same day,
   34 minutes after the comment posts — the opposite of what the comment
   recommended.
3. **2026-06-11** — #663 (`infallible` knob), #664 (`borrow_args` knob), #668
   (S7-dispatch perf research) all close as COMPLETED. Not re-audited in this
   pass; worth a spot-check if picking this back up.
4. **2026-06-13** — PR #1019 merges (`e74ec527`): re-lands *only* the 16
   `analysis/scaffolding-*` docs/scripts and two inert divan benches
   (`c_side_attribution.rs`, `error_path_attribution.rs`) from the
   `e3279011`/`ea4d22b4` chain, explicitly scoped "No codegen, no regen, no
   fixtures." Its own description says the real feature would land via a
   stacked pair: **#950 (G6-PR1, `error_direct`, #665) → #1018 (G6-PR2, the
   fast-knob base + `fast-default` cargo feature, #666)** — i.e. the docs
   PR merged *before*, and independently of, the code PRs it describes.
5. **2026-07-01** — `Mossa` reviews #950 (`error_direct`) and comments:
   *"I don't think this is a good idea actually. It would need a bunch of
   analysis and audits to ensure that this strategy is as good as the
   current condition-object signals."* — a legitimate correctness concern
   about `error_direct`'s C-side-direct-raise approach vs. the existing
   tagged-condition path, **not** a comment about the fast knob. `Mossa`
   closes both #950 and #1018 within 14 seconds of each other. #1018 was
   stacked on #950's branch (`perf/error-direct-knob-665`), so closing the
   base took the fast-knob PR down with it even though nobody raised an
   objection to the fast-knob content itself.
6. **2026-07-08 (today)** — `analysis/scaffolding-perf-roadmap.md` and
   `analysis/scaffolding-fast-bench.R` (both still on `main` from step 4)
   still read as if the feature is live: the roadmap's "quick start" tells
   the next session to run `Rscript -e 'library(miniextendr);
   fast_i32_fast(42L)'`, which errors. Issue #669 remains closed. Issues
   #666 (`default-fast` cargo feature), #667 (should `internal` imply
   `fast`?), #670 (typed-error transport), #1017 (per-method fast override)
   remain open, all silently blocked on a base that was built twice and
   shipped zero times.

## Root cause

Two independent things, not one:

- **Branch-stacking makes an uncontroversial feature hostage to a
  controversial one.** #1018's actual content (four new `#[miniextendr]`
  flags, `fast-default` cargo feature, 15 testthat cases, UI snapshots) has
  no recorded technical objection anywhere in this investigation. It died
  solely because its branch's parent (`perf/error-direct-knob-665`) carried
  a feature the maintainer decided needed more analysis. Nothing forced
  `#666` to depend on `#665` other than PR-authoring convenience ("G6 landing
  order").
- **Docs-first landing order inverted the safety property it was supposed to
  have.** PR #1019 was deliberately scoped as the safe, inert tail of a
  3-PR stack ("No codegen, no regen, no fixtures") — reasonable in isolation,
  but it merged the evidence *before* the code, so for the three weeks
  between #1019 merging and #950/#1018 closing (and every day since), `main`
  has had documentation asserting a shipped feature with zero corresponding
  code. The 2026-06-09 triage comment predicted almost exactly this failure
  mode for the *first* abandonment and it recurred anyway for the second.

Separately, **the issue-closing pattern is closing things that aren't
resolved.** #669 was closed against its own comment's explicit
recommendation. Given the volume of triage in this repo (84 open issues per
the 2026-07-07 ledger), this reads like closing-as-acknowledgment rather than
closing-as-resolved — worth watching for elsewhere.

## What's actually recoverable

`origin/feat/g6-fast-default-666` (PR #1018's head) still exists and is not
force-pushed over:

```
3985865b docs(rpkg): regen fast_fixtures.Rd to match current roxygen source
00c8ee01 feat(macros): fast knob codegen base + fast-default cargo feature (#666)
54e17a3e fix(error_direct): handle RCondition data field; forward data on fallback   <- from the #950 stack, not this feature
```

86 commits behind `main`, 4 ahead. The bottom commit (`54e17a3e`) belongs to
the `error_direct` stack and would need to be dropped/rebased out; the top
two are the fast-knob feature itself. A clean re-land would mean rebasing
just `00c8ee01` + `3985865b` onto current `main` (not onto `#950`'s branch),
which touches `miniextendr-macros/src/{lib,miniextendr_fn,miniextendr_impl,
r_class_formatter}.rs` and all 6 class generators — files the 2026-07
audit wave (A1-A14) has also been touching, so expect real conflicts, not a
clean cherry-pick.

## Fix

Not applied in this pass — this review documents the investigation only.
Options for whoever picks this up (flat, no priority implied):

- Rebase `00c8ee01`/`3985865b` onto `main` standalone (drop the `error_direct`
  dependency entirely), reopen #666/#1017 against it, close #669 for real
  once `analysis/scaffolding-fast-bench.R` is runnable again.
- Or: correct `analysis/scaffolding-perf-roadmap.md` and
  `analysis/scaffolding-fast-bench.R` to stop asserting the feature is live
  (mark them historical/aspirational) and leave re-implementation for later —
  cheaper, but leaves #666/#667/#670/#1017 blocked indefinitely.
- Either way: reopen #669 (or file a fresh consolidating issue) — its
  underlying problem was never resolved, closing it just hid that fact.

## Lesson

`[[feedback_stop_punting_questions]]`-style automated recommendations can be
overridden by a human closing the issue anyway — a comment saying "keep this
open" is not self-enforcing. When a PR stack's tail (docs/benches) is scoped
as "safe to merge independently," that safety is about *review risk*, not
about *truth* — the docs still assert facts about the unmerged base PRs, and
if those never land, the "safe" PR is the one now shipping a falsehood.
Stack order matters for correctness, not just for review convenience: don't
stack an uncontroversial feature under a controversial one on the same
branch chain if the two are severable, or the controversial one's rejection
takes the uncontroversial one with it.
