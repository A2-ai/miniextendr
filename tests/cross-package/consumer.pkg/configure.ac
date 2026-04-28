AC_INIT([consumer.pkg], [0.1.0])

dnl Use tools/ directory for auxiliary build files (config.guess, config.sub)
dnl This avoids the CRAN NOTE about non-standard top-level files
AC_CONFIG_AUX_DIR([tools])

dnl Verify we're running from the R package directory (not the monorepo root)
dnl This prevents accidental creation of src/, config.log, etc. in the wrong place
if test ! -f "DESCRIPTION" || test ! -f "NAMESPACE"; then
  AC_MSG_ERROR([configure must be run from the package directory (where DESCRIPTION exists).

  If you're in the monorepo root, run:
    cd tests/cross-package/consumer.pkg && ./configure
  Or use:
    just cross-install])
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

dnl ---- canonical paths ----
abs_top_srcdir="$(cd "$srcdir" && pwd)"
AC_SUBST([ABS_TOP_SRCDIR], ["$abs_top_srcdir"])
abs_rpkg_src="$abs_top_srcdir/src"
AC_SUBST([ABS_RPKG_SRCDIR], ["$abs_rpkg_src"])
RUST_SRC_DIR="$abs_rpkg_src/rust"

dnl ---- install-mode detection ----
dnl Single signal: presence of inst/vendor.tar.xz.
dnl   Tarball install (R CMD INSTALL <built-tarball>): tarball is shipped inside
dnl     the build artifact by `just vendor`. configure unpacks it, writes a
dnl     vendored cargo source replacement, and builds offline.
dnl   Source install (R CMD INSTALL ., devtools::install, etc.):
dnl     no tarball. configure leaves vendoring alone and lets cargo resolve
dnl     deps via the existing [patch.crates-io] path overrides in Cargo.toml.
dnl
dnl NOTE: these cross-package fixtures never ship inst/vendor.tar.xz in
dnl practice (they only build inside the monorepo). The detection block is
dnl kept for structural symmetry with rpkg.
if test -f "$abs_top_srcdir/inst/vendor.tar.xz"; then
  IS_TARBALL_INSTALL=true
  _install_mode_label="tarball install (offline, vendored)"
else
  IS_TARBALL_INSTALL=false
  _install_mode_label="source install (path deps)"
fi
AC_SUBST([IS_TARBALL_INSTALL])
AC_MSG_NOTICE([install mode         = $_install_mode_label])

dnl In tarball mode cargo must run --offline so the build cannot accidentally
dnl reach for crates.io. In source mode we leave cargo's default resolution.
if test "$IS_TARBALL_INSTALL" = "true"; then
  CARGO_OFFLINE_FLAG="--offline"
else
  CARGO_OFFLINE_FLAG=""
fi
AC_SUBST([CARGO_OFFLINE_FLAG])

dnl Set cargo features flag
dnl
dnl Users can enable optional features via CARGO_FEATURES environment variable.
dnl By default, no extra features are enabled.
AC_ARG_VAR([CARGO_FEATURES], [Comma-separated cargo features to enable (e.g., "nonapi,rayon")])

if test -z "${CARGO_FEATURES+x}"; then
  if test -f "${srcdir}/tools/detect-features.R"; then
    CARGO_FEATURES=$("${R_HOME}/bin/Rscript" "${srcdir}/tools/detect-features.R" 2>/dev/null || echo "")
    if test -n "$CARGO_FEATURES"; then
      AC_MSG_NOTICE([Auto-detected features: $CARGO_FEATURES])
    fi
  else
    CARGO_FEATURES=""
  fi
fi

if test -n "$CARGO_FEATURES"; then
  CARGO_FEATURES_FLAG="--features=$CARGO_FEATURES"
else
  CARGO_FEATURES_FLAG=""
fi
AC_SUBST([CARGO_FEATURES_FLAG])

dnl ---- tool discovery ----
AC_PATH_TOOL([CARGO],[cargo],[no])
AC_PATH_TOOL([RUSTC],[rustc],[no])
AC_PATH_PROG([SED],[sed],[no])
AC_PATH_PROG([CYGPATH],[cygpath],[no])
AS_IF([test "x$CARGO" = "xno"], [AC_MSG_ERROR([`cargo` not found])])
AS_IF([test "x$RUSTC" = "xno"], [AC_MSG_ERROR([`rustc` not found])])
AS_IF([test "x$SED" = "xno"], [AC_MSG_ERROR([`sed` not found])])
AC_SUBST([CARGO])

dnl Use native paths for Cargo on Windows/MSYS to avoid path mangling.
abs_rpkg_src_cargo="$abs_rpkg_src"
abs_top_srcdir_cargo="$abs_top_srcdir"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test "x$CYGPATH" != "xno"; then
      abs_rpkg_src_cargo="$($CYGPATH -m "$abs_rpkg_src")"
      abs_top_srcdir_cargo="$($CYGPATH -m "$abs_top_srcdir")"
    fi
    ;;
esac
AC_SUBST([ABS_RPKG_SRC_CARGO], ["$abs_rpkg_src_cargo"])

dnl Report rustc version (required by CRAN for Rust packages)
RUSTC_VERSION="$($RUSTC --version)"
AC_MSG_NOTICE([using $RUSTC_VERSION])

dnl Check minimum rustc version (edition 2024 requires rustc 1.85+)
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

CARGO_CMD="$CARGO $RUST_TOOLCHAIN"

dnl ---- Cargo target/profile configuration ----
AC_ARG_VAR([CARGO_TARGET_DIR], [Cargo target directory (default: rust-target, outside src/)])
: ${CARGO_TARGET_DIR="$abs_top_srcdir/rust-target"}
AC_SUBST([CARGO_TARGET_DIR])

CARGO_TARGET_DIR_CARGO="$abs_top_srcdir_cargo/rust-target"
AC_SUBST([CARGO_TARGET_DIR_CARGO])

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (dev or release)])
: ${CARGO_PROFILE="release"}
if test "$CARGO_PROFILE" = "debug"; then
  AC_MSG_WARN([CARGO_PROFILE=debug is invalid for cargo; using 'dev' instead])
  CARGO_PROFILE="dev"
fi
AC_SUBST([CARGO_PROFILE])

if test "$CARGO_PROFILE" = "dev"; then
  CARGO_PROFILE_DIR="debug"
else
  CARGO_PROFILE_DIR="$CARGO_PROFILE"
fi

AC_ARG_VAR([ENV_RUSTFLAGS], [Rust compiler flags passed as RUSTFLAGS])
: ${ENV_RUSTFLAGS="$RUSTFLAGS"}
AC_SUBST([ENV_RUSTFLAGS])

AC_ARG_VAR([CARGO_BUILD_TARGET], [Cargo build target triple (e.g. aarch64-apple-darwin)])
if test -z "$CARGO_BUILD_TARGET"; then
  if test -n "$host" && test -n "$build" && test "$host" != "$build"; then
    CARGO_BUILD_TARGET="$host"
    AC_MSG_NOTICE([CARGO_BUILD_TARGET not set; using host triple $host])
    AC_MSG_WARN([Autoconf host triple '$host' may differ from Rust target triple.
  If the build fails, set CARGO_BUILD_TARGET explicitly (e.g. x86_64-unknown-linux-gnu).
  Run 'rustc --print target-list' to see valid Rust targets.])
  else
    CARGO_BUILD_TARGET=""
  fi
fi
AC_SUBST([CARGO_BUILD_TARGET])

if test -n "$CARGO_BUILD_TARGET"; then
  CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_BUILD_TARGET/$CARGO_PROFILE_DIR"
else
  CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_PROFILE_DIR"
fi
AC_SUBST([CARGO_LIBDIR])

AC_MSG_NOTICE([R_HOME               = $R_HOME])
AC_MSG_NOTICE([CARGO_TARGET_DIR     = $CARGO_TARGET_DIR])
AC_MSG_NOTICE([CARGO_PROFILE        = $CARGO_PROFILE])
AC_MSG_NOTICE([ENV_RUSTFLAGS        = $ENV_RUSTFLAGS])
AC_MSG_NOTICE([CARGO_LIBDIR         = $CARGO_LIBDIR])
AS_IF([test -n "$RUST_TOOLCHAIN"],     [AC_MSG_NOTICE([RUST_TOOLCHAIN       = $RUST_TOOLCHAIN])])
AS_IF([test -n "$CARGO_BUILD_TARGET"], [AC_MSG_NOTICE([CARGO_BUILD_TARGET   = $CARGO_BUILD_TARGET])])
AS_IF([test -n "$CARGO_FEATURES"],     [AC_MSG_NOTICE([CARGO_FEATURES       = $CARGO_FEATURES])])

dnl ---- package name → Rust-safe variants ----
CARGO_STATICLIB_NAME="$(echo "$PACKAGE_NAME" | $SED 's/-/_/g; s/\./_/g')"
AC_SUBST([CARGO_STATICLIB_NAME])
AC_MSG_NOTICE([CARGO_STATICLIB_NAME = $CARGO_STATICLIB_NAME])

dnl MSYS2/MinGW tar interprets D: as remote host — --force-local fixes this
case "$host_os" in
  *msys*|*cygwin*|*mingw*) TAR_FORCE_LOCAL="--force-local" ;;
  *)                       TAR_FORCE_LOCAL="" ;;
esac
AC_SUBST([TAR_FORCE_LOCAL])

dnl ---- cdylib platform extension (for R wrapper generation) ----
case "$host_os" in
  darwin*)                  CDYLIB_PREFIX="lib"; CDYLIB_EXT="dylib" ;;
  *mingw*|*msys*|*cygwin*)  CDYLIB_PREFIX="";    CDYLIB_EXT="dll" ;;
  *)                        CDYLIB_PREFIX="lib"; CDYLIB_EXT="so" ;;
esac
AC_SUBST([CDYLIB_PREFIX])
AC_SUBST([CDYLIB_EXT])

dnl ---- Native R package include paths (for bindgen C shim compilation) ----
dnl Populated by minirextendr::use_native_package(). Each native package
dnl adds -I<path> for its inst/include/ headers so the C compiler can
dnl find them when compiling bindgen-generated static inline wrappers.
NATIVE_PKG_CPPFLAGS=""
dnl MINIREXTENDR: native-pkg-cppflags insertion marker
AC_SUBST([NATIVE_PKG_CPPFLAGS])

AC_CONFIG_SRCDIR([src/rust/lib.rs])

dnl ---- vendor directory paths (used in tarball mode only) ----
VENDOR_OUT="$abs_top_srcdir/vendor"
VENDOR_OUT_CARGO="$abs_top_srcdir_cargo/vendor"
AC_SUBST([VENDOR_OUT])
AC_SUBST([VENDOR_OUT_CARGO])

dnl ---- output files ----
dnl AC_CONFIG_FILES handles Makevars and the win.def. The cargo config is
dnl written by AC_CONFIG_COMMANDS below because its contents differ across
dnl install modes.
AS_MKDIR_P([src/rust/.cargo])

AC_CONFIG_FILES([
  src/Makevars:src/Makevars.in
  src/$PACKAGE_NAME-win.def:src/win.def.in
])

dnl ---- 1) Tarball install: unpack inst/vendor.tar.xz ----
dnl Source mode is a no-op here (these fixtures never ship a tarball).
dnl The block is kept for structural symmetry with rpkg/configure.ac.
AC_CONFIG_COMMANDS([unpack-vendor-tarball],
[
  if test "$IS_TARBALL_INSTALL" = "true"; then
    if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      echo "configure: tarball install — unpacking inst/vendor.tar.xz"
      (cd "$abs_top_srcdir" && tar $TAR_FORCE_LOCAL -xJf "$abs_top_srcdir/inst/vendor.tar.xz")
      if test $? -ne 0; then
        echo "configure: error: failed to unpack inst/vendor.tar.xz" >&2
        exit 1
      fi
      dnl Vendored crates ship with empty checksums; strip the [checksum]
      dnl entries from Cargo.lock so cargo accepts the vendored tree.
      if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
        $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && \
          rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
      fi
    else
      echo "configure: tarball install — vendor/ already populated"
    fi
  fi
],
[IS_TARBALL_INSTALL="$IS_TARBALL_INSTALL"
 abs_top_srcdir="$abs_top_srcdir"
 abs_rpkg_src="$abs_rpkg_src"
 VENDOR_OUT="$VENDOR_OUT"
 SED="$SED"
 TAR_FORCE_LOCAL="$TAR_FORCE_LOCAL"])

dnl ---- 2) Write .cargo/config.toml per install mode ----
dnl
dnl Tarball install:
dnl   [source.crates-io]                      replace-with = "vendored-sources"
dnl   [source.vendored-sources]               directory = "<vendor>"
dnl   [build]                                 target-dir = "<rust-target>"
dnl
dnl Source install (tarball never present for these fixtures):
dnl   [build]                                 target-dir = "<rust-target>"
dnl   (cargo follows the [patch.crates-io] path overrides already in Cargo.toml)
AC_CONFIG_COMMANDS([cargo-config],
[
  RPKG_CFG="src/rust/.cargo/config.toml"
  rm -f "$RPKG_CFG"

  if test "$IS_TARBALL_INSTALL" = "true"; then
    {
      echo "# Autogenerated by configure — DO NOT EDIT"
      echo "# Tarball install: redirect crates.io to the unpacked vendor tarball."
      echo ""
      echo "@<:@source.crates-io@:>@"
      echo 'replace-with = "vendored-sources"'
      echo ""
      echo "@<:@source.vendored-sources@:>@"
      echo "directory = \"$VENDOR_OUT_CARGO\""
      echo ""
      echo "@<:@build@:>@"
      echo "target-dir = \"$CARGO_TARGET_DIR_CARGO\""
    } > "$RPKG_CFG"
    echo "configure: wrote $RPKG_CFG (tarball mode)"

  else
    {
      echo "# Autogenerated by configure — DO NOT EDIT"
      echo "# Source install: Cargo.toml already contains [patch.crates-io] path overrides"
      echo "# for the monorepo workspace crates. No additional patches needed here."
      echo ""
      echo "@<:@build@:>@"
      echo "target-dir = \"$CARGO_TARGET_DIR_CARGO\""
    } > "$RPKG_CFG"
    echo "configure: wrote $RPKG_CFG (source mode)"
  fi
],
[IS_TARBALL_INSTALL="$IS_TARBALL_INSTALL"
 VENDOR_OUT_CARGO="$VENDOR_OUT_CARGO"
 CARGO_TARGET_DIR_CARGO="$CARGO_TARGET_DIR_CARGO"])

dnl ---- 3) Cargo.lock compatibility ----
dnl Older cargo releases (<1.78) cannot read lockfile version 4. Regenerate
dnl in-place if the toolchain on this machine can't read what's committed.
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
      echo "configure: WARNING: cargo $CARGO_VERSION cannot read Cargo.lock v$LOCKFILE_VERSION; regenerating" >&2
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
[CARGO_CMD="$CARGO_CMD" CARGO_OFFLINE_FLAG="$CARGO_OFFLINE_FLAG" SED="$SED"])

dnl ---- 4) Ensure Cargo.lock exists; touch Cargo.toml to force rebuild ----
AC_CONFIG_COMMANDS([post-config],
[
  if test ! -f "src/rust/Cargo.lock"; then
    $CARGO_CMD generate-lockfile --manifest-path src/rust/Cargo.toml $CARGO_OFFLINE_FLAG
  fi

  dnl Touch Cargo.toml so make rebuilds the staticlib when CARGO_TARGET_DIR
  dnl points at a shared location that already has artifacts (e.g. R CMD check).
  touch "$ABS_RPKG_SRC/rust/Cargo.toml"
],
[ABS_RPKG_SRC="$abs_rpkg_src" CARGO_CMD="$CARGO_CMD" CARGO_OFFLINE_FLAG="$CARGO_OFFLINE_FLAG"])

AC_OUTPUT
