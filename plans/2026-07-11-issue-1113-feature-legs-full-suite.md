# Plan: #1113 — default-agnostic fixtures + full-suite r6/s7 feature legs

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `ci/1113-feature-legs-full-suite`.

Decision baked in (the issue's own "cleaner path" + one matrix restructure):

- Make every bare `#[miniextendr]` **impl block** in `rpkg/src/rust/` explicit
  (`#[miniextendr(env)]` — env IS the compiled default, so this is
  behavior-preserving), EXCEPT the deliberate class-system probe.
- Split the strict/coerce riders onto their own filtered rows so the class
  legs can run the full suite: the blocker for full-suite was never only the
  class flip — `strict-default`/`coerce-default` change conversion semantics
  suite-wide and CANNOT go full-suite without a second sweep that is out of
  scope here.

Sequencing note: `ci/1244-fast-default-leg`
(`plans/2026-07-11-issue-1244-fast-default-ci-leg.md`) edits the same matrix
block — whichever lands second rebases (trivial adjacent-row conflict).

## Work items (flat order)

1. **Sweep** (own commit, mechanical): inventory bare impl-block attributes:
   `rg -U -n '#\[miniextendr\]\s*\nimpl ' rpkg/src/rust/` (expect ≈52 per the
   issue; the count validates the pattern — if you find <40 or >70, stop and
   report). For each hit, rewrite `#[miniextendr]` → `#[miniextendr(env)]`,
   EXCEPT the class-system probe region in
   `rpkg/src/rust/feature_default_fixtures.rs:69-94` (its bareness is the
   probe — leave it, add a comment `// deliberately bare: flips under
   r6-default/s7-default (#1113)` if not already stated). Bare `#[miniextendr]`
   on FNS is untouched (class flips don't affect functions).
   Behavior-preservation proof: full regen loop, then `git status` — NAMESPACE
   and `man/*.Rd` must show ZERO diff. If any diff appears, stop and report.
2. **Matrix restructure** in `.github/workflows/ci.yml` feature-legs
   (`:1188-1207`): replace the two bundled rows with four:
   ```yaml
   - leg: r6-default
     features: r6-default
     filter: ""
   - leg: s7-default
     features: s7-default
     filter: ""
   - leg: strict-default
     features: strict-default
     filter: feature-defaults
   - leg: coerce-default
     features: coerce-default
     filter: feature-defaults
   ```
   Keep the existing comment about strict/coerce needing separate rows
   (`:1198-1201`), rewritten to explain the new split (class legs full-suite
   per #1113; strict/coerce stay fixture-filtered because they flip
   conversion semantics suite-wide). Update the job header comment
   (`:1173-1178`) which currently documents the filtered r6/s7 behavior.
3. **Stress-file exclusion**: the job currently leaves
   `MINIEXTENDR_SKIP_STRESS` unset, so full-suite legs run the gctorture
   files (slow interpreter-heavy tests whose coverage lives in the sharded
   `r-stress-tests` job + nightly). Add `MINIEXTENDR_SKIP_STRESS: "1"` to the
   feature-legs job `env:` block (`:1209-1212`) with a one-line comment. This
   also applies to the existing extras/worker rows — deliberate; note it in
   the PR body.
4. **Full-suite proof for the class legs** (local, one leg is enough —
   r6-default; s7 is symmetric and CI-verified weekly):
   ```bash
   export CARGO_FEATURES="$(Rscript rpkg/tools/detect-features.R 2>/dev/null | tail -1),r6-default"
   ```
   — NO: compose it exactly the way the CI step does (`:1248-1256` — read
   the "Compose CARGO_FEATURES" step and replicate its base-detection
   command verbatim; do not invent a different composition). Then in the SAME
   shell: `just configure && just rcmdinstall`, then
   `MINIEXTENDR_SKIP_STRESS=1 Rscript -e 'testthat::test_local()'` from
   `rpkg/` — must be FAIL 0. Triage any failure: it is either a fixture the
   sweep missed (fix: make it explicit) or a genuinely flip-sensitive test
   (fix: branch it on `miniextendr_has_feature()` like
   `test-feature-defaults.R` does) — those two remedies are in scope; any
   third kind of failure → escalate. Restore the default build afterwards
   (`unset CARGO_FEATURES && just configure && just rcmdinstall`).

## Exact commands (worktree)

```bash
rig default 4.6 && R --version | head -1        # must print 4.6.x
just worktree-sync                               # FIRST
just configure && just rcmdinstall && just force-document
# sweep is attr-only, no new exports — single install; expect zero NAMESPACE/man diff
just devtools-test 2>&1 > /tmp/1113-devtools.log      # default build stays green
grep -E '\[ FAIL [0-9]+' /tmp/1113-devtools.log       # devtools::test always exits 0
# r6-default full-suite proof per item 4 (redirect to /tmp/1113-r6leg.log, Read it)
just test 2>&1 > /tmp/1113-rust.log
cargo clippy --workspace --all-targets --locked -- -D warnings  # + all/all_s7 legs per ci.yml
cargo fmt --all
python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'
```

## Must NOT touch

- `test-feature-defaults.R`'s divergence probes (they remain the deliberate
  flip-sensitive coverage).
- Bare `#[miniextendr]` on functions; conversion fixtures' strict/coerce
  knobs (out of scope — strict/coerce rows stay filtered).
- The extras/worker rows beyond the shared env addition.
- cross-package crates.

## Done criteria

- r6-default and s7-default rows run `testthat::test_local()` unfiltered;
  local r6 full-suite run FAIL 0; sweep commit shows zero tracked-artifact
  diff; strict/coerce rows still cover their fixtures; three clippy legs +
  default suite green; `Fixes #1113`.

## Escalation rule

If reality diverges from this plan — the sweep count is far off, explicit
`env` changes generated output, a full-suite failure fits neither remedy in
item 4 — **stop, commit nothing further, and report back. Do not improvise.**
