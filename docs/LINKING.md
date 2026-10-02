# Linking Strategy

This document explains how miniextendr links against R's shared library (`libR`)
for both R packages and standalone Rust binaries.

## Two Linking Contexts

miniextendr supports two usage patterns with different linking requirements:

| Context | Crate | Linking | rpath |
|---------|-------|---------|-------|
| R package | miniextendr-api | R handles linking | N/A |
| Standalone binary | miniextendr-engine | build.rs | Yes |

## R Packages (miniextendr-api)

When building an R package, R's build system handles all linking:

1. `R CMD INSTALL` compiles the Rust code via `cargo build`
2. R links the resulting static library into the package's shared object
3. The shared object is loaded by R at runtime via `dyn.load()`

**No rpath needed**: R is already running, so `libR` is already loaded.

### Build Configuration

The `Makevars` (generated from `Makevars.in`) tells R how to link:

```makefile
CARGO_AR = $(CARGO_LIBDIR)/lib$(CARGO_STATICLIB_NAME).a
PKG_LIBS = $(CARGO_AR) $(BLAS_LAPACK_PKG_LIBS)
```

This uses the full path to the static archive (not `-L`/`-l` flags) to ensure
the linker picks up the staticlib directly. `BLAS_LAPACK_PKG_LIBS` is empty
unless the package enables the `blas-lapack` feature (see
[BLAS and LAPACK](#blas-and-lapack-blas-lapack)). On Windows,
`Makevars.win` also adds system libraries (`-lws2_32`, `-lntdll`, etc.).

## Standalone Binaries (miniextendr-engine)

When embedding R in a Rust binary (benchmarks, tools, etc.), the binary must:
1. Link against `libR` at compile time
2. Find `libR` at runtime

### build.rs Strategy

The `miniextendr-engine/build.rs` handles this:

```rust
// 1. Find R_HOME
let r_home = env::var("R_HOME")
    .unwrap_or_else(|_| run_r_rhome());

// 2. Add library search path
let r_libdir = format!("{}/lib", r_home);
println!("cargo:rustc-link-search=native={}", r_libdir);
println!("cargo:rustc-link-lib=R");

// 3. Add rpath for runtime (non-Windows)
println!("cargo:rustc-link-arg=-Wl,-rpath,{}", r_libdir);
```

### R Discovery

R's location is discovered in order:
1. `R_HOME` environment variable (if set)
2. `R RHOME` command output (requires R on PATH)

Set `R_HOME` explicitly for reproducible builds:

```bash
R_HOME=/usr/lib/R cargo build
```

### rpath Behavior

The build script mirrors `R CMD LINK` behavior:

| Platform | rpath Strategy |
|----------|---------------|
| Linux/macOS | `-Wl,-rpath,<R_HOME>/lib` embedded in binary |
| Windows | No rpath; uses PATH at runtime |

**Which targets get the rpath**: `cargo:rustc-link-arg` reaches only the
emitting crate's own binaries, tests, benches and examples. A crate that
depends on `miniextendr-engine` gets no rpath from it. The engine therefore
exports R's locations as `links = "R"` build-script metadata, which a direct
dependent's build script reads as `DEP_R_LIBDIR` / `DEP_R_HOME`:

```rust
// build.rs of a crate with `miniextendr-engine` in [dependencies]
fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if let Ok(libdir) = std::env::var("DEP_R_LIBDIR")
        && target_os != "windows"
    {
        println!("cargo::rustc-link-arg=-Wl,-rpath,{libdir}");
    }
}
```

`miniextendr-bench/build.rs` does exactly this. Without it, a dependent's
Linux binary needs `LD_LIBRARY_PATH=$R_HOME/lib` (macOS R.framework
libraries carry absolute install names, so there it works either way).
`rustc-link-search` / `rustc-link-lib` do propagate, so linking itself needs
nothing extra.

## BLAS and LAPACK (`blas-lapack`)

The `blas-lapack` feature of miniextendr-api declares the BLAS and LAPACK
that R itself uses (`sys::dgemm_`, `sys::dgesv_`, and the safe
`linalg::matrix_product` / `linalg::solve`). These routines live in
`libRblas` / `libRlapack` (or the external BLAS/LAPACK R was configured
with), not in libR, so every consumer links them itself.

### R packages

Writing R Extensions prescribes `PKG_LIBS = $(LAPACK_LIBS) $(BLAS_LIBS)
$(FLIBS)` for packages that call R's BLAS/LAPACK. A Rust staticlib carries no
link directives, so this has to come from Makevars:

1. Declare a package feature that forwards to the api feature, in
   `src/rust/Cargo.toml`:

   ```toml
   [features]
   blas-lapack = ["miniextendr-api/blas-lapack"]
   ```

2. `tools/detect-features.R` enables it like any other declared feature, and
   `configure` sees `blas-lapack` in `CARGO_FEATURES` and substitutes
   `BLAS_LAPACK_PKG_LIBS = $(LAPACK_LIBS) $(BLAS_LIBS) $(FLIBS)` into
   `src/Makevars`, which appends it to `PKG_LIBS` after the Rust archive
   (`Makevars.win` reuses the same variable). The value is literal make
   syntax, so the three variables are expanded from R's `Makeconf` (or an
   `R_MAKEVARS_USER` override) when R links the package.

Enabling `features = ["blas-lapack"]` on the `miniextendr-api` dependency
line instead compiles the declarations but bypasses configure, so nothing
links R's libraries. The package still builds, and its unresolved `dgemm_` /
`dgesv_` bind at load time to whatever BLAS/LAPACK the R process already has
(on CRAN macOS R 4.6, `dgesv_` then comes from Apple Accelerate's
`libLAPACK.dylib`, not R's `libRlapack`), or fail to resolve where there is
none. Use the forwarding feature.

`$(FLIBS)` names the Fortran runtime R was built with. On macOS that is
`-L/opt/gfortran/... -lgfortran`: installing such a package from source needs
CRAN's gfortran, as for any package that follows the WRE recipe.

A generic mechanism that forwards cargo's `native-static-libs` into
`PKG_LIBS` would make this per-feature line unnecessary; it is a parked plan,
not implemented.

### Cargo-built tests and binaries

With the feature on, `miniextendr-api/build.rs` (non-wasm targets) runs
`$R_HOME/bin/R CMD config LAPACK_LIBS` and `BLAS_LIBS` and turns the flags
into `rustc-link-search` / `rustc-link-lib` directives (dropping `-L`
directories that don't exist). Those propagate to every target that links
miniextendr-api: its own tests, and dependents' tests and CLIs. For the R
package staticlib they are inert.

`FLIBS` is deliberately not linked here: R's BLAS/LAPACK are shared libraries
that record their own Fortran-runtime dependency (CRAN macOS: R's bundled
`libgfortran.5.dylib`, by absolute path), and nothing on the Rust side is
Fortran. `-lgfortran` would fail on machines without the Fortran toolchain.

Two runtime requirements:

- **Start R first.** R replaces the BLAS/LAPACK error handler `xerbla` with
  one in libR that raises an R error, and `libRlapack` links libR. A
  standalone binary must initialise R (`miniextendr-engine`) before calling
  these routines, then call them on R's main thread.
- **Find the libraries.** R's own libRblas / libRlapack sit in R's library
  directory, so the `DEP_R_LIBDIR` rpath above covers them. An external BLAS
  in a non-system directory needs its own runtime path.

### webR

webR's R has its own wasm `libRblas.so` / `libRlapack.so` (flang-compiled);
the same `PKG_LIBS` line links them. `rwasm`'s `webr-vars.mk` points
`BLAS_LIBS` / `LAPACK_LIBS` at those wasm libraries. webR's own
`packages/webr-vars.mk` does not: it leaves them at the host R's native
libraries, which `wasm-ld` rejects ("unknown file type"), so a build through
it needs the two overrides (see `.github/workflows/webr.yml`, Phase 2).

## Platform Notes

### Linux

Standard R installations put `libR.so` in `$R_HOME/lib/`. The rpath ensures
the binary finds it at runtime without modifying `LD_LIBRARY_PATH`.

For system R installations (e.g., `/usr/lib/R`), `libR.so` may already be in
a standard library path and work without rpath.

### macOS

Similar to Linux. R.framework installations use `$R_HOME/lib/` for `libR.dylib`.
The rpath uses `-Wl,-rpath,<path>` which macOS linker accepts.

For Homebrew R: `R_HOME=$(brew --prefix r)/lib/R`

### Windows

Windows doesn't use rpath. Instead:
- `R.dll` must be on PATH at runtime
- Typically handled by R's installer adding R to PATH
- Or set `PATH=%R_HOME%\bin\x64;%PATH%` before running

The build script skips rpath emission on Windows.

## Troubleshooting

### "libR.so: cannot open shared object file"

The binary can't find `libR` at runtime. Solutions:
1. Set `LD_LIBRARY_PATH=$R_HOME/lib` (temporary)
2. Rebuild with correct `R_HOME` to embed proper rpath
3. Install R to a standard system location

### "R RHOME failed"

The build script couldn't find R. Solutions:
1. Install R and ensure it's on PATH
2. Set `R_HOME` explicitly: `R_HOME=/path/to/R cargo build`

### Wrong R version linked

If multiple R versions are installed:
1. Set `R_HOME` to the desired version
2. Verify with: `R_HOME=/path/to/R R --version`

### BLAS/LAPACK symbols unresolved, or bound to another library

A package built with miniextendr-api's `blas-lapack` feature but without R's
BLAS/LAPACK in `PKG_LIBS` either fails to resolve `dgemm_` / `dgesv_` or
binds them to another library already in the process. Check the link:

```bash
nm -m src/<pkg>.so | grep -E '_dgemm_|_dgesv_'   # macOS: "(from libRblas)", "(from libRlapack)"
readelf -d src/<pkg>.so | grep NEEDED            # Linux: R's BLAS/LAPACK libraries listed
```

If they are missing, declare the forwarding package feature (see "R packages"
above) so configure adds them.

### Static vs Dynamic Linking

miniextendr always links dynamically to `libR`:
- R is designed for dynamic linking
- Static linking would require R source modifications
- Dynamic linking allows version flexibility

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `R_HOME` | R installation directory | `/usr/lib/R` |
| `R_ARCH` | R architecture subdirectory | `/x64` (Windows) |
| `LD_LIBRARY_PATH` | Runtime library search (Linux) | `$R_HOME/lib` |
| `DYLD_LIBRARY_PATH` | Runtime library search (macOS) | `$R_HOME/lib` |

## See Also

- [OPENMP.md](OPENMP.md)
- [R Installation and Administration](https://cran.r-project.org/doc/manuals/r-release/R-admin.html)
- [Writing R Extensions: Linking](https://cran.r-project.org/doc/manuals/r-release/R-exts.html#Linking-GUIs-and-other-front_002dends-to-R)
