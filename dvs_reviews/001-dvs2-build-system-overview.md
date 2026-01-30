# DVS2 Build System Overview

## Summary

DVS2 uses a cleaner, more streamlined approach to R package configuration compared to miniextendr. Key differences include simpler vendor handling, auto-generated feature flags, and a more elegant `--locked` flag strategy.

## Key Observations

### 1. Static Library Name Extraction (configure.ac:140-144)

DVS2 extracts `CARGO_STATICLIB_NAME` directly from `Cargo.toml.in` at configure time using sed:

```m4
CARGO_STATICLIB_NAME="$($SED -n 's/^name = "\(.*\)"/\1/p' "$RUST_SRC_DIR/Cargo.toml.in" | head -1 | tr '-' '_')"
```

**Comparison with miniextendr:**
- miniextendr uses a placeholder (`__CARGO_STATICLIB_NAME_PLACEHOLDER__`) and later patches it via `cargo pkgid` in `AC_CONFIG_COMMANDS`
- DVS2's approach is simpler and doesn't require network access or cargo invocation

**Recommendation:** Consider adopting DVS2's sed-based extraction for `CARGO_STATICLIB_NAME`. The `cargo pkgid` approach adds complexity and may fail if cargo cache isn't populated.

### 2. `--locked` Flag Strategy (configure.ac:37-45)

DVS2's approach:

```m4
if test "$NOT_CRAN" = "true"; then
  CARGO_OFFLINE_FLAG=""
  CARGO_LOCKED_FLAG="--locked"  # DEV uses --locked
else
  CARGO_OFFLINE_FLAG="--offline"
  CARGO_LOCKED_FLAG=""          # CRAN does NOT use --locked
fi
```

**Rationale:**
- DEV mode uses `--locked` for reproducible builds from the committed lockfile
- CRAN mode omits `--locked` because checksums are stripped from vendored crates

**Comparison with miniextendr:**
- miniextendr recently fixed its backwards `--locked` logic (was swapped)
- Now matches: DEV=no lock, CRAN=lock
- DVS2's approach is the OPPOSITE: DEV=lock, CRAN=no lock

**Analysis:**
DVS2's logic is arguably more correct:
1. In dev mode, you want reproducible builds matching the committed lockfile
2. In CRAN mode, checksums are cleared from vendor tarball, so `--locked` fails

**Recommendation:** Re-evaluate miniextendr's `--locked` logic. The DEV mode should use `--locked` for reproducibility; CRAN mode should NOT because vendor tarballs have cleared checksums.

### 3. Automatic Feature Flag to CPPFLAGS (configure.ac:49-68)

DVS2 auto-generates C preprocessor flags from cargo features:

```m4
for _feature in $DVS_FEATURES; do
  _feature_upper=$(echo "$_feature" | tr 'a-z-' 'A-Z_')
  CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_$_feature_upper"
done
```

**Comparison with miniextendr:**
- miniextendr manually lists features in case statements:
```m4
case ",$MINIEXTENDR_FEATURES," in
  *,nonapi,*) CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_NONAPI" ;;
esac
```

**Recommendation:** Adopt DVS2's automatic approach. It's DRY and automatically handles new features.

### 4. Git Source Vendoring (configure.ac:220-236)

DVS2 explicitly lists git sources for replacement:

```m4
CARGO_SOURCE_REPLACE="$(cat <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/CGMossa/miniextendr"]
git = "https://github.com/CGMossa/miniextendr"
replace-with = "vendored-sources"

[source."git+https://github.com/A2-ai/dvs2"]
git = "https://github.com/A2-ai/dvs2"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$VENDOR_OUT_CARGO"
EOF
)"
```

**Comparison with miniextendr:**
- miniextendr uses `[patch.crates-io]` with path overrides in dev mode
- miniextendr uses `[source.crates-io]` replacement only in CRAN mode

**Analysis:**
DVS2's approach handles git dependencies more cleanly by explicitly mapping them to vendored sources. This is necessary when depending on git repos (like miniextendr itself).

**Recommendation:** If miniextendr ever has downstream projects depending on it via git, they should follow this pattern.

### 5. Package Name Variants (configure.ac:181-190)

DVS2 generates multiple name variants:

```m4
pkg_rs="$(echo "$PACKAGE_TARNAME" | $SED 's/-/_/g')"
AC_SUBST([PACKAGE_TARNAME_RS], [$pkg_rs])
pkg_rs_upper=`printf '%s' "$pkg_rs" | tr 'a-z' 'A-Z'`
AC_SUBST([PACKAGE_TARNAME_RS_UPPERCASE], [$pkg_rs_upper])
```

Both projects do this identically - good alignment.

### 6. Vendor Tarball Handling (configure.ac:319-441)

DVS2's vendor handling is cleaner:

1. Uses `_use_prevendored` flag to detect context
2. Handles R CMD check context explicitly (vendor empty but tarball exists)
3. Removes `--locked` from Makevars when unpacking tarball
4. Clears `.unpacked-from-tarball` marker

```m4
# Key insight: detect R CMD check running on extracted tarball
elif test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
  if test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
    echo "configure: vendor directory empty but vendor.tar.xz exists"
    echo "configure: falling back to pre-vendored sources (R CMD check context)"
    _use_prevendored=1
  fi
fi
```

**Recommendation:** The R CMD check context detection is valuable. Miniextendr should adopt this pattern.

## Files Changed

| DVS2 File | Purpose |
|-----------|---------|
| `configure.ac` | Main autoconf template |
| `Makevars.in` | Make build rules |
| `Cargo.toml.in` | Cargo manifest template |
| `document.rs.in` | R wrapper generator |
| `entrypoint.c.in` | R DLL entrypoint |
| `mx_abi.c.in` | Trait ABI implementation |
| `cargo-config.toml.in` | Cargo source replacement |
| `.gitignore` | Comprehensive ignore patterns |

## Action Items

1. **HIGH**: Re-evaluate `--locked` flag logic (DEV=lock, CRAN=no-lock)
2. **MEDIUM**: Adopt automatic feature→CPPFLAGS generation
3. **MEDIUM**: Consider sed-based staticlib name extraction
4. **LOW**: Add R CMD check context detection for vendor handling
