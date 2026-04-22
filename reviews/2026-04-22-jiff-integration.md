# Review: jiff integration (feat/jiff-integration)

**Date:** 2026-04-22
**Reviewer:** Mossa (main thread)
**Branch:** `feat/jiff-integration` — 4 agent commits on top of my spec/plan commits
**Commits reviewed:**

- `f0d473d5` — `[phase 1-7]` jiff: cargo wiring, core conversions, adapter traits, ALTREP, vctrs **(also includes phases 10+11)**
- `dfdd3775` — `[phase 8]` docs: FEATURES.md + CONVERSION_MATRIX.md
- `9304bb69` — `[phase 9]` rpkg fixtures + testthat tests
- `371675de` — `chore: update rpkg Cargo.lock`

**Verification reproduced locally:**

- `cargo clippy -p miniextendr-api --features jiff -- -D warnings` — clean
- `cargo clippy --workspace --all-targets --locked --features <clippy_all incl. jiff> -- -D warnings` — clean
- `just devtools-test` — `[ FAIL 0 | WARN 0 | SKIP 15 | PASS 4619 ]` (skips unrelated: indicatif, default-worker, thread-broken)

**Verdict:** merge-worthy after addressing the 4 must-fix items below. Six nice-to-haves / follow-ups recorded for the fix agent's discretion.

---

## Findings by phase

### Phase 1 — Cargo wiring + CI (✅ passes)

- jiff dep + feature declared correctly with `default-features = false, features = ["std", "tzdb-bundle-always"]`.
- `optionals.rs` module declaration + re-exports match the `time` pattern.
- `rpkg/src/rust/Cargo.toml` has `jiff = ["miniextendr-api/jiff"]`.
- CI `clippy_all` list extended with `jiff` in `.github/workflows/ci.yml`.

No issues.

### Phase 2 — Core UTC conversions (✅ passes)

- Timestamp + civil::Date, Option + Vec + Vec<Option> variants all present (`jiff_impl.rs:48–538`).
- Floor-based fractional-seconds split is correct.
- NA handling (`NaN` / `i32::MIN`) per convention.

No issues.

### Phase 3 — Zoned ↔ POSIXct (+ tzone) (⚠️ gap in test coverage)

Implementation is correct (`jiff_impl.rs:540–875`):

- `set_posixct_tz(sexp, iana)` helper added in `cached_class.rs:200–230` with proper `Rf_protect`/`Rf_unprotect` discipline.
- Unknown IANA → `SexpError::InvalidValue("unknown IANA tz {name}: {jiff error}")`. Matches spec.
- Empty / `"UTC"` tzone fast-paths to `TimeZone::UTC`.
- `Vec<Zoned>::into_sexp` picks first element's IANA for the vector-level `tzone` attr and logs a `log::warn!` on heterogeneous tz (gated on `#[cfg(feature = "log")]`). Matches spec.

**Gap (must-fix) — missing error-path test:**

The spec explicitly required a testthat test for "unknown tz error path", and the plan's phase 9.5 required the test `expect_error(jiff_zoned_round_trip(bad), "Mars/Olympus")`. **Nothing in `rpkg/tests/testthat/test-jiff.R` exercises the unknown-IANA error.** grep `Mars|unknown tz|invalid` → no match.

**Gap (nice-to-have) — missing heterogeneous-tz test:**

No test exercises `Vec<Zoned>::into_sexp` with mixed tzs. At minimum, a test that constructs a `Vec<Zoned>` with two different tzs (via rpkg fixture) and asserts the resulting POSIXct carries the first tz would catch regressions.

### Phase 4 — SignedDuration ↔ difftime (✅ passes)

Implementation at `jiff_impl.rs:877–1111`, `RSignedDuration` trait at `1113–1174`. Difftime class + `units="secs"` attr handled correctly. Tests cover zero, one-hour, negative, extraction.

No issues.

### Phase 5 — `Span` + `RSpan` adapter trait (⚠️ no fixtures, no R-side exercise)

`RSpan` trait is implemented (`jiff_impl.rs:1176–1246`) with all 14 methods as the plan specified. **But no rpkg fixture exposes `Span` or the trait to R.** grep `Span` in `rpkg/src/rust/jiff_adapter_tests.rs` → zero matches.

This means the adapter trait compiles but is untested end-to-end. The vctrs `span_vec_to_rcrd` helper (phase 10) is also uncovered (see phase 10).

**Gap (must-fix)** — add at minimum a `JiffSpan` `#[derive(ExternalPtr)]` fixture with `#[miniextendr] impl` forwarding to `RSpan`, plus a testthat block asserting component extraction (matches plan phase 9.5 which showed exactly this).

### Phase 6 — civil::DateTime + civil::Time (⚠️ no fixtures)

`RDateTime` trait at `1248–1299`, `RTime` trait at `1301–1332`. Same concern as phase 5: no rpkg fixture, no R-side test.

**Gap (nice-to-have)** — add minimal fixtures and ≥1 test per type. Lower priority than Span because DateTime/Time have less R-user expectation.

### Phase 7 — `RTimestamp`, `RZoned`, `RDate` adapter traits (✅ code, ⚠️ partial coverage)

All three traits implemented (`jiff_impl.rs:1334–1473`). `jiff_date_year/month/day` fixtures exercise `RDate` components, `jiff_zoned_year/month/tz_name` fixtures exercise `RZoned`. But **no fixture uses `RTimestamp::strftime`, `RZoned::start_of_day`, `RDate::weekday/day_of_year/first_of_month/last_of_month/tomorrow/yesterday`**. Implementation-only surface is fine if these are reachable from Rust callers only, but the plan requested coverage.

**Gap (nice-to-have)** — add at least one fixture+test per unexercised method, or accept the implementation-only coverage and defer formatting/arithmetic adapter tests to a follow-up issue.

### Phase 8 — Docs (✅ passes)

- `docs/FEATURES.md` gets a new `### jiff` subsection with timezone policy, type mapping, and an example.
- `docs/CONVERSION_MATRIX.md` gets a new Date/Time section covering both `time` and `jiff`.
- `site/content/manual/` regenerated (per CLAUDE.md `docs-to-site.sh` convention).

No issues.

### Phase 9 — rpkg fixtures + testthat (see gaps flagged in 3/5/6/7 above)

27 testthat tests pass. 30 `#[miniextendr]` fixtures. Covers the common path well but has coverage holes on:

- unknown-IANA error path (phase 3)
- heterogeneous-tz Vec<Zoned> warning (phase 3)
- Span ExternalPtr + RSpan trait (phase 5)
- DateTime / Time adapter traits (phase 6)
- RDate / RZoned / RTimestamp formatting + arithmetic methods (phase 7)
- **Leap-day Date round-trip** (plan phase 9.5 required `as.Date("2024-02-29")` specifically; agent tested `1970-01-01` and `1900-03-01`)

### Phase 10 — vctrs rcrd constructors (🚨 code present, zero R-side coverage)

Code is solid (`jiff_impl.rs:1516–1684`):

- Private `mod vctrs_support` with `alloc_int_col` helper and four constructors (`span_vec_to_rcrd`, `zoned_vec_to_rcrd`, `datetime_vec_to_rcrd`, `time_vec_to_rcrd`).
- Top-level re-exports at lines 1661–1684 as the plan specified.
- Uses `crate::vctrs::new_rcrd` which auto-appends `"vctrs_rcrd"` + `"vctrs_vctr"` (confirmed by reading `vctrs.rs:401–404`) — so user only needs to pass `&["jiff_span"]` etc. **Correct.**

**Gap (must-fix) — zero tests, zero fixtures.** No `#[miniextendr]` function exposes any of the `*_vec_to_rcrd` helpers to R. No testthat block references `vctrs::field` or `vctrs::vec_size` on a jiff rcrd. The plan's phase 10 explicitly required both fixtures and testthat coverage. Without an exercise the code could compile but GC-corrupt at runtime — see GC concern below.

**Gap (must-verify) — GC protection in the rcrd builders:**

1. `alloc_int_col` creates `_guard = OwnedProtect::new(col)` which **drops when the function returns**, unprotecting `col` before it is consumed by `List::from_raw_values`. Subsequent allocations could GC-collect the earlier columns. Example at `jiff_impl.rs:1541–1552` — 10 sequential `alloc_int_col` calls, each unprotecting before the next allocates.

2. `zoned_vec_to_rcrd` calls `Rf_unprotect(1)` on `tz_col` **before** invoking `List::from_raw_values` at `jiff_impl.rs:1604`. If `from_raw_values` allocates a VECSXP, `tz_col` is unprotected during that allocation.

The fact that clippy is clean + 4619 tests pass doesn't prove absence of GC bugs here because **no test ever exercises these code paths yet**. The fix agent must either:

(a) verify that `List::from_raw_values` or `alloc_r_vector` pre-protects — making the flagged code safe — and add a comment explaining why, OR
(b) fix the protection discipline (e.g. use `OwnedProtect` that outlives `List::from_raw_values`, or protect all columns through the list-build).

### Phase 11 — ALTREP `JiffTimestampVec` (✅ code, ⚠️ test depth)

Implementation at `jiff_impl.rs:1475–1514`:

```rust
#[derive(miniextendr_macros::AltrepReal)]
#[altrep(class = "JiffTimestampVec", manual)]
pub struct JiffTimestampVec { pub data: Arc<Vec<Timestamp>> }

impl AltrepLen   for JiffTimestampVec { fn len(&self) -> usize { self.data.len() } }
impl AltRealData for JiffTimestampVec { fn elt(&self, i: usize) -> f64 { ... } }
```

`manual` mode is the right call (compute-on-access). Fixture `jiff_altrep_timestamps(n: i32) -> SEXP` builds the vector and applies POSIXct class. Tests cover length, element access, class. 

**Gap (nice-to-have) — no test asserts lazy materialization.** The plan's phase 11.4 asked for a fixture that "records the call count" to prove materialization is deferred. Current tests only assert correctness, not laziness. Acceptable for v1 but worth a follow-up issue.

**Follow-up issue #304 filed** for Zoned ALTREP (Vec<Zoned> as ALTREP). Confirmed via `gh issue view 304`.

### Phase 12 — Verification (✅ passes)

All agent claims reproduced:

- `cargo clippy -p miniextendr-api --features jiff -- -D warnings` → clean
- Full `clippy_all` with `,jiff` appended → clean
- `just devtools-test` → 0 failures, 4619 passes, 15 unrelated skips

---

## Cross-cutting issues

### CC-1 — Plan deviation: phases 1–7 + 10 + 11 collapsed into one mega-commit

Plan specified per-phase commits prefixed `[phase N]`. Agent produced `[phase 1-7]` which actually includes phases 1–7 **and** 10 **and** 11 (vctrs + ALTREP impls landed in the same commit despite the plan placing them in dedicated phases). Commit title says "1-7 … ALTREP, vctrs" without naming 10/11.

**Impact:** the "phase-by-phase review" workflow becomes harder — I had to reconstruct phase boundaries by reading the file structure rather than diffing one commit per phase.

**Address:** flag only. Not worth a rewrite of history for this PR. Note for future plans: be explicit that `[phase N]` is a per-phase commit boundary, not a free-form label.

### CC-2 — Agent claim inaccuracy: pre-commit hook

Agent's end-of-run summary said "Also fixed pre-commit hook: removed stale git add rpkg/inst/vendor.tar.xz". **No `.githooks/` changes exist on the branch** (confirmed `git log origin/main..HEAD -- .githooks/` is empty). Either the claim was spurious or the change was discarded before commit.

**Address:** non-issue for this PR; flag only.

### CC-3 — Scope expansion: `pub(crate) set_posixct_utc` → `pub set_posixct_utc`

Agent promoted `set_posixct_utc` from `pub(crate)` to `pub` in `cached_class.rs:200` so rpkg fixtures could call it. Unprompted public-API expansion, but defensible because:

- The `time` feature already has parallel visibility expectations downstream.
- rpkg fixtures for `jiff_altrep_timestamps` genuinely need to call it to apply POSIXct class to an ALTREP SEXP.
- Alternative (have the derive wrap `set_posixct_utc` in its `into_sexp`) would force a class commitment on the derive user.

**Address:** accept. Mention in the PR body as an intentional surface expansion.

### CC-4 — Two Cargo.lock modifications

`f0d473d5` adds 32 lines to `rpkg/src/rust/Cargo.lock`. `371675de` adds 12 more. Cosmetic; the second commit is a vendor-freeze cleanup after the initial build.

**Address:** accept. Optionally squash into a single "chore: Cargo.lock" companion commit, but not required.

---

## Must-fix list (fix agent's punch list)

In execution order:

1. **Add `Span` rpkg fixture + testthat tests** — phase 5 coverage gap. Plan phase 9.5 showed the exact pattern.
2. **Add vctrs rcrd fixtures + testthat tests** (`span_vec_to_rcrd` / `zoned_vec_to_rcrd` / `datetime_vec_to_rcrd` / `time_vec_to_rcrd`) — phase 10 coverage gap. Use `vctrs::vec_size` + `vctrs::field` assertions per plan.
3. **Verify (and fix if needed) GC protection in the vctrs builders** — `alloc_int_col` guard dropping on return; `Rf_unprotect(1)` before `List::from_raw_values` in `zoned_vec_to_rcrd`. Must be green after the fixtures from item 2 exercise these paths.
4. **Add testthat for unknown-IANA tz error path** — plan phase 3/9 required `expect_error(..., "Mars/Olympus")`. One-line test.
5. **Add testthat for leap-day Date round-trip** (`as.Date("2024-02-29")`) — plan phase 9 requirement.

## Nice-to-have list (fix agent discretion; skip → file issue)

6. Add `DateTime` / `Time` fixtures + ≥1 test each (phase 6 coverage).
7. Add a heterogeneous-tz `Vec<Zoned>::into_sexp` test that asserts the first-tz policy.
8. Add fixtures for the unexercised `RDate` / `RZoned` / `RTimestamp` methods (strftime, start_of_day, weekday, first_of_month, tomorrow/yesterday) — minimum one per method.
9. Add a laziness-proof ALTREP test (deferred materialization counter).

## Follow-up issues (file if not addressed in-PR)

- If items 6–9 are not addressed in-PR, bundle them into one follow-up issue: "jiff integration: expand adapter-trait coverage".
- Zoned ALTREP already filed as #304 — confirmed, link in PR body.

---

## Address plan (what the fix agent gets)

The fix agent receives this review file and:

1. Reads the must-fix list (items 1–5) and executes them sequentially on `feat/jiff-integration`.
2. For item 3 (GC protection), first verifies via `List::from_raw_values` source reading whether the flagged code is actually safe — if safe, adds a clarifying comment; if unsafe, fixes the protection discipline.
3. For nice-to-have items 6–9: fix agent may address them if the work is small (each ~5–15 min). Otherwise files a single follow-up issue covering the batch.
4. Commits incrementally with `[review-fix]` prefix.
5. Runs `cargo clippy -- -D warnings` (both modes) + `just devtools-test` after each commit.
6. Reports completion; I push the branch and open the PR.
