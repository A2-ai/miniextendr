# R's Package Build System for Shared Libraries

How R builds packages with compiled code, and how miniextendr integrates with it.

## The Big Picture

When `R CMD INSTALL` encounters a package with a `src/` directory, it:

1. Runs the package's top-level `configure` script (if present), which can
   generate `src/Makevars` from `src/Makevars.in`
2. Compiles C/C++/Fortran sources into `.o` object files
3. Links those objects into a shared library (`.so` on Unix, `.dll` on Windows)
4. Installs the shared library into `libs/`

miniextendr adds a Rust step: Cargo builds a static library (`.a`) which R's
linker folds into the final shared library alongside a minimal C anchor.
`R_init_*` and all registered entry points are defined in Rust.

## Reusing caches across tarball installs

Set both `CARGO_TARGET_DIR` and `VENDOR_OUT` to retain compiled dependencies
across installations of the same tarball from different extraction directories:

```sh
CARGO_TARGET_DIR="$HOME/.cache/miniextendr/target" \
VENDOR_OUT="$HOME/.cache/miniextendr/vendor" \
R CMD INSTALL mypackage_0.1.0.tar.gz
```

`VENDOR_OUT` is an optional cache root. Configure selects
`<VENDOR_OUT>/<vendor-archive-md5>/vendor`, extracting the archive once and
publishing the completed entry atomically. A different archive selects a new
entry, so an unrelated package cannot accidentally reuse an old vendor tree.
Both variables must have the same values on subsequent installs. Cargo's
configured target directory follows the requested target directory too.

Shared-cache tarballs should be produced with the matching cargo-revendor:
`--freeze` records relative vendor paths in `[patch.crates-io]`, with the
corresponding dependencies resolved through those patches. Configure writes
absolute cache patches into `.cargo/config.toml`; it never rewrites the shipped
Cargo manifest or lockfile. Shared-cache builds pass package-specific C objects
only to the package crate, avoiding dependency rebuilds caused by changing
extraction paths in global Rust flags.

Tarball cleanup preserves caller-selected target and vendor caches, including
ones under the usual package-local cleanup paths. Cache entries are retained
until the caller removes them. The install-mode latch and wrapper-generation
guards still apply. Without `VENDOR_OUT`, extraction stays in the package's
`vendor/` directory and uses the ordinary build command. With neither variable
set, package-local build directories are cleaned as before. This cache does
not change the behavior of the separate `cleanup` script invoked by R's
`--preclean` and `--clean` options.
## Pre-shipped wrapper freshness

A native tarball install can reuse `R/<package>-wrappers.R` without loading the
shared library for generation (#1022). Its size alone cannot establish that it
matches the Rust sources: adding an S3 method after generation previously
allowed an install to succeed with a missing-method warning (#1512).

`tools/wrapper-freshness.R` records content fingerprints after a successful
native generation pass. The generated `tools/wrapper-inputs.rds` travels in the
package tarball and is gitignored. It binds the wrapper bytes to the package's
Rust source paths and contents, `Cargo.toml`, `Cargo.lock`, and the configured
Cargo features and profile. It excludes generated `wasm_registry.rs` and build
output directories. All recorded paths are relative to the package, so copying
the package or resetting its timestamps does not invalidate the record.

The native tarball fast path and the source-mode roxygen reuse optimization
require a matching record. `miniextendr_build()` regenerates the source
wrappers (and the record) under `MINIEXTENDR_FORCE_WRAPPER_GEN=1` before
roxygen2 runs, so its `document()` step reuses them and its single install
ships them (#1549). Missing, corrupt, or mismatched records trigger
generation from the freshly linked library.
`bootstrap.R` keeps a current record current across `cargo revendor --freeze`,
which rewrites the fingerprinted Cargo files without changing what the wrappers
are generated from; any other rewrite of those files invalidates the record,
even when the resulting wrappers are identical. If the existing R wrapper
changes, installation stops with its filename and recovery instructions before
R's namespace load check. Generation compares a temporary copy, preserving the
shipped wrapper on failure so a repeated install cannot bypass the check.
`MINIEXTENDR_FORCE_WRAPPER_GEN=1` still forces generation and performs the same
consistency check.

For example, after adding a `summary.my_class` Rust method, regenerate wrappers
and documentation in the original source package before producing the tarball:

```r
minirextendr::miniextendr_build("path/to/package")
devtools::build("path/to/package")
```

The fingerprint is separate from the R wrapper. The generator still leaves
unchanged R code, timestamps, and source-position comments alone. This guard
applies to the native tarball fast path; wasm uses its existing host-generated
wrapper and registry snapshot workflow.

The wrapper file itself opens with two header lines: the `AUTO-GENERATED`
marker, then the `miniextendr-api` version that generated it and an FNV-1a
64-bit digest of the rest of the file with source positions normalized
(`# miniextendr-api 0.x.y | fnv1a-64 of the normalized lines below: …`,
#1552). Two copies with the same digest contain the same wrapper code,
exports and docstrings, whatever their `(file.rs:line:col)` comments say;
that is the same view the generator uses to decide whether to rewrite the
file, so a Rust edit that only shifts line numbers changes neither.

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
     SHLIB='miniextendr.so' OBJECTS='stub.o'
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
| `PKG_LIBS` | Extra libraries to link | `$(CARGO_AR)` (the full static-library path) |
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
    stub.o \
    /path/to/rust-target/release/librpkg.a \    # ← our PKG_LIBS
    $(SHLIB_LIBADD) $(SAN_LIBS) -lR $(LIBINTL)  # ← R's system libs
```

### Windows

```bash
gcc -shared -o miniextendr.dll \
    miniextendr-win.def \                        # ← auto-generated exports
    stub.o \
    /path/to/rust-target/release/librpkg.a \      # ← our PKG_LIBS
    -lws2_32 -lntdll -luserenv -lbcrypt \        # ← Windows system libs
    -ladvapi32 -lsecur32 \
    $(SHLIB_LIBADD) $(SAN_LIBS) -lR $(LIBINTL)
```

## How miniextendr Integrates

### Our Makevars.in (Unix)

```makefile
# Use the full path so linkers cannot select a same-named import library.
PKG_LIBS = $(CARGO_AR)

# Add Cargo build and the configuration snapshot as dependencies of the
# shared library
$(SHLIB): $(OBJECTS) $(CARGO_AR) $(CARGO_LINK_CONFIG)

# Build the Rust static library via Cargo
$(CARGO_AR): FORCE_CARGO $(OBJECTS)
    $(CARGO) build --lib --profile $(CARGO_PROFILE) ...

# Snapshot the configured Makevars; copy only when the contents changed so a
# no-op install keeps the old mtime and skips the link (#1498).
$(CARGO_LINK_CONFIG): FORCE_CARGO $(CARGO_AR)
    @if ! cmp -s "$(ABS_RPKG_SRCDIR)/Makevars" "$(CARGO_LINK_CONFIG)"; then \
      cp "$(ABS_RPKG_SRCDIR)/Makevars" "$(CARGO_LINK_CONFIG)"; \
    fi

# Link first, then generate wrappers from that same shared library.
all: $(SHLIB) $(WRAPPERS_R)
$(WRAPPERS_R): $(SHLIB)
    Rscript -e "dyn.load(...); .Call('miniextendr_write_wrappers', ...)"
```

Key design decisions:

1. **No link recipe on `$(SHLIB)`**: we only add dependencies. The recipe comes
   from shlib.mk (`$(SHLIB_LINK) -o $@ $(OBJECTS) $(ALL_LIBS)`).

2. **FORCE_CARGO phony target**: ensures Cargo is always invoked, letting Cargo's
   own incremental build system decide what to rebuild.

3. **`all: $(SHLIB) $(WRAPPERS_R)` ordering**: links the package library first,
   then loads that same library to generate the R wrapper and wasm registry.
   The final `all` recipe handles development touches and tarball cleanup.

4. **`$(CARGO_LINK_CONFIG)` prerequisite**: Cargo keeps a cached archive's old
   mtime when a feature, profile, or target-directory switch selects it again,
   so a `.so` linked from a *different* configuration can be newer than the
   archive and make would skip the link (#1498). The link therefore also
   depends on `rust-target/.miniextendr-link-config`, a copy of the configured
   `src/Makevars` that is refreshed only when its contents change. Any
   configuration change relinks and regenerates the wrappers; a no-op install
   leaves the `.so` untouched.

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
                         → .cargo/config.toml (written by AC_CONFIG_COMMANDS)

R CMD INSTALL:
  1. Run configure (generates Makevars, etc.)
  2. make all:
     a. Compile stub.c → stub.o (R's CC)
     b. cargo build → librpkg.a (Rust staticlib, includes R_init_*)
     c. $(SHLIB_LINK) -o miniextendr.so stub.o librpkg.a (R's linker)
     d. In development/fallback mode, load miniextendr.so and write
        R/miniextendr-wrappers.R + src/rust/wasm_registry.rs
  3. Install miniextendr.so to libs/
  4. Install R/ files, man/, etc.
```

The wrapper and wasm registry writers are registered routines in
`miniextendr-api`. Roxygen2 runs separately (`just force-document` after
macro/wrapper changes) and derives the tracked `NAMESPACE` and `man/*.Rd` files
from the generated R wrapper.

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

## Keep the Cargo cache during development installs

Avoid `R CMD INSTALL --preclean` / `--clean` in the development loop: they
invoke `cleanup` and normally erase `rust-target`, `src/rust/target`, and
`ra-target`, making the next Cargo build cold. If another tool requires those
flags, set `MINIEXTENDR_KEEP_TARGET=1` for that source install. The opt-in spares
only those target directories during R CMD INSTALL; configure files are still
refreshed, and R CMD build still cleans its staged tree. Leave the variable
unset for a deliberate full clean rebuild.

```sh
# These cleanup flags normally erase the Cargo cache; the opt-in retains it.
MINIEXTENDR_KEEP_TARGET=1 R CMD INSTALL --preclean --clean path/to/rpkg
```

`just rcmdinstall` does not add either flag, and `miniextendr_build()` does not
request them. Scaffolded DESCRIPTION files set `Config/build/never-clean: true`
so pkgbuild also avoids adding preclean automatically. A plain install remains
the preferred development command.

R sets `R_INSTALL_PKG` while installing, including before and after configure.
The cleanup script requires that marker as well as the opt-in; simply running
`./cleanup` or building a package still removes targets. Existing tarball-mode
Makevars cleanup remains unchanged.

## Development bootstrap

For local installs from a package with path-dependency siblings:

```r
withr::with_envvar(c(MINIEXTENDR_BOOTSTRAP_MODE = "dev"), {
  devtools::install("path/to/rpkg", build = TRUE, upgrade = FALSE)
})
```

`minirextendr::miniextendr_build()` selects this mode for its own installs unless
you explicitly set a bootstrap mode or already have a distribution vendor
archive. It uses `cargo revendor --dev` when that tool is on PATH and the base-R
stager in `tools/dev-bootstrap.R` otherwise; both produce the same layout.

Bootstrap packages only path dependencies, including their transitive path
siblings, into uncompressed `src/rust/vendor/<name>-<version>/` directories.
Cargo package resolves workspace inheritance. Registry and Git dependencies
continue to resolve normally; this mode does not make an offline release
artifact, generate source replacement, or compress `inst/vendor.tar.xz`. The
portable manifest points each path dependency at its staged directory and adds
`exclude = ["vendor"]` to its `[workspace]` so staged crates never become
workspace members.

## Distribution bootstrap without cargo-revendor

A distribution build (`MINIEXTENDR_BOOTSTRAP_MODE` unset or `dist`) normally
runs `cargo revendor --freeze` into `inst/vendor.tar.xz`. When `cargo-revendor`
is not on PATH, bootstrap falls back to the same base-R stager whenever a path
dependency lies **outside the package directory**, because R CMD build seals
only that directory and a `path` dependency is not source-replaceable. The
result builds with network access but is not CRAN-ready (#1580). A package whose
path dependencies all live inside it has nothing to stage and builds from source.
Either way bootstrap ends with a `warning()` that the tarball downloads crates.io
and git dependencies at install time and is not CRAN-ready, naming the
`cargo install` line and `minirextendr::miniextendr_vendor()` as the fixes.

R CMD build activates the staging by running `./cleanup` in its copy, and skips a
`cleanup` that is not executable. pak's git client (`git::` and `gitlab::` refs)
writes every file without mode bits, so bootstrap restores the executable bit on
`cleanup` whenever it leaves a staging behind.

Installers only reach an outside sibling when they start from the repository:

- **pak** with a repository ref and a subdirectory (`pak::pak("owner/repo/rpkg")`,
  `gitlab::…/-/rpkg`, or any ref that pkgdepends resolves through git) downloads
  the whole repository, runs `bootstrap.R` in the subdirectory, and builds the
  tarball there, so the staging is what gets installed.
- **rv 0.23.0 or later** with a git source plus `directory = "rpkg"`, or a local
  source whose `path` is the repository root plus `directory = "rpkg"`, copies
  the whole repository, runs `bootstrap.R` in the subdirectory, runs `R CMD
  build` there, and installs the extracted tarball. The staging is what gets
  built: with rv 0.23.1 the sibling compiles from `src/rust/vendor/`.

rv before 0.23.0 never runs `bootstrap.R`, and a local source takes no
`directory` yet. With a git source plus `directory = "rpkg"`, what reaches
`R CMD INSTALL` depends on the release:

| rv | Build directory | The outside sibling |
|---|---|---|
| up to 0.12.0 | Real directories whose files are symlinks into the package directory of rv's checkout (a copy on Windows) | configure stages it from that checkout, see below |
| 0.12.1 to 0.16.x | A copy of the package directory alone | missing: configure stops with the error below |
| 0.17.0 to 0.22.x | A copy of the whole repository, installed in place | still next to the package, so cargo builds it where it is |

Pointing any installer at the package directory itself (pak
`local::<repo>/rpkg`, rv `path = "<repo>/rpkg"`, a plain `R CMD build` of
`rpkg/`) takes only that directory, so the sibling is gone before anything can
stage it, with or without `cargo-revendor`. Use rv 0.23.0 or later or pak with
one of the forms above, or build the tarball in the checkout
(`devtools::build()`) and install that.

Configure catches a stranded sibling before cargo runs, instead of the opaque
`failed to load manifest for dependency` cargo error. Outside tarball mode,
`tools/dev-bootstrap.R configure` scans `src/rust/Cargo.toml` for `path`
entries in the tables cargo loads: every dependency table (dev,
build and target-specific ones included, `[workspace.dependencies]`) and
`[patch.*]` sources. A path inside the package, or one whose directory holds a
`Cargo.toml`, passes. That covers the checkout itself, the whole-repository
copy, a checkout that bootstrap staged, and the tarball R CMD build seals after
cleanup activates the staging. For a missing path it does one of two things:

- When the build directory is a tree of symlinks (rv up to 0.12.0), it follows
  `src/rust/Cargo.toml` to the package directory it mirrors. It acts only when
  that manifest sits at `<origin>/src/rust/Cargo.toml`, the tree's
  `DESCRIPTION` resolves to `<origin>/DESCRIPTION`, and `<origin>` is not the
  build directory. If every missing path exists relative to the origin
  manifest, the base-R stager packages those crates from the origin into the
  tree's `src/rust/vendor/`. It then renames the portable manifest, and a copy
  of the lockfile, over the tree's symlinks. Every write is a new entry of the
  build directory, so neither configure nor cargo writes through a link into
  the checkout.
- Otherwise (a copy, or a tree of hardlinks, where nothing leads back to the
  checkout) configure stops:

```text
Path dependencies outside the package directory are missing:
  - `path = "../../../core"` under [dependencies] in src/rust/Cargo.toml: /tmp/core has no Cargo.toml
bootstrap.R did not run for this build. It stages the crates that the package's
src/rust/Cargo.toml reaches outside the package directory, but the installer took
the package directory out of its repository without running it.
Install the package in one of these ways, which run bootstrap.R while the repository is present:
  - rv >= 0.23.0 with a git source plus `directory` naming the package's subdirectory
  - pak with a repository ref and a subdirectory, e.g. pak::pak("<owner>/<repo>/<subdirectory>")
  - build the tarball in the checkout with devtools::build() and install that tarball
configure: error: a path dependency outside the package is missing; see the message above
```

A missing `[workspace.dependencies]` path counts even when no dependency
inherits it, since the base-R stager rejects such an entry too. An unused
`[replace]` path and a target path such as `[lib] path` are left to cargo.

The stager runs `cargo metadata --no-deps` for discovery and `cargo package
--no-verify --allow-dirty` per sibling, so it never writes to the source tree.
Two constraints follow from `cargo package`:

- A dependency **between siblings** (normal or build kind) needs a `version`
  key, because `cargo package` rewrites it to a version-only entry; the stager
  then points it back at the staged copy. The R crate's own path dependencies
  need no version key: its manifest is rewritten in place, never packaged.
- A sibling cannot carry a `git` dependency, which `cargo package` rejects or
  turns into a crates.io one. Install `cargo-revendor` for such a graph.

Bootstrap reports every blocking problem in one error before staging anything:
each missing `version`, each sibling git dependency, each `path` that points at
a directory without a `Cargo.toml`, and each other path literal in the R crate
that leaves the package. That last group covers a `[patch]` or `[replace]`
source, which the fallback does not stage, and a target such as `[lib] path`.
The R crate may declare a staged dependency in any TOML form cargo accepts
(inline or `[dependencies.<name>]` tables, dotted keys, either quote style,
absolute or `./`-prefixed paths, `[workspace.dependencies]` inheritance,
`[target.*]` tables, multi-line inline tables); the minirextendr test suite
exercises each form.

The staged state records its mode and an md5 fingerprint of the source files
behind every staged crate (plus an inherited workspace manifest). A staging
left in the checkout is only reused by a later build of the same mode, and
cleanup stops with `path dependency <name> changed since bootstrap; rerun
bootstrap.R` when a source it can still see has changed. Sources that no
longer exist are not compared.

The source `Cargo.toml` stays byte-for-byte unchanged. Bootstrap prepares
`.Cargo.toml.dev` and `.dev-bootstrap.rds` beside it. R CMD build runs cleanup
inside its temporary copy, where the portable manifest is activated. Cleanup
checks both the recorded origin path and source digest, so running cleanup in
the checkout leaves its manifest alone and stale preparation fails clearly.
Configure never rewrites either source manifest.

A staging is bootstrap's own only while `.dev-bootstrap.rds` sits beside it.
Every bootstrap removes that previous staging (vendor directory, portable
manifest, state) before writing a new one, so repeated installs keep exactly
one copy and no backups. A `src/rust/vendor` without the state file is not
bootstrap's, so bootstrap stops instead of moving or deleting it. The dev
sidecars must reach R's build-copy cleanup and therefore are only gitignored.
Cleanup consumes them before the artifact is sealed, keeping its activation
backup under `src/rust/.dev-vendor-backup-*` inside the build copy, which is
excluded from the artifact.

Ordinary bootstrap calls still default to distribution mode. Unset the variable
(or set it to `dist`) to produce `inst/vendor.tar.xz` with the existing full
vendor/freeze workflow. Bootstrap removes prior dev staging before preparing
that distribution artifact.

Development bundles follow [Cargo’s package file-selection rules](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields). Check `cargo package --list` for an ancestor core crate: Git ignore rules require its manifest to be tracked. For an untracked source tree, use `package.include` or `package.exclude` to keep generated output out of the crate. In a Git checkout, untracked files that Git does not ignore are packaged too, and ignored ones are left out. Bootstrap never stages files in Git.

## See Also

- [LINKING.md](LINKING.md): how miniextendr links to libR (engine vs package)
- [ENTRYPOINT.md](ENTRYPOINT.md): the C entry point design
- [CRAN_COMPATIBILITY.md](CRAN_COMPATIBILITY.md): dependency vendoring for CRAN
- [TEMPLATES.md](TEMPLATES.md): how configure.ac templates work
- R sources: `share/make/shlib.mk`, `share/make/winshlib.mk`
- R sources: `src/library/tools/R/install.R` (the `R CMD INSTALL` implementation)
