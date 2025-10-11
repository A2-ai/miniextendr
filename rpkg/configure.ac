AC_INIT([rpkg])
: ${R_HOME=`R RHOME`}                 dnl ensure R_HOME is set
AC_CONFIG_SRCDIR([src/entrypoint.c])  dnl safety check

dnl ---- canonical paths ----
abs_top_srcdir=`(cd "$srcdir" && pwd)`
abs_srcdir="$abs_top_srcdir/src"
AC_SUBST([ABS_SRCDIR], ["$abs_srcdir"])

dnl ---- Cargo configuration ----
AC_ARG_VAR([CARGO_TARGET_DIR], [Cargo target directory (default: src/rust/target)])
: ${CARGO_TARGET_DIR="$abs_srcdir/rust/target"}
AC_SUBST([CARGO_TARGET_DIR])

AC_ARG_VAR([CARGO_PROFILE], [Cargo build profile (debug or release)])
: ${CARGO_PROFILE="release"}
AC_SUBST([CARGO_PROFILE])

# TODO: read this from Cargo.toml manually.. or somewhere else
AC_ARG_VAR([CARGO_STATICLIB_NAME], [Name of Rust static library])
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

dnl ---- output files ----
AC_CONFIG_FILES([
  src/Makevars
  .cargo/config.toml
])

AC_CONFIG_COMMANDS([postcopy], [
  mkdir -p ../.cargo 2>/dev/null || true
  cp .cargo/config.toml ../.cargo/config.toml 2>/dev/null || true
])

AC_OUTPUT