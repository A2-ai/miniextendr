# Plan: #1253 — CI check for the CLAUDE.md ↔ AGENTS.md sibling invariant

Date: 2026-07-11. Anchors verified against main @ 17f634d8.
Branch: `ci/1253-agents-md-check`.

## Invariant (from PR #1236, restated in root CLAUDE.md)

1. Every tracked `CLAUDE.md` has a sibling `AGENTS.md` in the same directory.
2. Every tracked `AGENTS.md` has a sibling `CLAUDE.md` (no orphans).
3. Every **subdirectory** `AGENTS.md` contains the `@CLAUDE.md` import line
   (a line that is exactly `@CLAUDE.md`); the ROOT `AGENTS.md` is exempt
   (hand-kept mirror).

Current state (verified): 13 perfectly-paired directories; every
subdirectory AGENTS.md is a one-comment-line + `@CLAUDE.md` file (see
`miniextendr-api/AGENTS.md` for the canonical shape).

## Work items

1. New `scripts/agents-md-check.sh` (bash, `set -euo pipefail`; mirror the
   header style of `scripts/skill-freshness-audit.sh` — purpose comment,
   exit-nonzero-on-violation contract). Logic over `git ls-files` ONLY (never
   the working tree — untracked/ignored files like `background/` must not
   trip it):
   ```bash
   claude=$(git ls-files '*CLAUDE.md' | grep -E '(^|/)CLAUDE\.md$' | sed 's|CLAUDE\.md$||' | sort)
   agents=$(git ls-files '*AGENTS.md' | grep -E '(^|/)AGENTS\.md$' | sed 's|AGENTS\.md$||' | sort)
   comm -23 <(echo "$claude") <(echo "$agents")   # dirs missing AGENTS.md → violation
   comm -13 <(echo "$claude") <(echo "$agents")   # orphan AGENTS.md → violation
   ```
   (The `grep -E '(^|/)CLAUDE\.md$'` guard keeps a hypothetical
   `FOO-CLAUDE.md` from matching.) For rule 3: for every tracked AGENTS.md
   except the root one, require `grep -qx '@CLAUDE.md'`; violations named
   individually. Print each violation with a one-line fix hint (the root
   CLAUDE.md "Orientation" section documents the convention) and exit 1 if
   any rule fired; exit 0 silently otherwise (CI-friendly).
2. `justfile`: add an `agents-md-check` recipe (`bash scripts/agents-md-check.sh`)
   near `templates-check` (`justfile:1097`). Follow the file's existing
   single-line recipe style; mind the recipe-line-isolation gotcha (each line
   is its own `bash -c` unless `[script("bash")]` — a single command line
   needs nothing special).
3. `.github/workflows/ci.yml`, Sync Checks job (`:195-265`): add a step
   `- name: AGENTS.md sibling check` / `run: just agents-md-check`
   immediately after the `just templates-check` step (`:265`). No new deps —
   pure git+coreutils.
4. Self-test the three failure modes locally (create violation, run script,
   assert exit 1, revert — do NOT commit the violations):
   (a) delete a sibling: `git rm --cached miniextendr-bench/AGENTS.md` form —
   use a scratch `git stash`-able edit, or simpler: run the script in a
   throwaway clone under `/tmp`; (b) add an orphan `AGENTS.md`;
   (c) replace a subdir AGENTS.md body with prose lacking `@CLAUDE.md`.
   Record the three observed failure outputs in the PR body.
5. Root `CLAUDE.md` + root `AGENTS.md` (hand-kept mirror — project rule: a
   project-wide rule changed in one must be mirrored in the other): add one
   sentence to the existing AGENTS.md convention paragraph noting CI enforces
   the invariant via `just agents-md-check`.

## Exact commands (worktree)

```bash
just worktree-sync            # FIRST (not strictly needed — no R/Rust builds — but keep the invariant)
bash scripts/agents-md-check.sh; echo "exit=$?"     # expect exit=0 on clean tree
just agents-md-check                                 # recipe wiring works
python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'
shellcheck scripts/agents-md-check.sh || true        # advisory; fix what it flags if installed
```

No Rust/R builds, no regen loop, no snapshots.

## Must NOT touch

- Any existing CLAUDE.md/AGENTS.md content beyond item 5's one sentence
  (both root files, mirrored).
- `minirextendr/inst/templates/**` (scaffolded-package files are out of
  scope; none are tracked CLAUDE.md today — the git-ls-files basis keeps it
  that way by construction).
- Other Sync Checks steps.

## Done criteria

- Clean tree passes; each of the three violation classes turns the script
  red with a named path (evidence in PR body); CI Sync Checks runs the new
  step; `Fixes #1253`.

## Escalation rule

If reality diverges from this plan — the tracked-file inventory contains a
legitimate exception the rules can't express, the Sync Checks job structure
changed — **stop, commit nothing further, and report back. Do not improvise.**
