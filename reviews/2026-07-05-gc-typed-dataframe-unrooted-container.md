# 2026-07-05 — gc_stress_typed_dataframe: unrooted container UAF flaking unrelated PRs

## What was attempted

Post-sweep CI verification of the rebased open PRs (all had been force-pushed
onto current main during the 2026-07-04/05 review sweep).

## What went wrong

Three unrelated branches went red on the same rerun wave:

- #1043 and #1044: `gc_stress_typed_dataframe` failed the gctorture sweep with
  `DataFrame always carries a names attribute` (iterations 3 and 5
  respectively). Neither PR touches typed dataframes.
- #1134 (minirextendr-only diff): rpkg testthat segfaulted on the R-devel
  check job with `*** recursive gc invocation` spam — plausibly the same bug
  surfacing as a crash instead of a clean panic.

Main's own CI was green throughout.

## Root cause

`List::from_raw_pairs` PROTECTs its list and names vectors only *during*
construction — the `OwnedProtect` guards drop on return, so the returned
`List` wraps an unprotected VECSXP. `gc_stress_typed_dataframe` (added with
the #1129 sweep) then called `as_data_frame()` on it, which allocates the
class STRSXP and row.names INTSXP, and drove `TheophDf::try_from_sexp` over
the result. Under gctorture the container node is reaped at the first of
those allocations.

The read-after-free is **silent until R reuses the node**. That is why:
- main's CI passed (schedule happened not to reuse the node in the window),
- unrelated PRs failed stochastically (their extra fixtures/tests shifted the
  allocation schedule),
- the failure iteration differed per branch (3 vs 5),
- one branch got a segfault instead of the panic (whatever reused the node
  determined the failure mode).

This is the #1128 hazard class verbatim: `DataFrame`/`List` are unrooted Copy
SEXP wrappers; "freed-but-intact reads pass tests silently."

## Fix

PR #1145: root the container into the fixture's existing `ProtectScope`
immediately after `from_raw_pairs`, before `as_data_frame()` allocates —
both halves of the fixture. Verified 100/100 gctorture iterations post-fix.

Production callers are unaffected (their input SEXP arrives from R rooted in
the call frame); only the fixture synthesises an unrooted container.

## Lessons

- A gc_stress fixture that *itself* violates PROTECT discipline produces
  flaky red CI on innocent branches — the failure lands wherever the
  allocation schedule shifts, not where the bug lives. When an unrelated PR
  trips a gc_stress fixture, first suspect the fixture's own rooting.
- `from_raw_pairs` (and friends) protecting "during construction only" is an
  API trap: every synthetic-container caller must remember to root the result
  before the next allocation. The #1128 redesign discussion is the durable
  fix; until then, grep new fixtures for `from_raw_pairs` → allocation
  sequences.
- The same UAF can present as a clean panic, a wrong-value assert, or a
  segfault ("recursive gc invocation") — do not treat those as three bugs
  before checking for one unrooted SEXP.
