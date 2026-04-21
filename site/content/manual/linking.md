+++
title = "Linking Strategy"
weight = 47
description = "This document explains how miniextendr links against R's shared library (libR) for both R packages and standalone Rust binaries."
+++

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
PKG_LIBS = $(CARGO_LIBDIR)/lib$(CARGO_STATICLIB_NAME).a
```

This uses the full path to the static archive (not `-L`/`-l` flags) to ensure
the linker picks up the staticlib directly. On Windows, `Makevars.win` also
adds system libraries (`-lws2_32`, `-lntdll`, etc.).

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

**When rpath is emitted**: Only for executable targets (bin, test, bench, example).
Library targets skip rpath since the final binary determines runtime paths.

Detection logic:
```rust
fn should_emit_rpath() -> bool {
    env::var_os("CARGO_BIN_NAME").is_some()
        || env::var_os("CARGO_TEST_NAME").is_some()
        || env::var_os("CARGO_BENCH_NAME").is_some()
        || env::var_os("CARGO_EXAMPLE_NAME").is_some()
}
```

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

- [R Installation and Administration](https://cran.r-project.org/doc/manuals/r-release/R-admin.html)
- [Writing R Extensions: Linking](https://cran.r-project.org/doc/manuals/r-release/R-exts.html#Linking-GUIs-and-other-front_002dends-to-R)
