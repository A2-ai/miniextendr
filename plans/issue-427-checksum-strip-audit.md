+++
title = "Issue #427 — Audit the Cargo.lock checksum strip in tarball-mode configure"
+++

# Audit the Cargo.lock checksum strip in tarball-mode configure

Closes #427. Three `configure.ac` files strip `checksum = ` lines from
`Cargo.lock` at tarball-install time. PR #408 rewrote `cargo-revendor` to
preserve the original `package` hash in each `.cargo-checksum.json` (so
`Cargo.lock`'s `checksum =` line still matches the vendored copy after
CRAN-trim). The strip was necessary before #408 because the old behaviour
wrote `{"files":{}}` (null package), leaving the lockfile with a checksum
that cargo could not verify — producing the "checksum for X could not be
calculated" error (see also #426 lockfile-mode-unification and #342 original
checksum-strip rationale). With #408 in place the three possible outcomes are:
(1) the strip is now redundant — checksums agree, build succeeds without it;
(2) retaining checksums actively strengthens integrity — cargo now verifies
lockfile vs `.cargo-checksum.json` end-to-end and that verification should
be left intact; (3) the strip is still required — cargo still errors, meaning
something in the tarball path nullifies the `package` field after #408.

## Work items

1. Run `just configure && just vendor && just r-cmd-build` on origin/main to
   produce a clean CRAN-trimmed tarball whose vendor/ was created with the
   post-#408 `cargo-revendor`. Confirm `Cargo.lock` inside the tarball still
   has `checksum = ` lines (the strip fires in configure, not at vendor time).

2. Unpack the built tarball into a scratch directory, patch its `configure.ac`
   to skip the sed strip (wrap the `$SED -i.bak …` block in `if false; then
   … fi`), regenerate `configure` with `autoconf`, then run
   `R CMD check --as-cran` against the patched tarball. Capture all output.
   Watch for "checksum for … could not be calculated" or "checksum for …
   changed between lock files" cargo errors.

3. Decide based on observed output:
   - **Success, no checksum errors** → outcome (1)/(2): strip is redundant
     (or actively harmful to integrity, since it discards a valid check).
     Either way: remove the `$SED` block and its comment from all three
     `configure.ac` files, regenerate each `configure`, run
     `just templates-approve`, verify `just templates-check` clean.
   - **cargo checksum error** → outcome (3): strip is still required.
     Restore it, extend comment to mention #408 and why the strip persists,
     update `docs/CARGO_LOCK_SHAPE.md` with the exact cargo error observed,
     so the next person who tries to remove it has the empirical record.

4. Regardless of outcome: verify `just r-cmd-check` (aliased `rcmdcheck`)
   passes end-to-end in tarball mode. Commit plan + outcome change together.

## Files to edit (outcome: strip is redundant)

- `rpkg/configure.ac` — remove lines 303–314 (comment block + sed + rm bak)
- `rpkg/configure` — regenerate via `autoconf` inside `rpkg/`
- `minirextendr/inst/templates/rpkg/configure.ac` — remove same block (~334–344)
- `minirextendr/inst/templates/monorepo/rpkg/configure.ac` — remove same block (~334–344)
- `patches/templates.patch` — regenerate via `just templates-approve`

## Files to edit (outcome: strip still needed)

- `rpkg/configure.ac` — extend comment to explain #408 does not fully fix it
- `docs/CARGO_LOCK_SHAPE.md` — add empirical error message + explanation

## Acceptance

- `just r-cmd-check` (tarball mode) passes with no cargo checksum errors.
- `just templates-check` passes (no unexpected template drift).
- All three `configure.ac` files are in sync (same decision applied everywhere).
