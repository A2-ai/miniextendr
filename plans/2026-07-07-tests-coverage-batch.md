# Tests / coverage batch — #1190, #1139, #996(+#1103 verify)

Re-verified 2026-07-11 against main @ 17f634d8 (original pass: 88c493fd).
Three PR-sized units, independent — any order:

- TC-1 #1190 → branch `test/1190-altrep-clamp-live`
- TC-2 #1139 → branch `fix/1139-main-thread-id-test-override`
- TC-3 #1103+#996 → branch `fix/996-1103-condition-data-roundtrip`

Former pointers now live elsewhere: #1097 →
`plans/2026-07-11-issue-1097-intoras-batched-errors.md`; #1113 →
`plans/2026-07-11-issue-1113-feature-legs-full-suite.md`.

Shared commands (each unit; worktree):

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
cargo test -p miniextendr-api 2>&1 > /tmp/tc-api.log     # Read the log
just configure && just rcmdinstall && just force-document && just rcmdinstall
#                                   ^ ×2 ONLY when the unit adds a new rpkg export (TC-3 fixture)
just test 2>&1 > /tmp/tc-rust.log
just devtools-test 2>&1 > /tmp/tc-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/tc-devtools.log    # devtools::test always exits 0
just cross-test 2>&1 > /tmp/tc-cross.log         # TC-3 only (cross-package test)
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
```

Escalation rule (all units): if reality diverges from this plan — an anchor
doesn't match, a premise-check comes out differently than both branches
described — **stop, commit nothing further, and report back. Do not
improvise.**

---

## TC-1 — #1190: live-SEXP coverage for ALTREP negative-index clamps

Clamp anchors (verified): `miniextendr-api/src/altrep_impl/macros.rs:494`
(`i.max(0)`), `:515` (`start < 0 || len <= 0`), `:944`, `:1100`, `:1186`.
Nothing exercises them at the SEXP-taking bridge level.

**Gating decision (updated from the issue)**: NO gating. `miniextendr-api/tests/`
is an existing live-R integration suite (boots R via miniextendr-engine
through `tests/r_test_utils.rs::with_r_thread`, one process per test binary;
`altrep_extract.rs` / `altrep_from_data_matrix.rs` / `altrep_thread.rs`
already do exactly this under plain `cargo test`). Follow that convention.

1. New `miniextendr-api/tests/altrep_clamp.rs` (copy the harness boilerplate
   from `altrep_extract.rs` — same `r_test_utils` include pattern, same
   thread shape).
2. Inside `with_r_thread`: register/construct ONE live ALTREP instance the
   same way the existing altrep integration tests do (reuse their fixture
   type — read `altrep_extract.rs` and reuse its class; do not invent a new
   registration path).
3. Call the bridge-level entry points with negative indices:
   `elt(x, -1)` → element-0 value (the `i.max(0)` clamp), and
   `get_region(x, -1, ...)` / a negative-start region → the `:515` guard's
   empty/zero result. Assert clamped results, not panics. Cover one
   integer-family case and one logical-or-string case so both macro arms
   (`:494` and `:944`) execute; the list arm (`:1186`) if the reused fixture
   family allows it cheaply, else note in PR body.
4. No rpkg change, no regen, no snapshots. `Fixes #1190`.

Done: `cargo test -p miniextendr-api --test altrep_clamp` green; the clamp
lines are reachable from a real R session (delete-the-clamp → test fails —
verify once locally, do not commit the mutation).

## TC-2 — #1139: R_MAIN_THREAD_ID OnceLock blocks in-crate unit tests

Anchors (verified): `miniextendr-api/src/worker.rs:69`
(`static R_MAIN_THREAD_ID: OnceLock<ThreadId>`), reader `:90`, the
already-set diagnostic at `:246-249`.

Decision baked (option a): test-only override.

1. Replace the raw `OnceLock` read path with a two-layer lookup: production
   stays `OnceLock` (zero cost); add a `#[cfg(test)]`-only
   `static R_MAIN_THREAD_ID_TEST_OVERRIDE: Mutex<Option<ThreadId>>` checked
   FIRST by the getter (`:90`), plus
   `#[cfg(test)] pub(crate) fn force_main_thread_id_for_test(id: ThreadId)`.
   Keep the public surface unchanged.
2. Surface the silent loss: where registration does `let _ = ...set(...)`,
   log under `#[cfg(test)]` (eprintln) when a set loses the race, naming both
   thread ids — mirror the wording of the `:246-249` diagnostic.
3. Regression proof: a new `#[cfg(test)]` mod in worker.rs (or the crate's
   unit-test convention) that spins a dedicated R-like thread, calls
   `force_main_thread_id_for_test`, and exercises one `pub(crate)`
   FFI-checked fn — passing under FULL parallel `cargo test -p
   miniextendr-api --lib` (run it 3× to shake the race).
4. `Fixes #1139`. No rpkg change, no regen.

## TC-3 — #1103 (verify, likely close) + #996 (or_raise data splicing)

Premise state (verified 2026-07-11):
- `ConditionData` is `Vec<(String, crate::RValue)>` (`condition.rs:168`) and
  `RValue` is Option-bearing — #1103's "NA silently dropped" premise is
  probably stale.
- `RCondition::from_tagged_sexp` (`condition.rs:667`) ALREADY reads slot [4]
  (`:732-749`, `len >= 5` branch) — #996 path 1 is probably already
  implemented and only needs the cross-package proof.
- `raise_rust_condition_via_stop` (`unwind_protect.rs:104`) has NO data
  splicing (verified — no data handling in `:104-165`) — #996 path 2 is the
  real work.

1. **#1103 step 0 (test-first)**: rpkg testthat —
   `error!(data = ("count", <NA integer>))`-shaped fixture (find the existing
   condition-data fixture in `rpkg/src/rust/condition_demo.rs` and extend
   with an NA-bearing case; new export → ×2 install) asserting `e$count` is
   `NA` (present, not dropped). If green: #1103 is FIXED — keep the pinning
   test in this PR and state in the PR body that #1103 closes as
   already-fixed (`Fixes #1103`). If red: STOP and report (escalation rule) —
   do not design Opt* variants ad hoc.
2. **#996 path 1 (verify + pin)**: cross-package test in
   `tests/cross-package/` — producer.pkg raises `error!(data = (...))`
   through the trait-ABI re-panic path; consumer.pkg asserts `e$field`.
   Follow the existing cross-package test layout (see its testthat dirs).
   Expected green given `:749`; if red, that's in-scope wiring: connect the
   slot-[4] read's `ConditionData` into the re-panic reconstruction where
   `from_tagged_sexp`'s result is consumed (`condition.rs:839` region).
3. **#996 path 2 (implement)**: extend `raise_rust_condition_via_stop`
   (`unwind_protect.rs:104`) to carry the condition's data fields into the
   evaluated `stop(structure(...))` — mirror how the main-thread path
   materializes `ConditionData` into the condition object
   (`error_value.rs` / `condition.rs` `into_sexp` side — grep
   `make_rust_condition_value` data handling and reuse its materializer).
   CAUTION: `error_value.rs` is PROTECT-sensitive; if the change touches it,
   run the gctorture pass (`docs/GCTORTURE_TESTING.md` harness) before
   commit. Test: `error!(data = ...)` raised from an ALTREP callback
   (`rpkg` has ALTREP condition fixtures — `altrep_condition_tests.rs`)
   → `e$field` present in R.
4. PR body: `Fixes #996` + `Fixes #1103` (or the step-0 caveat), and the
   note that path 1 was already wired and is now pinned.

Must NOT touch (TC-3): the tagged-condition transport shape (slot layout),
`RValue` variants, worker channel paths (#989 is a separate open decision).
