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
NOT_CRAN=true just devtools-document  # Regenerate R/miniextendr-wrappers.R
NOT_CRAN=true just rcmdinstall        # Rebuild with new wrappers
```

---

## R Function Issues

### "could not find function" in R tests

Functions exist in Rust but aren't callable from R. Check in order:

1. **Function is `pub`** -- non-pub functions don't get `@export` in R wrappers
2. **Function has `#[miniextendr]`** -- check the attribute is present
3. **NAMESPACE is stale** -- run `just devtools-document` to regenerate

Quick fix:
```bash
NOT_CRAN=true just devtools-document && NOT_CRAN=true just rcmdinstall
```

**Note:** The lint tool (`just lint`) catches issues #1 and #2 at build time.

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
just configure          # Refresh vendored copies
```

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

Ensure the function is `pub` and has `#[miniextendr]`, then run `just devtools-document` to regenerate R wrappers.

---

## Windows Issues

Windows uses `configure.win` / `configure.ucrt` instead of the autoconf-based `configure` script. If Windows builds fail:

1. Check `rpkg/configure.win` and `rpkg/configure.ucrt` exist
2. Check `rpkg/cleanup.win` and `rpkg/cleanup.ucrt` exist
3. Ensure paths use forward slashes in R code
