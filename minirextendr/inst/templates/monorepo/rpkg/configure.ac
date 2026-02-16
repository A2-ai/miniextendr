AC_INIT([{{package}}], [1.0])

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

dnl Set cargo offline/locked flags based on NOT_CRAN
dnl - NOT_CRAN=true (dev): no --offline, no --locked (cargo updates lockfile for patches)
dnl - NOT_CRAN=false (CRAN): use --offline, omit --locked (checksums cleared)
if test "$NOT_CRAN" = "true"; then
  CARGO_OFFLINE_FLAG=""
  dnl Dev mode omits --locked so cargo can update source locations for [patch] entries
  CARGO_LOCKED_FLAG=""
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
dnl Users can enable optional features via MINIEXTENDR_FEATURES environment variable.
dnl By default, no extra features are enabled.
dnl
dnl Note: 'nonapi' feature causes R CMD check warnings/errors (non-API calls).
dnl Only enable when you specifically need those features and understand the risks.
AC_ARG_VAR([MINIEXTENDR_FEATURES], [Comma-separated cargo features to enable (e.g., "nonapi,rayon")])

dnl Default: no extra features enabled
dnl Users can set MINIEXTENDR_FEATURES to enable specific features
if test -z "${MINIEXTENDR_FEATURES+x}"; then
  dnl MINIEXTENDR_FEATURES not set - use empty (no extra features)
  MINIEXTENDR_FEATURES=""
fi

if test -n "$MINIEXTENDR_FEATURES"; then
  CARGO_FEATURES_FLAG="--features=$MINIEXTENDR_FEATURES"
  # Auto-generate C preprocessor flags from cargo features
  CARGO_FEATURE_CPPFLAGS=""
  miniextendr_saved_IFS=$IFS
  IFS=','
  for _feature in $MINIEXTENDR_FEATURES; do
    _feature_upper=$(echo "$_feature" | tr 'a-z-' 'A-Z_')
    CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_$_feature_upper"
  done
  IFS=$miniextendr_saved_IFS
  unset miniextendr_saved_IFS
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

dnl ---- tool discovery ----
AC_PATH_TOOL([CARGO],[cargo],[no])
AC_PATH_TOOL([RUSTC],[rustc],[no])
AC_PATH_PROG([SED],[sed],[no])
AC_PATH_PROG([CYGPATH],[cygpath],[no])
AS_IF([test "x$CARGO" = "xno"], [AC_MSG_ERROR([`cargo` not found])])
AS_IF([test "x$RUSTC" = "xno"], [AC_MSG_ERROR([`rustc` not found])])
AS_IF([test "x$SED" = "xno"], [AC_MSG_ERROR([`sed` not found])])
AC_SUBST([CARGO])
AC_SUBST([RUSTC])
AC_SUBST([SED])
AC_SUBST([CYGPATH])

dnl Use native paths for Cargo on Windows/MSYS to avoid path mangling.
dnl MSYS2 paths like /d/a/foo become D:/d/a/foo when interpreted by Cargo,
dnl so we convert them to proper Windows paths (D:/a/foo) with cygpath -m.
abs_rpkg_src_cargo="$abs_rpkg_src"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test "x$CYGPATH" != "xno"; then
      abs_rpkg_src_cargo="$($CYGPATH -m "$abs_rpkg_src")"
    fi
    ;;
esac
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

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (dev or release)])
: ${CARGO_PROFILE="release"}
dnl Map legacy 'debug' → 'dev' (cargo rejects --profile debug)
if test "$CARGO_PROFILE" = "debug"; then
  AC_MSG_WARN([CARGO_PROFILE=debug is invalid for cargo; using 'dev' instead])
  CARGO_PROFILE="dev"
fi
AC_SUBST([CARGO_PROFILE])

dnl Cargo output directory differs from profile name:
dnl   --profile dev     → target/debug/
dnl   --profile release → target/release/
if test "$CARGO_PROFILE" = "dev"; then
  CARGO_PROFILE_DIR="debug"
else
  CARGO_PROFILE_DIR="$CARGO_PROFILE"
fi

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
    AC_MSG_WARN([Autoconf host triple '$host' may differ from Rust target triple.
  If the build fails, set CARGO_BUILD_TARGET explicitly (e.g. x86_64-unknown-linux-gnu).
  Run 'rustc --print target-list' to see valid Rust targets.])
  else
    CARGO_BUILD_TARGET=""
  fi
fi
AC_SUBST([CARGO_BUILD_TARGET])

dnl Compute Cargo libdir path:
dnl - If CARGO_BUILD_TARGET is set, use target/<that>/<profile_dir>
dnl - Otherwise, use target/<profile_dir>
dnl Note: CARGO_PROFILE_DIR maps dev→debug, release→release
if test -n "$CARGO_BUILD_TARGET"; then
  CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_BUILD_TARGET/$CARGO_PROFILE_DIR"
else
  CARGO_LIBDIR="$CARGO_TARGET_DIR/$CARGO_PROFILE_DIR"
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
dnl Use PACKAGE_NAME (case-preserving) not PACKAGE_TARNAME (lowercased)
dnl so the entrypoint's R_init_*_miniextendr symbol matches the Rust module exactly.
pkg_rs="$(echo "$PACKAGE_NAME" | $SED 's/-/_/g; s/\./_/g')"
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

dnl ---- vendoring (at package root, outside src/) ----
dnl IMPORTANT: vendor directory MUST be outside src/ to avoid pkgbuild scanning
dnl thousands of .rs files in vendor/ during needs_compile() checks (same reasoning
dnl as CARGO_TARGET_DIR above).
VENDOR_OUT="$abs_top_srcdir/vendor"
AC_SUBST([VENDOR_OUT])
dnl Cargo-specific version with Windows-native paths (for .cargo/config.toml)
abs_top_srcdir_cargo="$abs_top_srcdir"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test "x$CYGPATH" != "xno"; then
      abs_top_srcdir_cargo="$($CYGPATH -m "$abs_top_srcdir")"
    fi
    ;;
esac
VENDOR_OUT_CARGO="$abs_top_srcdir_cargo/vendor"
AC_SUBST([VENDOR_OUT_CARGO])

dnl Note: Cargo.toml uses path dependencies to vendor/ (unlike rpkg which uses git deps).
dnl The cargo config template (cargo-config.toml.in) handles CRAN source replacement.
dnl In dev mode, we remove the cargo config so cargo uses normal resolution.

dnl ---- output files ----
dnl Create .cargo directory (it's a hidden dir not included in tarball)
AS_MKDIR_P([src/rust/.cargo])

AC_CONFIG_FILES([
  src/rust/.cargo/config.toml:src/rust/cargo-config.toml.in
  src/Makevars:src/Makevars.in
  src/entrypoint.c:src/entrypoint.c.in
  src/mx_abi.c:src/mx_abi.c.in
  src/{{package}}-win.def:src/win.def.in
])

dnl 1) Remove cargo config in dev mode (vendored sources not needed for development)
dnl    In dev mode, Cargo.toml uses git deps directly
AC_CONFIG_COMMANDS([dev-cargo-config],
[
  RPKG_CFG="src/rust/.cargo/config.toml"

  if test "$NOT_CRAN" = "true"; then
    dnl In dev mode, remove the generated cargo config so cargo uses normal resolution
    if test -f "$RPKG_CFG"; then
      rm "$RPKG_CFG"
      echo "configure: removed cargo config (dev mode - using git deps)"
    fi
  fi
],
[NOT_CRAN="$NOT_CRAN"])

dnl ---- AC_CONFIG_COMMANDS run after AC_CONFIG_FILES generates files ----
dnl Order matters: lockfile-compat, then cargo-vendor, then patch-staticlib-name

dnl 2) Ensure Cargo.lock is compatible with the installed cargo version.
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
            tar -xJf "$abs_rpkg_dir/inst/vendor.tar.xz" -C "$abs_rpkg_dir"
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

dnl 3) Ensure vendor/ exists for offline builds
dnl    In dev mode: vendor/ is pre-populated by scaffolding, leave it alone.
dnl    In CRAN mode: unpack vendor.tar.xz if vendor/ is missing.
dnl    Vendoring is NOT done by configure — use `just vendor` for CRAN prep.
AC_CONFIG_COMMANDS([cargo-vendor],
[
  if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
    if test -f "$abs_rpkg_dir/inst/vendor.tar.xz"; then
      echo "configure: unpacking inst/vendor.tar.xz"
      tar -xJf "$abs_rpkg_dir/inst/vendor.tar.xz" -C "$abs_rpkg_dir"
      if test $? -ne 0; then
        echo "configure: error: failed to unpack vendor.tar.xz" >&2
        exit 1
      fi
      if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
        $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
      fi
    fi
  fi

  if test "$NOT_CRAN" = "true"; then
    if test "$FORCE_VENDOR" = "1"; then
      echo "configure: FORCE_VENDOR — running cargo vendor"
      mkdir -p "$VENDOR_OUT"
      $CARGO_CMD vendor --manifest-path src/rust/Cargo.toml "$VENDOR_OUT"
      if test $? -ne 0; then
        echo "configure: error: cargo vendor failed" >&2
        exit 1
      fi
      for _crate_dir in "$VENDOR_OUT"/*/; do
        if test -d "$_crate_dir"; then
          rm -rf "$_crate_dir/tests" "$_crate_dir/benches" "$_crate_dir/examples" \
                 "$_crate_dir/.github" "$_crate_dir/docs" "$_crate_dir/ci"
          for _dotfile in "$_crate_dir"/.*; do
            case "$(basename "$_dotfile")" in
              .|..|.cargo-checksum.json) ;;
              *) rm -rf "$_dotfile" ;;
            esac
          done
          echo '{"files":{}}' > "$_crate_dir/.cargo-checksum.json"
        fi
      done
      if test -f "$abs_rpkg_src/rust/Cargo.lock"; then
        $SED -i.bak '/^checksum = /d' "$abs_rpkg_src/rust/Cargo.lock" && rm -f "$abs_rpkg_src/rust/Cargo.lock.bak"
      fi
    else
      echo "configure: dev mode — using vendor/ from scaffolding"
    fi
  else
    dnl CRAN/offline mode: vendor/ must exist
    if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      echo "configure: error: CRAN build requires vendored sources." >&2
      echo "configure:        Run 'just vendor' before CRAN submission." >&2
      exit 1
    fi
    echo "configure: CRAN build — vendor ready"
  fi
],
[NOT_CRAN="$NOT_CRAN" VENDOR_OUT="$VENDOR_OUT" abs_rpkg_src="$abs_rpkg_src" abs_rpkg_dir="$abs_top_srcdir" SED="$SED" FORCE_VENDOR="$FORCE_VENDOR" CARGO_CMD="$CARGO_CMD"])

dnl 4) Post-vendor tasks: touch Cargo.toml to force rebuild
AC_CONFIG_COMMANDS([post-vendor],
[
  dnl Touch Cargo.toml to ensure make rebuilds the staticlib.
  dnl This is necessary because CARGO_TARGET_DIR may point to a shared location
  dnl where the staticlib already exists (e.g., during R CMD check).
  touch "$ABS_RPKG_SRC/rust/Cargo.toml"
  echo "configure: touched $ABS_RPKG_SRC/rust/Cargo.toml to force rebuild" >&2

  dnl Note: Cargo.lock updates for patched sources are handled by cargo build itself.
  dnl Dev mode does not use --locked, so cargo resolves source locations at build time.
],
[ABS_RPKG_SRC="$abs_rpkg_src"])

dnl 5) Extract CARGO_STATICLIB_NAME from Cargo.toml and patch generated files
AC_CONFIG_COMMANDS([patch-staticlib-name],
[
  dnl Ensure Cargo.lock exists (cargo pkgid requires it)
  if test ! -f "src/rust/Cargo.lock"; then
    if test "$NOT_CRAN" = "true"; then
      $CARGO_CMD generate-lockfile --manifest-path src/rust/Cargo.toml
    else
      $CARGO_CMD generate-lockfile --manifest-path src/rust/Cargo.toml --offline
    fi
  fi

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
