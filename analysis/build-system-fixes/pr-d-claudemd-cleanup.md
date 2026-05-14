# PR-D: CLAUDE.md cleanup — stale --freeze docs + stale force_load description

**Context**: `analysis/build-system-investigation-2026-05-11.md` §5.4, §7.3, §12.6, §12.7.

## Problems

### D1. CLAUDE.md describes `-Wl,--whole-archive` / `-force_load`, but Makevars doesn't use them

The root `CLAUDE.md` "How the build works" section says:

> `cargo rustc --crate-type cdylib` (links via `-Wl,-force_load` on macOS / `--whole-archive` on Linux)

The current `Makevars.in` does **not** use either flag. The design
uses `stub.c` as a force-link anchor:
- `stub.c` declares `extern const char miniextendr_force_link`.
- `miniextendr_init!()` macro emits `pub static miniextendr_force_link: c_char = 0` in the Rust crate.
- Linker sees `stub.o` referencing the symbol → must extract its archive
  member → with `codegen-units = 1` that pulls in the entire user crate →
  all `#[distributed_slice]` entries follow.

This is documented in investigation §5.4.

### D2. CLAUDE.md describes "Stale frozen-vendor recovery" but `--freeze` is never invoked

`docs/CRAN_COMPATIBILITY.md:221-228` explicitly lists `cargo revendor --freeze
invocations from just vendor` under "Removed entirely from this codebase."
Neither `just vendor` nor `rpkg/bootstrap.R` passes `--freeze`. The
"frozen-vendor recovery" section in CLAUDE.md (under "The install-mode latch")
describes a recovery scenario that doesn't arise in current code.

## Files to change

### D1

- `CLAUDE.md` — root file, section "How the build works":
  - Remove the `(links via -Wl,-force_load on macOS / --whole-archive on Linux)` parenthetical.
  - Add a one-line note explaining the `stub.c` anchor + `codegen-units = 1`
    mechanism. Two sentences max.
- `AGENTS.md` — per investigation, this is the parallel for non-Claude
  agents (codex/cursor) and "shares most content" with `CLAUDE.md`.
  Mirror the change.

### D2

- `CLAUDE.md` — remove the entire "Stale frozen-vendor recovery" subsection
  under "The install-mode latch (`inst/vendor.tar.xz`)". (Per investigation
  §7.3, this is historical.)
- `AGENTS.md` — mirror.

## Optional polish (low priority, include if straightforward)

- If `docs/CRAN_COMPATIBILITY.md` or other docs reference the `--freeze`
  flow as current, scrub those too. Only do this if the references are
  clearly miscategorized as current; do not touch historical/archived sections.

## Tests / verification

- Read the updated `CLAUDE.md` end-to-end. Confirm no other section
  references `-Wl,--whole-archive`, `-force_load`, or `--freeze` as
  current behavior.
- Diff `CLAUDE.md` and `AGENTS.md` after the change to confirm the
  mirrored content. The two should differ only in their respective
  meta-headers (per investigation §Orientation).
- Run `markdownlint` or equivalent if your repo has one; otherwise
  visually confirm no broken section refs.
- Smoke: `grep -n 'whole-archive\|force_load\|--freeze' CLAUDE.md AGENTS.md`
  should return nothing referring to current behavior.

## Not in scope

- Updating per-subtree CLAUDE.md files (e.g., `miniextendr-api/CLAUDE.md`)
  unless they contain the same stale claims — quick grep, then decide.
- Cleaning up references in /Users/elea/.../user-private files. Out of repo.

## PR title

`docs(claude,agents): drop stale --whole-archive / --freeze descriptions`

## PR body

Reference investigation §5.4 and §7.3. Note that current design uses
`stub.c` anchor + `codegen-units = 1`, and `--freeze` was removed from
the codebase (per `docs/CRAN_COMPATIBILITY.md`).

## Branch

`docs/claude-md-cleanup`
