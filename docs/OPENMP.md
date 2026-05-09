# OpenMP Runtime Loading

This document explains why packages that use OpenMP can install or load on
Linux but fail on macOS, and what to check in a miniextendr-backed R package.

## The Short Version

OpenMP is not only a compile-time feature. Code compiled with OpenMP often
needs a runtime library such as `libomp`, `libgomp`, or a vendor-specific
equivalent when the final package shared object is loaded by R.

For R packages, the dependency has to be attached to the final package shared
object loaded by `dyn.load()`. It is not enough for an intermediate Rust crate,
C object, or C++ object to have been compiled with an OpenMP flag.

macOS exposes missing runtime linkage more often because Apple `clang` does not
ship native OpenMP support. OpenMP usually comes from a separate `libomp.dylib`,
and the dynamic loader has to find a version of that dylib compatible with the
compiler used to build the package.

## Rust Perspective

Pure Rust code does not use OpenMP. Rust has no `#pragma omp` directive, no
`-fopenmp` flag, and no implicit dependency on `libomp` or `libgomp`. A
miniextendr package whose Rust dependencies are all pure Rust will not link
against an OpenMP runtime and is unaffected by this document.

OpenMP enters a miniextendr package through native code compiled by a Rust
crate's `build.rs` — typically a `*-sys` crate wrapping a C, C++, or Fortran
library that itself uses OpenMP. Common arrival paths:

- BLAS/LAPACK backends (`openblas-src` with the `system` feature,
  `intel-mkl-src`, `accelerate-src` on macOS) brought in by linear-algebra
  crates such as `ndarray-linalg`, `nalgebra` with a BLAS backend, `faer`, or
  `polars` with a BLAS feature.
- C++ ML runtimes wrapped via `-sys` crates (ONNX Runtime, Torch via
  `tch-rs`, certain `xgboost` / `lightgbm` bindings).
- Image, audio, and numerical libraries with parallel C/C++ kernels gated
  behind a Cargo feature flag.

`rayon`, `std::thread`, `tokio`, and `crossbeam` are not OpenMP — they are
pure-Rust parallelism. A package that uses only Rayon needs no
`SHLIB_OPENMP_*FLAGS` plumbing in `Makevars`.

### What Cargo Does Not Do

Cargo build scripts that compile C, C++, or Fortran via the `cc` crate run
with the host compiler's defaults. They do not read R's `SHLIB_OPENMP_CFLAGS`
/ `SHLIB_OPENMP_CXXFLAGS` / `SHLIB_OPENMP_FFLAGS` macros, and they do not know
that the final shared object will be loaded by R. The consequences:

1. Native source compiled inside the crate may or may not have been built
   with an OpenMP flag — that depends on the crate's own `build.rs`,
   environment variables, and feature flags, not on R.
2. The Rust `staticlib` artifact handed to `R CMD INSTALL` may carry
   unresolved OpenMP symbols (`omp_get_thread_num`, `__kmpc_fork_call`,
   `GOMP_parallel`, …).
3. R's link line — driven by `rpkg/src/Makevars` / `Makevars.in` — is the
   only place those symbols can be resolved into the final `pkg.so`. If
   `PKG_LIBS` does not include `$(SHLIB_OPENMP_CFLAGS)` (or the matching
   CXX/FFLAGS macro), the link succeeds on Linux because lazy binding masks
   the missing reference, and fails at `dyn.load()` time on macOS.

### Linking Through R, With `openmp-sys` Doing the Compile Half

The split for miniextendr packages: the OpenMP **compile flag** (the
`-fopenmp` / `/openmp` passed to `cc::Build` in a Rust crate's
`build.rs`) comes from
[`openmp-sys`](https://lib.rs/crates/openmp-sys) (source:
<https://gitlab.com/kornelski/openmp-rs>); the OpenMP **runtime
library** (`libomp.dylib`, `libgomp.so`, `vcomp.dll`) is linked into
the final `pkg.so` by R's `SHLIB_OPENMP_*FLAGS` on the
`R CMD INSTALL` line. Cargo and R cooperate at different layers — they
do not race to discover the runtime independently.

What `openmp-sys` does on the Rust side:

- Searches `cc -print-search-dirs`, Homebrew, MacPorts, and MSVC's
  `vcomp.dll` for an OpenMP install at build time.
- Exposes `DEP_OPENMP_FLAG` so a downstream crate's `build.rs` can
  call `cc::Build::new().flag(env!("DEP_OPENMP_FLAG")).compile(...)`
  and have its C/C++ sources compile with the right OpenMP pragma
  enabled.
- Emits `cargo:rustc-link-lib=dylib=gomp` / `omp` so a standalone
  binary built from those crates would link a runtime.

For a miniextendr package, the third item is what would otherwise
collide with R. The trick is: that `cargo:rustc-link-lib` directive
travels into the Rust staticlib's metadata and surfaces on R's link
line, *and* R independently adds `$(SHLIB_OPENMP_CFLAGS)`. As long as
both name the same runtime — i.e., the OpenMP `openmp-sys` discovered
matches the one R was built against — the two link directives collapse
to a single recorded dependency in `pkg.so` and the package loads
cleanly.

The practical rules:

1. **Prefer pure-Rust parallelism** when the choice is yours.
   `rayon`, `std::thread`, `tokio`, `crossbeam` need no `Makevars`
   plumbing and dodge this entire document.
2. **When you do need OpenMP-using C/C++ compiled inside a Rust
   crate**, depend on `openmp-sys`, use its `DEP_OPENMP_FLAG` in
   `cc::Build`, and on the R side declare the runtime via
   `PKG_LIBS = $(SHLIB_OPENMP_CFLAGS)` (or `CXXFLAGS` / `FFLAGS`
   matching the underlying language) in `rpkg/src/Makevars.in`. The
   rule R-Extensions states for plain C/C++ packages applies unchanged.
3. **Make sure the two halves agree on which runtime is in use.** On
   macOS the typical hazard is `openmp-sys` finding Homebrew's
   `libomp.dylib` while R's `clang` was configured against a different
   `libomp` (system, Apple-supplied, or a CRAN binary build's vendored
   one). Set `LIBRARY_PATH` / `CFLAGS` for `openmp-sys` discovery, or
   point R at the same toolchain via `~/.R/Makevars`, so that one
   runtime is visible to both.
4. **Do not hand-roll** `cargo:rustc-link-lib=dylib=omp` (or `gomp`)
   in a miniextendr-package `build.rs`. `openmp-sys` already does this
   correctly per platform; replicating it is how runtimes start
   disagreeing.
5. **On macOS, verify after install.** Run `otool -L src/pkg.so` and
   confirm exactly one OpenMP runtime is recorded, with a path the
   loader can resolve. Two entries (e.g., `@rpath/libomp.dylib` from
   Cargo and `/usr/local/opt/libomp/lib/libomp.dylib` from R) is the
   canary for the runtimes having drifted apart.
6. **If R's `SHLIB_OPENMP_*` macro is empty or wrong**, the user's R
   install is the problem — fix the R install, or add a
   `SystemRequirements:` entry pointing at the OpenMP runtime, rather
   than papering over it from the package's `build.rs`.

## Why It Looks macOS-Specific

The bug is portable; the masking behavior is not.

On Linux with GCC, `-fopenmp` typically selects `libgomp`, and the ELF dynamic
linker often tolerates unresolved or already-satisfied symbols in ways that make
a missing package-level OpenMP link flag appear harmless. If another library has
already loaded the OpenMP runtime into the R process, the package may load by
accident.

On macOS, package code is loaded as a Mach-O bundle by `dyn.load()`. If the
bundle references OpenMP symbols but does not record a loadable dependency on
the right runtime, loading fails immediately. The common symptom is an error
mentioning `libomp.dylib`, `@rpath/libomp.dylib`, or an unresolved `omp_*` /
`__kmpc_*` symbol.

Windows can have the same class of problem, but it usually appears as a missing
DLL or as a toolchain mismatch rather than as a `libomp.dylib` error.

## The Rule for R Package Makevars

If package C or C++ code is compiled with an OpenMP macro, the matching macro
must also be present in `PKG_LIBS`:

```makefile
PKG_CFLAGS = $(SHLIB_OPENMP_CFLAGS)
PKG_LIBS = $(SHLIB_OPENMP_CFLAGS)
```

For C++:

```makefile
PKG_CXXFLAGS = $(SHLIB_OPENMP_CXXFLAGS)
PKG_LIBS = $(SHLIB_OPENMP_CXXFLAGS)
```

For Fortran, R packages are usually linked by the C or C++ compiler, so the
compile flag and link flag can intentionally differ:

```makefile
PKG_FFLAGS = $(SHLIB_OPENMP_FFLAGS)
PKG_LIBS = $(SHLIB_OPENMP_CFLAGS)
```

If a package contains only Fortran sources using OpenMP and needs the Fortran
compiler to link the shared object, use R's `USE_FC_TO_LINK` pattern instead:

```makefile
USE_FC_TO_LINK =
PKG_FFLAGS = $(SHLIB_OPENMP_FFLAGS)
PKG_LIBS = $(SHLIB_OPENMP_FFLAGS)
```

Do not hard-code `-lgomp` or `-lomp` in portable package code. The R macros
carry the compiler-specific flag, and that flag may also set the runtime search
path correctly.

## What Changes for miniextendr Packages

miniextendr packages still end as ordinary R package shared objects:

1. Cargo builds Rust code into a static library.
2. `R CMD INSTALL` links that static library into the package shared object.
3. R loads that shared object with `dyn.load()`.

That final shared object is where the OpenMP runtime dependency must be visible.
If a Rust dependency compiles C/C++ code with OpenMP internally, R's
`SHLIB_OPENMP_*FLAGS` macros are not automatically propagated into Cargo's
native build scripts. You need to ensure both sides agree:

- the native code was compiled with the intended OpenMP-capable compiler;
- the final package link line includes the matching OpenMP runtime flag;
- on macOS, the resulting shared object can find the matching `libomp.dylib` at
  load time.

## Inspection Commands

After installing or building the package, inspect the compiled shared object.
Replace `mypkg` with the actual package name.

On macOS:

```bash
otool -L src/mypkg.so
```

Look for `libomp.dylib` when the package really uses OpenMP. If the dependency
is recorded as `@rpath/libomp.dylib`, also check that the package or R process
has a usable rpath for the installed OpenMP runtime.

On Linux:

```bash
ldd src/mypkg.so
```

Look for `libgomp.so`, `libomp.so`, or the runtime used by the selected
compiler. A package that uses OpenMP but shows no OpenMP runtime dependency is
usually relying on accidental symbol availability.

On Windows:

```bash
objdump -p src/mypkg.dll | grep 'DLL Name'
```

Check for the OpenMP runtime DLL expected by the active Rtools or compiler
toolchain.

## Practical Guidance

Keep OpenMP use in one compiler family when possible. Mixing Apple `clang`,
LLVM `clang`, GCC, `gfortran`, and vendor OpenMP runtimes can create packages
that compile successfully but fail at load time.

On macOS, avoid assuming that OpenMP is available because `clang` exists.
Apple `clang` needs an external OpenMP setup, and the `libomp.dylib` used at
runtime should match the compiler support used at compile time.

When a package loads on Linux but not macOS, check the final shared object
before changing Rust code. The common failure is not miniextendr initialization;
it is that R cannot load the package bundle because the OpenMP runtime
dependency is missing, mismatched, or unreachable.

## See Also

- [LINKING.md](LINKING.md)
- [R_BUILD_SYSTEM.md](R_BUILD_SYSTEM.md)
- [Writing R Extensions: OpenMP support](https://cran.r-project.org/doc/manuals/r-release/R-exts.html#OpenMP-support)
- [R Installation and Administration: OpenMP Support](https://cran.r-project.org/doc/manuals/r-release/R-admin.html#OpenMP-Support)
