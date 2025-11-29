AC_INIT([rpkg], [1.0])

dnl ensure R_HOME is set
: ${R_HOME=`R RHOME`}

dnl Set NOT_CRAN=true if not already set (enables vendoring in Makevars)
: ${NOT_CRAN="true"}
export NOT_CRAN

dnl Set cargo offline flag based on NOT_CRAN
dnl - NOT_CRAN=true (dev): no --offline, regenerate lockfile
dnl - NOT_CRAN not true (CRAN): use --offline, don't regenerate lockfile
if test "$NOT_CRAN" = "true"; then
  CARGO_OFFLINE_FLAG=""
  GENERATE_LOCKFILE="yes"
else
  CARGO_OFFLINE_FLAG="--offline"
  GENERATE_LOCKFILE="no"
fi
AC_SUBST([CARGO_OFFLINE_FLAG])
AC_SUBST([GENERATE_LOCKFILE])
AC_SUBST([NOT_CRAN])

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

dnl ---- vendoring (in inst/vendor for self-contained builds) ----
VENDOR_OUT="$abs_top_srcdir/inst/vendor"
SYNC1="--sync=$abs_rpkg_src/rust/Cargo.toml"
SYNC2="--sync=$VENDOR_OUT/miniextendr-api/Cargo.toml"
SYNC3="--sync=$VENDOR_OUT/miniextendr-macros/Cargo.toml"
VENDOR_SYNC_FLAGS="$SYNC1 $SYNC2 $SYNC3"
AC_SUBST([VENDOR_SYNC_FLAGS])
AC_SUBST([VENDOR_OUT])

dnl ---- output files ----
AC_CONFIG_FILES([
  src/rust/Cargo.toml
  src/rust/.cargo/config.toml
  src/Makevars
  src/entrypoint.c
])

dnl Also install a copy at top-level .cargo for dev ergonomics
dnl this is specific to miniextendr repository

AC_OUTPUT

dnl Keep root .cargo in sync when running configure inside the repo.
if test -d "$ROOT_MINIEXTENDR_REPO/.git"; then
  mkdir -p "$ROOT_MINIEXTENDR_REPO/.cargo" \
    || AC_MSG_ERROR([cannot mkdir $ROOT_MINIEXTENDR_REPO/.cargo])
  cp "$ABS_RPKG_SRCDIR/rust/.cargo/config.toml" \
    "$ROOT_MINIEXTENDR_REPO/.cargo/config.toml"
fi
