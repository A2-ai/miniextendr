# Plan: audit 2026-07-12 P2 — make the standard green path warning-clean

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `fix/audit-standard-path-warnings`.

Covers 2026-07-12 audit worklist item 14. Verified DISTINCT from existing
issues: #1292 tracks two *clippy* warnings in rpkg
(`serde_json_adapter_tests.rs:101` etc. — cross-reference, do not fold);
#1261 (R CMD check WARNINGs) was closed by `d10462ce`. No open issue
covers the two findings below.

## Verified defects (on 656e5cdd)

**A. Two impl-level roxygen-tag deprecation warnings on every
`just check` / `just clippy` / `just rcmdinstall`.**
- Emission sites: `miniextendr-macros/src/roxygen.rs:719` and `:896`
  ("miniextendr: @{} on impl block `{}` has no effect — move it to the
  method. Tag: {}"). The nudge was deliberately activated by #1296
  (`e504646c`, 2026-07-11) — and rpkg's own fixture still trips it.
- Tripping site: `rpkg/src/rust/impl_dots_tests.rs:52-53` — impl-level
  `/// @param seed` + `/// @param ...` on the doc block above
  `#[miniextendr(s3)] impl ImplDotsS3` (:55-56). The methods already carry
  their own `@param` tags (:57-58, :72-73), so the impl-level pair is
  redundant, not load-bearing.
- The warning path has dedicated UI coverage
  (`miniextendr-macros/tests/ui/impl_method_tag_deprecated_deny.stderr`),
  so the rpkg fixture is NOT needed as warning-coverage.
- `rpkg/tests/testthat/test-impl-dots.R:8-10` pins only `formals(...)`
  shapes — unaffected by moving doc tags.
- Note the struct-level tags at `impl_dots_tests.rs:7-8` (on the
  `ImplDotsR6` struct) feed constructor docs legitimately — leave them;
  only the *impl-block* tags at :52-53 warn.

**B. `proc-macro-error2 v2.0.1` future-incompatibility notice** on release
installs (also visible in webR CI build logs). Dependency chain, verified
in both lockfiles: `miniextendr-api` optional feature `tabled`
(`miniextendr-api/Cargo.toml:97-98,230` — `tabled = "0.20.0"`) →
`tabled_derive 0.11.0` → `proc-macro-error2 2.0.1`
(root `Cargo.lock:2663,3315`; `rpkg/src/rust/Cargo.lock:2463,3040`;
rpkg re-exports the feature at `rpkg/src/rust/Cargo.toml:53`).

## Work items (flat order)

1. **Fix A**: delete the two impl-level `@param` lines
   (`impl_dots_tests.rs:52-53`), keeping the class description sentence.
   Then the doc-regeneration loop:
   `just configure && just rcmdinstall && just force-document` — expect
   little/no `man/*.Rd` churn (params were "no effect") and NO `NAMESPACE`
   change; if force-document flips `S3method`↔`export` lines, apply the
   known revert-NAMESPACE lesson and keep only intended diffs. Commit any
   legitimate man/ diffs with the Rust change.
2. **Fix B — bump the chain**: check crates.io for a `tabled` release
   whose derive no longer depends on `proc-macro-error2` (check
   `tabled_derive` changelogs; the future-incompat is in the ecosystem's
   sights). If one exists: bump `miniextendr-api/Cargo.toml:230`, refresh
   the root `Cargo.lock` AND `rpkg/src/rust/Cargo.lock`, run
   `just vendor-sync-check`, and confirm the notice is gone from a release
   install log. Spot-check the `tabled` API surface we use
   (`grep -rn "tabled" miniextendr-api/src | head`) against the new
   version's breaking changes.
3. **Fix B — if no upstream fix exists**: run
   `cargo report future-incompatibilities --id <id>` for the exact
   offense, file a tracked repo issue (implementer files it — repo
   policy: no untracked known-issues) with the upstream link, and state in
   the PR body that the notice is upstream-blocked. Do NOT silence it via
   `[future-incompat-report]` config — visibility is the point.
4. **Keep it clean — cheap regression guard**: add a grep gate to an
   existing CI step that already captures an rpkg build log (candidates:
   the wrappers-sync-check or install step): fail if the log contains
   `miniextendr: @` warning lines. Scope it narrowly (our own macro's
   nudge prefix) so upstream noise can't flake it. If no existing step's
   log is convenient, note the gap in the PR body instead of adding a new
   build job — the audit's broader "warning-free default gate for every
   manifest" is a maintainer-scoped follow-up.

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just check 2>&1 > /tmp/audit-warn-check.log      # Read it — zero miniextendr warnings
just clippy 2>&1 > /tmp/audit-warn-clippy.log    # Read it — zero warnings
just configure && just rcmdinstall 2>&1 > /tmp/audit-warn-install.log && just force-document
grep -n "miniextendr: @\|future" /tmp/audit-warn-install.log   # expect empty (or upstream-blocked note)
just devtools-test 2>&1 > /tmp/audit-warn-devtools.log
grep -E '\[ FAIL [0-9]+' /tmp/audit-warn-devtools.log
cargo clippy --workspace --all-targets --locked -- -D warnings   # + all/all_s7 legs per ci.yml
cargo fmt --all
```

## Must NOT touch

- The nudge itself (`roxygen.rs:719,:896`) and its UI test — the warning
  is correct; the fixture is what's wrong.
- #1292's two clippy warnings (separate issue, separate fix — cross-ref
  only).
- `impl_dots_tests.rs` behavior/fixtures beyond the two doc lines (the
  dots formals are pinned by test-impl-dots.R).
- `rpkg/R/miniextendr-wrappers.R` / `wasm_registry.rs` (generated,
  gitignored).

## Done criteria

- `just check`, `just clippy`, `just rcmdinstall` produce zero
  miniextendr-emitted warnings on a clean tree.
- `proc-macro-error2` notice gone (bump) or tracked in a referenced issue
  (upstream-blocked), stated in the PR body.
- Suites + three clippy legs green; NAMESPACE unchanged.

## Escalation rule

If reality diverges from this plan — removing the impl-level tags changes
rendered Rd in a way tests pin, the tabled bump breaks our formatting
API, or the warnings turn out to come from a second site — **stop, commit
nothing further, and report back. Do not improvise.**
