AC_INIT([rpkg])
: ${R_HOME=`R RHOME`}
AC_CONFIG_SRCDIR([src/entrypoint.c])

abs_top_srcdir=`(cd "$srcdir" && pwd)`
abs_srcdir="$abs_top_srcdir/src"
AC_SUBST([ABS_SRCDIR], ["$abs_srcdir"])

# Decide the cargo target dir: honour existing env vars if present
if test -n "$CARGO_BUILD_TARGET_DIR"; then
  cargo_target_dir="$CARGO_BUILD_TARGET_DIR"
elif test -n "$CARGO_TARGET_DIR"; then
  cargo_target_dir="$CARGO_TARGET_DIR"
else
  cargo_target_dir="$abs_srcdir/rust/target"
fi
AC_SUBST([RUST_TARGET_DIR], ["$cargo_target_dir"])
AC_SUBST([ENV_RUSTFLAGS], ["$RUSTFLAGS"])

# Collect Cargo-generated .d files so Makevars doesn't need to use wildcard
# instead of _just_ looking at release-profile, look at all subfolders of target and their deps files.
cargo_dep_files=""
if test -d "$cargo_target_dir"; then
  cargo_dep_files="$(
    find "$cargo_target_dir" -type f -name '*.d' -print0 \
      | xargs -0 printf ' %s'
  )"
fi
AC_SUBST([CARGO_DEP_FILES], ["$cargo_dep_files"])

AC_CONFIG_FILES([src/Makevars])
AC_OUTPUT