# Calling R Package C APIs from Rust via bindgen

This document explains how to use C headers from installed R packages in a
miniextendr Rust crate. The technique uses `bindgen` to generate Rust FFI
bindings at development time, and R's standard build system to compile the
required C shim files.

## Overview

Many R packages expose C APIs via `inst/include/` headers. These headers
typically use `R_GetCCallable()` to resolve function pointers at runtime,
wrapped in `static R_INLINE` functions. Examples: `cli` (progress bars),
`nanoarrow` (Arrow C data interface), `vctrs` (vector types), `processx`
(process management).

The integration has three layers:

1. **bindgen** (development time): parses the C headers and generates:
   - A Rust FFI module (`*_ffi.rs`) with `extern "C"` declarations
   - A C shim file (`*_static_wrappers.c`) that wraps `static inline` functions
     into normal linkable symbols

2. **R's build system** (install time): compiles the C shims into `.o` files
   alongside `stub.c` and any other C sources in `src/`

3. **Makevars**: passes all compiled `.o` files to cargo as link arguments so
   both the cdylib (for wrapper generation) and the staticlib (for the final
   `.so`) can resolve the shim symbols

## Quick Start: `minirextendr::use_native_package()`

For the most common pattern (a package whose C API is a set of
`R_GetCCallable` calls wrapped in `static R_INLINE` functions), the
minirextendr helper automates every step below:

```r
library(minirextendr)

# Add cli's progress-bar C API to the current miniextendr package.
use_native_package("cli", headers = "cli/progress.h")

# Sanity-check before committing.
check_native_package("cli")
```

`use_native_package()`:

1. Adds `cli` to `LinkingTo:` in `DESCRIPTION`
2. Writes `src/cli_wrapper.h` and runs bindgen to produce
   `src/rust/native/cli_ffi.rs` + `src/cli_static_wrappers.c`
3. Patches `src/Makevars.in` to include `$(NATIVE_PKG_CPPFLAGS)` and to
   forward every `OBJECTS` entry to cargo as a `-C link-arg=…`
4. Probes the target package: if it's pure C, the shim is built in C
   mode (so `--wrap-static-fns` actually emits the shim file); C++
   packages fall through to C++17/C++14 with wrappers skipped
5. Walks `LinkingTo:` recursively so transitive headers resolve

`check_native_package()` re-runs the detection step and reports whether
the corpus-wide bindgen probe (308/594 CRAN packages parse successfully
today) classifies this package as pure C, C++, or unparseable.

The rest of this document covers the manual workflow, useful when
`use_native_package()` isn't a fit (non-standard header layouts, C++
packages that also expose `static inline` functions, custom link flags).

## Step-by-step: adding a native R package

### 1. Run bindgen to generate the FFI

```bash
R_INCLUDE="$(Rscript -e 'cat(R.home("include"))')"
PKG_INCLUDE="$(Rscript -e 'cat(system.file("include", package = "cli"))')"

# Create wrapper header
cat > src/cli_wrapper.h << 'EOF'
#include <Rinternals.h>
#include <cli/progress.h>
EOF

# Run bindgen
bindgen \
  --merge-extern-blocks \
  --no-layout-tests \
  --no-doc-comments \
  --wrap-static-fns \
  --wrap-static-fns-path src/cli_static_wrappers.c \
  --allowlist-file '.*/cli/progress\.h' \
  --blocklist-type 'SEXPREC' \
  --blocklist-type 'SEXP' \
  --raw-line 'use miniextendr_api::ffi::SEXP;' \
  src/cli_wrapper.h \
  -- \
  -I"$R_INCLUDE" \
  -I"$PKG_INCLUDE" \
  > src/rust/native/cli_ffi.rs
```

Key bindgen flags:

| Flag | Purpose |
|------|---------|
| `--wrap-static-fns` | Generates C shim wrappers for `static` and `static inline` functions |
| `--wrap-static-fns-path` | Where to write the C shim file |
| `--allowlist-file` | Only emit bindings for declarations from matching files |
| `--blocklist-type SEXPREC` | Don't re-define `SEXPREC` (already in miniextendr-api) |
| `--blocklist-type SEXP` | Don't re-define `SEXP` |
| `--raw-line 'use ...'` | Import miniextendr's `SEXP` type instead |
| `--merge-extern-blocks` | Combine all `extern "C"` declarations into one block |
| `--no-layout-tests` | Skip layout verification tests |
| `--no-doc-comments` | Omit C doc comments from output |

### 2. Fix the C shim include path

bindgen writes the C shim with an absolute include path. Change it to relative:

```c
// Before (wrong):
#include "/absolute/path/to/src/cli_wrapper.h"

// After (correct):
#include "cli_wrapper.h"
```

### 3. Add `#![allow(...)]` to the generated Rust file

At the top of the generated `cli_ffi.rs`, add:

```rust
#![allow(unused, non_camel_case_types, non_upper_case_globals, clippy::all)]
```

### 4. What gets generated

**`src/cli_wrapper.h`**: bridge header that includes R and the package headers:

```c
#include <Rinternals.h>
#include <cli/progress.h>
```

**`src/cli_static_wrappers.c`**: C shims for static inline functions. Each
function `foo()` becomes `foo__extern()`:

```c
#include "cli_wrapper.h"

void cli_progress_done__extern(SEXP bar) { cli_progress_done(bar); }
int cli_progress_num__extern(void) { return cli_progress_num(); }
// ... one per static inline function
```

**`src/rust/native/cli_ffi.rs`**: Rust FFI declarations with `__extern` link names:

```rust
use miniextendr_api::ffi::SEXP;

unsafe extern "C" {
    #[link_name = "cli_progress_num__extern"]
    pub fn cli_progress_num() -> ::std::os::raw::c_int;
    // ...
}
```

The `#[link_name = "cli_progress_num__extern"]` tells the linker to find the
`cli_progress_num__extern` symbol (provided by the C shim), even though Rust
code calls it as `cli_progress_num()`.

### 5. Wire into the Rust crate

Create `src/rust/native.rs`:

```rust
pub mod cli_ffi;
```

Add to `src/rust/lib.rs`:

```rust
mod native;
```

Use the FFI:

```rust
use crate::native::cli_ffi;

#[miniextendr]
pub fn cli_active_progress_bars() -> i32 {
    unsafe { cli_ffi::cli_progress_num() }
}
```

### 6. Update DESCRIPTION

Two entries needed:

```
LinkingTo: cli
Imports: cli
```

`LinkingTo` tells R to add `-I<cli-include-path>` when compiling C files.
`Imports` ensures cli's DLL is loaded at runtime (required for
`R_GetCCallable()` to resolve symbols).

You also need an `@importFrom` in at least one roxygen block to trigger the
NAMESPACE import:

```rust
/// @importFrom cli cli_progress_bar
#[miniextendr]
pub fn cli_active_progress_bars() -> i32 { ... }
```

This causes `importFrom(cli,cli_progress_bar)` in NAMESPACE, which forces R
to load cli's DLL when your package loads.

### 7. Update configure.ac

Add native package include discovery:

```m4
dnl ---- Native R package include paths ----
NATIVE_PKG_CPPFLAGS=""

CLI_INCLUDE=$("${R_HOME}/bin/Rscript" -e "cat(system.file('include', package='cli'))")
if test -n "$CLI_INCLUDE" && test -d "$CLI_INCLUDE"; then
  NATIVE_PKG_CPPFLAGS="$NATIVE_PKG_CPPFLAGS -I$CLI_INCLUDE"
  AC_MSG_NOTICE([cli include: $CLI_INCLUDE])
fi
AC_SUBST([NATIVE_PKG_CPPFLAGS])
```

### 8. Update Makevars.in: the OBJECTS pattern

This is the critical piece. R's build system automatically compiles all `.c`
files in `src/` into `.o` files and collects them in `$(OBJECTS)`. By placing
the C shim files (`cli_static_wrappers.c`, `cli_wrapper.h`) in `src/`, they
compile automatically.

The key change: **pass all `$(OBJECTS)` to cargo as link arguments**. This
makes the shim symbols available to both the cdylib and staticlib Rust builds.

```makefile
# Add include paths for native package headers
PKG_CPPFLAGS = $(NATIVE_PKG_CPPFLAGS)

# The cargo staticlib target now depends on $(OBJECTS) and passes them as link args
$(CARGO_AR): FORCE_CARGO $(WRAPPERS_R) $(OBJECTS)
    @set -e; \
    TARGET_OPT=""; \
    LINK_ARGS=""; \
    for obj in $(OBJECTS); do \
      LINK_ARGS="$$LINK_ARGS -C link-arg=$(ABS_RPKG_SRCDIR)/$$obj"; \
    done; \
    if [ -n "$(CARGO_BUILD_TARGET)" ]; then \
      TARGET_OPT="--target $(CARGO_BUILD_TARGET)"; \
    fi; \
    RUSTFLAGS="$(ENV_RUSTFLAGS) $$LINK_ARGS" \
    $(CARGO) $(RUST_TOOLCHAIN) build $(CARGO_OFFLINE_FLAG) \
      $(CARGO_FEATURES_FLAG) $$TARGET_OPT \
      --lib --profile $(CARGO_PROFILE) \
      --manifest-path $(CARGO_TOML) \
      --target-dir $(CARGO_TARGET_DIR); \
    test -f "$(CARGO_AR)"

# Same pattern for the cdylib (wrapper generation)
$(CARGO_CDYLIB): FORCE_CARGO $(OBJECTS)
    @set -e; \
    TARGET_OPT=""; \
    CDYLIB_LINK_ARGS=""; \
    for obj in $(OBJECTS); do \
      CDYLIB_LINK_ARGS="$$CDYLIB_LINK_ARGS -C link-arg=$(ABS_RPKG_SRCDIR)/$$obj"; \
    done; \
    # ... rest of cdylib build ...
    RUSTFLAGS="$(ENV_RUSTFLAGS)" \
    $(CARGO) $(RUST_TOOLCHAIN) rustc ... \
      -- $$CDYLIB_LINK_ARGS
```

**How the OBJECTS pattern works:**

1. R's build system compiles `stub.c` → `stub.o` and
   `cli_static_wrappers.c` → `cli_static_wrappers.o`
2. These go into `$(OBJECTS)` automatically
3. The `for obj in $(OBJECTS)` loop converts each `.o` file to a
   `-C link-arg=/absolute/path/to/obj.o` RUSTFLAG
4. Cargo passes these to the linker, making the `*__extern` symbols
   available when linking the Rust crate
5. Both the cdylib (temporary, for wrapper generation) and the staticlib
   (permanent, for the final `.so`) get the symbols

**Why this works for both cdylib and staticlib:**

- The **cdylib** is a shared library that cargo builds for R wrapper
  generation. It needs the shim symbols to link successfully.
- The **staticlib** is the archive that becomes part of the final R package
  `.so`. The `*__extern` symbols are resolved when R links `$(OBJECTS)` +
  `$(CARGO_AR)` into `miniextendr.so`.

### 9. File layout

After setup, your `src/` directory looks like:

```
src/
├── cli_wrapper.h              # Bridge header (Rinternals.h + cli/progress.h)
├── cli_static_wrappers.c      # C shims for static inline functions
├── stub.c                     # Minimal C stub for R's build system
├── Makevars.in                # Build rules (configure template)
└── rust/
    ├── lib.rs                 # Rust crate root (has: mod native;)
    ├── native.rs              # Module declarations (has: pub mod cli_ffi;)
    ├── native/
    │   └── cli_ffi.rs         # bindgen-generated Rust FFI
    ├── native_cli_test.rs     # Test/demo using the FFI
    └── Cargo.toml
```

## Why static inline functions need shims

Most R packages that export C APIs use this pattern in their headers:

```c
static R_INLINE int cli_progress_num(void) {
    static int (*ptr)(void) = NULL;
    if (ptr == NULL) {
        ptr = (int (*)(void)) R_GetCCallable("cli", "cli_progress_num");
    }
    return ptr();
}
```

These are `static inline` functions that exist only in the header file, not in any
compiled library. bindgen can't just declare them as `extern "C"` because
there's no compiled symbol to link against.

The `--wrap-static-fns` flag solves this: bindgen generates a C file with
non-inline wrapper functions:

```c
int cli_progress_num__extern(void) {
    return cli_progress_num();  // calls the static inline version
}
```

The Rust FFI then links against `cli_progress_num__extern` instead of
`cli_progress_num`.

## Runtime resolution

At runtime, the call chain is:

```
Rust: cli_ffi::cli_progress_num()
  → linker resolves to: cli_progress_num__extern  (C shim in your package)
    → calls: cli_progress_num  (static inline from cli/progress.h)
      → first call: R_GetCCallable("cli", "cli_progress_num")  (resolves DLL symbol)
      → subsequent calls: cached function pointer (fast path)
```

The `R_GetCCallable` mechanism requires the cli package's DLL to be loaded.
This is why `importFrom(cli, cli_progress_bar)` in NAMESPACE is essential:
it triggers `library.dynam("cli", ...)` during package loading.

## Corpus: which packages work with bindgen

308 of 594 tested CRAN packages (52%) work with bindgen when using C++ mode.
See `dev/bindgen-compatible-packages-v3.csv` for the full list.

### Progression

| Version | Flags added | Successes |
|---------|-------------|-----------|
| v1 | C-only, no special flags | 69 / 594 (12%) |
| v2 | + `R_NO_REMAP` + `-x c++17` + `--enable-cxx-namespaces` | 204 / 594 (34%) |
| v3 | + `-isysroot` + LinkingTo resolution + c++14 fallback | 308 / 594 (52%) |

### Recommended bindgen flags (always use these)

```bash
bindgen \
  --enable-cxx-namespaces \
  --merge-extern-blocks \
  --no-layout-tests \
  --no-doc-comments \
  --wrap-static-fns \
  --wrap-static-fns-path "$STATIC_C" \
  --blocklist-type 'SEXPREC' \
  --blocklist-type 'SEXP' \
  --raw-line 'use miniextendr_api::ffi::SEXP;' \
  "$WRAPPER" \
  -- \
  -x c++ -std=c++17 \
  -isysroot "$(xcrun --show-sdk-path)" \
  -I"$R_INCLUDE" \
  -I"$PKG_INCLUDE" \
  -I"$TRANSITIVE_DEP_INCLUDES"
```

The wrapper header must define `R_NO_REMAP` before including `Rinternals.h`:

```c
#define R_NO_REMAP
#include <Rinternals.h>
#include <pkg/header.h>
```

### Why `R_NO_REMAP` is essential

R's `Rinternals.h` defines macros like `#define length Rf_length`,
`#define error Rf_error`, `#define allocVector Rf_allocVector`. These
collide with C++ identifiers. `rapidjson::Document` has a `length`
member, for instance. `R_NO_REMAP` suppresses these macros, keeping
only the `Rf_` prefixed versions.

### Why always use `-x c++`

Many `.h` files in R packages contain C++ code (`#include <string>`,
`#include <cmath>`, templates, namespaces). Using `-x c++` for all
headers avoids misclassification. bindgen handles pure C code fine
in C++ mode.

### Workaround: bindgen panics on Boost anonymous types

bindgen 0.72.1 panics with `"/*<unnamed>*/" is not a valid Ident` when
processing anonymous struct types inside Boost headers (e.g., through
`wdm` → `boost` transitive includes). The workaround:

```bash
--blocklist-file '.*/boost/.*'
--blocklist-file '.*/wdm/.*'
```

This prevents bindgen from constructing IR for Boost internals while
still allowing the package's own headers to reference Boost types
opaquely. The package's public API bindings generate correctly.

Tested: `svines` (25k lines) and `vinereg` (31k lines), both producing
valid bindings with the blocklist.

### Remaining failure categories (286 packages)

| Category | Count | Cause | Fixable? |
|----------|-------|-------|----------|
| cxx_stdlib | 122 | Deep Rcpp/RcppArmadillo dependency chains | Partially (needs recursive LinkingTo resolution) |
| compile_error | 80 | C++ template errors, deprecated APIs | No (package-specific issues) |
| missing_header | 59 | System libs (HDF5, GL, petscsnes) | Yes (install system deps) |
| rcpp_dep | 9 | Direct `#include <Rcpp.h>` | No (Rcpp ecosystem) |
| bindgen_panic | 2 | anonymous types in Boost/wdm headers | Yes (`--blocklist-file '.*/boost/.*'`) |

### Notable working packages

| Package | Mode | Std | Lines | Static fns |
|---------|------|-----|-------|------------|
| cli | c | - | 969 | yes |
| nanoarrow | c | - | 1,257 | yes |
| vctrs | c | - | 959 | no |
| processx | c | - | 1,280 | yes |
| wk | c | - | 981 | no |
| checkmate | c | - | 1,246 | yes |
| nloptr | cpp | c++17 | 12,394 | yes |
| BH (Boost subset) | cpp | c++17 | 18,767 | yes |
| AsioHeaders | cpp | c++17 | 24,664 | yes |
| piton (PEGTL) | cpp | c++17 | 15,661 | no |
| rjsoncons | cpp | c++17 | 13,374 | no |
| openxlsx2 | cpp | c++17 | 16,507 | no |
| dqrng | cpp | c++17 | 13,134 | no |
| ipaddress | cpp | c++17 | 22,743 | no |

## Applying to mirai/NNG

The NNG integration in the mirai worktree currently bundles NNG and mbedtls
source code in `src/nng/` and `src/mbedtls/`, compiling them via explicit
Makevars source lists and pattern rules. This produces static archives
(`libnng.a`, `libmbedtls_all.a`) that are passed to cargo via
`-C link-arg=`.

The OBJECTS pattern is a cleaner alternative when the C sources are simple
enough to live directly in `src/`. For NNG this may not apply directly (NNG
needs platform-specific defines and has deep subdirectories), but the
OBJECTS link-arg loop should still be used to pass `stub.o` and any other
`src/*.c` objects to cargo. This ensures any future C shim files
(e.g., for inline functions from other R packages) get linked automatically.

The key change for mirai:

```makefile
# Instead of hardcoding PKG_LIBS:
PKG_LIBS = $(CARGO_AR) $(LIBNNG) $(LIBMBEDTLS) $(NNG_LIBS)

# Pass OBJECTS to cargo too:
$(CARGO_AR): FORCE_CARGO $(WRAPPERS_R) $(OBJECTS) $(LIBNNG) $(LIBMBEDTLS)
    @set -e; \
    LINK_ARGS=""; \
    for obj in $(OBJECTS); do \
      LINK_ARGS="$$LINK_ARGS -C link-arg=$(ABS_RPKG_SRCDIR)/$$obj"; \
    done; \
    LINK_ARGS="$$LINK_ARGS -C link-arg=$(ABS_RPKG_SRCDIR)/$(LIBNNG)"; \
    LINK_ARGS="$$LINK_ARGS -C link-arg=$(ABS_RPKG_SRCDIR)/$(LIBMBEDTLS)"; \
    RUSTFLAGS="$(ENV_RUSTFLAGS) $$LINK_ARGS" \
    $(CARGO) ... build ...
```

This way adding a new R package header binding (e.g., for `later` or
`processx`) is just: drop the wrapper `.h` and `_static_wrappers.c` in
`src/`, add `LinkingTo:`, and it compiles automatically.

## Known limitations

### `--wrap-static-fns` only works in C mode

bindgen's `--wrap-static-fns` flag generates C shim wrappers for
`static` and `static inline` functions. This only works when parsing
headers in C mode (`-x c`). In C++ mode (`-x c++`), the flag is
silently ignored - no `*_static_wrappers.c` file is generated.

This matters for R packages that use the `R_GetCCallable()` pattern
via `static R_INLINE` functions (e.g., `cli`, `nanoarrow`). For these
packages, `use_native_package()` detects them as pure C and uses C
mode, preserving static wrapper generation. For C++ packages that also
have `static inline` functions, users would need to write the C shim
manually or invoke bindgen separately in C mode for those functions.

### Windows

The `-isysroot` flag is macOS-specific. On Windows (MSYS2/MinGW), the
C++ stdlib is provided differently. The `Makevars.win` template does
not yet include `NATIVE_PKG_CPPFLAGS` or the OBJECTS link-arg pattern.
Windows support requires:

- Detecting the MinGW C++ include path
- Updating `Makevars.win` / `configure.win` templates
- Testing with `R CMD INSTALL` on Windows

### LinkingTo resolution

`resolve_include_paths()` walks the LinkingTo dependency tree
recursively via BFS. However, some packages have LinkingTo deps that
aren't installed (e.g., Bioconductor packages). Missing deps are
silently skipped - the include path just won't be added, and bindgen
will fail with "file not found" for headers from those deps.
