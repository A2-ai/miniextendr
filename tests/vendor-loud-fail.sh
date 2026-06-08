#!/usr/bin/env bash
# Regression test for #876: `just vendor` must fail LOUDLY when a framework
# crate would be vendored from git instead of the local workspace.
#
# Background: cargo-revendor reads `[patch."<git-url>"]` path overrides from
# .cargo/config.toml to vendor miniextendr-{api,lint,macros} from the local
# checkout. When configure runs in tarball mode (inst/vendor.tar.xz present) it
# writes a `[source]` replacement but NO `[patch]`, so cargo-revendor finds
# "0 local packages" and silently ships git@main — dropping in-PR framework
# edits (the #865 latch leak). The `just vendor` recipe now asserts every
# framework crate appears in `--json local_crates`; this test proves that
# assertion fires on the leak shape.
#
# This test is hermetic: it builds a tiny package whose only dependency is a
# crate served from a LOCAL bare git repo (file:// URL, no crates.io deps), so
# `cargo vendor` makes no network calls. It does NOT touch the real rpkg tree.
#
# Requirements:
#   - cargo-revendor on PATH (skips, not fails, if missing)
#   - git, cargo
set -euo pipefail

if ! command -v cargo-revendor >/dev/null 2>&1; then
    echo "SKIP: cargo-revendor not on PATH (install with: cargo install --path cargo-revendor)"
    exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# --- 1. A bare git repo standing in for a framework crate ("miniextendr-api").
SRC="$WORK/src/miniextendr-api"
mkdir -p "$SRC/src"
cat >"$SRC/Cargo.toml" <<'EOF'
[package]
name = "miniextendr-api"
version = "0.1.0"
edition = "2021"
publish = false
EOF
echo 'pub fn upstream() {}' >"$SRC/src/lib.rs"
git -C "$SRC" init -q -b main
git -C "$SRC" -c user.email=t@t.com -c user.name=t add .
git -C "$SRC" -c user.email=t@t.com -c user.name=t commit -q -m init
BARE="$WORK/miniextendr-api.git"
git clone --bare -q "$SRC" "$BARE"
GIT_URL="file://$BARE"

# --- 2. A package that depends on the framework crate via the bare git URL,
#         with NO `[patch]` override — the latch-leak shape.
PKG="$WORK/pkg"
mkdir -p "$PKG"
cat >"$PKG/Cargo.toml" <<EOF
[package]
name = "leak-pkg"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
miniextendr-api = { git = "$GIT_URL" }
EOF
echo 'pub use miniextendr_api::upstream;' >"$PKG/lib.rs"
git -C "$PKG" init -q
git -C "$PKG" -c user.email=t@t.com -c user.name=t add .
git -C "$PKG" -c user.email=t@t.com -c user.name=t commit -q -m init

# --- 3. Vendor with --json and apply the EXACT assertion the recipe uses.
revendor_json="$(cargo-revendor revendor \
  --manifest-path "$PKG/Cargo.toml" \
  --output "$PKG/vendor" \
  --json)"

leaked=""
for crate in miniextendr-api miniextendr-lint miniextendr-macros; do
  if ! grep -qE "\"$crate\"" <<<"$revendor_json"; then
    leaked="$leaked $crate"
  fi
done

# miniextendr-api was fetched from git, so it must be reported as the leak.
case "$leaked" in
  *miniextendr-api*)
    echo "OK: loud-fail assertion correctly flagged git-vendored framework crate(s):$leaked"
    ;;
  *)
    echo "FAIL: a git-vendored framework crate was NOT flagged — the loud-fail" >&2
    echo "      assertion (#876) would have silently passed on the latch-leak shape." >&2
    echo "      local_crates JSON was:" >&2
    echo "$revendor_json" >&2
    exit 1
    ;;
esac
