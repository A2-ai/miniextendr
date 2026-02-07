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
just configure          # Sync workspace crates to vendor/
just rcmdinstall        # Compile and install
```

If you changed **macros** specifically:

```bash
just configure          # 1. Sync macro crate to vendor/
just rcmdinstall        # 2. Build with new macros
just devtools-document  # 3. Regenerate R wrappers using new macros
just rcmdinstall        # 4. Rebuild with regenerated wrappers
```

### Stale R wrappers after macro changes

Symptoms: R wrappers don't reflect your new `#[miniextendr]` functions.

```bash
NOT_CRAN=true just devtools-document  # Regenerate R/miniextendr_wrappers.R
NOT_CRAN=true just rcmdinstall        # Rebuild with new wrappers
```

---

## R Function Issues

### "could not find function" in R tests

Functions exist in Rust but aren't callable from R. Check in order:

1. **Function is `pub`** -- non-pub functions don't get `@export` in R wrappers
2. **Function is in `miniextendr_module!`** -- check the module declaration in the same file
3. **Sub-module is `use`d** -- check the parent module's `miniextendr_module!` has `use submodule;`
4. **NAMESPACE is stale** -- run `just devtools-document` to regenerate

Quick fix:
```bash
NOT_CRAN=true just devtools-document && NOT_CRAN=true just rcmdinstall
```

**Note:** The lint tool (`just lint`) catches issues #1 and #2 but does NOT catch missing `use submodule;` in the parent module. This is a known limitation.

### "package not found" when running tests

The rpkg package needs to be installed first:

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
just configure          # Refresh vendored copies
```

### Editing generated files has no effect

Many files in rpkg are generated from `.in` templates. Always edit the template:

| Generated file | Edit this instead |
|---|---|
| `rpkg/src/rust/Cargo.toml` | `rpkg/src/rust/Cargo.toml.in` |
| `rpkg/src/rust/.cargo/config.toml` | `rpkg/src/rust/cargo-config.toml.in` |
| `rpkg/src/rust/document.rs` | `rpkg/src/rust/document.rs.in` |
| `rpkg/src/Makevars` | `rpkg/src/Makevars.in` |
| `rpkg/src/entrypoint.c` | `rpkg/src/entrypoint.c.in` |
| `rpkg/src/mx_abi.c` | `rpkg/src/mx_abi.c.in` |
| `rpkg/configure` | `rpkg/configure.ac` (then run `autoconf`) |

---

## Lint Issues

### Lint reports "not listed in miniextendr_module!"

Add the function to the appropriate `miniextendr_module!` block:

```rust
miniextendr_module! {
    mod my_module;
    fn my_new_function;  // Add this
}
```

### Lint reports "has no #[miniextendr] attribute"

Either add `#[miniextendr]` to the function, or remove it from the module declaration.

### Lint passes but functions are still invisible to R

The lint doesn't check that child modules are wired into the parent. Verify the parent module has `use child_module;` in its `miniextendr_module!`.

---

## Windows Issues

Windows uses `configure.win` / `configure.ucrt` instead of the autoconf-based `configure` script. If Windows builds fail:

1. Check `rpkg/configure.win` and `rpkg/configure.ucrt` exist
2. Check `rpkg/cleanup.win` and `rpkg/cleanup.ucrt` exist
3. Ensure paths use forward slashes in R code
