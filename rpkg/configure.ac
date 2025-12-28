AC_INIT([rpkg], [1.0])

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
dnl - NOT_CRAN=true (dev): no --offline
dnl - NOT_CRAN=false (CRAN): use --offline (pre-vendored tarball)
if test "$NOT_CRAN" = "true"; then
  CARGO_OFFLINE_FLAG=""
  CARGO_LOCKED_FLAG="--locked"
else
  CARGO_OFFLINE_FLAG="--offline"
  dnl CRAN build may need to regenerate lockfile with vendored paths
  CARGO_LOCKED_FLAG=""
fi
AC_SUBST([CARGO_OFFLINE_FLAG])
AC_SUBST([CARGO_LOCKED_FLAG])
AC_SUBST([NOT_CRAN])

dnl Set cargo features flag
dnl
dnl By default, use no extra features (safe, passes R CMD check).
dnl Advanced users can enable features via environment variables:
dnl - MINIEXTENDR_FEATURES="nonapi" for non-API R functions (advanced threading, stack control)
dnl - MINIEXTENDR_FEATURES="rayon" for parallel computation support
dnl - MINIEXTENDR_FEATURES="nonapi,rayon" for both
dnl
dnl Note: 'nonapi' feature causes R CMD check warnings/errors (non-API calls).
dnl Only enable when you specifically need those features and understand the risks.
AC_ARG_VAR([MINIEXTENDR_FEATURES], [Comma-separated cargo features to enable (e.g., "nonapi,rayon")])
: ${MINIEXTENDR_FEATURES=""}

if test -n "$MINIEXTENDR_FEATURES"; then
  CARGO_FEATURES_FLAG="--features=$MINIEXTENDR_FEATURES"
  # Set C preprocessor flags based on enabled features
  CARGO_FEATURE_CPPFLAGS=""
  case ",$MINIEXTENDR_FEATURES," in
    *,nonapi,*) CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_NONAPI" ;;
  esac
  case ",$MINIEXTENDR_FEATURES," in
    *,rayon,*) CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_RAYON" ;;
  esac
  case ",$MINIEXTENDR_FEATURES," in
    *,connections,*) CARGO_FEATURE_CPPFLAGS="$CARGO_FEATURE_CPPFLAGS -DCARGO_FEATURE_CONNECTIONS" ;;
  esac
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
root_miniextendr_repo_cargo="$root_miniextendr_repo"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test "x$CYGPATH" != "xno"; then
      root_miniextendr_repo_cargo="$($CYGPATH -m "$root_miniextendr_repo")"
    fi
    ;;
esac
AC_SUBST([ROOT_MINIEXTENDR_REPO_CARGO], ["$root_miniextendr_repo_cargo"])

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
AC_ARG_VAR([CARGO_TARGET_DIR], [Cargo target directory (default: src/rust/target)])
: ${CARGO_TARGET_DIR="$abs_rpkg_src/rust/target"}
AC_SUBST([CARGO_TARGET_DIR])

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (debug or release)])
: ${CARGO_PROFILE="release"}
AC_SUBST([CARGO_PROFILE])

dnl CARGO_STATICLIB_NAME will be extracted after Cargo.toml generation
dnl Use placeholder for now - AC_CONFIG_COMMANDS will substitute the real value
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
AC_MSG_NOTICE([CARGO_TARGET_DIR      = $CARGO_TARGET_DIR])
AC_MSG_NOTICE([CARGO_PROFILE         = $CARGO_PROFILE])
AC_MSG_NOTICE([CARGO_STATICLIB_NAME  = $CARGO_STATICLIB_NAME])
AC_MSG_NOTICE([ENV_RUSTFLAGS         = $ENV_RUSTFLAGS])
AC_MSG_NOTICE([CARGO_LIBDIR          = $CARGO_LIBDIR])
AS_IF([test -n "$RUST_TOOLCHAIN"],
      [AC_MSG_NOTICE([RUST_TOOLCHAIN       = $RUST_TOOLCHAIN])])
AC_MSG_NOTICE([ROOT_MINIEXTENDR_REPO = $ROOT_MINIEXTENDR_REPO])

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

dnl Force refresh if you want
AC_ARG_VAR([FORCE_VENDOR],[Set to 1 to force `cargo vendor` refresh in dev mode])
: ${FORCE_VENDOR="0"}
AC_SUBST([FORCE_VENDOR])

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
directory = "$VENDOR_OUT"
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
  src/rust/Cargo.toml
  src/rust/.cargo/config.toml:src/rust/cargo-config.toml.in
  src/Makevars
  src/entrypoint.c
  src/mx_abi.c
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
    cp "$ROOT_CFG_SRC" "$ROOT_CFG_DST"
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
[CARGO_CMD="$CARGO_CMD" CARGO_OFFLINE_FLAG="$CARGO_OFFLINE_FLAG" SED="$SED"])

dnl 2) Run cargo vendor (dev mode only, after Cargo.toml is generated)
AC_CONFIG_COMMANDS([cargo-vendor],
[
  dnl Only vendor in dev mode (NOT_CRAN=true).
  dnl In CRAN/offline mode we MUST already have a populated vendor dir (from the tarball).
  if test "$NOT_CRAN" != "true"; then
    if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      echo "configure: error: CRAN/offline build but vendor directory is missing or empty: $VENDOR_OUT" >&2
      echo "configure:        Either build from a release tarball that includes src/vendor, or rerun configure with NOT_CRAN=true (and optionally FORCE_VENDOR=1)." >&2
      exit 1
    fi
    echo "configure: CRAN build - skipping cargo vendor (using pre-vendored tarball)"
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
    _new="$(cksum < "$LOCKFILE" | awk '{print $1":"$2}')"
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
        echo "configure: running cargo vendor (Cargo.lock changed or vendor missing)"
        mkdir -p "$VENDOR_OUT"
        $CARGO_CMD vendor \
          --manifest-path "$MANIFEST" \
          --sync "$ROOT_MINIEXTENDR_REPO_CARGO/Cargo.toml" \
          --sync "$ROOT_MINIEXTENDR_REPO_CARGO/miniextendr-bench/Cargo.toml" \
          "$VENDOR_OUT"
        if test $? -ne 0; then
          echo "configure: error: cargo vendor failed" >&2
          exit 1
        fi
        echo "$_new" > "$STAMP"
      else
        dnl Monorepo not available (e.g., R CMD check from tarball with NOT_CRAN=true)
        if test -d "$VENDOR_OUT" && test -n "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
          echo "configure: skipping cargo vendor (monorepo not found, using existing vendor)"
        else
          echo "configure: error: cargo vendor needed but monorepo not found and vendor is empty" >&2
          echo "configure:        Cannot build without either monorepo or pre-vendored dependencies." >&2
          exit 1
        fi
      fi
    else
      echo "configure: skipping cargo vendor (Cargo.lock unchanged)"
    fi
  fi
],
[CARGO_CMD="$CARGO_CMD" NOT_CRAN="$NOT_CRAN" FORCE_VENDOR="$FORCE_VENDOR" VENDOR_OUT="$VENDOR_OUT" ROOT_MINIEXTENDR_REPO="$root_miniextendr_repo" ROOT_MINIEXTENDR_REPO_CARGO="$root_miniextendr_repo_cargo"])

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
      if grep -q '\\.workspace = true' "$_cargo_toml" 2>/dev/null; then
        echo "configure: error: $_crate/Cargo.toml still contains '.workspace = true' after patching" >&2
        exit 1
      fi
      dnl Create .cargo-checksum.json for vendored crates (required by cargo)
      echo '{"files":{}}' > "$VENDOR_OUT/$_crate/.cargo-checksum.json"
    done

    echo "configure: miniextendr crates synced to $VENDOR_OUT" >&2
  else
    echo "configure: miniextendr source crates not found; switching to vendored sources" >&2

    dnl When source crates aren't found (e.g., during R CMD check from tarball),
    dnl we must rewrite the cargo config to use vendored sources instead of
    dnl the absolute path patches that were generated for dev mode.
    if test ! -d "$VENDOR_OUT" || test -z "`ls -A \"$VENDOR_OUT\" 2>/dev/null`"; then
      echo "configure: error: vendor directory missing or empty: $VENDOR_OUT" >&2
      echo "configure:        Cannot build without either source crates or vendored dependencies." >&2
      exit 1
    fi

    dnl Force CARGO_TARGET_DIR to be local when source crates aren't found.
    dnl This prevents using a stale target directory from a previous build.
    CARGO_TARGET_DIR="$ABS_RPKG_SRC/rust/target"
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

    dnl Clear checksum files to avoid "file not found" errors from R CMD build exclusions
    dnl R CMD build excludes .gitignore files etc, but cargo-checksum.json references them
    for _cksum in "$VENDOR_OUT"/*/.cargo-checksum.json; do
      if test -f "$_cksum"; then
        echo '{"files":{}}' > "$_cksum"
      fi
    done
    echo "configure: cleared vendor checksum files" >&2

    dnl Also strip checksums from Cargo.lock to avoid verification mismatch
    if test -f "src/rust/Cargo.lock"; then
      $SED -i.bak '/^checksum = /d' src/rust/Cargo.lock && rm -f src/rust/Cargo.lock.bak
      echo "configure: stripped checksums from Cargo.lock" >&2
    fi

    dnl Rewrite cargo config to use vendored sources (both locations)
    for _cfg in "src/rust/.cargo/config.toml" ".cargo/config.toml"; do
      mkdir -p "$(dirname "$_cfg")"
      cat > "$_cfg" <<VENDORED_CONFIG_EOF
# Autogenerated by configure -- DO NOT EDIT

@<:@source.crates-io@:>@
replace-with = "vendored-sources"

@<:@source.vendored-sources@:>@
directory = "$VENDOR_OUT"

@<:@build@:>@
target-dir = "$CARGO_TARGET_DIR"
VENDORED_CONFIG_EOF
      echo "configure: rewrote $_cfg for vendored sources" >&2
    done
  fi

  dnl Touch Cargo.toml to ensure make rebuilds the staticlib.
  dnl This is necessary because CARGO_TARGET_DIR may point to a shared location
  dnl where the staticlib already exists (e.g., during R CMD check).
  touch "$ABS_RPKG_SRC/rust/Cargo.toml"
  echo "configure: touched $ABS_RPKG_SRC/rust/Cargo.toml to force rebuild" >&2
],
[RSYNC="$RSYNC" SED="$SED" VENDOR_OUT="$VENDOR_OUT" ROOT_MINIEXTENDR_REPO="$root_miniextendr_repo" \
 MINIEXTENDR_API_SRC="$MINIEXTENDR_API_SRC" MINIEXTENDR_MACROS_SRC="$MINIEXTENDR_MACROS_SRC" \
 MINIEXTENDR_LINT_SRC="$MINIEXTENDR_LINT_SRC" MINIEXTENDR_ENGINE_SRC="$MINIEXTENDR_ENGINE_SRC" \
 CARGO_TARGET_DIR="$CARGO_TARGET_DIR" CARGO_PROFILE="$CARGO_PROFILE" ABS_RPKG_SRC="$abs_rpkg_src"])

dnl 3) Extract CARGO_STATICLIB_NAME from generated Cargo.toml and patch files
AC_CONFIG_COMMANDS([patch-staticlib-name],
[
  dnl cargo pkgid outputs: path+file:///path/to/project#name@version
  dnl Extract package name between # and @, convert hyphens to underscores for Rust
  dnl Use --offline only for CRAN builds (NOT_CRAN != true) to avoid network access
  dnl Dev builds (NOT_CRAN=true) may not have cache populated yet
  if test "$NOT_CRAN" = "true"; then
    _pkgid_flags="--color=never"
  else
    _pkgid_flags="--offline --color=never"
  fi
  _pkgid_output="$( $CARGO_CMD pkgid $_pkgid_flags --manifest-path src/rust/Cargo.toml )"
  _pkgid_status=$?
  if test $_pkgid_status -ne 0; then
    echo "configure: error: cargo pkgid failed with status $_pkgid_status" >&2
    echo "configure: cargo pkgid output: $_pkgid_output" >&2
    exit 1
  fi
  REAL_STATICLIB_NAME="$(echo "$_pkgid_output" | "$SED" 's/.*#\(@<:@^@@:>@*\)@.*/\1/' | tr '-' '_')"
  if test -z "$REAL_STATICLIB_NAME"; then
    echo "configure: error: Could not parse package name from cargo pkgid output: $_pkgid_output" >&2
    exit 1
  fi
  echo "configure: extracted CARGO_STATICLIB_NAME = $REAL_STATICLIB_NAME"

  dnl Patch generated files to replace placeholder (portable: temp file + mv)
  for f in src/Makevars src/rust/document.rs; do
    if test -f "$f"; then
      "$SED" "s/__CARGO_STATICLIB_NAME_PLACEHOLDER__/$REAL_STATICLIB_NAME/g" "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    fi
  done
],
[CARGO_CMD="$CARGO_CMD" NOT_CRAN="$NOT_CRAN" SED="$SED"])

AC_OUTPUT
