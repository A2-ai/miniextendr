# Troubleshooting

Common issues and solutions when developing with miniextendr.

---

## Build Issues

### "configure: command not found"

The `configure` script hasn't been generated from `configure.ac`.

```bash
cd rpkg && autoconf && ./configure
# or
just configure
```

**Prerequisites:** GNU autotools must be installed:
```bash
# macOS
brew install autoconf automake

# Debian/Ubuntu
apt-get install autoconf automake
```

### "Could not find tools necessary to compile a package"

R can't find a C compiler or Rust toolchain.

1. Verify Rust is installed: `rustc --version`
2. Verify C compiler: `gcc --version` or `clang --version`
3. On macOS, install Xcode command line tools: `xcode-select --install`

### Compilation errors after changing Rust code

Run the full rebuild workflow:

```bash
just configure          # Generate Makevars and .cargo/config.toml
just rcmdinstall        # Compile and install
```

If you changed **macros** specifically:

```bash
just configure && just rcmdinstall && just force-document
```

`rcmdinstall` regenerates `R/miniextendr-wrappers.R` from the new macro output;
`force-document` then runs roxygen2 over the updated wrappers. Use
`force-document` (not `devtools-document`) here — it bypasses
`needs_roxygenize()`'s mtime cache, which may not catch macro-layer changes.

### Stale R wrappers after macro changes

Symptoms: R wrappers don't reflect your new `#[miniextendr]` functions.

```bash
just rcmdinstall        # Regenerate R/miniextendr-wrappers.R from macro output
just force-document     # Re-run roxygen2 over the updated wrappers
```

---

## R Function Issues

### "could not find function" in R tests

Functions exist in Rust but aren't callable from R. Check in order:

1. **Function is `pub`** -- non-pub functions don't get `@export` in R wrappers
2. **Function has `#[miniextendr]`** -- check the attribute is present
3. **NAMESPACE is stale** -- run [`just force-document`](https://github.com/A2-ai/miniextendr/blob/main/justfile) (after `rcmdinstall`) to regenerate

Quick fix:
```bash
just rcmdinstall && just force-document
```

**Note:** The lint tool ([`just lint`](https://github.com/A2-ai/miniextendr/blob/main/justfile)) catches issues #1 and #2 at build time.

### "package not found" when running tests

The example package (`rpkg/`) needs to be installed first:

```bash
just rcmdinstall
```

---

## Installation Issues

### Permission errors ("no permission to install to directory")

Use a local library path:

```bash
R_LIBS=/tmp/miniextendr_lib R CMD INSTALL rpkg
# or
R_LIBS=/tmp/miniextendr_lib just rcmdinstall
```

Alternatively, use `devtools::install()` which handles library paths:

```bash
just devtools-install
```

---

## Cross-Package Issues

### Cross-package tests fail

Rebuild both packages:

```bash
just cross-install  # Installs producer.pkg and consumer.pkg
just cross-test     # Run tests
```

### TypedExternal dispatch fails across packages

Ensure both packages are built against the same version of miniextendr-api. The `TypedExternal` trait uses R symbols for type identification, and version mismatches can cause type lookups to fail silently.

---

## Vendor / Configure Issues

### Vendored crates are out of date

After modifying workspace crates (miniextendr-api, miniextendr-macros, etc.):

```bash
just vendor-sync-check  # Check for drift
just vendor             # Refresh vendored copies (rpkg/vendor/ + inst/vendor.tar.xz)
```

### `failed to load manifest for dependency` for a local crate

Symptom: a `path = ...` dependency on a crate outside your package (an "engine"
crate, a sibling library) builds fine with `cargo build` but fails under
`R CMD INSTALL` / `devtools::install()`:

```
error: failed to load manifest for dependency `my_engine`
  ... No such file or directory
```

Cause: R copies the package into a temporary build directory before compiling
the Rust staticlib, so a **relative** path (`../../../engine`) resolves against
the temp location, which doesn't contain the crate. Fix: use an **absolute**
path in `src/rust/Cargo.toml`:

```toml
# ❌ relative — breaks under R's temp-copy
my_engine = { path = "../../../engine" }
# ✅ absolute — read live from its real location
my_engine = { path = "/abs/path/to/engine" }
```

See [MINIREXTENDR.md](MINIREXTENDR.md#wrapping-a-local-rust-crate-use-an-absolute-path)
for the full explanation (and how this interacts with `use_vendor_lib()` and
vendoring).

### Editing generated files has no effect

Many files in rpkg are generated from `.in` templates. Always edit the template:

| Generated file | Edit this instead |
|---|---|
| `rpkg/src/rust/.cargo/config.toml` | `rpkg/src/rust/cargo-config.toml.in` |
| `rpkg/src/Makevars` | `rpkg/src/Makevars.in` |
| `rpkg/configure` | `rpkg/configure.ac` (then run `autoconf`) |

---

## Lint Issues

### Lint reports missing `#[miniextendr]` attribute

Add `#[miniextendr]` to the function or impl block. Registration is automatic via linkme distributed slices.

### Lint passes but functions are still invisible to R

Ensure the function is `pub` and has `#[miniextendr]`, then run `just rcmdinstall && just force-document` to regenerate R wrappers.

---

## Windows Issues

Windows uses `configure.win` / `configure.ucrt` instead of the autoconf-based `configure` script. If Windows builds fail:

1. Check `rpkg/configure.win` and `rpkg/configure.ucrt` exist
2. Check `rpkg/cleanup.win` and `rpkg/cleanup.ucrt` exist
3. Ensure paths use forward slashes in R code

### `R CMD check` hangs at "checking tests" on Windows

If you depend on a crate that spins up a multi-threaded Tokio runtime
(`DataFusion`, `reqwest` with the default client, etc.), `R CMD check`
on Windows can hang indefinitely at `* checking tests ...`. The worker
threads outlive R's main thread and keep the Rterm stdout pipe open;
`system2(stdout = TRUE)` then waits forever.

Fix by constructing a `current_thread` runtime on the worker thread:

```rust
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .expect("tokio current_thread runtime");
rt.block_on(async { /* ... */ });
```

No background threads, no stranded pipe handles, no hang. Applies on
Linux/macOS too; they just don't manifest as a hang because the pipe
closes on process exit.
