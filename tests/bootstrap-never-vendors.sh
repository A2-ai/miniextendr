#!/usr/bin/env bash
# bootstrap.R never vendors: pkgbuild runs it for every build frontend
# (devtools::build/install/check, pak, rv), and only the explicit tarball
# workflow (`just vendor`, miniextendr_vendor()) may seal inst/vendor.tar.xz.
# rpkg reaches no crate by `path` outside the package, so bootstrap must leave
# the source tree exactly as it found it, and the tarball R CMD build seals
# from it must not carry a vendor archive.
set -euo pipefail

cd "$(dirname "$0")/.."

if [ -f rpkg/inst/vendor.tar.xz ]; then
    echo "FAIL: rpkg/inst/vendor.tar.xz is present before the test; run 'just clean-vendor-leak'" >&2
    exit 1
fi

MARKER=$(mktemp)
trap 'rm -f miniextendr_*.tar.gz "$MARKER"' EXIT

# Vendor outputs are gitignored, so git status cannot see them. A `just vendor`
# may legitimately have left rpkg/vendor behind; record what exists now.
VENDOR_PATHS="rpkg/inst/vendor.tar.xz rpkg/vendor rpkg/src/rust/vendor rpkg/src/rust/.dev-bootstrap.rds"
existed=""
for path in $VENDOR_PATHS; do
    if [ -e "$path" ]; then existed="$existed $path"; fi
done

before=$(git status --porcelain --untracked-files=all -- rpkg)
sleep 1 # second-resolution mtimes: anything bootstrap writes is newer than MARKER

( cd rpkg && Rscript bootstrap.R )

after=$(git status --porcelain --untracked-files=all -- rpkg)
if [ "$before" != "$after" ]; then
    echo "FAIL: bootstrap.R changed the source tree:" >&2
    diff <(printf '%s\n' "$before") <(printf '%s\n' "$after") >&2 || true
    exit 1
fi
for path in $VENDOR_PATHS; do
    [ -e "$path" ] || continue
    case " $existed " in
        *" $path "*)
            if [ -n "$(find "$path" -maxdepth 1 -newer "$MARKER" -print -quit)" ]; then
                echo "FAIL: bootstrap.R modified $path" >&2
                exit 1
            fi ;;
        *)
            echo "FAIL: bootstrap.R created $path" >&2
            exit 1 ;;
    esac
done

R CMD build --no-manual rpkg

TARBALL=$(ls -t miniextendr_*.tar.gz 2>/dev/null | head -n1)
if [ -z "$TARBALL" ]; then
    echo "FAIL: R CMD build did not produce a tarball" >&2
    exit 1
fi
# Materialise the listing first: `tar | grep -q` under pipefail can report a
# SIGPIPE from tar as a failure (#551).
TAR_LISTING=$(tar -tzf "$TARBALL" 2>/dev/null)
if grep -qE '(^|/)inst/vendor\.tar\.xz$|(^|/)vendor/' <<<"$TAR_LISTING"; then
    echo "FAIL: $TARBALL carries vendored sources; only the explicit tarball workflow may vendor" >&2
    exit 1
fi

echo "OK: bootstrap.R left rpkg unchanged and $TARBALL carries no vendored sources"
