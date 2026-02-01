AC_INIT([miniextendr], [1.0])

dnl Use tools/ directory for auxiliary build files (config.guess, config.sub)
dnl This avoids the CRAN NOTE about non-standard top-level files
AC_CONFIG_AUX_DIR([tools])

dnl Verify we're running from the R package directory (not the monorepo root)
dnl This prevents accidental creation of src/, config.log, etc. in the wrong place
if test ! -f "DESCRIPTION" || test ! -f "NAMESPACE"; then
  AC_MSG_ERROR([configure must be run from the rpkg directory (where DESCRIPTION exists).

  If you're in the monorepo root, run:
    cd rpkg && ./configure
  Or use:
    just configure])
fi

dnl Canonicalize build/host triples for cross-compilation support
AC_CANONICAL_HOST

dnl Ensure R_HOME is set and valid
if test -z "$R_HOME"; then
  R_HOME="$(R RHOME 2>/dev/null)"
  if test $? -ne 0 || test -z "$R_HOME"; then
    AC_MSG_ERROR([R not found. Please ensure R is installed and on PATH, or set R_HOME.])
  fi
fi
if test ! -d "$R_HOME"; then
  AC_MSG_ERROR([R_HOME directory does not exist: $R_HOME])
fi

dnl NOT_CRAN handling:
dnl - CRAN builds: NOT_CRAN is unset or empty → default to false (offline mode)
dnl - Dev builds: set NOT_CRAN=true in environment to enable network/vendoring
dnl Normalize to "true" or "false" for consistent checks
: ${NOT_CRAN:=false}
case "$NOT_CRAN" in
  true|TRUE|1) NOT_CRAN=true ;;
  *)           NOT_CRAN=false ;;
esac
export NOT_CRAN

dnl Set cargo offline flag based on NOT_CRAN
dnl - NOT_CRAN=true (dev): no --offline, use --locked for reproducibility
dnl - NOT_CRAN=false (CRAN): use --offline, omit --locked (checksums cleared)
if test "$NOT_CRAN" = "true"; then
  CARGO_OFFLINE_FLAG=""
  dnl Dev mode uses --locked for reproducible builds matching committed lockfile
  CARGO_LOCKED_FLAG="--locked"
else
  CARGO_OFFLINE_FLAG="--offline"
  dnl CRAN mode omits --locked because checksums are cleared from vendor tarball
  CARGO_LOCKED_FLAG=""
fi
AC_SUBST([CARGO_OFFLINE_FLAG])
AC_SUBST([CARGO_LOCKED_FLAG])
AC_SUBST([NOT_CRAN])

dnl Set cargo features flag
dnl
dnl rpkg is a demo package that demonstrates ALL miniextendr features.
dnl Enable all optional features by default (both dev and CRAN modes).
dnl Advanced users can override via MINIEXTENDR_FEATURES environment variable.
dnl
dnl Note: 'nonapi' feature causes R CMD check warnings/errors (non-API calls).
dnl Only enable when you specifically need those features and understand the risks.
AC_ARG_VAR([MINIEXTENDR_FEATURES], [Comma-separated cargo features to enable (e.g., "nonapi,rayon")])

dnl Enable all features by default for this demo package
dnl Users can override by setting MINIEXTENDR_FEATURES explicitly (even to empty string)
if test -z "${MINIEXTENDR_FEATURES+x}"; then
  dnl MINIEXTENDR_FEATURES not set - enable all optional features
  MINIEXTENDR_FEATURES="rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,vctrs"
fi

if test -n "$MINIEXTENDR_FEATURES"; then
  CARGO_FEATURES_FLAG="--features=$MINIEXTENDR_FEATURES"
  # Auto-generate C preprocessor flags from cargo features
  CARGO_FEATURE_CPPFLAGS=""
  IFS=','
  for _feature in $MINIEXTENDR_FEATURES; do
    _feature_upper=$(echo "$_feature" | tr 'a-z-' 'A-Z_')
    CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_$_feature_upper"
  done
  unset IFS
else
  CARGO_FEATURES_FLAG=""
  CARGO_FEATURE_CPPFLAGS=""
fi
AC_SUBST([CARGO_FEATURES_FLAG])
AC_SUBST([CARGO_FEATURE_CPPFLAGS])

dnl rust-analyzer configuration: For IDE support, configure rust-analyzer to use
dnl the standalone workspace, not the vendored rpkg workspace:
dnl   "rust-analyzer.linkedProjects": ["/path/to/miniextendr/Cargo.toml"]
dnl This allows rust-analyzer to see the full workspace during development.


dnl ---- canonical paths ----
abs_top_srcdir="$(cd "$srcdir" && pwd)"
AC_SUBST([ABS_TOP_SRCDIR], ["$abs_top_srcdir"])
abs_rpkg_src="$abs_top_srcdir/src"
AC_SUBST([ABS_RPKG_SRCDIR], ["$abs_rpkg_src"])
RUST_SRC_DIR="$abs_rpkg_src/rust"

dnl Canonical path two levels above src (rpkg/src → miniextendr)
root_miniextendr_repo="$(cd "$abs_rpkg_src/../.." && pwd)"
AC_SUBST([ROOT_MINIEXTENDR_REPO], ["$root_miniextendr_repo"])

dnl Miniextendr crate source paths (in monorepo)
MINIEXTENDR_API_SRC="$root_miniextendr_repo/miniextendr-api"
MINIEXTENDR_MACROS_SRC="$root_miniextendr_repo/miniextendr-macros"
MINIEXTENDR_LINT_SRC="$root_miniextendr_repo/miniextendr-lint"
MINIEXTENDR_ENGINE_SRC="$root_miniextendr_repo/miniextendr-engine"

dnl ---- tool discovery ----
AC_PATH_TOOL([CARGO],[cargo],[no])
AC_PATH_TOOL([RUSTC],[rustc],[no])
AC_PATH_PROG([RSYNC],[rsync],[no])
AC_PATH_PROG([SED],[sed],[no])
AC_PATH_PROG([CYGPATH],[cygpath],[no])
AS_IF([test "x$CARGO" = "xno"], [AC_MSG_ERROR([`cargo` not found])])
AS_IF([test "x$RUSTC" = "xno"], [AC_MSG_ERROR([`rustc` not found])])
dnl rsync is only required when syncing from monorepo sources
AS_IF([test "x$SED" = "xno"], [AC_MSG_ERROR([`sed` not found])])
AC_SUBST([CARGO])
AC_SUBST([RUSTC])
AC_SUBST([RSYNC])
AC_SUBST([SED])
AC_SUBST([CYGPATH])

dnl Use native paths for Cargo on Windows/MSYS to avoid path mangling.
dnl MSYS2 paths like /d/a/foo become D:/d/a/foo when interpreted by Cargo,
dnl so we convert them to proper Windows paths (D:/a/foo) with cygpath -m.
root_miniextendr_repo_cargo="$root_miniextendr_repo"
abs_rpkg_src_cargo="$abs_rpkg_src"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test "x$CYGPATH" != "xno"; then
      root_miniextendr_repo_cargo="$($CYGPATH -m "$root_miniextendr_repo")"
      abs_rpkg_src_cargo="$($CYGPATH -m "$abs_rpkg_src")"
    fi
    ;;
esac
AC_SUBST([ROOT_MINIEXTENDR_REPO_CARGO], ["$root_miniextendr_repo_cargo"])
AC_SUBST([ABS_RPKG_SRC_CARGO], ["$abs_rpkg_src_cargo"])

dnl Report rustc version (required by CRAN for Rust packages)
RUSTC_VERSION="$($RUSTC --version)"
AC_MSG_NOTICE([using $RUSTC_VERSION])

dnl Check minimum rustc version (edition 2024 requires rustc 1.85+)
dnl Extract version number: "rustc 1.85.0 (..." -> "1.85.0"
RUSTC_VERSION_NUM=`echo "$RUSTC_VERSION" | "$SED" 's/rustc \(@<:@0-9@:>@*\.@<:@0-9@:>@*\.@<:@0-9@:>@*\).*/\1/'`
RUSTC_MAJOR=`echo "$RUSTC_VERSION_NUM" | cut -d. -f1`
RUSTC_MINOR=`echo "$RUSTC_VERSION_NUM" | cut -d. -f2`
MIN_RUSTC_MAJOR=1
MIN_RUSTC_MINOR=85
if test "$RUSTC_MAJOR" -lt "$MIN_RUSTC_MAJOR" || \
   (test "$RUSTC_MAJOR" -eq "$MIN_RUSTC_MAJOR" && test "$RUSTC_MINOR" -lt "$MIN_RUSTC_MINOR"); then
  AC_MSG_ERROR([rustc $MIN_RUSTC_MAJOR.$MIN_RUSTC_MINOR+ required for Rust edition 2024, found $RUSTC_VERSION_NUM])
fi

dnl Allow explicit toolchain (e.g. +nightly); default empty (use default toolchain)
AC_ARG_VAR([RUST_TOOLCHAIN],[Rust toolchain selector like '+stable' or '+nightly'])
: ${RUST_TOOLCHAIN=""}
AC_SUBST([RUST_TOOLCHAIN])

dnl Convenience: cargo command that includes optional toolchain selector
CARGO_CMD="$CARGO $RUST_TOOLCHAIN"
AC_SUBST([CARGO_CMD])

dnl ---- Cargo configuration ----
dnl IMPORTANT: target directory MUST be outside src/ to avoid pkgbuild scanning
dnl generated .rs files in target/build/.../out/ during needs_compile() checks.
dnl pkgbuild's sources() function recursively finds .rs files in src/, and
dnl if any generated file has a newer mtime than the DLL, it triggers rebuild.
AC_ARG_VAR([CARGO_TARGET_DIR], [Cargo target directory (default: rust-target, outside src/)])
: ${CARGO_TARGET_DIR="$abs_top_srcdir/rust-target"}
AC_SUBST([CARGO_TARGET_DIR])

dnl Cargo-specific paths (Windows-native on MSYS2, same as shell paths otherwise)
dnl These are used in .cargo/config.toml which Cargo reads directly
CARGO_TARGET_DIR_CARGO="$(cd "$abs_top_srcdir" && pwd)/rust-target"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test "x$CYGPATH" != "xno"; then
      CARGO_TARGET_DIR_CARGO="$($CYGPATH -m "$abs_top_srcdir")/rust-target"
    fi
    ;;
esac
AC_SUBST([CARGO_TARGET_DIR_CARGO])

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (debug or release)])
: ${CARGO_PROFILE="release"}
AC_SUBST([CARGO_PROFILE])

dnl CARGO_STATICLIB_NAME will be extracted via cargo pkgid in AC_CONFIG_COMMANDS
dnl Use placeholder for now - will be substituted after Cargo.toml is available
CARGO_STATICLIB_NAME="__CARGO_STATICLIB_NAME_PLACEHOLDER__"
AC_SUBST([CARGO_STATICLIB_NAME])

AC_ARG_VAR([ENV_RUSTFLAGS], [Rust compiler flags passed as RUSTFLAGS])
: ${ENV_RUSTFLAGS="$RUSTFLAGS"}
AC_SUBST([ENV_RUSTFLAGS])

AC_ARG_VAR([CARGO_BUILD_TARGET], [Cargo build target triple (e.g. aarch64-apple-darwin)])
if test -z "$CARGO_BUILD_TARGET"; then
  if test -n "$host" && test -n "$build" && test "$host" != "$build"; then
    CARGO_BUILD_TARGET="$host"
    AC_MSG_NOTICE([CARGO_BUILD_TARGET not set; using host triple $host])
  else
    CARGO_BUILD_TARGET=""
  fi
fi
AC_SUBST([CARGO_BUILD_TARGET])

dnl Compute Cargo libdir path:
dnl - If CARGO_BUILD_TARGET is set, use target/<that>/<profile>
dnl - Otherwise, use target/<profile>
if test -n "$CARGO_BUILD_TARGET"; then
  CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_BUILD_TARGET/$CARGO_PROFILE"
else
  CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_PROFILE"
fi
AC_SUBST([CARGO_LIBDIR])

dnl ---- user feedback ----
AC_MSG_NOTICE([R_HOME                = $R_HOME])
AC_MSG_NOTICE([NOT_CRAN              = $NOT_CRAN])
AC_MSG_NOTICE([CARGO_TARGET_DIR      = $CARGO_TARGET_DIR])
AC_MSG_NOTICE([CARGO_PROFILE         = $CARGO_PROFILE])
AC_MSG_NOTICE([CARGO_STATICLIB_NAME  = $CARGO_STATICLIB_NAME])
AC_MSG_NOTICE([ENV_RUSTFLAGS         = $ENV_RUSTFLAGS])
AC_MSG_NOTICE([CARGO_LIBDIR          = $CARGO_LIBDIR])
AS_IF([test -n "$RUST_TOOLCHAIN"],
      [AC_MSG_NOTICE([RUST_TOOLCHAIN        = $RUST_TOOLCHAIN])])
AS_IF([test -n "$CARGO_BUILD_TARGET"],
      [AC_MSG_NOTICE([CARGO_BUILD_TARGET    = $CARGO_BUILD_TARGET])])
AC_MSG_NOTICE([ROOT_MINIEXTENDR_REPO = $ROOT_MINIEXTENDR_REPO])
AS_IF([test -n "$MINIEXTENDR_FEATURES"],
      [AC_MSG_NOTICE([MINIEXTENDR_FEATURES  = $MINIEXTENDR_FEATURES])])

dnl ---- package name → Rust-safe variants ----
pkg_rs="$(echo "$PACKAGE_TARNAME" | $SED 's/-/_/g')"
AC_SUBST([PACKAGE_TARNAME_RS], [$pkg_rs])
pkg_rs_upper=`printf '%s' "$pkg_rs" | tr 'a-z' 'A-Z'`
AC_SUBST([PACKAGE_TARNAME_RS_UPPERCASE], [$pkg_rs_upper])

dnl Generate files from .in templates (templates must be committed)
AC_CONFIG_FILES([src/rust/document.rs])
AC_CONFIG_SRCDIR([src/entrypoint.c.in])  dnl safety check

dnl Note: R CMD build already tracks .rs files for rebuild detection via pkgbuild,
dnl so we don't need to enumerate RUST_SOURCES for Make dependencies.

dnl ---- optional cargo deps: include if present ----
OPTIONAL_CARGO_DEPS=
for f in \
  "$RUST_SRC_DIR/build.rs" \
  "$RUST_SRC_DIR/.cargo/config.toml" \
  "$RUST_SRC_DIR/rust-toolchain.toml" \
  "$RUST_SRC_DIR/rust-toolchain"
do
  test -f "$f" && OPTIONAL_CARGO_DEPS="$OPTIONAL_CARGO_DEPS $f"
done
AC_SUBST([OPTIONAL_CARGO_DEPS])

dnl ---- vendoring (cached, in src/vendor) ----
VENDOR_OUT="$abs_rpkg_src/vendor"
AC_SUBST([VENDOR_OUT])
dnl Cargo-specific version with Windows-native paths (for .cargo/config.toml)
VENDOR_OUT_CARGO="$abs_rpkg_src_cargo/vendor"
AC_SUBST([VENDOR_OUT_CARGO])

dnl Force refresh if you want
AC_ARG_VAR([FORCE_VENDOR],[Set to 1 to force `cargo vendor` refresh in dev mode])
: ${FORCE_VENDOR="0"}
AC_SUBST([FORCE_VENDOR])

dnl Include extra manifests in `cargo vendor` (pulls in bench/tooling deps).
dnl Default is minimal vendoring based on rpkg/src/rust/Cargo.toml only.
AC_ARG_VAR([VENDOR_SYNC_EXTRA],[Set to 1 to include extra `cargo vendor --sync` manifests (workspace/bench)])
: ${VENDOR_SYNC_EXTRA="0"}
AC_SUBST([VENDOR_SYNC_EXTRA])

dnl Syncing miniextendr-* crates and patching their Cargo.toml is done in an
dnl AC_CONFIG_COMMANDS step (after Cargo.toml generation and optional cargo vendor).

dnl Source replacement config: only use vendored sources on CRAN (offline builds)
dnl Dev mode uses local path overrides for miniextendr-*
AS_IF([test "$NOT_CRAN" = "true"], [
  dnl Dev mode: use local path overrides for miniextendr crates
  CARGO_SOURCE_REPLACE="$(cat <<EOF
@<:@patch.crates-io@:>@
miniextendr-api = { path = "$ROOT_MINIEXTENDR_REPO_CARGO/miniextendr-api" }
miniextendr-macros = { path = "$ROOT_MINIEXTENDR_REPO_CARGO/miniextendr-macros" }
miniextendr-lint = { path = "$ROOT_MINIEXTENDR_REPO_CARGO/miniextendr-lint" }
miniextendr-engine = { path = "$ROOT_MINIEXTENDR_REPO_CARGO/miniextendr-engine" }
miniextendr-bench = { path = "$ROOT_MINIEXTENDR_REPO_CARGO/miniextendr-bench" }
EOF
)"
], [
  dnl CRAN/offline mode: use vendored sources only
  CARGO_SOURCE_REPLACE="$(cat <<EOF
@<:@source.crates-io@:>@
replace-with = "vendored-sources"

@<:@source.vendored-sources@:>@
directory = "$VENDOR_OUT_CARGO"
EOF
)"
])

dnl Sanity check CARGO_SOURCE_REPLACE before AC_SUBST
if test -z "$CARGO_SOURCE_REPLACE"; then
  AC_MSG_ERROR([CARGO_SOURCE_REPLACE is empty; Cargo config templates will be missing source/patch directives])
fi
case "$CARGO_SOURCE_REPLACE" in
  *patch.crates-io*|*source.crates-io*) ;;
  *)
    AC_MSG_ERROR([CARGO_SOURCE_REPLACE does not contain expected [patch.crates-io] or [source.crates-io] header; check bracket escaping and quoting])
    ;;
esac
AC_SUBST([CARGO_SOURCE_REPLACE])

dnl ---- output files ----
dnl Create .cargo directory (it's a hidden dir not included in tarball)
AS_MKDIR_P([.cargo])
AS_MKDIR_P([src/rust/.cargo])


AC_CONFIG_FILES([
  .cargo/config.toml:src/rust/cargo-config.toml.in
  src/rust/.cargo/config.toml:src/rust/cargo-config.toml.in
  src/Makevars:src/Makevars.in
  src/entrypoint.c:src/entrypoint.c.in
  src/mx_abi.c:src/mx_abi.c.in
])

dnl 2a) Ensure the monorepo root has a cargo config (needed for workspace resolution)
AC_CONFIG_COMMANDS([sync-root-cargo-config],
[
  ROOT_CFG_SRC=".cargo/config.toml"
  ROOT_CFG_DIR="$ROOT_MINIEXTENDR_REPO/.cargo"
  ROOT_CFG_DST="$ROOT_CFG_DIR/config.toml"

  if test -f "$ROOT_CFG_SRC" && test "$NOT_CRAN" = "true" && \
     test -d "$ROOT_MINIEXTENDR_REPO/miniextendr-api"; then
    mkdir -p "$ROOT_CFG_DIR"
    _cfg_diff=0
    _cfg_exists=0
    if test -f "$ROOT_CFG_DST"; then
      _cfg_exists=1
      if command -v cmp >/dev/null 2>&1; then
        cmp -s "$ROOT_CFG_SRC" "$ROOT_CFG_DST" || _cfg_diff=1
      elif command -v diff >/dev/null 2>&1; then
        diff -q "$ROOT_CFG_SRC" "$ROOT_CFG_DST" >/dev/null 2>&1 || _cfg_diff=1
      else
        dnl No comparison tool available - assume different to be safe
        _cfg_diff=1
      fi
    fi

    if test "$_cfg_diff" -eq 1; then
      echo "======================================================================" >&2
      echo "configure: WARNING: Monorepo root cargo config differs from rpkg" >&2
      echo "configure:   Existing: $ROOT_CFG_DST" >&2
      echo "configure:   New:      $ROOT_CFG_SRC" >&2
      echo "configure:" >&2
      echo "configure:   This will OVERWRITE your existing cargo config." >&2
      echo "configure:   If you have custom settings, back them up first." >&2
      echo "configure:" >&2
      echo "configure:   To prevent this, remove the monorepo from the parent" >&2
      echo "configure:   directory or run configure from outside the monorepo." >&2
      echo "======================================================================" >&2
      dnl Give user 3 seconds to read the warning
      sleep 3
    fi

    cp "$ROOT_CFG_SRC" "$ROOT_CFG_DST"

    if test "$_cfg_exists" -eq 0; then
      echo "configure: created $ROOT_CFG_DST" >&2
    elif test "$_cfg_diff" -eq 1; then
      echo "configure: overwrote $ROOT_CFG_DST" >&2
    fi
  fi
],
[ROOT_MINIEXTENDR_REPO="$root_miniextendr_repo" NOT_CRAN="$NOT_CRAN"])

dnl ---- AC_CONFIG_COMMANDS run after AC_CONFIG_FILES generates files ----
dnl Order matters: lockfile-compat, then cargo-vendor, then patch-staticlib-name

dnl 1) Ensure Cargo.lock is compatible with the installed cargo version.
dnl    Older cargo releases (e.g. 1.75) cannot read lockfile version 4.
AC_CONFIG_COMMANDS([cargo-lockfile-compat],
[
  LOCKFILE_PATH="src/rust/Cargo.lock"
  if test -f "$LOCKFILE_PATH"; then
    LOCKFILE_VERSION="$("$SED" -n 's/^version = //p' "$LOCKFILE_PATH" | head -n1)"
  else
    LOCKFILE_VERSION=""
  fi

  CARGO_VERSION="$( $CARGO_CMD --version | "$SED" 's/cargo \(@<:@0-9@:>@*\.@<:@0-9@:>@*\.@<:@0-9@:>@*\).*/\1/' )"
  CARGO_MAJOR=`echo "$CARGO_VERSION" | cut -d. -f1`
  CARGO_MINOR=`echo "$CARGO_VERSION" | cut -d. -f2`

  if test -n "$LOCKFILE_VERSION" && test "$LOCKFILE_VERSION" -ge 4; then
    if test "$CARGO_MAJOR" -lt 1 || \
       (test "$CARGO_MAJOR" -eq 1 && test "$CARGO_MINOR" -lt 78); then
      dnl In CRAN/offline mode, ensure vendor sources exist before regenerating lockfile.
      if test "$NOT_CRAN" != "true"; then
        if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
          if test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
            echo "configure: CRAN build - unpacking inst/vendor.tar.xz (for lockfile regeneration)" >&2
            tar -xJf "$abs_rpkg_dir/inst/vendor.tar.xz" -C "$abs_rpkg_src"
            if test $? -ne 0; then
              echo "configure: error: failed to unpack vendor.tar.xz" >&2
              exit 1
            fi
            dnl Strip checksums from Cargo.lock (vendored crates have empty checksums)
            if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
              $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
            fi
          else
            echo "configure: error: CRAN/offline build requires vendored sources to regenerate Cargo.lock" >&2
            echo "configure:        Expected inst/vendor.tar.xz or a populated vendor/ directory." >&2
            exit 1
          fi
        fi
      fi
      echo "configure: WARNING: cargo $CARGO_VERSION cannot read Cargo.lock v$LOCKFILE_VERSION; regenerating lockfile" >&2
      rm -f "$LOCKFILE_PATH"
      $CARGO_CMD generate-lockfile --manifest-path src/rust/Cargo.toml $CARGO_OFFLINE_FLAG
      _gen_status=$?
      if test $_gen_status -ne 0; then
        echo "configure: error: cargo generate-lockfile failed with status $_gen_status" >&2
        exit 1
      fi
    fi
  fi
],
[CARGO_CMD="$CARGO_CMD" CARGO_OFFLINE_FLAG="$CARGO_OFFLINE_FLAG" SED="$SED" NOT_CRAN="$NOT_CRAN" VENDOR_OUT="$VENDOR_OUT" abs_rpkg_dir="$abs_top_srcdir" abs_rpkg_src="$abs_rpkg_src"])

dnl 2) Run cargo vendor (dev mode only, after Cargo.toml is generated)
AC_CONFIG_COMMANDS([cargo-vendor],
[
  dnl Only vendor in dev mode (NOT_CRAN=true).
  dnl In CRAN/offline mode we prefer vendor/ dir OR vendor.tar.xz, but fall back to network if needed.
  if test "$NOT_CRAN" != "true"; then
    dnl Check if vendor directory exists and is populated
    if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      dnl Vendor missing - check for compressed vendor.tar.xz in inst/ (created by configure)
      if test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
        echo "configure: CRAN build - unpacking inst/vendor.tar.xz"
        tar -xJf "$abs_rpkg_dir/inst/vendor.tar.xz" -C "$abs_rpkg_src"
        if test $? -ne 0; then
          echo "configure: error: failed to unpack vendor.tar.xz" >&2
          exit 1
        fi
        dnl Strip checksums from Cargo.lock (vendored crates have empty checksums)
        if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
          $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
        fi
      else
        dnl No vendor available - fall back to network (e.g., GitHub installs)
        echo "configure: no cached vendor found, will proceed with network access" >&2
        echo "configure:   (This is normal for GitHub installs via remotes::install_github())" >&2
        dnl Continue to dev mode vendoring logic below
        NOT_CRAN=true
      fi
    fi
    if test "$NOT_CRAN" != "true"; then
      echo "configure: CRAN build - skipping cargo vendor (using pre-vendored tarball)"
    fi
  else
    MANIFEST="src/rust/Cargo.toml"
    LOCKFILE="src/rust/Cargo.lock"
    STAMP="$VENDOR_OUT/.vendor.lock.cksum"

    dnl Generate Cargo.lock if missing (required for fingerprinting)
    if test ! -f "$LOCKFILE"; then
      echo "configure: generating Cargo.lock"
      $CARGO_CMD generate-lockfile --manifest-path "$MANIFEST"
      if test $? -ne 0; then
        echo "configure: error: cargo generate-lockfile failed" >&2
        exit 1
      fi
    fi

    dnl POSIX fingerprint: checksum+size of Cargo.lock contents
    _new="$(cksum < "$LOCKFILE" | awk '{print [$]1":"[$]2}')"
    _old="$(cat "$STAMP" 2>/dev/null || true)"

    _need=0
    if test "$FORCE_VENDOR" = "1"; then
      _need=1
    elif test ! -f "$STAMP"; then
      _need=1
    elif test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      _need=1
    elif test "x$_new" != "x$_old"; then
      _need=1
    fi

	    if test $_need -eq 1; then
	      dnl Check if monorepo Cargo.toml exists (won't exist during R CMD check)
	      if test -f "$ROOT_MINIEXTENDR_REPO/Cargo.toml"; then
	        echo "configure: vendoring dependencies using cargo package approach"
	        mkdir -p "$VENDOR_OUT"

	        dnl Create staging directory for .crate extraction
	        STAGING_DIR="$abs_rpkg_src/.vendor-tarball-staging"
	        mkdir -p "$STAGING_DIR"

	        dnl Package workspace crates using cargo package
	        echo "configure: packaging workspace crates..."
	        for _crate_dir in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
	          if test -d "$ROOT_MINIEXTENDR_REPO/$_crate_dir"; then
	            echo "configure:   packaging $_crate_dir"
	            $CARGO_CMD package --manifest-path "$ROOT_MINIEXTENDR_REPO/$_crate_dir/Cargo.toml" \
	              --target-dir "$STAGING_DIR/target" --allow-dirty --no-verify 2>&1 | \
	              grep -v "warning: manifest has no" || true
	            if test ${PIPESTATUS[[0]]} -ne 0; then
	              echo "configure: error: cargo package failed for $_crate_dir" >&2
	              exit 1
	            fi
	          fi
	        done

	        dnl Extract .crate files to vendor directory
	        echo "configure: extracting packaged crates to vendor..."
	        for _crate_file in "$STAGING_DIR/target/package/"*.crate; do
	          if test -f "$_crate_file"; then
	            _basename=$(basename "$_crate_file" .crate)
	            echo "configure:   extracting $_basename"
	            mkdir -p "$VENDOR_OUT/$_basename"
	            tar -xzf "$_crate_file" -C "$VENDOR_OUT/$_basename" --strip-components=1
	          fi
	        done

	        dnl Run cargo vendor for external dependencies only
	        echo "configure: running cargo vendor for external dependencies"
	        $CARGO_CMD vendor --manifest-path "$MANIFEST" "$VENDOR_OUT"
	        if test $? -ne 0; then
	          echo "configure: error: cargo vendor failed" >&2
	          exit 1
	        fi

	        dnl Clean up staging directory
	        rm -rf "$STAGING_DIR"

	        dnl Strip checksums from Cargo.lock (vendored crates have empty checksums)
	        if test -f "$LOCKFILE"; then
	          $SED -i.bak '/^checksum = /d' "$LOCKFILE" && rm -f "$LOCKFILE.bak"
	        fi
	        echo "$_new" > "$STAMP"
      else
        dnl Monorepo not available (e.g., R CMD check from tarball with NOT_CRAN=true)
        if test -d "$VENDOR_OUT" && test -n "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
          dnl Vendor exists - check if we need to update it for new dependencies
          if test "$FORCE_VENDOR" = "1"; then
            echo "configure: FORCE_VENDOR=1, running cargo vendor to fetch new dependencies from crates.io"
            dnl Run cargo vendor without monorepo to fetch from crates.io
            $CARGO_CMD vendor --manifest-path "$MANIFEST" "$VENDOR_OUT"
            if test $? -eq 0; then
              dnl Strip checksums from Cargo.lock (vendored crates have empty checksums)
              if test -f "$LOCKFILE"; then
                $SED -i.bak '/^checksum = /d' "$LOCKFILE" && rm -f "$LOCKFILE.bak"
              fi
              echo "$_new" > "$STAMP"
              echo "configure: cargo vendor complete (updated with new dependencies)"
            else
              echo "configure: warning: cargo vendor failed, continuing with existing vendor" >&2
            fi
          else
            echo "configure: skipping cargo vendor (monorepo not found, using existing vendor)"
          fi
        elif test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
          echo "configure: monorepo not found, unpacking inst/vendor.tar.xz"
          tar -xJf "$abs_rpkg_dir/inst/vendor.tar.xz" -C "$abs_rpkg_src"
          dnl Strip checksums from Cargo.lock (vendored crates have empty checksums)
          if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
            $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
          fi
        else
          dnl Fallback: use crates.io (dev mode only)
          echo "======================================================================" >&2
          echo "configure: WARNING: Falling back to crates.io" >&2
          echo "configure:   - Monorepo not found" >&2
          echo "configure:   - No vendored dependencies available" >&2
          echo "configure:" >&2
          echo "configure:   Will fetch miniextendr crates from crates.io." >&2
          echo "configure:   This requires network access and may use older versions." >&2
          echo "configure:" >&2
          echo "configure:   For development, consider:" >&2
          echo "configure:   - Clone the full miniextendr monorepo" >&2
          echo "configure:   - Or use a release tarball with vendored dependencies" >&2
          echo "======================================================================" >&2
          sleep 2

          dnl Clear any stale lockfile that might reference vendored or path sources
          if test -f "$LOCKFILE"; then
            echo "configure: removing stale Cargo.lock (may contain vendored/path references)" >&2
            rm -f "$LOCKFILE"
          fi

          dnl Regenerate Cargo.lock from crates.io
          echo "configure: regenerating Cargo.lock from crates.io" >&2
          $CARGO_CMD generate-lockfile --manifest-path "$MANIFEST"
          if test $? -ne 0; then
            echo "configure: error: cargo generate-lockfile failed (network required)" >&2
            echo "configure:        Ensure you have internet access or use a vendored build." >&2
            exit 1
          fi

          dnl Clear CARGO_LOCKED_FLAG since we just regenerated the lockfile
          CARGO_LOCKED_FLAG=""

          dnl Rewrite cargo config to remove vendor source
          CARGO_CONFIG="src/rust/.cargo/config.toml"
          if test -f "$CARGO_CONFIG"; then
            echo "configure: updating cargo config to use crates.io instead of vendored sources" >&2
            $SED -i.bak '/^\[source\.vendored-sources\]/,/^$/d' "$CARGO_CONFIG" && rm -f "$CARGO_CONFIG.bak"
            $SED -i.bak '/^replace-with = "vendored-sources"$/d' "$CARGO_CONFIG" && rm -f "$CARGO_CONFIG.bak"
          fi

          echo "configure: crates.io fallback configured successfully" >&2
        fi
      fi
    else
      echo "configure: skipping cargo vendor (Cargo.lock unchanged)"
    fi
  fi
],
[CARGO_CMD="$CARGO_CMD" NOT_CRAN="$NOT_CRAN" FORCE_VENDOR="$FORCE_VENDOR" VENDOR_SYNC_EXTRA="$VENDOR_SYNC_EXTRA" VENDOR_OUT="$VENDOR_OUT" ROOT_MINIEXTENDR_REPO="$root_miniextendr_repo" ROOT_MINIEXTENDR_REPO_CARGO="$root_miniextendr_repo_cargo" abs_rpkg_src="$abs_rpkg_src" abs_rpkg_dir="$abs_top_srcdir" SED="$SED" CARGO_LOCKED_FLAG="$CARGO_LOCKED_FLAG"])

dnl 2b) Sync miniextendr crates from monorepo to vendor and patch Cargo.toml

dnl This must run after Cargo.toml generation (AC_CONFIG_FILES) and after the
dnl optional cargo-vendor step, so the vendor directory is populated first.
AC_CONFIG_COMMANDS([sync-miniextendr-crates],
[
  dnl Sync miniextendr crates from monorepo to vendor (if sources exist)
  echo "configure: checking for miniextendr source crates" >&2

  if test -d "$MINIEXTENDR_API_SRC" && test -d "$MINIEXTENDR_MACROS_SRC" && \
     test -d "$MINIEXTENDR_LINT_SRC" && test -d "$MINIEXTENDR_ENGINE_SRC"; then

    dnl rsync is required when syncing from monorepo
    if test "x$RSYNC" = "xno"; then
      echo "configure: error: rsync not found (required for syncing miniextendr crates from monorepo)" >&2
      exit 1
    fi

    echo "configure: found miniextendr crates; syncing to $VENDOR_OUT" >&2

    dnl Extract workspace.package values from root Cargo.toml
    ROOT_CARGO_TOML="$ROOT_MINIEXTENDR_REPO/Cargo.toml"

    dnl Extract simple string values from [workspace.package]
    WS_VERSION="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^version = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_EDITION="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^edition = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_LICENSE="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^license = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_REPOSITORY="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^repository = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_HOMEPAGE="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^homepage = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1)"

    dnl For arrays, extract the whole value (keep brackets)
    WS_KEYWORDS="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^keywords = //p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_CATEGORIES="$($SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^categories = //p; }' "$ROOT_CARGO_TOML" | head -1)"

    dnl Extract workspace.dependencies
    WS_DEP_PROC_MACRO2="$($SED -n '/\@<:@workspace\.dependencies\@:>@/,/^\@<:@/{ s/^proc-macro2 = //p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_DEP_QUOTE="$($SED -n '/\@<:@workspace\.dependencies\@:>@/,/^\@<:@/{ s/^quote = //p; }' "$ROOT_CARGO_TOML" | head -1)"
    WS_DEP_SYN="$($SED -n '/\@<:@workspace\.dependencies\@:>@/,/^\@<:@/{ s/^syn = //p; }' "$ROOT_CARGO_TOML" | head -1)"

    echo "configure: workspace.package: version=$WS_VERSION edition=$WS_EDITION" >&2

    dnl Create vendor directories
    mkdir -p "$VENDOR_OUT/miniextendr-api" \
             "$VENDOR_OUT/miniextendr-macros" \
             "$VENDOR_OUT/miniextendr-lint" \
             "$VENDOR_OUT/miniextendr-engine"

    dnl Sync crates (excluding target, .git, Cargo.toml, and CRAN-unfriendly files)
    _rsync_exclude="--exclude=target --exclude=.git --exclude=Cargo.toml --exclude=.DS_Store --exclude=.gitignore --exclude=.github"

    $RSYNC -a --delete $_rsync_exclude \
      "$MINIEXTENDR_API_SRC/" "$VENDOR_OUT/miniextendr-api/"

    $RSYNC -a --delete $_rsync_exclude \
      "$MINIEXTENDR_MACROS_SRC/" "$VENDOR_OUT/miniextendr-macros/"

    $RSYNC -a --delete $_rsync_exclude \
      "$MINIEXTENDR_LINT_SRC/" "$VENDOR_OUT/miniextendr-lint/"

    $RSYNC -a --delete $_rsync_exclude \
      "$MINIEXTENDR_ENGINE_SRC/" "$VENDOR_OUT/miniextendr-engine/"

    dnl Escape special chars for `sed` replacement using | delimiter (handles URLs with /)
    dnl Escape & and | characters
    dnl Note: @S|@1 is m4 quadrigraph for $1
    escape_sed_repl () {
      printf '%s' "@S|@1" | $SED 's/@<:@&|@:>@/\\&/g'
    }
    WS_REPOSITORY_ESC="$(escape_sed_repl "$WS_REPOSITORY")"
    WS_HOMEPAGE_ESC="$(escape_sed_repl "$WS_HOMEPAGE")"
    WS_KEYWORDS_ESC="$(escape_sed_repl "$WS_KEYWORDS")"
    WS_CATEGORIES_ESC="$(escape_sed_repl "$WS_CATEGORIES")"
    WS_DEP_PROC_MACRO2_ESC="$(escape_sed_repl "$WS_DEP_PROC_MACRO2")"
    WS_DEP_QUOTE_ESC="$(escape_sed_repl "$WS_DEP_QUOTE")"
    WS_DEP_SYN_ESC="$(escape_sed_repl "$WS_DEP_SYN")"

    dnl If keywords/categories are empty, set to empty array to avoid TOML parse error
    test -z "$WS_KEYWORDS_ESC" && WS_KEYWORDS_ESC="@<:@@:>@"
    test -z "$WS_CATEGORIES_ESC" && WS_CATEGORIES_ESC="@<:@@:>@"

    dnl Patch Cargo.toml to remove workspace inheritance
    dnl Use | as delimiter to handle URLs with /
    $SED -e "s|edition\.workspace = true|edition = \"$WS_EDITION\"|" \
         -e "s|version\.workspace = true|version = \"$WS_VERSION\"|" \
         -e "s|license\.workspace = true|license = \"$WS_LICENSE\"|" \
         -e "s|repository\.workspace = true|repository = \"$WS_REPOSITORY_ESC\"|" \
         -e "s|homepage\.workspace = true|homepage = \"$WS_HOMEPAGE_ESC\"|" \
         -e "s|keywords\.workspace = true|keywords = $WS_KEYWORDS_ESC|" \
         -e "s|categories\.workspace = true|categories = $WS_CATEGORIES_ESC|" \
         -e "s|miniextendr-macros = { workspace = true }|miniextendr-macros = { version = \"$WS_VERSION\", path = \"../miniextendr-macros\" }|" \
         -e "s|miniextendr-engine = { workspace = true }|miniextendr-engine = { version = \"$WS_VERSION\", path = \"../miniextendr-engine\" }|" \
         "$MINIEXTENDR_API_SRC/Cargo.toml" > "$VENDOR_OUT/miniextendr-api/Cargo.toml"

    $SED -e "s|edition\.workspace = true|edition = \"$WS_EDITION\"|" \
         -e "s|version\.workspace = true|version = \"$WS_VERSION\"|" \
         -e "s|license\.workspace = true|license = \"$WS_LICENSE\"|" \
         -e "s|repository\.workspace = true|repository = \"$WS_REPOSITORY_ESC\"|" \
         -e "s|homepage\.workspace = true|homepage = \"$WS_HOMEPAGE_ESC\"|" \
         -e "s|keywords\.workspace = true|keywords = $WS_KEYWORDS_ESC|" \
         -e "s|categories\.workspace = true|categories = $WS_CATEGORIES_ESC|" \
         -e "s|proc-macro2 = { workspace = true }|proc-macro2 = $WS_DEP_PROC_MACRO2_ESC|" \
         -e "s|quote = { workspace = true }|quote = $WS_DEP_QUOTE_ESC|" \
         -e "s|syn = { workspace = true }|syn = $WS_DEP_SYN_ESC|" \
         -e "/miniextendr-api = { workspace = true }/d" \
         "$MINIEXTENDR_MACROS_SRC/Cargo.toml" > "$VENDOR_OUT/miniextendr-macros/Cargo.toml"

    $SED -e "s|edition\.workspace = true|edition = \"$WS_EDITION\"|" \
         -e "s|version\.workspace = true|version = \"$WS_VERSION\"|" \
         -e "s|license\.workspace = true|license = \"$WS_LICENSE\"|" \
         -e "s|repository\.workspace = true|repository = \"$WS_REPOSITORY_ESC\"|" \
         -e "s|homepage\.workspace = true|homepage = \"$WS_HOMEPAGE_ESC\"|" \
         -e "s|proc-macro2 = { workspace = true }|proc-macro2 = $WS_DEP_PROC_MACRO2_ESC|" \
         -e "s|quote = { workspace = true }|quote = $WS_DEP_QUOTE_ESC|" \
         -e "s|syn = { workspace = true }|syn = $WS_DEP_SYN_ESC|" \
         "$MINIEXTENDR_LINT_SRC/Cargo.toml" > "$VENDOR_OUT/miniextendr-lint/Cargo.toml"

    dnl miniextendr-engine: simple crate, just edition/version/license
    $SED -e "s|edition\.workspace = true|edition = \"$WS_EDITION\"|" \
         -e "s|version\.workspace = true|version = \"$WS_VERSION\"|" \
         -e "s|license\.workspace = true|license = \"$WS_LICENSE\"|" \
         -e "s|repository\.workspace = true|repository = \"$WS_REPOSITORY_ESC\"|" \
         -e "s|homepage\.workspace = true|homepage = \"$WS_HOMEPAGE_ESC\"|" \
         "$MINIEXTENDR_ENGINE_SRC/Cargo.toml" > "$VENDOR_OUT/miniextendr-engine/Cargo.toml"

    dnl Verify patching was complete - fail if any .workspace = true remains
    for _crate in miniextendr-api miniextendr-macros miniextendr-lint miniextendr-engine; do
      _cargo_toml="$VENDOR_OUT/$_crate/Cargo.toml"
      if grep -q '\.workspace = true' "$_cargo_toml" 2>/dev/null; then
        echo "configure: error: $_crate/Cargo.toml still contains '.workspace = true' after patching" >&2
        exit 1
      fi
      dnl Miniextendr crates are synced from monorepo, not cargo vendor, so they need checksums
      echo '{"files":{}}' > "$VENDOR_OUT/$_crate/.cargo-checksum.json"
    done

    dnl Remove files that cause R CMD check warnings (GNU extensions in Makefiles)
    dnl These are vendored crate build files not used by the R package
    rm -f "$VENDOR_OUT/r-efi/Makefile"

    echo "configure: miniextendr crates synced to $VENDOR_OUT" >&2

    dnl Strip checksums from Cargo.lock when using vendored sources
    dnl (miniextendr crates are synced/modified, so checksums don't match)
    if test -f "src/rust/Cargo.lock"; then
      $SED -i.bak '/^checksum = /d' src/rust/Cargo.lock && rm -f src/rust/Cargo.lock.bak
      echo "configure: stripped checksums from Cargo.lock (vendored sources)" >&2
    fi
  else
    echo "configure: miniextendr source crates not found; switching to vendored sources or crates.io" >&2

    _have_vendor=0
    if test -d "$VENDOR_OUT" && test -n "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      _have_vendor=1
    elif test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
      echo "configure: monorepo not found, unpacking inst/vendor.tar.xz" >&2
      tar -xJf "$abs_rpkg_dir/inst/vendor.tar.xz" -C "$abs_rpkg_src"
      if test $? -ne 0; then
        echo "configure: error: failed to unpack vendor.tar.xz" >&2
        exit 1
      fi
      dnl Strip checksums from Cargo.lock (vendored crates have empty checksums)
      if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
        $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
      fi
      _have_vendor=1
    fi

    if test $_have_vendor -eq 1; then
      dnl Force CARGO_TARGET_DIR to be local when source crates aren't found.
      dnl This prevents using a stale target directory from a previous build.
      dnl Use rust-target/ (outside src/) to avoid pkgbuild scanning generated .rs files.
      dnl Note: Use $abs_rpkg_dir (from init-commands), not $abs_top_srcdir
      CARGO_TARGET_DIR="$abs_rpkg_dir/rust-target"
      CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_PROFILE"
      echo "configure: forcing local CARGO_TARGET_DIR=$CARGO_TARGET_DIR" >&2

      dnl Update Makevars with the new paths (it was generated before we knew crates were missing)
      dnl Also clear CARGO_LOCKED_FLAG since we need to regenerate lockfile with vendored paths
      $SED -i.bak \
        -e "s|^CARGO_TARGET_DIR = .*|CARGO_TARGET_DIR = $CARGO_TARGET_DIR|" \
        -e "s|^CARGO_LIBDIR = .*|CARGO_LIBDIR = $CARGO_LIBDIR|" \
        -e "s|^CARGO_LOCKED_FLAG = .*|CARGO_LOCKED_FLAG = |" \
        src/Makevars && rm -f src/Makevars.bak
      echo "configure: updated src/Makevars with local target dir and cleared CARGO_LOCKED_FLAG" >&2
      echo "configure: CARGO_TARGET_DIR in Makevars: $(grep '^CARGO_TARGET_DIR' src/Makevars | head -1)" >&2
      echo "configure: CARGO_LIBDIR in Makevars: $(grep '^CARGO_LIBDIR' src/Makevars | head -1)" >&2
      echo "configure: CARGO_AR would be: $CARGO_LIBDIR/librpkg.a" >&2
      echo "configure: checking if $CARGO_LIBDIR exists: $(test -d "$CARGO_LIBDIR" && echo yes || echo no)" >&2

      dnl NOTE: Do NOT clear checksum files here. When building from a tarball,
      dnl the vendor directory comes from inst/vendor.tar.xz which was created
      dnl by `cargo vendor` with valid checksums. Clearing them breaks the build.
      dnl
      dnl The checksum clearing that used to be here was intended to handle files
      dnl excluded by R CMD build (.gitignore etc), but this is now handled by
      dnl the compress-vendor step which clears checksums BEFORE creating the tarball.

      dnl Remove files that cause R CMD check warnings (GNU extensions in Makefiles)
      rm -f "$VENDOR_OUT/r-efi/Makefile"

      dnl Rewrite cargo config to use vendored sources (both locations)
      dnl Use _CARGO paths for Windows-native path format
      for _cfg in "src/rust/.cargo/config.toml" ".cargo/config.toml"; do
        mkdir -p "$(dirname "$_cfg")"
        cat > "$_cfg" <<VENDORED_CONFIG_EOF
# Autogenerated by configure -- DO NOT EDIT

@<:@source.crates-io@:>@
replace-with = "vendored-sources"

@<:@source.vendored-sources@:>@
directory = "$VENDOR_OUT_CARGO"

@<:@build@:>@
target-dir = "$CARGO_TARGET_DIR_CARGO"
VENDORED_CONFIG_EOF
        echo "configure: rewrote $_cfg for vendored sources" >&2
      done
    else
      echo "configure: WARNING: vendor missing and no vendor.tar.xz; falling back to crates.io" >&2
      dnl Rewrite cargo config to use crates.io (no source replacement)
      for _cfg in "src/rust/.cargo/config.toml" ".cargo/config.toml"; do
        mkdir -p "$(dirname "$_cfg")"
        cat > "$_cfg" <<CRATESIO_CONFIG_EOF
# Autogenerated by configure -- DO NOT EDIT

@<:@build@:>@
target-dir = "$CARGO_TARGET_DIR_CARGO"
CRATESIO_CONFIG_EOF
        echo "configure: rewrote $_cfg for crates.io sources" >&2
      done
      dnl Regenerate Cargo.lock to clear any stale path/vendored sources.
      _lockfile="$ABS_RPKG_SRC/rust/Cargo.lock"
      echo "configure: regenerating Cargo.lock (crates.io fallback)" >&2
      rm -f "$_lockfile"
      $CARGO_CMD generate-lockfile --manifest-path "$ABS_RPKG_SRC/rust/Cargo.toml"
      if test $? -ne 0; then
        echo "configure: error: cargo generate-lockfile failed" >&2
        exit 1
      fi
    fi
  fi

  dnl Touch Cargo.toml to ensure make rebuilds the staticlib.
  dnl This is necessary because CARGO_TARGET_DIR may point to a shared location
  dnl where the staticlib already exists (e.g., during R CMD check).
  touch "$ABS_RPKG_SRC/rust/Cargo.toml"
  echo "configure: touched $ABS_RPKG_SRC/rust/Cargo.toml to force rebuild" >&2

  dnl Update Cargo.lock to reflect patched/vendored sources.
  dnl When [patch.crates-io] redirects miniextendr-* to local paths, the lock file
  dnl needs updating so cargo --locked doesn't complain about source mismatches.
  dnl This doesn't change versions, just updates source locations.
  if test -f "$ABS_RPKG_SRC/rust/Cargo.lock"; then
    $CARGO_CMD update --workspace --manifest-path "$ABS_RPKG_SRC/rust/Cargo.toml" >&2 2>/dev/null || true
    echo "configure: updated Cargo.lock for patched sources" >&2
  fi
],
[RSYNC="$RSYNC" SED="$SED" VENDOR_OUT="$VENDOR_OUT" VENDOR_OUT_CARGO="$VENDOR_OUT_CARGO" \
 ROOT_MINIEXTENDR_REPO="$root_miniextendr_repo" \
 MINIEXTENDR_API_SRC="$MINIEXTENDR_API_SRC" MINIEXTENDR_MACROS_SRC="$MINIEXTENDR_MACROS_SRC" \
 MINIEXTENDR_LINT_SRC="$MINIEXTENDR_LINT_SRC" MINIEXTENDR_ENGINE_SRC="$MINIEXTENDR_ENGINE_SRC" \
 CARGO_CMD="$CARGO_CMD" CARGO_TARGET_DIR="$CARGO_TARGET_DIR" CARGO_TARGET_DIR_CARGO="$CARGO_TARGET_DIR_CARGO" \
 CARGO_PROFILE="$CARGO_PROFILE" ABS_RPKG_SRC="$abs_rpkg_src" abs_rpkg_dir="$abs_top_srcdir" abs_rpkg_src="$abs_rpkg_src"])

dnl 2c) Compress vendor directory for CRAN tarballs (after sync-miniextendr-crates)
dnl This must run after both cargo-vendor and sync-miniextendr-crates so the
dnl compressed archive includes all vendored dependencies including miniextendr crates.
dnl Only recompress if vendor contents have changed (based on Cargo.lock + miniextendr sources).
AC_CONFIG_COMMANDS([compress-vendor],
[
  dnl Only compress in dev mode (NOT_CRAN=true) when vendor directory exists
  if test "$NOT_CRAN" = "true" && test -d "$VENDOR_OUT" && test -n "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
    VENDOR_TARBALL="$abs_rpkg_dir/inst/vendor.tar.xz"
    VENDOR_STAMP="$abs_rpkg_dir/inst/.vendor.tar.xz.cksum"

    dnl Calculate fingerprint based on inputs that affect vendor contents:
    dnl - Cargo.lock checksum (third-party deps)
    dnl - miniextendr crate directory checksums (ALL files, not just Cargo.toml)
    _lock_ck=""
    if test -f "src/rust/Cargo.lock"; then
      _lock_ck="$(cksum < "src/rust/Cargo.lock" | awk '{print [$]1":"[$]2}')"
    fi
    dnl Checksum all source files in each miniextendr crate (excluding target dirs)
    _api_ck=""
    if test -d "$VENDOR_OUT/miniextendr-api"; then
      _api_ck="$(find "$VENDOR_OUT/miniextendr-api" -path '*/target' -prune -o -type f -print | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | awk '{print [$]1":"[$]2}')"
    fi
    _macros_ck=""
    if test -d "$VENDOR_OUT/miniextendr-macros"; then
      _macros_ck="$(find "$VENDOR_OUT/miniextendr-macros" -path '*/target' -prune -o -type f -print | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | awk '{print [$]1":"[$]2}')"
    fi
    _lint_ck=""
    if test -d "$VENDOR_OUT/miniextendr-lint"; then
      _lint_ck="$(find "$VENDOR_OUT/miniextendr-lint" -path '*/target' -prune -o -type f -print | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | awk '{print [$]1":"[$]2}')"
    fi
    _engine_ck=""
    if test -d "$VENDOR_OUT/miniextendr-engine"; then
      _engine_ck="$(find "$VENDOR_OUT/miniextendr-engine" -path '*/target' -prune -o -type f -print | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | awk '{print [$]1":"[$]2}')"
    fi

    _new_fingerprint="lock:$_lock_ck;api:$_api_ck;macros:$_macros_ck;lint:$_lint_ck;engine:$_engine_ck"
    _old_fingerprint="$(cat "$VENDOR_STAMP" 2>/dev/null || true)"

    _need_compress=0
    if test ! -f "$VENDOR_TARBALL"; then
      _need_compress=1
      echo "configure: vendor.tar.xz does not exist"
    elif test "x$_new_fingerprint" != "x$_old_fingerprint"; then
      _need_compress=1
      echo "configure: vendor contents changed (fingerprint mismatch)"
    fi

    if test $_need_compress -eq 1; then
      echo "configure: compressing vendor directory to inst/vendor.tar.xz"
      mkdir -p "$abs_rpkg_dir/inst"

      dnl Create temp staging area with vendor/ subdirectory to preserve correct archive structure
      _staging_tmp="$ABS_RPKG_SRC/.vendor-tarball-staging"
      rm -rf "$_staging_tmp"
      mkdir -p "$_staging_tmp"
      cp -R "$VENDOR_OUT" "$_staging_tmp/vendor"

      dnl Note: We cannot strip /// doc comments because displaydoc crate uses them
      dnl to generate Display impls. Stripping them would break error types.
      dnl However, we CAN strip #@<:@doc = include_str! lines since we delete examples/docs dirs
      find "$_staging_tmp/vendor" -name '*.rs' -type f -exec $SED -i.bak -e '/^#\@<:@doc = include_str!/d' -e '/^#!\@<:@doc = include_str!/d' {} \; 2>/dev/null || true
      find "$_staging_tmp/vendor" -name '*.rs.bak' -type f -delete 2>/dev/null || true

      dnl Clear checksums for ALL crates in temp copy.
      dnl Required because we exclude tests/, benches/, examples/, .github/ etc. which are in the checksums.
      for _crate_dir in "$_staging_tmp/vendor"/*/; do
        test -d "$_crate_dir" && echo '{"files":{}}' > "${_crate_dir}.cargo-checksum.json"
      done

      dnl Remove directories we're excluding (faster than tar --exclude for many files)
      find "$_staging_tmp/vendor" -type d -name 'tests' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name 'benches' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name 'examples' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name '.github' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name '.vim' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name 'ci' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name 'scripts' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name 'testdata' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -type d -name 'docs' -exec rm -rf {} + 2>/dev/null || true
      find "$_staging_tmp/vendor" -name '.travis.yml' -type f -delete 2>/dev/null || true
      find "$_staging_tmp/vendor" -name '.gitlab-ci.yml' -type f -delete 2>/dev/null || true
      find "$_staging_tmp/vendor" -name '*.a' -type f -delete 2>/dev/null || true
      find "$_staging_tmp/vendor" -name '*.lib' -type f -delete 2>/dev/null || true
      dnl Truncate all .md files (keeps file existing for include_str! but removes content)
      find "$_staging_tmp/vendor" -name '*.md' -type f -exec sh -c 'printf "" > "$1"' sh {} \;

      dnl Create tarball from staging area
      tar -cJf "$VENDOR_TARBALL" -C "$_staging_tmp" vendor
      _tar_status=$?

      dnl Clean up staging area
      rm -rf "$_staging_tmp"

      if test $_tar_status -ne 0; then
        echo "configure: warning: failed to compress vendor directory" >&2
      else
        echo "$_new_fingerprint" > "$VENDOR_STAMP"
      fi
    else
      echo "configure: skipping vendor compression (unchanged)"
    fi
  fi
],
[NOT_CRAN="$NOT_CRAN" VENDOR_OUT="$VENDOR_OUT" ABS_RPKG_SRC="$abs_rpkg_src" abs_rpkg_dir="$abs_top_srcdir" SED="$SED"])

dnl 3) Extract CARGO_STATICLIB_NAME from Cargo.toml and patch generated files
AC_CONFIG_COMMANDS([patch-staticlib-name],
[
  dnl Use cargo pkgid to extract the actual package name from Cargo.toml
  dnl This handles hyphens correctly (converted to underscores for Rust lib name)
  if test "$NOT_CRAN" = "true"; then
    _pkgid_flags="--color=never"
  else
    _pkgid_flags="--offline --color=never"
  fi
  _pkgid_output="$( $CARGO_CMD pkgid $_pkgid_flags --manifest-path src/rust/Cargo.toml )"
  _pkgid_status=$?
  if test $_pkgid_status -ne 0; then
    echo "configure: error: cargo pkgid failed with status $_pkgid_status" >&2
    exit 1
  fi
  REAL_STATICLIB_NAME="$(echo "$_pkgid_output" | "$SED" 's/.*#\(@<:@^@@:>@*\)@.*/\1/' | tr '-' '_')"
  if test -z "$REAL_STATICLIB_NAME"; then
    echo "configure: error: Could not parse package name from cargo pkgid output" >&2
    exit 1
  fi
  echo "configure: extracted CARGO_STATICLIB_NAME = $REAL_STATICLIB_NAME"

  dnl Patch generated files to replace placeholder
  for f in src/Makevars src/rust/document.rs; do
    if test -f "$f"; then
      "$SED" "s/__CARGO_STATICLIB_NAME_PLACEHOLDER__/$REAL_STATICLIB_NAME/g" "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    fi
  done
],
[CARGO_CMD="$CARGO_CMD" NOT_CRAN="$NOT_CRAN" SED="$SED"])

AC_OUTPUT
