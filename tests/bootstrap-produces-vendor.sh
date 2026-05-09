#!/usr/bin/env bash
# Regression test for #441: bootstrap.R must produce inst/vendor.tar.xz from a
# clean source tree. If a previous run left a stale copy, the build would
# silently bundle it instead of running bootstrap fresh — this test catches that.
#
# Requirements:
#   - cargo-revendor on PATH (bootstrap.R invokes it for auto-vendor)
#   - pkgbuild installed in R
#
# This test deletes any pre-existing inst/vendor.tar.xz before building, so
# it is safe to run even when a leftover tarball is present in the source tree.
set -euo pipefail

cd "$(dirname "$0")/.."

# Check for required tools and R packages; skip (not fail) if missing.
if ! command -v cargo-revendor >/dev/null 2>&1; then
    echo "SKIP: cargo-revendor not on PATH (install with: cargo install --git https://github.com/A2-ai/miniextendr cargo-revendor)"
    exit 0
fi

if ! Rscript -e 'if (!requireNamespace("pkgbuild", quietly = TRUE)) quit(status = 1)' 2>/dev/null; then
    echo "SKIP: pkgbuild not installed in R (install with: install.packages('pkgbuild'))"
    exit 0
fi

# Pre: clean state — delete any leftover tarball so we can assert the build
# regenerates it from scratch via bootstrap.R.
rm -f rpkg/inst/vendor.tar.xz

# Build into a throwaway lib to avoid touching the user's R library.
TMP_LIB=$(mktemp -d)
trap 'rm -rf "$TMP_LIB" rpkg/inst/vendor.tar.xz miniextendr_*.tar.gz' EXIT

R_LIBS_USER="$TMP_LIB" Rscript -e \
  ".libPaths(c('${TMP_LIB}', .libPaths())); pkgbuild::build('rpkg', dest_path = '.')"

# Find the produced tarball (pkgbuild writes miniextendr_X.Y.Z.tar.gz).
TARBALL=$(ls -t miniextendr_*.tar.gz 2>/dev/null | head -n1)
if [ -z "$TARBALL" ]; then
    echo "FAIL: pkgbuild::build did not produce a tarball" >&2
    exit 1
fi

# Assert: the tarball ships inst/vendor.tar.xz (produced by bootstrap.R).
if ! tar -tJf "$TARBALL" 2>/dev/null | grep -q 'inst/vendor\.tar\.xz$'; then
    # Try plain tar (non-xz tarball from pkgbuild on some platforms)
    if ! tar -tzf "$TARBALL" 2>/dev/null | grep -q 'inst/vendor\.tar\.xz$'; then
        echo "FAIL: $TARBALL does not contain inst/vendor.tar.xz" >&2
        echo "      Bootstrap pipeline regression — see #441/#440." >&2
        exit 1
    fi
fi

echo "OK: $TARBALL contains inst/vendor.tar.xz"
