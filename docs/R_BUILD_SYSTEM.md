# R's Package Build System for Shared Libraries

How R builds packages with compiled code, and how miniextendr integrates with it.

## The Big Picture

When `R CMD INSTALL` encounters a package with a `src/` directory, it:

1. Runs `src/configure` (if present) to generate `Makevars` from `Makevars.in`
2. Compiles C/C++/Fortran sources into `.o` object files
3. Links those objects into a shared library (`.so` on Unix, `.dll` on Windows)
4. Installs the shared library into `libs/`

miniextendr adds a Rust step: Cargo builds a static library (`.a`) which R's
linker folds into the final shared library alongside the C entry points.

## Makefile Include Chain

R's build system is a hierarchy of makefiles included in a specific order.
Understanding this order is essential because later includes can override
earlier definitions.

```text
┌─────────────────────────────────────────┐
│ 1. Package's src/Makevars              │  ← We define PKG_LIBS, deps, recipes
│    (or src/Makevars.win on Windows)     │
├─────────────────────────────────────────┤
│ 2. $R_HOME/etc/Makeconf                │  ← R's system config (compiler, flags)
│    (or etc/Makeconf.win)               │
├─────────────────────────────────────────┤
│ 3. $R_HOME/etc/Makevars.site           │  ← Optional site-wide overrides
├─────────────────────────────────────────┤
│ 4. $R_HOME/share/make/shlib.mk        │  ← The link recipe (see below)
│    (or share/make/winshlib.mk)         │
├─────────────────────────────────────────┤
│ 5. ~/.R/Makevars                       │  ← Optional user overrides
└─────────────────────────────────────────┘
```

R invokes make with all of these as `-f` arguments:

```bash
make -f Makevars -f Makeconf -f Makevars.site -f shlib.mk -f ~/.R/Makevars \
     SHLIB='miniextendr.so' OBJECTS='entrypoint.o mx_abi.o'
```

## Variable Flow

### Makeconf (R's system configuration)

Set once when R was compiled. Key variables:

```makefile
# Linker command and flags
SHLIB_LD       = gcc              # or clang, etc.
SHLIB_LDFLAGS  = -shared          # or -dynamiclib on macOS
SHLIB_LINK     = $(SHLIB_LD) $(SHLIB_LDFLAGS) $(LIBR0) $(LDFLAGS)

# All libraries to link
ALL_LIBS = $(PKG_LIBS) $(SHLIB_LIBADD) $(SAN_LIBS) $(LIBR) $(LIBINTL)

# Compiler flags (with PKG_* hooks for package authors)
ALL_CFLAGS   = $(R_XTRA_CFLAGS) $(PKG_CFLAGS) $(CPICFLAGS) $(SHLIB_CFLAGS) $(CFLAGS)
ALL_CPPFLAGS = $(R_XTRA_CPPFLAGS) $(R_INCLUDES) -DNDEBUG $(PKG_CPPFLAGS) $(CPPFLAGS)
```

### Package's Makevars (what we control)

We can set these `PKG_*` variables:

| Variable | Purpose | Our value |
|----------|---------|-----------|
| `PKG_LIBS` | Extra libraries to link | `-L$(CARGO_LIBDIR) -l$(CARGO_STATICLIB_NAME)` |
| `PKG_CPPFLAGS` | C preprocessor flags | (not used) |
| `PKG_CFLAGS` | Extra C compiler flags | (not used) |

### shlib.mk (the link recipe)

This is the heart of R's shared library build:

```makefile
# Unix (share/make/shlib.mk)
all: $(SHLIB)

$(SHLIB): $(OBJECTS)
    $(SHLIB_LINK) -o $@ $(OBJECTS) $(ALL_LIBS)

shlib-clean:
    rm -Rf .libs _libs
    rm -f $(OBJECTS) symbols.rds
```

And on Windows:

```makefile
# Windows (share/make/winshlib.mk)
$(SHLIB): $(OBJECTS)
    if test -e "$(BASE)-win.def"; then
        $(SHLIB_LD) ... -o $@ $(BASE)-win.def $(OBJECTS) $(ALL_LIBS)
    else
        # Auto-generate .def from nm output
        EXPORTS > tmp.def
        $(NM) $(OBJECTS) | sed ... >> tmp.def
        $(SHLIB_LD) ... -o $@ tmp.def $(OBJECTS) $(ALL_LIBS)
    fi
```

## The Final Link Command

When everything expands, R links our package like this:

### Unix

```bash
gcc -shared -o miniextendr.so \
    entrypoint.o mx_abi.o \
    -L/path/to/rust-target/release -lrpkg \    # ← our PKG_LIBS
    $(SHLIB_LIBADD) $(SAN_LIBS) -lR $(LIBINTL)  # ← R's system libs
```

### Windows

```bash
gcc -shared -o miniextendr.dll \
    miniextendr-win.def \                        # ← auto-generated exports
    entrypoint.o mx_abi.o \
    -L/path/to/rust-target/release -lrpkg \      # ← our PKG_LIBS
    -lws2_32 -lntdll -luserenv -lbcrypt \        # ← Windows system libs
    -ladvapi32 -lsecur32 \
    $(SHLIB_LIBADD) $(SAN_LIBS) -lR $(LIBINTL)
```

## How miniextendr Integrates

### Our Makevars.in (Unix)

```makefile
# PKG_LIBS tells R to link our Rust static library
PKG_LIBS = -L$(CARGO_LIBDIR) -l$(CARGO_STATICLIB_NAME)

# Add Cargo build as a dependency of the shared library
$(SHLIB): $(OBJECTS) $(CARGO_AR) $(R_WRAPPERS_CURRENT)

# Build the Rust static library via Cargo
$(CARGO_AR): FORCE_CARGO $(CARGO_TOML) $(CARGO_LOCK)
    $(CARGO) build --lib --profile $(CARGO_PROFILE) ...
```

Key design decisions:

1. **No link recipe on `$(SHLIB)`**: we only add dependencies. The recipe comes
   from shlib.mk (`$(SHLIB_LINK) -o $@ $(OBJECTS) $(ALL_LIBS)`).

2. **FORCE_CARGO phony target**: ensures Cargo is always invoked, letting Cargo's
   own incremental build system decide what to rebuild.

3. **`all: $(SHLIB)` with post-build recipe**: runs after R links the shared lib
   to handle dev-mode touches and CRAN cleanup.

### Our Makevars.win (Windows)

```makefile
include Makevars          # Reuse all Unix logic
PKG_LIBS = ... -lws2_32 -lntdll ...   # Override with Windows system libs
```

### Object files

R auto-detects `.c` files in `src/` and compiles them:

- `stub.c` → `stub.o`: empty file required by R's build system to invoke the linker

This is the only C file. All entry points (`R_init_*`), registration, and
runtime initialization are defined in Rust via `miniextendr_init!`. The Rust
code lives in the static library referenced by `PKG_LIBS`.

### Symbol visibility

We use explicit symbol registration, not dynamic lookup:

```rust
// In lib.rs - generates R_init_miniextendr() with all registration
miniextendr_api::miniextendr_init!(miniextendr);
```

The generated `R_init_miniextendr()` calls `package_init()` which registers
all `.Call` routines and locks down symbol visibility.

This means:
- R never uses `dlsym()` to find our symbols at runtime
- All function dispatch goes through the registered routines table
- The `.def` file on Windows only needs to export `R_init_miniextendr`

## Build Flow Summary

```text
configure.ac → configure → Makevars (from Makevars.in)
                         → .cargo/config.toml (from cargo-config.toml.in)

R CMD INSTALL:
  1. Run configure (generates Makevars, etc.)
  2. make all:
     a. Compile stub.c → stub.o (R's CC)
     b. cargo build → librpkg.a (Rust staticlib, includes R_init_*)
     c. $(SHLIB_LINK) -o miniextendr.so stub.o librpkg.a (R's linker)
  3. Install miniextendr.so to libs/
  4. Install R/ files, man/, etc.
```

Note: R wrapper generation (`miniextendr-wrappers.R`) happens separately via
[`just devtools-document`](https://github.com/A2-ai/miniextendr/blob/main/justfile), not during R CMD INSTALL.

## Build Contexts

The configure script resolves one of two install modes from a single signal:

| Mode | When | Behavior |
|---|---|---|
| Source | `inst/vendor.tar.xz` absent | Cargo resolves through `[patch."git+url"]` to monorepo siblings if present, otherwise fetches the git URL declared in `Cargo.toml`. |
| Tarball | `inst/vendor.tar.xz` present | Configure unpacks the tarball into `vendor/`, writes `[source]` replacement to `vendored-sources`, build runs `--offline`. |

There is no env var for install mode (`NOT_CRAN`, `PREPARE_CRAN`,
`FORCE_VENDOR` are all gone). See [CRAN Compatibility](CRAN_COMPATIBILITY.md)
for the full table and rationale.

**IFS save/restore:** The configure script saves and restores `IFS` around
any code that modifies it (`miniextendr_saved_IFS=$IFS` / `IFS=$miniextendr_saved_IFS`).
This prevents corrupting autoconf 2.72's internal state, which relies on `IFS`
being set to its default value.

## See Also

- [LINKING.md](LINKING.md): how miniextendr links to libR (engine vs package)
- [ENTRYPOINT.md](ENTRYPOINT.md): the C entry point design
- [CRAN_COMPATIBILITY.md](CRAN_COMPATIBILITY.md): dependency vendoring for CRAN
- [TEMPLATES.md](TEMPLATES.md): how configure.ac templates work
- R sources: `share/make/shlib.mk`, `share/make/winshlib.mk`
- R sources: `src/library/tools/R/install.R` (the `R CMD INSTALL` implementation)
