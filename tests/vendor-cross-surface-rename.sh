#!/usr/bin/env bash
# Regression test for #883: a PR that adds/renames a cargo feature on a
# framework crate (miniextendr-api) AND references it from rpkg in the SAME
# commit must vendor successfully.
#
# The new feature exists only in the local working tree, never on git@main.
# The old bootstrap.R / `just vendor` pre-resolved Cargo.lock against the bare
# git URL (origin/main) with the dev [patch] override moved aside — so cargo
# resolved miniextendr-api at main, which lacks the new feature, and errored:
#
#   package `miniextendr` depends on `miniextendr-api` with feature
#   `mxl883-probe` but `miniextendr-api` does not have that feature.
#
# That forced an admin-merge (PR #710 was the last one). After #883, bootstrap.R
# runs ./configure FIRST (writing the [patch] override) and cargo-revendor
# resolves against the LOCAL workspace, then stamps the git+url#<sha> source
# back into Cargo.lock. So this cross-surface rename now vendors with no
# admin-merge, and the lock stays CRAN-installable (tarball-shape).
#
# Requirements: cargo-revendor on PATH (SKIP if absent, like the sibling tests).
#
# Asserts:
#   1. bootstrap.R (configure + cargo-revendor) exits 0 on the cross-rename.
#   2. src/rust/Cargo.lock is tarball-shape: every framework crate carries
#      source = "git+https://github.com/A2-ai/miniextendr#<sha>".
#   3. The vendored miniextendr-api carries the local-only feature — proving the
#      vendor came from the working tree, not a git@main fetch.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v cargo-revendor >/dev/null 2>&1; then
    echo "SKIP: cargo-revendor not on PATH (install with: cargo install --git https://github.com/A2-ai/miniextendr cargo-revendor)"
    exit 0
fi

API_TOML="miniextendr-api/Cargo.toml"
RPKG_TOML="rpkg/src/rust/Cargo.toml"
LOCK="rpkg/src/rust/Cargo.lock"

# Probe feature: a name that exists ONLY in this working tree, never on
# origin/main. This is what makes the test a faithful #883 reproduction.
PROBE_API_FEATURE="mxl883-probe"
PROBE_RPKG_FEATURE="mxl883"

# Stash the tracked files we mutate, plus configure/vendor outputs, and restore
# everything on exit (success, failure, or interrupt) so the test is idempotent.
LOCK_BACKUP="/tmp/vendor-cross-rename-cargo-lock-$$.bak"
cp "$LOCK" "$LOCK_BACKUP"
trap '
  git checkout -- "'"$API_TOML"'" "'"$RPKG_TOML"'" 2>/dev/null || true
  if [ -f "'"$LOCK_BACKUP"'" ]; then mv "'"$LOCK_BACKUP"'" "'"$LOCK"'"; fi
  rm -f rpkg/inst/vendor.tar.xz rpkg/src/Makevars rpkg/src/rust/.cargo/config.toml
  rm -f rpkg/src/rust/.cargo/config.toml.tmp_bootstrap_vendor
  rm -rf rpkg/vendor
' EXIT

# Inject the cross-crate feature: define it on miniextendr-api, reference it
# from rpkg. awk insertion is portable across GNU/BSD (sed -i is not).
inject_after_features() {
  # $1 = file, $2 = line to insert right after the first [features] header
  awk -v ins="$2" '
    /^\[features\]/ && !done { print; print ins; done = 1; next }
    { print }
  ' "$1" > "$1.mxl883.tmp" && mv "$1.mxl883.tmp" "$1"
}

inject_after_features "$API_TOML" "$PROBE_API_FEATURE = []"
inject_after_features "$RPKG_TOML" "$PROBE_RPKG_FEATURE = [\"miniextendr-api/$PROBE_API_FEATURE\"]"

# Confirm the injection landed (guards against a [features]-less manifest).
grep -q "^$PROBE_API_FEATURE = \[\]" "$API_TOML" \
  || { echo "FAIL: could not inject probe feature into $API_TOML" >&2; exit 1; }
grep -q "miniextendr-api/$PROBE_API_FEATURE" "$RPKG_TOML" \
  || { echo "FAIL: could not inject probe feature into $RPKG_TOML" >&2; exit 1; }

# Start from a clean latch so bootstrap.R actually runs the vendor branch.
rm -f rpkg/inst/vendor.tar.xz

# Drive the exact Bootstrap Vendor path: configure (writes the [patch]
# override) then cargo-revendor (resolves local, stamps git source).
echo "Running bootstrap.R with cross-surface rename injected..."
if ! ( cd rpkg && Rscript bootstrap.R ); then
    echo "FAIL: bootstrap.R errored on a coordinated cross-crate feature rename." >&2
    echo "      This is the #883 regression: the framework crate must resolve" >&2
    echo "      against the local workspace, not git@main." >&2
    exit 1
fi

# Assert 2: tarball-shape lock (framework crates carry the git source).
for crate in miniextendr-api miniextendr-lint miniextendr-macros; do
    if ! grep -A3 "^name = \"$crate\"\$" "$LOCK" \
            | grep -q '^source = "git+https://github\.com/A2-ai/miniextendr'; then
        echo "FAIL: $LOCK — $crate is missing the canonical git+url source." >&2
        echo "      The stamp step did not run; the offline tarball would not install." >&2
        exit 1
    fi
done

# Assert 3: the vendored miniextendr-api carries the local-only feature,
# proving it came from the working tree (not a git@main fetch). Capture into a
# variable first — `tar | grep -q` under `set -o pipefail` can SIGPIPE (#551).
API_VENDORED_TOML="$(tar -xJOf rpkg/inst/vendor.tar.xz vendor/miniextendr-api/Cargo.toml 2>/dev/null || true)"
if ! grep -q "$PROBE_API_FEATURE" <<<"$API_VENDORED_TOML"; then
    echo "FAIL: vendored miniextendr-api is missing the local-only feature '$PROBE_API_FEATURE'." >&2
    echo "      The vendor came from git@main, not the working tree (#876 angle)." >&2
    exit 1
fi

echo "OK: cross-surface rename vendored from the local workspace; lock is tarball-shape."
