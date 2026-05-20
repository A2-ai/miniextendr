#!/usr/bin/env bash
# Regression test for #441: bootstrap.R must produce inst/vendor.tar.xz from a
# clean source tree. If a previous run left a stale copy, the build would
# silently bundle it instead of running bootstrap fresh — this test catches that.
#
# Requirements:
#   - cargo-revendor on PATH (bootstrap.R invokes it for auto-vendor)
#
# This test deletes any pre-existing inst/vendor.tar.xz before building, so
# it is safe to run even when a leftover tarball is present in the source tree.
#
# Implementation note (#551): we drive `Rscript bootstrap.R` + `R CMD build rpkg`
# directly rather than going through `pkgbuild::build()`. This mirrors what
# `just r-cmd-build` (the recipe every r-check-* CI job depends on) does —
# minus the explicit `just vendor` step, so bootstrap.R is exercised
# end-to-end exactly as intended. The earlier `pkgbuild::build()` invocation
# was reported as "losing inst/vendor.tar.xz on CI", but post-rewrite
# diagnostics (PR #653) proved the file was always present in the sealed
# tarball — the bug lived in the assertion (see "Assert" block below).
set -euo pipefail

cd "$(dirname "$0")/.."

# Check for required tools; skip (not fail) if missing.
if ! command -v cargo-revendor >/dev/null 2>&1; then
    echo "SKIP: cargo-revendor not on PATH (install with: cargo install --git https://github.com/A2-ai/miniextendr cargo-revendor)"
    exit 0
fi

# Pre: clean state — delete any leftover tarball so we can assert the build
# regenerates it from scratch via bootstrap.R.
rm -f rpkg/inst/vendor.tar.xz

# Stash the tracked Cargo.lock: bootstrap.R unconditionally deletes it and
# runs `cargo generate-lockfile` when the latch is absent (see
# rpkg/bootstrap.R lines ~78-100). Without this stash, a dev whose lockfile
# pins different transitive checksums than `generate-lockfile` would produce
# (e.g. workspace tip moved between revendor runs) ends up with a dirty
# tracked file each time this test runs. The trap restores from the backup.
CARGO_LOCK_BACKUP="/tmp/bootstrap-vendor-test-cargo-lock-$$.bak"
cp rpkg/src/rust/Cargo.lock "$CARGO_LOCK_BACKUP"

# Trap-clean producer artifacts (latch + configure outputs + built tarball)
# so the test is idempotent on dev machines and matches the latch-leak
# hygiene of `just r-cmd-build` (justfile r-cmd-build trap on line ~632).
# Also restore Cargo.lock from the stash (see above) and clear bootstrap.R's
# tmp_bootstrap_vendor sidecar in case bootstrap.R fails mid-way before its
# own restore step runs.
# Note: this trap also removes Makevars and .cargo/config.toml, so a
# `just configure` is needed before the next dev iteration in this checkout.
trap '
  rm -f rpkg/inst/vendor.tar.xz rpkg/src/Makevars rpkg/src/rust/.cargo/config.toml miniextendr_*.tar.gz
  rm -f rpkg/src/rust/.cargo/config.toml.tmp_bootstrap_vendor
  rm -rf rpkg/vendor
  if [ -f "$CARGO_LOCK_BACKUP" ]; then
    mv "$CARGO_LOCK_BACKUP" rpkg/src/rust/Cargo.lock
  fi
' EXIT

# Run bootstrap.R in the package source dir. This produces inst/vendor.tar.xz
# via cargo-revendor and runs ./configure to generate Makevars / .cargo/config.toml.
( cd rpkg && Rscript bootstrap.R )

# R CMD build seals the tarball. With Config/build/bootstrap: TRUE in
# DESCRIPTION, pkgbuild would re-run bootstrap.R; the bare `R CMD build`
# invocation does NOT, which is what we want here (bootstrap already ran).
# --no-manual matches `just r-cmd-build` and avoids needing pdflatex on CI.
R CMD build --no-manual rpkg

# Find the produced tarball (R CMD build writes miniextendr_X.Y.Z.tar.gz).
TARBALL=$(ls -t miniextendr_*.tar.gz 2>/dev/null | head -n1)
if [ -z "$TARBALL" ]; then
    echo "FAIL: R CMD build did not produce a tarball" >&2
    exit 1
fi

# Assert: the tarball ships inst/vendor.tar.xz (produced by bootstrap.R).
#
# IMPORTANT: do NOT use `if ! tar -tzf … | grep -q PATTERN; then FAIL`. Under
# `set -o pipefail` (active here via `set -euo pipefail`), `grep -q` short-
# circuits on the first match and closes its stdin. `tar` continues writing
# and gets SIGPIPE → exits non-zero. pipefail propagates the non-zero rc to
# the pipeline; `!` inverts it; the `if` body fires; FAIL is reported even
# though the file IS in the tarball. The bug manifests reliably on Ubuntu
# (Linux pipe buffer 64 KB) but not macOS (16 KB) — process-scheduling
# accident, not a real failure. See #551 / PR #653 investigation history.
#
# Fix: materialise the tar listing into a variable first (no pipe involved in
# the check), then grep the variable. tar is fully drained, no SIGPIPE risk.
TAR_LISTING=$(tar -tzf "$TARBALL" 2>/dev/null)
if ! grep -qE 'inst/vendor\.tar\.xz$' <<<"$TAR_LISTING"; then
    echo "FAIL: $TARBALL does not contain inst/vendor.tar.xz" >&2
    echo "      Bootstrap pipeline regression — see #441/#440." >&2
    exit 1
fi

echo "OK: $TARBALL contains inst/vendor.tar.xz"
