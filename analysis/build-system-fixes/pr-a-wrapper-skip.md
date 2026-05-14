# PR-A: Skip wrapper-gen for tarball install

**Context**: `analysis/build-system-investigation-2026-05-11.md` §6.4, §12.1.

## Problem

`R CMD INSTALL <tarball>` currently does the full wrapper-gen pass even though
`R/<pkg>-wrappers.R` is already committed and shipped in the tarball. The
write itself is already a no-op (write-if-changed in `registry.rs:918-921`),
but we still build the cdylib (10–30 s) and spin up R to call `dyn.load`.
Pure waste.

## Why it is safe

1. The committed `R/<pkg>-wrappers.R` is enforced in sync with the Rust crate
   by the pre-commit hook (`.githooks/pre-commit`).
2. `R CMD build` seals both into the same tarball.
3. Therefore wrappers in the tarball always match the bundled Rust by
   construction — the cdylib regeneration would produce byte-identical
   bytes.

## Detection

`IS_TARBALL_INSTALL` is substituted into `Makevars` at configure time
(`configure.ac:102-119`). Already visible to every make rule.

## Files to change

- `rpkg/src/Makevars.in` (the master Makevars template)
- `minirextendr/inst/templates/rpkg/Makevars.in` (standalone-template mirror)
- `minirextendr/inst/templates/monorepo/rpkg/Makevars.in` (monorepo-template mirror)

The three are byte-identical per investigation §2.3 — must remain in sync.

## Change

In each `Makevars.in`, replace the current `$(WRAPPERS_R)` rule with a three-way
`ifeq` (wasm / tarball / dev). Sketch:

```make
ifeq ($(IS_WASM_INSTALL),true)
$(WRAPPERS_R):
	@echo "wasm32 install: skipping wrapper-gen ..."
	@touch $(WRAPPERS_R)
else ifeq ($(IS_TARBALL_INSTALL),true)
$(WRAPPERS_R):
	@echo "tarball install: using pre-shipped R/$(PACKAGE_NAME)-wrappers.R"
	@if [ ! -f '$(WRAPPERS_R)' ]; then \
	  echo "ERROR: tarball is missing pre-generated $(WRAPPERS_R)" >&2; \
	  exit 1; \
	fi
	@touch $(WRAPPERS_R)
else
$(WRAPPERS_R): $(CARGO_CDYLIB)
	@... existing dev-mode recipe unchanged ...
endif
```

The existence guard is important: catches the (rare) case of a maintainer
manually deleting `R/<pkg>-wrappers.R` from a built tarball — fail loudly
instead of silently shipping a broken install.

When `IS_TARBALL_INSTALL=true`, `$(CARGO_CDYLIB)` is never targeted →
cdylib build is fully skipped → no wasted cargo invocation.

## Optional escape hatch (recommended)

Add `MINIEXTENDR_FORCE_WRAPPER_GEN` env var: if set, the tarball branch
falls through to the dev recipe. Useful when debugging wrapper drift in a
tarball install. Implementation: wrap the tarball branch in another
`ifndef MINIEXTENDR_FORCE_WRAPPER_GEN`. Five lines.

## Templates sync

After editing `rpkg/src/Makevars.in`, run `just templates-approve` to lock
the delta into `patches/templates.patch`. Verify the templates copies
end up with the same change.

## Tests / verification

1. Build a tarball: `just r-cmd-build`. Confirm tarball contains
   `R/<pkg>-wrappers.R` (it always does — sanity check).
2. Install the tarball in a temp R library:
   `R_LIBS=/tmp/test-lib R CMD INSTALL <tarball>`. Capture output.
3. Assert install log contains `"tarball install: using pre-shipped"`.
4. Assert install log does **not** contain `Compiling miniextendr` lines
   for the cdylib (cargo's cdylib build line — search install output).
5. Install in dev mode: `just rcmdinstall`. Assert log does NOT contain
   the "tarball install" message; assert it DOES contain the original
   "Generating R wrappers" message.
6. Negative path: build tarball, manually strip `R/<pkg>-wrappers.R`
   from it, re-install. Assert install fails with the
   "ERROR: tarball is missing" message.
7. wasm path: ensure the wasm branch still works (no regression). Visual
   inspection is enough — wasm build isn't easy to fully exercise in PR.

## Not in scope for this PR

- Test plan §T6 cells beyond the smoke checks above. Full harness is
  larger and tracked separately.
- Timing benchmarks (would be nice but not load-bearing).

## PR title

`build(rpkg,minirextendr): skip wrapper-gen cdylib for tarball install`

## PR body

Reference §6.4 of `analysis/build-system-investigation-2026-05-11.md`.
Brief explanation of the safety argument (pre-commit hook + atomic
tarball construction). Note the optional escape hatch.

## Branch

`build/wrapper-skip-tarball`
