#!/usr/bin/env bash
# Regression fixture for #1555: in tarball mode, configure reuses the source
# replacements cargo-revendor records in vendor/.cargo-config.toml.
#
# Cargo matches a source replacement by the exact source id, so a Git
# dependency pinned by `rev`, `branch` or `tag`, or one reached only
# transitively, needs its own `[source."git+<url>?<selector>"]` entry. Only
# `cargo vendor` knows the full set, and cargo-revendor keeps its output in the
# archive. This fixture builds that case with three local Git repositories:
#
#   package -> alpha (rev) -> beta (branch) -> gamma (tag)
#
# plus a path dependency outside the package, which `--freeze` redirects into
# vendor/ through a relative `[patch.crates-io]` entry. It then:
#   1. produces inst/vendor.tar.xz with `cargo revendor --freeze --compress`,
#      the invocation bootstrap.R uses, and checks the archive carries
#      vendor/.cargo-config.toml with all three selectors;
#   2. copies the package to a new directory (as installing a built tarball
#      does) and deletes the original package, the Git repositories and the
#      path dependency, so only the archive can supply the sources and the
#      recorded `directory =` is stale;
#   3. runs configure and checks the relocated `directory =`, the `[env]` and
#      `[build]` tables, and that `cargo metadata --locked --offline` and
#      `cargo check --locked --offline` succeed with an empty CARGO_HOME;
#   4. repeats the install with VENDOR_OUT (the shared vendor cache, #1513) and
#      checks `directory =` and the `[patch.crates-io]` paths name the same
#      cache entry;
#   5. removes vendor/.cargo-config.toml from the archive and checks configure
#      exits 1 with the rebuild instructions.
#
# Usage: bash rpkg/dev/tarball-cargo-config.sh [PACKAGE_DIR]
#   PACKAGE_DIR supplies configure, DESCRIPTION, NAMESPACE, tools/ and the
#   src/ templates (default: rpkg, the directory above this script). Pass a
#   scaffolded package to check the template's configure.
# Requires: bash, cargo, cargo-revendor, git, tar with xz, Rscript.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
pkg_src="$(cd "${1:-$here/..}" && pwd)"

for tool in cargo cargo-revendor git tar Rscript; do
  command -v "$tool" >/dev/null 2>&1 || { echo "FAIL: $tool not on PATH" >&2; exit 1; }
done
test -f "$pkg_src/configure" || { echo "FAIL: $pkg_src/configure missing (run autoconf)" >&2; exit 1; }

work="$(cd "$(mktemp -d)" && pwd -P)"
trap 'rm -rf "$work"' EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

# The fixture controls every cargo and configure input itself.
unset VENDOR_OUT CARGO_TARGET_DIR CARGO_BUILD_TARGET CARGO_NET_OFFLINE RUSTC_WRAPPER CARGO_BUILD_RUSTC_WRAPPER
export CARGO_FEATURES=""
export CARGO_PROFILE="dev"
export CARGO_TERM_COLOR="never"

# region: local Git dependencies

commit_repo() {
  (
    cd "$1"
    git init -q
    git add -A
    git -c user.name=fixture -c user.email=fixture@example.invalid \
      commit -q -m "fixture"
  )
}

crate_manifest() {
  printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n%s\n' "$1" "$2"
}

mkdir -p "$work/git/gamma/src" "$work/git/beta/src" "$work/git/alpha/src"

crate_manifest gamma "" > "$work/git/gamma/Cargo.toml"
echo 'pub fn value() -> i32 { 3 }' > "$work/git/gamma/src/lib.rs"
commit_repo "$work/git/gamma"
(cd "$work/git/gamma" && git tag v1.0.0)

crate_manifest beta "gamma = { git = \"file://$work/git/gamma\", tag = \"v1.0.0\" }" \
  > "$work/git/beta/Cargo.toml"
echo 'pub fn value() -> i32 { gamma::value() + 2 }' > "$work/git/beta/src/lib.rs"
commit_repo "$work/git/beta"
(cd "$work/git/beta" && git branch stable)

crate_manifest alpha "beta = { git = \"file://$work/git/beta\", branch = \"stable\" }" \
  > "$work/git/alpha/Cargo.toml"
echo 'pub fn value() -> i32 { beta::value() + 1 }' > "$work/git/alpha/src/lib.rs"
commit_repo "$work/git/alpha"
alpha_rev="$(cd "$work/git/alpha" && git rev-parse HEAD)"

mkdir -p "$work/delta/src"
crate_manifest delta "" > "$work/delta/Cargo.toml"
echo 'pub fn value() -> i32 { 4 }' > "$work/delta/src/lib.rs"

# endregion

# region: package and vendor archive

copy_package_skeleton() {
  mkdir -p "$2/src/rust" "$2/inst"
  cp "$1/configure" "$1/DESCRIPTION" "$1/NAMESPACE" "$2/"
  cp -R "$1/tools" "$2/tools"
  for f in "$1"/src/*.in; do cp "$f" "$2/src/"; done
}

pkg="$work/pkg"
copy_package_skeleton "$pkg_src" "$pkg"
cat > "$pkg/src/rust/Cargo.toml" <<EOF
[package]
name = "tarballprobe"
version = "0.1.0"
edition = "2021"

[workspace]

[lib]
path = "lib.rs"

[dependencies]
alpha = { git = "file://$work/git/alpha", rev = "$alpha_rev" }
delta = { path = "../../../delta" }
EOF
echo 'pub fn value() -> i32 { alpha::value() + delta::value() }' > "$pkg/src/rust/lib.rs"

# Resolve and vendor with a private Cargo home: the Git checkouts it caches must
# not be visible to the offline checks below.
export CARGO_HOME="$work/cargo-home-vendor"
cargo generate-lockfile --quiet --manifest-path "$pkg/src/rust/Cargo.toml"
(cd "$pkg" && cargo revendor --manifest-path src/rust/Cargo.toml --output vendor \
  --freeze --compress inst/vendor.tar.xz --blank-md --source-marker --force) \
  > "$work/revendor.log" 2>&1 || { cat "$work/revendor.log" >&2; fail "cargo revendor failed"; }

# Extract the one member rather than piping a listing into `grep -q`: under
# pipefail an early grep exit can SIGPIPE tar into a false failure (#551).
tar -xJOf "$pkg/inst/vendor.tar.xz" vendor/.cargo-config.toml > "$work/archived-config.toml" 2>/dev/null \
  || fail "inst/vendor.tar.xz does not contain vendor/.cargo-config.toml"
for selector in "?rev=$alpha_rev" "?branch=stable" "?tag=v1.0.0"; do
  grep -Fq "$selector\"]" "$work/archived-config.toml" \
    || fail "archived .cargo-config.toml has no source entry for $selector"
done
stale_dir="$(sed -n 's/^directory = "\(.*\)"$/\1/p' "$work/archived-config.toml")"
test -n "$stale_dir" || fail "archived .cargo-config.toml has no directory line"

# endregion

# region: configure checks

# Copy the package as a built tarball would carry it (vendor/ is Rbuildignored)
# and remove every original source, so the archived directory points nowhere
# and nothing but the archive can resolve a dependency.
built="$work/built"
cp -R "$pkg" "$built"
rm -rf "$built/vendor" "$built/src/rust/.cargo"
rm -rf "$pkg" "$work/git" "$work/delta"
install_copy() {
  rm -rf "$1"
  mkdir -p "$(dirname "$1")"
  cp -R "$built" "$1"
}
inst="$work/install/pkg"
install_copy "$inst"
test ! -e "$stale_dir" || fail "stale vendor directory $stale_dir still exists"

run_configure() {
  (cd "$inst" && bash ./configure) > "$work/configure.log" 2>&1
}

check_offline() {
  local cargo_home="$work/empty-cargo-home-$1"
  mkdir -p "$cargo_home"
  (cd "$inst/src/rust" && CARGO_HOME="$cargo_home" \
    cargo metadata --locked --offline --format-version 1) > "$work/metadata.json" 2> "$work/metadata.log" \
    || { cat "$work/metadata.log" >&2; fail "cargo metadata --locked --offline failed ($1)"; }
  for name in alpha beta gamma delta; do
    grep -Fq "\"name\":\"$name\"" "$work/metadata.json" || fail "cargo metadata did not resolve $name ($1)"
  done
  (cd "$inst/src/rust" && CARGO_HOME="$cargo_home" cargo check --locked --offline --quiet) \
    > "$work/check.log" 2>&1 || { cat "$work/check.log" >&2; fail "cargo check --locked --offline failed ($1)"; }
}

cfg="$inst/src/rust/.cargo/config.toml"
directory_of() { sed -n 's/^directory = "\(.*\)"$/\1/p' "$cfg"; }

run_configure || { cat "$work/configure.log" >&2; fail "configure failed"; }
test "$(grep -v '^#' "$cfg" | grep -v '^$' | head -n 1)" = "[env]" || fail "[env] is not the first table"
test "$(directory_of)" = "$inst/vendor" || fail "directory is '$(directory_of)', expected '$inst/vendor'"
test "$(grep -c '^directory = ' "$cfg")" = 1 || fail "expected exactly one directory line"
if grep -Fq "$stale_dir" "$cfg"; then fail "config still names the stale directory $stale_dir"; fi
grep -Fxq '[build]' "$cfg" || fail "[build] table missing"
grep -Fxq "target-dir = \"$inst/rust-target\"" "$cfg" || fail "target-dir missing"
for selector in "?rev=$alpha_rev" "?branch=stable" "?tag=v1.0.0"; do
  grep -Fq "$selector\"]" "$cfg" || fail "config has no source entry for $selector"
done
check_offline default
echo "ok: tarball configure relocates vendor/.cargo-config.toml to $inst/vendor"

# Shared vendor cache: the entry configure unpacks into must be both the
# vendored-sources directory and the target of the frozen [patch.crates-io].
cache="$work/vendor-cache"
install_copy "$inst"
(export VENDOR_OUT="$cache"; run_configure) || { cat "$work/configure.log" >&2; fail "configure with VENDOR_OUT failed"; }
entry="$(directory_of)"
case "$entry" in
  "$cache"/*/vendor) ;;
  *) fail "directory is '$entry', expected an entry under $cache" ;;
esac
test -f "$entry/.cargo-config.toml" || fail "cache entry $entry has no .cargo-config.toml"
test ! -e "$inst/vendor" || fail "configure unpacked into the package despite VENDOR_OUT"
grep -Fxq '[patch.crates-io]' "$cfg" || fail "[patch.crates-io] missing with VENDOR_OUT"
grep -Fq "path = \"$entry/delta" "$cfg" || fail "[patch.crates-io] does not point into $entry"
check_offline cache
echo "ok: VENDOR_OUT cache entry $entry backs both directory and [patch.crates-io]"

# An archive without the file (e.g. a plain `cargo vendor`) must stop configure.
install_copy "$inst"
mkdir "$work/repack"
tar -xJf "$inst/inst/vendor.tar.xz" -C "$work/repack"
rm "$work/repack/vendor/.cargo-config.toml"
(cd "$work/repack" && COPYFILE_DISABLE=1 tar -cJf "$inst/inst/vendor.tar.xz" vendor)
set +e
run_configure
status=$?
set -e
test "$status" = 1 || { cat "$work/configure.log" >&2; fail "configure exited $status without .cargo-config.toml, expected 1"; }
grep -Fq "vendor/.cargo-config.toml" "$work/configure.log" || fail "configure did not name the missing file"
grep -Fq "miniextendr_vendor()" "$work/configure.log" || fail "configure did not say how to rebuild the archive"
echo "ok: configure stops when the archive has no vendor/.cargo-config.toml"

# endregion
