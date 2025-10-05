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
  cargo_target_dir="$abs_srcdir/target"
fi
AC_SUBST([RUST_TARGET_DIR], ["$cargo_target_dir"])
AC_SUBST([ENV_RUSTFLAGS], ["$RUSTFLAGS"])

# Collect Cargo-generated .d files so Makevars needn’t use wildcard
cargo_dep_files=""
if test -d "$cargo_target_dir/release"; then
  for f in "$cargo_target_dir"/release/*.d \
           "$cargo_target_dir"/release/deps/*.d; do
    test -f "$f" && cargo_dep_files="$cargo_dep_files $f"
  done
fi
AC_SUBST([CARGO_DEP_FILES], ["$cargo_dep_files"])

AC_CONFIG_FILES([src/Makevars])
AC_OUTPUT