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

First distinguish concrete-pointer conversion from trait dispatch:

- `ExternalPtr<T>` conversion uses `Any::downcast` as the authoritative type
  check and reports a type mismatch; `TypedExternal`'s R symbols are only for
  display and diagnostics.
- Cross-package trait calls use the trait ABI's tag and vtable query. Rebuild
  both provider and consumer against compatible trait definitions, then check
  that the trait and its impl are both annotated with `#[miniextendr]`.

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

Several files in rpkg are generated. Edit their actual source:

| Generated file | Edit this instead |
|---|---|
| `rpkg/src/rust/.cargo/config.toml` | the `AC_CONFIG_COMMANDS([cargo-config], ...)` block in `rpkg/configure.ac` |
| `rpkg/src/Makevars` | `rpkg/src/Makevars.in` |
| `rpkg/configure` | `rpkg/configure.ac` (then run `autoconf`) |

---

## Lint Issues

### Lint reports missing `#[miniextendr]` attribute

Add `#[miniextendr]` to the function or impl block. Registration is automatic via linkme distributed slices.

### Lint passes but functions are still invisible to R

Ensure the function is `pub` and has `#[miniextendr]`, then run
`just rcmdinstall && just force-document` to regenerate the R wrapper followed
by `NAMESPACE` and `man/*.Rd`.

---

## Load Issues

### `dlopen` fails: "symbol not found in flat namespace '_<symbol>'"

**Symptom** (macOS):

```
unable to load shared object '.../mypkg.so':
  dlopen(.../mypkg.so, 0x000A): symbol not found in flat namespace '_LLVMConstMul'
```

(Linux shows `undefined symbol` instead of "flat namespace".)

**Why it happens**

miniextendr links the Rust crate as a `staticlib` (`.a`) and R then links that
archive into `<pkg>.so` with:

```
clang ... -undefined dynamic_lookup ... libmypkg.a
```

`-undefined dynamic_lookup` defers every unresolved symbol to load time.
`dyn.load()` uses `RTLD_NOW`, so *every* unresolved symbol must be present in
an already-loaded library when the `.so` is first opened -- or the load fails.

A `cargo build`/`cargo test` binary of the same crate may link and run fine.
That's because `rustc`'s **binary** linker dead-strips unreachable code,
including dead references to symbols that are never actually called. R's `.so`
link does not dead-strip those references (and `-Wl,-dead_strip` in `PKG_LIBS`
doesn't help when `codegen-units = 1` produces a single object that keeps the
reference statically reachable even though no code path reaches it at runtime).

The result: a symbol that is *referenced but never called* -- e.g. a deprecated
API a dependency dropped for a newer toolchain version -- satisfies a `rustc`
binary link but breaks `dyn.load`.

**Diagnose**

Build the staticlib and find symbols that are undefined in the archive but not
provided by anything you link:

```sh
cargo build --release --lib
LIB=target/release/libmypkg.a

# Symbols undefined in the archive
comm -23 \
  <(nm -u "$LIB" | awk '{print $NF}' | sort -u) \
  <(nm -g --defined-only "$LIB" | awk '{print $NF}' | sort -u)
```

Cross-check the residual against the native libraries listed in your
`PKG_LIBS`. Any symbol that appears in the residual but is absent from all
linked libraries will cause `dyn.load` to fail.

**Fix: abort-on-call stubs in `src/stub.c`**

If the residual symbols are genuinely unreachable (the cargo binary runs
without them), satisfy the linker with abort-on-call stubs. R compiles every
`src/*.c` file and links the resulting `.o` files into the `.so` via
`$(OBJECTS)`, so stubs defined in `src/stub.c` resolve the references at link
time and abort loudly if ever invoked:

```c
// src/stub.c (add after the existing miniextendr_anchor block)

// ---------------------------------------------------------------------------
// Abort-on-call stubs for symbols referenced but unreachable via code paths
// that exist in the staticlib but are never executed through R.
//
// Background: R links the Rust staticlib into <pkg>.so with
//   clang -undefined dynamic_lookup
// which defers unresolved symbols to dyn.load() time (RTLD_NOW).  rustc's
// binary linker dead-strips these references; R's .so link does not.
// If a dep removed a symbol (e.g. LLVM 21 removed LLVMConstMul*) the
// staticlib still carries the reference but it is never called.  Stubbing
// here makes the .so load and aborts if the "impossible" path is reached.
// ---------------------------------------------------------------------------
#include <stdlib.h>  /* abort() */

void LLVMConstMul(void)    { abort(); }
void LLVMConstNSWMul(void) { abort(); }
void LLVMConstNUWMul(void) { abort(); }
```

Replace the symbol names with whichever symbols appear in your diagnostic
output. Remove a stub once the upstream dependency no longer references the
removed symbol (i.e. after the dep updates its minimum LLVM version).

> **Note on `stub.c` purpose:** `stub.c` already exists in every miniextendr
> scaffold to provide the `miniextendr_anchor` force-link symbol (see
> [Entrypoint](ENTRYPOINT.md#the-stubc-file)). The abort stubs above are a
> *separate* use: satisfying linker references to removed/missing native
> symbols. Both uses live in the same `stub.c`.

**Concrete example**

The dependency chain `diffsol → diffsl → inkwell` (an LLVM Rust binding)
references `LLVMConstMul`, `LLVMConstNSWMul`, and `LLVMConstNUWMul`.
LLVM 21 removed those three functions. `cargo test` of the same crate passes
because the rustc binary link never exercises those code paths. `dyn.load`
fails because R's `.so` link carries the references and `RTLD_NOW` rejects
any unresolved symbol. Adding the three abort stubs above made the package
load successfully.

This is a dependency-vs-toolchain issue, not a miniextendr bug -- but the
`staticlib → .so` linking model is miniextendr-specific, so the workaround is
not obvious without knowing how R links Rust code.

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
