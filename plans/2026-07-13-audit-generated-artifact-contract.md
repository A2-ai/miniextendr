# Plan: audit 2026-07-12 P2 — one generated-artifact contract (wrappers.R commit-status drift)

Date: 2026-07-13. Anchors verified against main @ 656e5cdd.
Branch: `docs/audit-generated-artifact-contract`.

Covers 2026-07-12 audit worklist item 15 and the "Generated-artifact
documentation has incompatible contracts" P2 finding. Distinct from the
older on-disk `plans/2026-07-01-docs-drift-sweep.md` (different files).

## Verified contradictions (all on 656e5cdd)

The authoritative truth (per `rpkg/.gitignore:8-18` — comment block +
`R/*-wrappers.R` ignore at :16 — and CLAUDE.md:112-118 "Generated
artifacts"): `rpkg/R/miniextendr-wrappers.R` and
`rpkg/src/rust/wasm_registry.rs` are **gitignored, regenerated**, ship in
the tarball from disk; `NAMESPACE` + `man/*.Rd` stay **tracked**.

Documents contradicting it:

1. `README.md:80-84` — "Generated artifacts that must stay committed:"
   lists `rpkg/R/miniextendr-wrappers.R` (plus `rpkg/configure`,
   `config.guess`, `config.sub`, which ARE tracked — only the wrappers
   entry is wrong).
2. `rpkg/README.md:39-50` — "Files that must be committed" lists
   `R/miniextendr-wrappers.R`, with the stale rationale "`R CMD build`
   expects the wrapper file to exist" (superseded by the `just
   r-cmd-build` regeneration + Makevars #1022 fallback). Also sweep
   `rpkg/README.md:83,106` ("rewrites `R/miniextendr-wrappers.R`",
   "regenerate ... before release") for consistency with the final text.
3. `AGENTS.md:130` — "Generated files (`rpkg/R/miniextendr-wrappers.R`,
   `NAMESPACE`, `man/*.Rd`) must be committed in sync…" vs
   `AGENTS.md:171-174` — wrappers.R "**Gitignored**". Internally
   inconsistent in one file.
4. `CLAUDE.md:269-270` — the same stale sentence as AGENTS.md:130 (the
   root AGENTS.md is a hand-kept mirror of CLAUDE.md — fix both, prose
   parity is hand-maintained per CLAUDE.md's own AGENTS.md section).
5. Audit also reports adjacent drift "around whether configure rewrites
   source files and whether source-mode development vendors crates" —
   e.g. `README.md:74-76` claims configure generates `src/rust/Cargo.toml`
   (verify against configure.ac's actual outputs: Makevars +
   `.cargo/config.toml` per CLAUDE.md's ".in templates" list). Sweep the
   flow bullets in both READMEs against configure.ac while in there;
   fix only what is verifiably wrong.

## Work items (flat order)

1. **Fix the four verified contradictions** (items 1-4 above): wrappers.R
   moves from every "must be committed" list to the
   generated/gitignored/ships-in-tarball description; the correct commit
   set is `NAMESPACE` + `man/*.Rd` (+ `configure`, `config.guess`,
   `config.sub`). Reuse the precise wording already in CLAUDE.md:112-118
   rather than inventing a third phrasing.
2. **Sweep the configure-behavior claims** (item 5): check each flow
   bullet in `README.md:70-78` and `rpkg/README.md:20-33` against
   `rpkg/configure.ac` outputs; correct only demonstrable mismatches, cite
   the configure.ac line in the commit message.
3. **Single source, machine-checked** (the audit's core recommendation).
   Minimal viable version — do not overbuild: a small table (artifact path
   → tracked | gitignored | generated-ships-in-tarball) in ONE place
   (suggest a short `docs/ARTIFACT_CONTRACT.md` or a section of
   `rpkg/README.md`), plus a check script wired into the existing Sync
   Checks CI job (`just artifact-contract-check`, sibling of
   `just agents-md-check`): for each row, assert `git check-ignore -q`
   /`git ls-files --error-unmatch` agrees with the declared status. That
   catches the *factual* half mechanically. Prose in the two READMEs then
   links to the table instead of restating the list. (Full
   generate-into-READMEs tooling is over-engineering for 6 rows — say so
   in the PR body if the maintainer asks.)
4. **AGENTS.md parity**: apply the CLAUDE.md fix to the root AGENTS.md
   mirror by hand; `just agents-md-check` verifies the structural
   invariant (it does not check prose — read the diff yourself).

## Exact commands (worktree)

```bash
just agents-md-check                       # structural invariant still green
just artifact-contract-check               # new check, green
git check-ignore rpkg/R/miniextendr-wrappers.R && echo ignored-ok
git ls-files --error-unmatch rpkg/NAMESPACE && echo tracked-ok
```

No R, no cargo (docs + one small script + justfile recipe + CI step).

## Must NOT touch

- `.gitignore` files themselves — the ignore status is the ground truth
  this plan aligns the prose TO; changing trackedness is out of scope.
- `rpkg/R/miniextendr-wrappers.R` / `wasm_registry.rs` / `NAMESPACE` /
  `man/` (no regeneration needed — pure prose).
- The subdirectory `AGENTS.md` files (they are `@CLAUDE.md` imports; only
  the root mirror is hand-kept).

## Done criteria

- No document in the repo claims `R/miniextendr-wrappers.R` must be
  committed; grep proof in the PR body
  (`grep -rn "wrappers.R" README.md rpkg/README.md AGENTS.md CLAUDE.md`).
- AGENTS.md is internally consistent; CLAUDE.md matches.
- `just artifact-contract-check` exists, runs in the Sync Checks job, and
  fails if a listed artifact's git status diverges from the table.

## Escalation rule

If reality diverges from this plan — a configure-behavior claim turns out
to be *correct* in a way that contradicts CLAUDE.md, or the check script
needs to special-case more than the listed artifacts — **stop, commit
nothing further, and report back. Do not improvise.**
