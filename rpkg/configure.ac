AC_INIT([rpkg], [1.0])
: ${R_HOME=`R RHOME`}                 dnl ensure R_HOME is set
AC_CONFIG_SRCDIR([src/entrypoint.c])  dnl safety check

# TODO: write a rust analyzer friendly command.
# see https://rust-analyzer.github.io/book/configuration.html?highlight=command#check.overrideCommand



dnl ---- canonical paths ----
abs_top_srcdir=`(cd "$srcdir" && pwd)`
abs_srcdir="$abs_top_srcdir/src"
AC_SUBST([ABS_SRCDIR], ["$abs_srcdir"])

dnl ---- Cargo configuration ----
# TODO: Add all these to the R build process
# https://doc.rust-lang.org/cargo/reference/environment-variables.html
AC_ARG_VAR([CARGO_TARGET_DIR], [Cargo target directory (default: src/rust/target)])
: ${CARGO_TARGET_DIR="$abs_srcdir/rust/target"}
AC_SUBST([CARGO_TARGET_DIR])

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (debug or release)])
: ${CARGO_PROFILE="release"}
AC_SUBST([CARGO_PROFILE])

# TODO: read this from Cargo.toml manually.. or somewhere else
AC_ARG_VAR([CARGO_STATICLIB_NAME], [Name of Rust static library])
# TODO: use cargo metadata to figure out what the library or libraries ought to be named!
: ${CARGO_STATICLIB_NAME="juice"}
AC_SUBST([CARGO_STATICLIB_NAME])


AC_ARG_VAR([ENV_RUSTFLAGS], [Rust compiler flags passed as RUSTFLAGS])
: ${ENV_RUSTFLAGS="$RUSTFLAGS"}
AC_SUBST([ENV_RUSTFLAGS])

dnl ---- user feedback ----
AC_MSG_NOTICE([R_HOME                = $R_HOME])
AC_MSG_NOTICE([CARGO_TARGET_DIR      = $CARGO_TARGET_DIR])
AC_MSG_NOTICE([CARGO_PROFILE         = $CARGO_PROFILE])
AC_MSG_NOTICE([CARGO_STATICLIB_NAME  = $CARGO_STATICLIB_NAME])
AC_MSG_NOTICE([ENV_RUSTFLAGS         = $ENV_RUSTFLAGS])

# TODO
dnl Create a document.rs.in file and replace certain places with the crate name.
# Rust-safe tarname (hyphens → underscores)
pkg_rs=`echo "$PACKAGE_TARNAME" | sed 's/-/_/g'`
AC_SUBST([PACKAGE_TARNAME_RS], [$pkg_rs])

# Ensure template exists
AS_IF([test ! -f "$srcdir/src/rust/document.rs.in"], [
  mkdir -p "$srcdir/src/rust"
  cat >"$srcdir/src/rust/document.rs.in" <<'EOF'
use juice::R_WRAPPERS_DEPS_@PACKAGE_TARNAME_RS@ as DEPS;
use juice::R_WRAPPERS_PARTS_@PACKAGE_TARNAME_RS@ as PARTS;

fn main() {
    let dep_flat: Vec<&'static str> = DEPS.concat();
    let roots: @<:@&@<:@&'static str@:>@; 2@:>@ = @<:@PARTS, &dep_flat@:>@;

    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("miniextendr_wrappers.R")
        .unwrap();

    let mut seen = std::collections::HashSet::new();
    for group in roots {
        for &s in group {
            if seen.insert(s) {
                use std::io::Write;
                f.write_all(s.as_bytes()).unwrap();
                f.write_all(b"\n\n").unwrap();
            }
        }
    }
}
EOF
])

# Generate the concrete file with substitutions
AC_CONFIG_FILES([src/rust/document.rs])


dnl Create the src/entrypoint.c file. It is very barebone and just need to
dnl adjust to changing package name.

# Ensure entrypoint.c.in exists
AS_IF([test ! -f "$abs_srcdir/entrypoint.c.in"], [
  mkdir -p "$abs_srcdir"
  cat >"$abs_srcdir/entrypoint.c.in" <<'EOF'
#include <R.h>
#include <Rinternals.h>
#include <R_ext/Rdynload.h>

extern void R_init_@PACKAGE_TARNAME_RS@_miniextendr(DllInfo * dll);

void R_init_@PACKAGE_TARNAME_RS@(DllInfo *dll) {
    R_init_@PACKAGE_TARNAME_RS@_miniextendr(dll);

    R_useDynamicSymbols(dll, FALSE);
    R_forceSymbols(dll, TRUE);
}
EOF
])


# TODO
dnl Create a `justfile.in` that replaces hardcoded rpkg.

dnl ---- output files ----
AC_CONFIG_FILES([
  src/Makevars
  .cargo/config.toml
  src/entrypoint.c
])

AC_CONFIG_COMMANDS([postcopy], [
  mkdir -p ../.cargo 2>/dev/null || true
  cp .cargo/config.toml ../.cargo/config.toml 2>/dev/null || true
])

AC_OUTPUT