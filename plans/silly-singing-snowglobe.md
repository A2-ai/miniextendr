# Simplify configure.ac: skip vendoring in dev mode

## Context

Every `just configure` (NOT_CRAN=true) currently runs the full vendoring pipeline:
cargo package for 5 workspace crates, cargo vendor for ~200 external crates, then
xz-compress everything into inst/vendor.tar.xz. This takes ~30s and produces artifacts
that dev mode never uses — cargo resolves deps via `.cargo/config.toml` patches.

**Goal**: In dev mode, skip vendoring entirely and clean up stale vendor artifacts.
Move vendoring to an explicit `just vendor` recipe for CRAN release prep.

## Three modes after this change

| Mode | Trigger | Vendoring behavior |
|------|---------|-------------------|
| Dev | `NOT_CRAN=true ./configure` (or `just configure`) | Remove `src/vendor/` and `inst/vendor.tar.xz`. No cargo vendor. |
| CRAN build | `./configure` (NOT_CRAN unset) | Unpack `inst/vendor.tar.xz` if needed (unchanged). |
| CRAN prep | `just vendor` | Full cargo package + cargo vendor + compress to tarball. |

## Files to modify

### 1. `rpkg/configure.ac`

**`cargo-vendor` section (line ~377)**: Replace the NOT_CRAN=true branch (lines 407-564) with:
```sh
if test "$NOT_CRAN" = "true"; then
    # Dev mode: clean up vendor artifacts, skip vendoring entirely
    if test -d "$VENDOR_OUT" && test -n "$(ls -A "$VENDOR_OUT" 2>/dev/null)"; then
        rm -rf "$VENDOR_OUT"
        echo "configure: removed vendor directory (dev mode)"
    fi
    if test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
        rm -f "$abs_rpkg_dir/inst/vendor.tar.xz" "$abs_rpkg_dir/inst/.vendor.tar.xz.cksum"
        echo "configure: removed vendor.tar.xz (dev mode)"
    fi
    # Remove vendor stamp
    rm -f "$VENDOR_OUT/.vendor.lock.cksum"
    echo "configure: dev mode - skipping cargo vendor (use 'just vendor' for CRAN prep)"
fi
```

Keep the NOT_CRAN=false (CRAN build) branch **unchanged** — it handles unpacking
vendor.tar.xz, GitHub install fallback, etc.

**`compress-vendor` section (line ~592)**: Replace entirely with a no-op:
```sh
echo "configure: skipping vendor compression (use 'just vendor' for CRAN prep)"
```

### 2. `justfile`

**Add `vendor` recipe** — extracted from configure.ac's old NOT_CRAN=true vendoring logic:
```just
# Vendor dependencies for CRAN release preparation
#
# This packages workspace crates, vendors external deps from crates.io,
# and compresses everything into inst/vendor.tar.xz for the CRAN tarball.
# Only needed when preparing a CRAN submission — not for day-to-day dev.
vendor:
    #!/usr/bin/env bash
    set -euo pipefail

    root="$(cd "$(dirname "$0")" && pwd)"  # monorepo root
    rpkg_src="$root/rpkg/src"
    vendor_out="$rpkg_src/vendor"
    manifest="$rpkg_src/rust/Cargo.toml"
    lockfile="$rpkg_src/rust/Cargo.lock"

    echo "=== CRAN vendor prep ==="

    # 1. Package workspace crates
    staging="$rpkg_src/.vendor-tarball-staging"
    rm -rf "$staging"
    mkdir -p "$staging" "$vendor_out"

    echo "Packaging workspace crates..."
    for crate in miniextendr-api miniextendr-macros miniextendr-macros-core miniextendr-lint miniextendr-engine; do
        if [ -d "$root/$crate" ]; then
            echo "  $crate"
            cargo package --manifest-path "$root/$crate/Cargo.toml" \
                --target-dir "$staging/target" --allow-dirty --no-verify 2>&1 | \
                grep -v "warning: manifest has no" || true
        fi
    done

    # 2. Extract .crate files to vendor/
    echo "Extracting packaged crates..."
    for crate_file in "$staging/target/package/"*.crate; do
        [ -f "$crate_file" ] || continue
        basename=$(basename "$crate_file" .crate)
        echo "  $basename"
        mkdir -p "$vendor_out/$basename"
        tar -xzf "$crate_file" -C "$vendor_out/$basename" --strip-components=1
    done

    # 3. Vendor external deps from crates.io
    echo "Vendoring external dependencies..."
    cargo vendor --manifest-path "$manifest" "$vendor_out"

    # 4. Strip checksums from Cargo.lock
    if [ -f "$lockfile" ]; then
        sed -i.bak '/^checksum = /d' "$lockfile" && rm -f "$lockfile.bak"
    fi

    # 5. Compress for CRAN tarball
    echo "Compressing vendor.tar.xz..."
    compress_staging="$rpkg_src/.vendor-compress-staging"
    rm -rf "$compress_staging"
    mkdir -p "$compress_staging"
    cp -R "$vendor_out" "$compress_staging/vendor"

    # Clear checksums and strip unneeded files
    for d in "$compress_staging/vendor"/*/; do
        [ -d "$d" ] && echo '{"files":{}}' > "${d}.cargo-checksum.json"
    done
    find "$compress_staging/vendor" -type d \( -name tests -o -name benches -o -name examples -o -name .github -o -name docs \) -exec rm -rf {} + 2>/dev/null || true
    find "$compress_staging/vendor" -name '*.md' -type f -exec truncate -s 0 {} \; 2>/dev/null || true

    mkdir -p "$root/rpkg/inst"
    tar -cJf "$root/rpkg/inst/vendor.tar.xz" -C "$compress_staging" vendor

    # Clean up staging
    rm -rf "$staging" "$compress_staging"

    echo "=== Done: rpkg/inst/vendor.tar.xz ready for CRAN ==="
```

**Update `configure-cran` recipe** — currently just runs configure without NOT_CRAN.
No change needed, since CRAN build just unpacks the tarball.

### 3. `CLAUDE.md` — Update build commands section

Add `just vendor` to the quick reference and CRAN workflow documentation.

## Verification

```bash
# 1. Dev mode is fast and clean
just configure
# Should NOT vendor, should remove any stale vendor/ and vendor.tar.xz

# 2. CRAN prep works
just vendor
# Should create rpkg/src/vendor/ and rpkg/inst/vendor.tar.xz

# 3. CRAN build still works
just configure-cran
# Should unpack vendor.tar.xz and configure for offline build

# 4. rpkg still builds
just rcmdinstall
just devtools-test
```
