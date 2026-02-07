# R Shared Library Integration Analysis

## Context

miniextendr embeds a Rust static library (`.a`) into an R package's shared library
(`.so`/`.dll`). R has a specific build pipeline for shared libraries defined in
`share/make/shlib.mk` (Unix) and `share/make/winshlib.mk` (Windows). This analysis
audits our Makevars.in / Makevars.win against what R expects.

## R's shlib Build Pipeline

### Include chain (order matters)

```
1. Package's src/Makevars         ← We set PKG_LIBS, extra deps
2. $R_HOME/etc/Makeconf           ← R's system config (SHLIB_LINK, ALL_LIBS, etc.)
3. $R_HOME/share/make/shlib.mk   ← Provides $(SHLIB) link recipe
```

### Key variables

| Variable | Defined in | Expands to |
|----------|-----------|------------|
| `SHLIB_LINK` | Makeconf | `$(SHLIB_LD) $(SHLIB_LDFLAGS) $(LIBR0) $(LDFLAGS)` |
| `ALL_LIBS` | Makeconf | `$(PKG_LIBS) $(SHLIB_LIBADD) $(SAN_LIBS) $(LIBR) $(LIBINTL)` |
| `PKG_LIBS` | Our Makevars | `-L$(CARGO_LIBDIR) -l$(CARGO_STATICLIB_NAME)` |
| `OBJECTS` | Auto-detected | `entrypoint.o mx_abi.o` (from our C files) |

### Link command (Unix)

```bash
$(SHLIB_LINK) -o miniextendr.so $(OBJECTS) $(ALL_LIBS)
# Expands to:
gcc -shared -o miniextendr.so entrypoint.o mx_abi.o \
    -L.../rust-target/release -lrpkg \
    $(SHLIB_LIBADD) $(SAN_LIBS) -lR $(LIBINTL)
```

### Link command (Windows)

winshlib.mk additionally:
1. Checks for `<base>-win.def` — we don't provide one
2. Auto-generates `.def` from `nm $(OBJECTS)` — finds C symbols only
3. Links with `.def` file controlling DLL exports

## Current State: What We Do Right

### Unix (shlib.mk compatibility)

| Requirement | Status | How |
|-------------|--------|-----|
| `$(SHLIB)` has a link recipe | OK | shlib.mk provides it via include chain |
| `PKG_LIBS` points to Rust static lib | OK | Makevars.in sets `-L... -l...` |
| `$(OBJECTS)` includes C entry points | OK | R auto-detects `entrypoint.c`, `mx_abi.c` |
| Extra deps trigger Cargo rebuild | OK | `$(SHLIB): $(CARGO_AR)` adds dependency |
| `all: $(SHLIB)` target exists | OK | Both our Makevars and shlib.mk define it |
| Symbol registration | OK | `R_useDynamicSymbols(dll, FALSE)` + `R_forceSymbols(dll, TRUE)` |

### Windows (winshlib.mk compatibility)

| Requirement | Status | How |
|-------------|--------|-----|
| `Makevars.win` exists | OK | Static file, includes Makevars then overrides PKG_LIBS |
| Windows system libs linked | OK | `-lws2_32 -lntdll -luserenv -lbcrypt -ladvapi32 -lsecur32` |
| DLL exports work | OK | Auto-generated `.def` finds `R_init_miniextendr` in entrypoint.o |
| MSYS2 path handling | OK | configure.ac uses `cygpath -m` for Cargo paths |

### Both platforms

| Requirement | Status | How |
|-------------|--------|-----|
| `shlib-clean` target | OK | Provided by shlib.mk include; our `clean` adds extra cleanup |
| `symbols.rds` | N/A | R generates this automatically; we use explicit registration |
| `FORCE_CARGO` phony target | OK | Lets Cargo handle incremental builds |

## Potential Improvements (Low Priority)

### 1. Provide explicit `miniextendr-win.def` on Windows

**Current**: winshlib.mk auto-generates a `.def` from `nm $(OBJECTS)`. Since
`$(OBJECTS)` only has C files, the `.def` only exports C symbols. Rust symbols
in the static lib are resolved at link time but not explicitly exported.

**Risk**: Low. R only accesses symbols through registered routines (`.Call`),
and `R_useDynamicSymbols(dll, FALSE)` prevents dynamic lookup. All C entry
points (`R_init_miniextendr`, `mx_wrap`, `mx_get`, `mx_query`) are in the
C objects and will appear in the auto-generated `.def`.

**Action**: No change needed. If DLL export issues arise on Windows, provide
`rpkg/src/miniextendr-win.def` listing required exports.

### 2. Verify SHLIB_LIBADD is not needed

**Current**: We don't set `SHLIB_LIBADD`. This variable carries BLAS/LAPACK
and ObjC libs from R's configuration.

**Risk**: None for current use. Only relevant if Rust code calls R's BLAS/LAPACK
via FFI, which we don't do.

**Action**: No change needed.

### 3. SAN_LIBS (sanitizer support)

**Current**: Sanitizer libraries are automatically included in `ALL_LIBS` by
R's Makeconf. No action needed from us.

**Action**: No change needed. Cargo builds with sanitizers require separate
RUSTFLAGS configuration, which users can set via `ENV_RUSTFLAGS`.

## Conclusion

**The shlib integration is sound.** Our Makevars.in correctly:
- Delegates the link recipe to R's shlib.mk
- Adds Cargo build as a dependency of `$(SHLIB)`
- Sets `PKG_LIBS` so R links our Rust static lib into the shared library
- Uses explicit symbol registration (not dynamic lookup)
- Has Windows support via Makevars.win with platform-specific system libraries

No code changes are required. This analysis serves as documentation of the
integration points for future reference.
