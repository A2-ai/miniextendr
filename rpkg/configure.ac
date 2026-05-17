AC_INIT([miniextendr], [0.1.0])

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

dnl ---- canonical paths ----
abs_top_srcdir="$(cd "$srcdir" && pwd)"
AC_SUBST([ABS_TOP_SRCDIR], ["$abs_top_srcdir"])
abs_rpkg_src="$abs_top_srcdir/src"
AC_SUBST([ABS_RPKG_SRCDIR], ["$abs_rpkg_src"])
RUST_SRC_DIR="$abs_rpkg_src/rust"

dnl ---- install-mode self-repair ----
dnl Goal: a tarball that arrives missing inst/vendor.tar.xz becomes self-repairing
dnl at install time, so end users (who don't have `just`) get an offline-mode
dnl build automatically without manual vendoring.
dnl
dnl Triggers cargo-revendor when ALL of these hold:
dnl   1. inst/vendor.tar.xz is absent (nothing to install offline from)
dnl   2. cargo-revendor is on PATH (end user has the toolchain to do it)
dnl   3. we are NOT in a developer source tree (no .git ancestor)
dnl
dnl Skipping in git-tracked source trees keeps `just configure` and
dnl `bash ./configure` from devsource fast: the dev workflow uses explicit
dnl `just vendor` / `miniextendr_vendor()` to produce the tarball when needed,
dnl and dev iteration runs in source-mode against monorepo path overrides.
dnl
dnl Failure mode: if cargo-revendor itself fails, fall through to source mode
dnl rather than aborting — installs without network or without a working cargo
dnl-revendor still proceed via cargo's network resolution at compile time.
dnl
dnl On CRAN's offline build farm the auto-vendor branch is never taken
dnl (cargo-revendor isn't installed there), so a maintainer who ships a tarball
dnl missing inst/vendor.tar.xz fails CRAN's offline check loudly. That is the
dnl intended canary.
SOURCE_IS_GIT=false
_walk_dir="$abs_top_srcdir"
while test -n "$_walk_dir" && test "$_walk_dir" != "/"; do
  if test -e "$_walk_dir/.git"; then
    SOURCE_IS_GIT=true
    break
  fi
  _walk_dir=`dirname "$_walk_dir"`
done

if test ! -f "$abs_top_srcdir/inst/vendor.tar.xz" \
   && test "$SOURCE_IS_GIT" = "false" \
   && command -v cargo-revendor >/dev/null 2>&1; then
  AC_MSG_NOTICE([auto-vendor: inst/vendor.tar.xz absent, running cargo-revendor])
  mkdir -p "$abs_top_srcdir/inst"
  if (cd "$abs_top_srcdir" && cargo revendor \
        --manifest-path src/rust/Cargo.toml \
        --output vendor \
        --compress inst/vendor.tar.xz \
        --blank-md --source-marker --force) >&2; then
    AC_MSG_NOTICE([auto-vendor: produced inst/vendor.tar.xz])
  else
    AC_MSG_WARN([auto-vendor: cargo-revendor failed; falling through to source mode])
    rm -f "$abs_top_srcdir/inst/vendor.tar.xz"
  fi
fi

dnl ---- install-mode detection ----
dnl Single signal: presence of inst/vendor.tar.xz (possibly produced by the
dnl self-repair block above, by bootstrap.R at build time, or by an explicit
dnl `just vendor` / `miniextendr_vendor()` call).
dnl   Tarball install (R CMD INSTALL <built-tarball>): tarball is in the
dnl     artifact. configure unpacks it, writes a vendored cargo source
dnl     replacement, and builds offline.
dnl   Source install (R CMD INSTALL ., devtools::install, install_github, etc.,
dnl     in a git-tracked source tree): no tarball, no auto-vendor. configure
dnl     leaves vendoring alone and lets cargo resolve deps over the network
dnl     (or via monorepo path overrides — see below).
dnl
dnl See docs/CRAN_COMPATIBILITY.md for the full decision table.
if test -f "$abs_top_srcdir/inst/vendor.tar.xz"; then
  IS_TARBALL_INSTALL=true
  _install_mode_label="tarball install (offline, vendored)"
else
  IS_TARBALL_INSTALL=false
  _install_mode_label="source install (cargo network)"
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
dnl rpkg is a demo package that demonstrates ALL miniextendr features.
dnl Enable all optional features by default. Advanced users can override
dnl via the CARGO_FEATURES environment variable.
dnl
dnl Note: 'nonapi' feature causes R CMD check warnings/errors (non-API calls).
dnl Only enable when you specifically need those features and understand the risks.
AC_ARG_VAR([CARGO_FEATURES], [Comma-separated cargo features to enable (e.g., "nonapi,rayon")])

if test -z "${CARGO_FEATURES+x}"; then
  if test -f "${srcdir}/tools/detect-features.R"; then
    CARGO_FEATURES=$("${R_HOME}/bin/Rscript" "${srcdir}/tools/detect-features.R" 2>/dev/null || echo "")
    if test -n "$CARGO_FEATURES"; then
      AC_MSG_NOTICE([Auto-detected features: $CARGO_FEATURES])
    fi
  fi
  if test -z "$CARGO_FEATURES"; then
    CARGO_FEATURES="worker-thread,rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,tinyvec,raw_conversions,vctrs,borsh,arrow,datafusion,log"
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

dnl ---- wasm32 / webR install detection ----
dnl When R CMD INSTALL passes CC=emcc (webR's Makevars injects this), we're
dnl cross-compiling for wasm32. Override the Cargo target, force nightly only
dnl if the user hasn't pinned RUST_TOOLCHAIN, and validate that the
dnl pre-generated wasm_registry.rs is present.
AC_MSG_CHECKING([whether CC is emscripten (webR/wasm32 install)])
case "${CC}" in
  *emcc*|*em++*)
    is_wasm_install=true
    CARGO_BUILD_TARGET=wasm32-unknown-emscripten
    if test -z "$RUST_TOOLCHAIN"; then
      RUST_TOOLCHAIN="+nightly"
    fi
    CARGO_BUILD_STD_FLAG="-Z build-std=std,panic_abort"
    AC_MSG_RESULT([yes])
    ;;
  *)
    is_wasm_install=false
    CARGO_BUILD_STD_FLAG=
    AC_MSG_RESULT([no])
    ;;
esac
if test "$is_wasm_install" = true; then
  if test ! -f "$srcdir/src/rust/wasm_registry.rs"; then
    AC_MSG_ERROR([wasm32 install requires src/rust/wasm_registry.rs.
Run 'R CMD INSTALL' (or 'just rcmdinstall') on the host first to regenerate
it from the cdylib slice contents, then ship the source tree to the wasm builder.])
  fi
fi
AC_SUBST([IS_WASM_INSTALL], [$is_wasm_install])
AC_SUBST([CARGO_BUILD_STD_FLAG])
dnl CARGO_BUILD_TARGET is AC_SUBST'd later in the Cargo target/profile section.

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

AC_ARG_VAR([CARGO_HOME], [cargo registry/cache directory (defaults to ~/.cargo; \
  useful in restricted environments where HOME is read-only)])
: ${CARGO_HOME=""}
AC_SUBST([CARGO_HOME])
AS_IF([test -n "$CARGO_HOME"], [AC_MSG_NOTICE([CARGO_HOME           = $CARGO_HOME])])

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
AS_IF([test "$IS_WASM_INSTALL" = "true"],  [AC_MSG_NOTICE([IS_WASM_INSTALL      = true (CC=emcc detected)])])
AS_IF([test -n "$CARGO_BUILD_STD_FLAG"],   [AC_MSG_NOTICE([CARGO_BUILD_STD_FLAG = $CARGO_BUILD_STD_FLAG])])

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
NATIVE_PKG_CPPFLAGS=""

CLI_INCLUDE=$("${R_HOME}/bin/Rscript" -e "cat(system.file('include', package='cli'))")
if test -n "$CLI_INCLUDE" && test -d "$CLI_INCLUDE"; then
  NATIVE_PKG_CPPFLAGS="$NATIVE_PKG_CPPFLAGS -I$CLI_INCLUDE"
  AC_MSG_NOTICE([cli include: $CLI_INCLUDE])
else
  AC_MSG_WARN([cli package not found — native CLI bindings will not compile])
fi
AC_SUBST([NATIVE_PKG_CPPFLAGS])

AC_CONFIG_SRCDIR([src/rust/lib.rs])

dnl ---- vendor directory paths (used in tarball mode only) ----
VENDOR_OUT="$abs_top_srcdir/vendor"
VENDOR_OUT_CARGO="$abs_top_srcdir_cargo/vendor"
AC_SUBST([VENDOR_OUT])
AC_SUBST([VENDOR_OUT_CARGO])

dnl ---- monorepo detection (for source-mode dev override) ----
dnl Walk up looking for the miniextendr workspace root.
dnl When found, configure writes a [patch."<git-url>"] block into
dnl .cargo/config.toml so cargo resolves the three workspace crates against
dnl local sources instead of fetching the git URL. Edits to
dnl miniextendr-{api,lint,macros} propagate immediately on the next build.
MONOREPO_ROOT=""
_probe="$abs_top_srcdir"
for _i in 1 2 3 4 5; do
  _probe="$(dirname "$_probe")"
  if test "$_probe" = "/" || test "$_probe" = "."; then break; fi
  if test -f "$_probe/miniextendr-api/Cargo.toml"; then
    MONOREPO_ROOT="$_probe"
    break
  fi
done

dnl Cargo path version (Windows-native if MSYS2)
MONOREPO_ROOT_CARGO="$MONOREPO_ROOT"
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    if test -n "$MONOREPO_ROOT" && test "x$CYGPATH" != "xno"; then
      MONOREPO_ROOT_CARGO="$($CYGPATH -m "$MONOREPO_ROOT")"
    fi
    ;;
esac

if test -n "$MONOREPO_ROOT"; then
  if test "$IS_TARBALL_INSTALL" = "true"; then
    AC_MSG_NOTICE([monorepo siblings    = $MONOREPO_ROOT (ignored — tarball mode takes precedence)])
  else
    AC_MSG_NOTICE([monorepo siblings    = $MONOREPO_ROOT (source-mode patch override active)])
  fi
fi

dnl ---- derive Cargo [env] block (R-ABI matching) ----
dnl Goal: every cargo invocation (local dev, CI, end-user R CMD INSTALL) inherits
dnl R-compatible toolchain settings so the Rust staticlib's ABI matches R's build.
dnl
dnl Shell env wins over `[env]` (no `force = true`) — users can still override
dnl via `MACOSX_DEPLOYMENT_TARGET=12.0 R CMD INSTALL .` etc.
dnl
dnl Per-platform derivations:
dnl   macOS   : MACOSX_DEPLOYMENT_TARGET (4-step fallback: env → R config → otool → arch default).
dnl   Windows : CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER (and aarch64) from rtools gcc when detected.
dnl   Linux   : conservative — only AR_<target>/RANLIB_<target> overrides if R's are non-default.
dnl   All     : CARGO_NET_GIT_FETCH_WITH_CLI = "true" (works around in-tree rustls/CA issues).
dnl
dnl We pre-compute derivation variables here so AC_CONFIG_COMMANDS just writes
dnl the lines out. The block is emitted as the FIRST table in each install-mode
dnl branch of .cargo/config.toml (TOML table-scoping requires it).

dnl Rust target triple for *_<target> environment variables. Prefer the user's
dnl explicit CARGO_BUILD_TARGET; otherwise ask rustc for its default host.
CARGO_ENV_RUST_TARGET="$CARGO_BUILD_TARGET"
if test -z "$CARGO_ENV_RUST_TARGET"; then
  CARGO_ENV_RUST_TARGET=`"$RUSTC" -vV 2>/dev/null | $SED -n 's/^host: //p'`
fi

dnl Uppercase + s/-/_/g, e.g. aarch64-apple-darwin -> AARCH64_APPLE_DARWIN.
CARGO_ENV_RUST_TARGET_UPPER=`echo "$CARGO_ENV_RUST_TARGET" | tr 'a-z-' 'A-Z_'`

dnl --- macOS: derive MACOSX_DEPLOYMENT_TARGET ----
dnl Derivation order (first match wins, logged via AC_MSG_NOTICE):
dnl   1. Shell env MACOSX_DEPLOYMENT_TARGET (caller's value)
dnl   2. -mmac(os|osx)-version-min=X.Y in R CMD config {CC,CXX,FC}
dnl   3. otool -l "$R_HOME/bin/exec/R" → LC_BUILD_VERSION minos
dnl   4. Arch default: aarch64 → 14.0, x86_64 → 11.0 (per R-admin §macOS).
CARGO_ENV_MACOS_DEPLOY_TARGET=""
CARGO_ENV_MACOS_DEPLOY_SOURCE=""
case "$host_os" in
  darwin*)
    dnl Step 1: shell env wins (cargo [env] no-force semantics mean we could
    dnl emit it and shell would still win, but skipping emission keeps the
    dnl config.toml output stable across invocations with different env vars).
    if test -n "${MACOSX_DEPLOYMENT_TARGET}"; then
      CARGO_ENV_MACOS_DEPLOY_TARGET="$MACOSX_DEPLOYMENT_TARGET"
      CARGO_ENV_MACOS_DEPLOY_SOURCE="shell env"
    fi

    dnl Step 2: parse R CMD config CC / CXX / FC. clang spells it
    dnl `-mmacos-version-min`; legacy gfortran / older clang use `-mmacosx-version-min`.
    if test -z "$CARGO_ENV_MACOS_DEPLOY_TARGET"; then
      for _cfg_key in CC CXX FC; do
        _flag_val=`"${R_HOME}/bin/R" CMD config "$_cfg_key" 2>/dev/null \
          | $SED -n 's/.*-mmac[ox]*-version-min=\(@<:@0-9@:>@*\.@<:@0-9@:>@*\).*/\1/p' \
          | head -n1`
        if test -n "$_flag_val"; then
          CARGO_ENV_MACOS_DEPLOY_TARGET="$_flag_val"
          CARGO_ENV_MACOS_DEPLOY_SOURCE="R CMD config $_cfg_key"
          break
        fi
      done
    fi

    dnl Step 3: otool -l on R binary. LC_BUILD_VERSION's `minos` field is
    dnl the authoritative target R was linked against.
    if test -z "$CARGO_ENV_MACOS_DEPLOY_TARGET"; then
      if command -v otool >/dev/null 2>&1; then
        _otool_val=`otool -l "${R_HOME}/bin/exec/R" 2>/dev/null \
          | awk '/LC_BUILD_VERSION/ {found=1} found && /minos/ {print $2; exit}'`
        if test -n "$_otool_val"; then
          CARGO_ENV_MACOS_DEPLOY_TARGET="$_otool_val"
          CARGO_ENV_MACOS_DEPLOY_SOURCE="otool LC_BUILD_VERSION"
        fi
      fi
    fi

    dnl Step 4: arch default. R-admin (Sec. "macOS") states current CRAN
    dnl targets are macOS 14.0 (arm64) and 11.0 (Intel).
    if test -z "$CARGO_ENV_MACOS_DEPLOY_TARGET"; then
      case "$host_cpu" in
        arm64|aarch64) CARGO_ENV_MACOS_DEPLOY_TARGET="14.0" ;;
        *)             CARGO_ENV_MACOS_DEPLOY_TARGET="11.0" ;;
      esac
      CARGO_ENV_MACOS_DEPLOY_SOURCE="arch default ($host_cpu)"
    fi

    AC_MSG_NOTICE([MACOSX_DEPLOYMENT_TARGET = $CARGO_ENV_MACOS_DEPLOY_TARGET (source: $CARGO_ENV_MACOS_DEPLOY_SOURCE)])
    ;;
esac

dnl --- Windows: derive rtools gcc linker path ----
dnl Strategy: R CMD config CC's first token is the C compiler path. If its
dnl sibling `gcc` (or `gcc.exe`) exists, use that as the cargo linker for
dnl x86_64-pc-windows-gnu / aarch64-pc-windows-gnu. This pins cargo to rtools'
dnl gcc rather than whichever gcc happens to be on PATH (MSYS2's, etc).
CARGO_ENV_WIN_LINKER=""
case "$host_os" in
  *msys*|*cygwin*|*mingw*)
    _r_cc=`"${R_HOME}/bin/R" CMD config CC 2>/dev/null | awk '{print $1}'`
    if test -n "$_r_cc"; then
      _r_cc_bindir=`dirname "$_r_cc"`
      for _gcc_cand in "$_r_cc_bindir/gcc.exe" "$_r_cc_bindir/gcc"; do
        if test -x "$_gcc_cand"; then
          dnl Cargo's [env] consumes the path raw; TOML needs forward slashes.
          dnl `cygpath -m` produces a Windows-native path with forward slashes,
          dnl which is exactly what cargo's TOML parser wants.
          _gcc_native="$_gcc_cand"
          if test "x$CYGPATH" != "xno"; then
            _gcc_native=`"$CYGPATH" -m "$_gcc_cand" 2>/dev/null || echo "$_gcc_cand"`
          fi
          CARGO_ENV_WIN_LINKER="$_gcc_native"
          break
        fi
      done
    fi
    if test -n "$CARGO_ENV_WIN_LINKER"; then
      AC_MSG_NOTICE([rtools linker        = $CARGO_ENV_WIN_LINKER])
    else
      AC_MSG_NOTICE([rtools linker        = (not detected — cargo will use default gcc on PATH)])
    fi
    ;;
esac

dnl --- AR / RANLIB overrides (all platforms, only if non-default) ----
dnl R may be configured with a non-default AR/RANLIB (e.g. a cross-toolchain
dnl prefix). Cargo's default is `ar` / `ranlib`, so we only emit overrides
dnl when R's values differ — otherwise we'd just add noise to the config.
CARGO_ENV_R_AR=`"${R_HOME}/bin/R" CMD config AR 2>/dev/null`
CARGO_ENV_R_RANLIB=`"${R_HOME}/bin/R" CMD config RANLIB 2>/dev/null`

dnl Individual env-derivation variables are propagated to AC_CONFIG_COMMANDS
dnl via the trailing variable-pass list, and the [env] block body is composed
dnl inline inside that shell context. Pre-composing here and passing a single
dnl newline-separated string trips on AC_CONFIG_COMMANDS' heredoc-style
dnl variable propagation (embedded `"` breaks the outer assignment quoting).

dnl ---- output files ----
dnl AC_CONFIG_FILES handles Makevars and the win.def. The cargo config is
dnl written by AC_CONFIG_COMMANDS below because its contents differ across
dnl the three install modes (tarball / source-monorepo / source-only).
AS_MKDIR_P([src/rust/.cargo])

AC_CONFIG_FILES([
  src/Makevars:src/Makevars.in
  src/$PACKAGE_NAME-win.def:src/win.def.in
])

dnl ---- 1) Tarball install: unpack inst/vendor.tar.xz ----
dnl Source mode is a no-op here. We unpack the vendor tarball before
dnl writing the cargo config so the [source.vendored-sources] directory
dnl actually exists by the time cargo runs.
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
    else
      echo "configure: tarball install — vendor/ already populated"
    fi
    dnl AUDIT-427: checksum strip removed. PR #408 rewrote cargo-revendor to
    dnl preserve the original `package` field in each .cargo-checksum.json
    dnl (matching the lockfile's `checksum = "..."` line). Cargo can therefore
    dnl verify checksums end-to-end; stripping them discards a valid integrity
    dnl check. See issue #427 and tools/lock-shape-check.R which explicitly
    dnl documents that checksum lines are allowed in tarball-shape locks.
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
dnl   [source."git+https://github.com/A2-ai/miniextendr"]
dnl                                           replace-with = "vendored-sources"
dnl   [source.vendored-sources]               directory = "<vendor>"
dnl   [build]                                 target-dir = "<rust-target>"
dnl
dnl Source install + monorepo:
dnl   [patch."https://github.com/A2-ai/miniextendr"]
dnl     miniextendr-api    = { path = "<root>/miniextendr-api" }
dnl     miniextendr-lint   = { path = "<root>/miniextendr-lint" }
dnl     miniextendr-macros = { path = "<root>/miniextendr-macros" }
dnl   [build]                                 target-dir = "<rust-target>"
dnl
dnl Source install + no monorepo:
dnl   [build]                                 target-dir = "<rust-target>"
dnl   (cargo follows the git URL declared in Cargo.toml)
dnl
dnl WARNING: stale inst/vendor.tar.xz hazard.
dnl   `just r-cmd-build` and `just r-cmd-check` create inst/vendor.tar.xz
dnl   transiently and remove it via `trap ... EXIT`.  If the process is killed
dnl   before the trap fires, the file is left behind.  Configure then detects
dnl   it and sets IS_TARBALL_INSTALL=true, which (a) writes a tarball-mode
dnl   config.toml here and (b) makes Makevars.in's post-build cleanup delete
dnl   src/rust/.cargo/ from the SOURCE TREE on the next `R CMD INSTALL .`.
dnl   Recovery: rm inst/vendor.tar.xz && bash ./configure
AC_CONFIG_COMMANDS([cargo-config],
[
  RPKG_CFG="src/rust/.cargo/config.toml"
  rm -f "$RPKG_CFG"

  dnl `[env]` is emitted as the FIRST table in every branch. TOML table-scoping
  dnl makes any later `[…]` heading end the previous table — so `[env]` must
  dnl precede `[source.*]` / `[patch.*]` / `[build]` to stay independent.
  dnl Cargo applies these to every cargo invocation in this package. Shell env
  dnl wins (no `force = true`), so users can override case-by-case.
  dnl
  dnl Derivation variables are passed individually via the variable-pass list
  dnl (see second arg) — composing the body here keeps quoting predictable.
  emit_env_block() {
    echo "@<:@env@:>@"
    if test -n "$CARGO_ENV_MACOS_DEPLOY_TARGET"; then
      echo "MACOSX_DEPLOYMENT_TARGET = \"$CARGO_ENV_MACOS_DEPLOY_TARGET\""
    fi
    if test -n "$CARGO_ENV_WIN_LINKER"; then
      case "$CARGO_ENV_RUST_TARGET_UPPER" in
        *AARCH64*WINDOWS*)
          echo "CARGO_TARGET_AARCH64_PC_WINDOWS_GNU_LINKER = \"$CARGO_ENV_WIN_LINKER\""
          ;;
        *)
          echo "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = \"$CARGO_ENV_WIN_LINKER\""
          ;;
      esac
    fi
    echo 'CARGO_NET_GIT_FETCH_WITH_CLI = "true"'
    if test -n "$CARGO_ENV_R_AR" && test "$CARGO_ENV_R_AR" != "ar"; then
      echo "AR_$CARGO_ENV_RUST_TARGET = \"$CARGO_ENV_R_AR\""
    fi
    if test -n "$CARGO_ENV_R_RANLIB" && test "$CARGO_ENV_R_RANLIB" != "ranlib"; then
      echo "RANLIB_$CARGO_ENV_RUST_TARGET = \"$CARGO_ENV_R_RANLIB\""
    fi
    echo ""
  }

  if test "$IS_TARBALL_INSTALL" = "true"; then
    {
      echo "# Autogenerated by configure — DO NOT EDIT"
      echo "# Tarball install: redirect crates.io and the miniextendr git source"
      echo "# to the unpacked inst/vendor.tar.xz."
      echo ""
      emit_env_block
      echo "@<:@source.crates-io@:>@"
      echo 'replace-with = "vendored-sources"'
      echo ""
      dnl Scan Cargo.toml for git URLs and emit one [source."git+<url>"] per
      dnl unique URL. Keeps the config self-consistent if more git deps are
      dnl added later.
      for _url in $("$SED" -n 's/.*git = "\(https:\/\/@<:@^"@:>@*\)".*/\1/p' src/rust/Cargo.toml | sort -u); do
        echo "@<:@source.\"git+$_url\"@:>@"
        echo "git = \"$_url\""
        echo 'replace-with = "vendored-sources"'
        echo ""
      done
      echo "@<:@source.vendored-sources@:>@"
      echo "directory = \"$VENDOR_OUT_CARGO\""
      echo ""
      echo "@<:@build@:>@"
      echo "target-dir = \"$CARGO_TARGET_DIR_CARGO\""
    } > "$RPKG_CFG"
    echo "configure: wrote $RPKG_CFG (tarball mode)"

  elif test -n "$MONOREPO_ROOT"; then
    {
      echo "# Autogenerated by configure — DO NOT EDIT"
      echo "# Source install (monorepo): patch the miniextendr git source to"
      echo "# resolve against local workspace crates."
      echo ""
      emit_env_block
      echo '@<:@patch."https://github.com/A2-ai/miniextendr"@:>@'
      echo "miniextendr-api    = { path = \"$MONOREPO_ROOT_CARGO/miniextendr-api\" }"
      echo "miniextendr-lint   = { path = \"$MONOREPO_ROOT_CARGO/miniextendr-lint\" }"
      echo "miniextendr-macros = { path = \"$MONOREPO_ROOT_CARGO/miniextendr-macros\" }"
      echo ""
      echo "@<:@build@:>@"
      echo "target-dir = \"$CARGO_TARGET_DIR_CARGO\""
    } > "$RPKG_CFG"
    echo "configure: wrote $RPKG_CFG (source mode, monorepo override)"

  else
    {
      echo "# Autogenerated by configure — DO NOT EDIT"
      echo "# Source install (no monorepo): cargo follows the git URL"
      echo "# declared in Cargo.toml. First build downloads, subsequent"
      echo "# builds use cargo's git cache."
      echo ""
      emit_env_block
      echo "@<:@build@:>@"
      echo "target-dir = \"$CARGO_TARGET_DIR_CARGO\""
    } > "$RPKG_CFG"
    echo "configure: wrote $RPKG_CFG (source mode, no monorepo siblings)"
  fi
],
[IS_TARBALL_INSTALL="$IS_TARBALL_INSTALL"
 MONOREPO_ROOT="$MONOREPO_ROOT"
 MONOREPO_ROOT_CARGO="$MONOREPO_ROOT_CARGO"
 VENDOR_OUT_CARGO="$VENDOR_OUT_CARGO"
 CARGO_TARGET_DIR_CARGO="$CARGO_TARGET_DIR_CARGO"
 CARGO_ENV_MACOS_DEPLOY_TARGET="$CARGO_ENV_MACOS_DEPLOY_TARGET"
 CARGO_ENV_WIN_LINKER="$CARGO_ENV_WIN_LINKER"
 CARGO_ENV_RUST_TARGET="$CARGO_ENV_RUST_TARGET"
 CARGO_ENV_RUST_TARGET_UPPER="$CARGO_ENV_RUST_TARGET_UPPER"
 CARGO_ENV_R_AR="$CARGO_ENV_R_AR"
 CARGO_ENV_R_RANLIB="$CARGO_ENV_R_RANLIB"
 SED="$SED"])

dnl ---- 3) Cargo.lock shape check (tarball mode only) ----
dnl In tarball mode, the lock must be in tarball-shape: no checksum = lines and
dnl no path+... sources for framework crates. Drift is fatal at offline install
dnl time; we catch it here before cargo starts building.
dnl In source mode, cargo silently rewrites the lock; drift is tolerated.
AC_CONFIG_COMMANDS([lock-shape-check],
[
  if test "$IS_TARBALL_INSTALL" = "true"; then
    if "${R_HOME}/bin/Rscript" tools/lock-shape-check.R "tarball" "src/rust/Cargo.lock"; then
      echo "configure: lock-shape OK"
    else
      AC_MSG_ERROR([Cargo.lock shape drift detected in tarball mode; see message above])
    fi
  fi
],
[IS_TARBALL_INSTALL="$IS_TARBALL_INSTALL"
 R_HOME="$R_HOME"])

dnl ---- 4) Cargo.lock compatibility ----
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

dnl ---- 5) Ensure Cargo.lock exists; touch Cargo.toml to force rebuild ----
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
