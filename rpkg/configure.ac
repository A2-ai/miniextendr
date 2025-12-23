AC_INIT([rpkg], [1.0])

dnl Canonicalize build/host triples for cross-compilation support
AC_CANONICAL_HOST

dnl Ensure R_HOME is set and valid
if test -z "$R_HOME"; then
  R_HOME=`R RHOME 2>/dev/null`
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
else
  CARGO_OFFLINE_FLAG="--offline"
fi
AC_SUBST([CARGO_OFFLINE_FLAG])
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
abs_top_srcdir=`(cd "$srcdir" && pwd)`
AC_SUBST([ABS_TOP_SRCDIR], ["$abs_top_srcdir"])
abs_rpkg_src="$abs_top_srcdir/src"
AC_SUBST([ABS_RPKG_SRCDIR], ["$abs_rpkg_src"])

dnl Canonical path two levels above src (rpkg/src → miniextendr)
root_miniextendr_repo=`(cd "$abs_rpkg_src/../.." && pwd)`
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
AS_IF([test "x$CARGO" = "xno"], [AC_MSG_ERROR([`cargo` not found])])
AS_IF([test "x$RUSTC" = "xno"], [AC_MSG_ERROR([`rustc` not found])])
dnl rsync is only required when syncing from monorepo sources
AS_IF([test "x$SED" = "xno"], [AC_MSG_ERROR([`sed` not found])])
AC_SUBST([CARGO])
AC_SUBST([RUSTC])
AC_SUBST([RSYNC])
AC_SUBST([SED])

dnl Report rustc version (required by CRAN for Rust packages)
RUSTC_VERSION=`$RUSTC --version`
AC_MSG_NOTICE([using $RUSTC_VERSION])

dnl Check minimum rustc version (edition 2024 requires rustc 1.85+)
dnl Extract version number: "rustc 1.85.0 (..." -> "1.85.0"
RUSTC_VERSION_NUM=`echo "$RUSTC_VERSION" | sed 's/rustc \(@<:@0-9@:>@*\.@<:@0-9@:>@*\.@<:@0-9@:>@*\).*/\1/'`
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
pkg_rs=`echo "$PACKAGE_TARNAME" | sed 's/-/_/g'`
AC_SUBST([PACKAGE_TARNAME_RS], [$pkg_rs])
pkg_rs_upper=`printf '%s' "$pkg_rs" | tr 'a-z' 'A-Z'`
AC_SUBST([PACKAGE_TARNAME_RS_UPPERCASE], [$pkg_rs_upper])

dnl Generate files from .in templates (templates must be committed)
AC_CONFIG_FILES([src/rust/document.rs])
AC_CONFIG_SRCDIR([src/entrypoint.c.in])  dnl safety check

dnl ---- enumerate Rust sources for Make dependencies (no make wildcards) ----
RUST_SRC_DIR="$abs_rpkg_src/rust"
RUST_SOURCES=
if test -d "$RUST_SRC_DIR"; then
  RUST_SOURCES=`(cd "$RUST_SRC_DIR" && find . -type f -name '*.rs' -print \
    | sed "s|^.|$RUST_SRC_DIR|" \
    | tr '\012' ' ')`
fi
AC_SUBST([RUST_SOURCES])

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

dnl Sync miniextendr crates from monorepo to vendor (if sources exist)
dnl This handles workspace inheritance by patching Cargo.toml during copy
AC_MSG_CHECKING([for miniextendr source crates])
if test -d "$MINIEXTENDR_API_SRC" && test -d "$MINIEXTENDR_MACROS_SRC" && \
   test -d "$MINIEXTENDR_LINT_SRC" && test -d "$MINIEXTENDR_ENGINE_SRC"; then

  dnl rsync is required when syncing from monorepo
  AS_IF([test "x$RSYNC" = "xno"], [
    AC_MSG_ERROR([`rsync` not found (required for syncing miniextendr crates from monorepo)])
  ])

  AC_MSG_RESULT([found, syncing to vendor])

  dnl Extract workspace.package values from root Cargo.toml
  ROOT_CARGO_TOML="$root_miniextendr_repo/Cargo.toml"

  dnl Extract simple string values from [workspace.package]
  WS_VERSION=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^version = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_EDITION=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^edition = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_LICENSE=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^license = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_REPOSITORY=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^repository = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_HOMEPAGE=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^homepage = "\(.*\)"/\1/p; }' "$ROOT_CARGO_TOML" | head -1`

  dnl For arrays, extract the whole value (keep brackets)
  WS_KEYWORDS=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^keywords = //p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_CATEGORIES=`$SED -n '/\@<:@workspace\.package\@:>@/,/^\@<:@/{ s/^categories = //p; }' "$ROOT_CARGO_TOML" | head -1`

  dnl Extract workspace.dependencies
  WS_DEP_PROC_MACRO2=`$SED -n '/\@<:@workspace\.dependencies\@:>@/,/^\@<:@/{ s/^proc-macro2 = //p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_DEP_QUOTE=`$SED -n '/\@<:@workspace\.dependencies\@:>@/,/^\@<:@/{ s/^quote = //p; }' "$ROOT_CARGO_TOML" | head -1`
  WS_DEP_SYN=`$SED -n '/\@<:@workspace\.dependencies\@:>@/,/^\@<:@/{ s/^syn = //p; }' "$ROOT_CARGO_TOML" | head -1`

  AC_MSG_NOTICE([workspace.package: version=$WS_VERSION edition=$WS_EDITION])

  dnl Create vendor directories
  AS_MKDIR_P(["$VENDOR_OUT/miniextendr-api"])
  AS_MKDIR_P(["$VENDOR_OUT/miniextendr-macros"])
  AS_MKDIR_P(["$VENDOR_OUT/miniextendr-lint"])
  AS_MKDIR_P(["$VENDOR_OUT/miniextendr-engine"])

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

  dnl Escape special chars for sed replacement using | delimiter (handles URLs with /)
  dnl Escape & and | characters
  escape_sed_repl () {
    printf '%s' "$1" | $SED 's/\@<:@&|\@:>@/\\&/g'
  }
  WS_REPOSITORY_ESC=`escape_sed_repl "$WS_REPOSITORY"`
  WS_HOMEPAGE_ESC=`escape_sed_repl "$WS_HOMEPAGE"`
  WS_KEYWORDS_ESC=`escape_sed_repl "$WS_KEYWORDS"`
  WS_CATEGORIES_ESC=`escape_sed_repl "$WS_CATEGORIES"`
  WS_DEP_PROC_MACRO2_ESC=`escape_sed_repl "$WS_DEP_PROC_MACRO2"`
  WS_DEP_QUOTE_ESC=`escape_sed_repl "$WS_DEP_QUOTE"`
  WS_DEP_SYN_ESC=`escape_sed_repl "$WS_DEP_SYN"`

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
      AC_MSG_ERROR([$_crate/Cargo.toml still contains '.workspace = true' after patching; incomplete workspace inheritance removal])
    fi
  done

  AC_MSG_NOTICE([miniextendr crates synced to $VENDOR_OUT])
else
  AC_MSG_RESULT([not found, using existing vendor])
  dnl Sources not found - assume we're building from tarball with pre-vendored crates
  if test ! -d "$VENDOR_OUT/miniextendr-api"; then
    AC_MSG_WARN([vendor/miniextendr-api not found - build may fail])
  fi
fi

dnl Source replacement config: only use vendored sources on CRAN (offline builds)
dnl Dev mode uses network for crates.io, path deps for miniextendr-*
AS_IF([test "$NOT_CRAN" = "true"], [
  CARGO_SOURCE_REPLACE=""
], [
  CARGO_SOURCE_REPLACE='[[source.crates-io]]
replace-with = "vendored-sources"

[[source.vendored-sources]]
directory = "'"$VENDOR_OUT"'"'
])
AC_SUBST([CARGO_SOURCE_REPLACE])

dnl ---- output files ----
dnl Create .cargo directory (it's a hidden dir not included in tarball)
AS_MKDIR_P([src/rust/.cargo])

AC_CONFIG_FILES([
  src/rust/Cargo.toml
  src/rust/.cargo/config.toml:src/rust/cargo-config.toml.in
  src/Makevars
  src/entrypoint.c
])

dnl ---- AC_CONFIG_COMMANDS run after AC_CONFIG_FILES generates files ----
dnl Order matters: lockfile-compat, then cargo-vendor, then patch-staticlib-name

dnl 1) Ensure Cargo.lock is compatible with the installed cargo version.
dnl    Older cargo releases (e.g. 1.75) cannot read lockfile version 4.
AC_CONFIG_COMMANDS([cargo-lockfile-compat],
[
  LOCKFILE_PATH="src/rust/Cargo.lock"
  if test -f "$LOCKFILE_PATH"; then
    LOCKFILE_VERSION=`"$SED" -n 's/^version = //p' "$LOCKFILE_PATH" | head -n1`
  else
    LOCKFILE_VERSION=""
  fi

  CARGO_VERSION=`$CARGO_CMD --version | "$SED" 's/cargo \(@<:@0-9@:>@*\.@<:@0-9@:>@*\.@<:@0-9@:>@*\).*/\1/'`
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
  dnl Only vendor in dev mode (NOT_CRAN=true)
  if test "$NOT_CRAN" != "true"; then
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
    _new=`cksum < "$LOCKFILE" | awk '{print $1":"$2}'`
    _old=`cat "$STAMP" 2>/dev/null || true`

    _need=0
    if test "$FORCE_VENDOR" = "1"; then
      _need=1
    elif test ! -f "$STAMP"; then
      _need=1
    elif test "x$_new" != "x$_old"; then
      _need=1
    fi

    if test $_need -eq 1; then
      echo "configure: running cargo vendor (Cargo.lock changed or vendor missing)"
      mkdir -p "$VENDOR_OUT"
      $CARGO_CMD vendor \
        --manifest-path "$MANIFEST" \
        --sync "$ROOT_MINIEXTENDR_REPO/Cargo.toml" \
        --sync "$ROOT_MINIEXTENDR_REPO/miniextendr-bench/Cargo.toml" \
        "$VENDOR_OUT"
      if test $? -ne 0; then
        echo "configure: error: cargo vendor failed" >&2
        exit 1
      fi
      echo "$_new" > "$STAMP"
    else
      echo "configure: skipping cargo vendor (Cargo.lock unchanged)"
    fi
  fi
],
[CARGO_CMD="$CARGO_CMD" NOT_CRAN="$NOT_CRAN" FORCE_VENDOR="$FORCE_VENDOR" VENDOR_OUT="$VENDOR_OUT" ROOT_MINIEXTENDR_REPO="$root_miniextendr_repo"])

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
  _pkgid_output=`$CARGO_CMD pkgid $_pkgid_flags --manifest-path src/rust/Cargo.toml`
  _pkgid_status=$?
  if test $_pkgid_status -ne 0; then
    echo "configure: error: cargo pkgid failed with status $_pkgid_status" >&2
    echo "configure: cargo pkgid output: $_pkgid_output" >&2
    exit 1
  fi
  REAL_STATICLIB_NAME=`echo "$_pkgid_output" | "$SED" 's/.*#\(@<:@^@@:>@*\)@.*/\1/' | tr '-' '_'`
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
