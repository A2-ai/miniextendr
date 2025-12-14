AC_INIT([rpkg], [1.0])

dnl ensure R_HOME is set
: ${R_HOME=`R RHOME`}

dnl Set NOT_CRAN=true if not already set (enables vendoring in Makevars)
: ${NOT_CRAN="true"}
export NOT_CRAN

dnl Set cargo offline flag based on NOT_CRAN
dnl - NOT_CRAN=true (dev): no --offline
dnl - NOT_CRAN not true (CRAN): use --offline
if test "$NOT_CRAN" = "true"; then
  CARGO_OFFLINE_FLAG=""
else
  CARGO_OFFLINE_FLAG="--offline"
fi
AC_SUBST([CARGO_OFFLINE_FLAG])
AC_SUBST([NOT_CRAN])

dnl Set cargo features flag
dnl Use --all-features to enable all features (including nonapi for thread utilities)
CARGO_FEATURES_FLAG="--all-features"
AC_SUBST([CARGO_FEATURES_FLAG])

dnl Set C preprocessor flags to match Cargo features
dnl When using --all-features, define all feature macros for C code
CARGO_FEATURE_CPPFLAGS="-DCARGO_FEATURE_NONAPI"
AC_SUBST([CARGO_FEATURE_CPPFLAGS])

# TODO: write a rust analyzer friendly command.
# see https://rust-analyzer.github.io/book/configuration.html?highlight=command#check.overrideCommand


dnl ---- canonical paths ----
abs_top_srcdir=`(cd "$srcdir" && pwd)`
AC_SUBST([ABS_TOP_SRCDIR], ["$abs_top_srcdir"])
abs_rpkg_src="$abs_top_srcdir/src"
AC_SUBST([ABS_RPKG_SRCDIR], ["$abs_rpkg_src"])

dnl Canonical path two levels above src (rpkg/src → miniextendr)
root_miniextendr_repo=`(cd "$abs_rpkg_src/../.." && pwd)`
AC_SUBST([ROOT_MINIEXTENDR_REPO], ["$root_miniextendr_repo"])

dnl ---- tool discovery ----
AC_PATH_TOOL([CARGO],[cargo],[no])
AC_PATH_TOOL([RUSTC],[rustc],[no])
AS_IF([test "x$CARGO" = "xno"], [AC_MSG_ERROR([`cargo` not found])])
AS_IF([test "x$RUSTC" = "xno"], [AC_MSG_ERROR([`rustc` not found])])
AC_SUBST([CARGO])
AC_SUBST([RUSTC])

dnl Report rustc version (required by CRAN for Rust packages)
RUSTC_VERSION=`$RUSTC --version`
AC_MSG_NOTICE([using $RUSTC_VERSION])

dnl Allow explicit toolchain (e.g. +nightly); default empty (use default toolchain)
AC_ARG_VAR([RUST_TOOLCHAIN],[Rust toolchain selector like '+stable' or '+nightly'])
: ${RUST_TOOLCHAIN=""}
AC_SUBST([RUST_TOOLCHAIN])

dnl ---- Cargo configuration ----
AC_ARG_VAR([CARGO_TARGET_DIR], [Cargo target directory (default: src/rust/target)])
: ${CARGO_TARGET_DIR="$abs_rpkg_src/rust/target"}
AC_SUBST([CARGO_TARGET_DIR])

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (debug or release)])
: ${CARGO_PROFILE="release"}
AC_SUBST([CARGO_PROFILE])

dnl TODO: read this from Cargo.toml manually.. or somewhere else
AC_ARG_VAR([CARGO_STATICLIB_NAME], [Name of Rust static library])
dnl TODO: use cargo metadata to figure out what the library or libraries ought to be named!
: ${CARGO_STATICLIB_NAME="juice"}
AC_SUBST([CARGO_STATICLIB_NAME])

AC_ARG_VAR([ENV_RUSTFLAGS], [Rust compiler flags passed as RUSTFLAGS])
: ${ENV_RUSTFLAGS="$RUSTFLAGS"}
AC_SUBST([ENV_RUSTFLAGS])

AC_ARG_VAR([CARGO_BUILD_TARGET], [Cargo build target triple (e.g. aarch64-apple-darwin)])
: ${CARGO_BUILD_TARGET=""}
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

dnl ---- vendoring (in src/vendor for builds, cleaned up post-install) ----
VENDOR_OUT="$abs_rpkg_src/vendor"
AC_SUBST([VENDOR_OUT])

dnl Source locations for miniextendr crates (relative to rpkg root)
dnl Note: miniextendr-engine is NOT vendored - it's only for standalone binaries
dnl that embed R, not for R packages.
MINIEXTENDR_API_SRC="$abs_top_srcdir/../miniextendr-api"
MINIEXTENDR_MACROS_SRC="$abs_top_srcdir/../miniextendr-macros"

dnl Sync miniextendr crates from monorepo to vendor (if sources exist)
dnl This handles workspace inheritance by patching Cargo.toml during copy
AC_MSG_CHECKING([for miniextendr source crates])
if test -d "$MINIEXTENDR_API_SRC" && test -d "$MINIEXTENDR_MACROS_SRC"; then
  AC_MSG_RESULT([found, syncing to vendor])

  dnl Create vendor directories
  mkdir -p "$VENDOR_OUT/miniextendr-api"
  mkdir -p "$VENDOR_OUT/miniextendr-macros"

  dnl Sync miniextendr-api (excluding target, .git, Cargo.toml)
  rsync -a --delete --exclude='target' --exclude='.git' --exclude='Cargo.toml' \
    "$MINIEXTENDR_API_SRC/" "$VENDOR_OUT/miniextendr-api/"

  dnl Sync miniextendr-macros (excluding target, .git, Cargo.toml)
  rsync -a --delete --exclude='target' --exclude='.git' --exclude='Cargo.toml' \
    "$MINIEXTENDR_MACROS_SRC/" "$VENDOR_OUT/miniextendr-macros/"

  dnl Clean up CRAN-unfriendly hidden files (e.g., .DS_Store, .gitignore)
  find "$VENDOR_OUT" -name ".*" -type f -delete 2>/dev/null || true
  find "$VENDOR_OUT" -name ".*" -type d -exec rm -rf {} + 2>/dev/null || true

  dnl Patch Cargo.toml to remove workspace inheritance
  sed -e 's/edition\.workspace = true/edition = "2024"/' \
      -e 's/version\.workspace = true/version = "0.1.0"/' \
      "$MINIEXTENDR_API_SRC/Cargo.toml" > "$VENDOR_OUT/miniextendr-api/Cargo.toml"

  sed -e 's/edition\.workspace = true/edition = "2024"/' \
      -e 's/version\.workspace = true/version = "0.1.0"/' \
      "$MINIEXTENDR_MACROS_SRC/Cargo.toml" > "$VENDOR_OUT/miniextendr-macros/Cargo.toml"

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
if test "$NOT_CRAN" = "true"; then
  CARGO_SOURCE_REPLACE=""
else
  CARGO_SOURCE_REPLACE="[source.crates-io]
replace-with = \"vendored-sources\"

[source.vendored-sources]
directory = \"$VENDOR_OUT\""
fi
AC_SUBST([CARGO_SOURCE_REPLACE])

dnl ---- output files ----
dnl Create .cargo directory (it's a hidden dir not included in tarball)
mkdir -p src/rust/.cargo

AC_CONFIG_FILES([
  src/rust/Cargo.toml
  src/rust/.cargo/config.toml:src/rust/cargo-config.toml.in
  src/Makevars
  src/entrypoint.c
])

dnl Also install a copy at top-level .cargo for dev ergonomics
dnl this is specific to miniextendr repository

AC_OUTPUT
