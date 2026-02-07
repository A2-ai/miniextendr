# Maintainer Guide

This document covers maintenance tasks for the miniextendr project.

## Version Management

Versions are tracked in two places that must stay in sync:

- `Cargo.toml` (`[workspace.package].version`)
- `rpkg/DESCRIPTION` (`Version:`)

Other locations derive their version automatically:

- All Rust crates use `version.workspace = true`
- `rpkg/configure.ac` reads from `Cargo.toml`

### Bumping Version

```bash
./scripts/bump-version.sh 0.2.0
```

This updates both `Cargo.toml` and `rpkg/DESCRIPTION`.

R development versions (e.g., `0.2.0.9000`) are allowed and will match the base version `0.2.0` in CI checks.

## Regenerating Configure Scripts

After modifying `rpkg/configure.ac`, regenerate the configure script:

```bash
cd rpkg
autoreconf -vif
```

Or use the justfile:

```bash
just configure  # runs autoconf + ./configure
```

### When to Regenerate

- After editing `configure.ac`
- After editing `Makevars.in`
- After changing autoconf macros
- Before committing changes to configure.ac

### Dependencies

Requires GNU autotools:

```bash
# macOS
brew install autoconf automake

# Debian/Ubuntu
apt-get install autoconf automake
```

## Development Workflow

### Full Rebuild (after macro changes)

When changing proc-macros, the full sequence is:

```bash
just configure          # 1. Vendor macro crates to rpkg/src/vendor/
just rcmdinstall        # 2. Build and install R package
just devtools-document  # 3. Regenerate R wrappers
just rcmdinstall        # 4. Rebuild with updated wrappers
```

### Quick Iteration

For most changes:

```bash
just check              # Fast cargo check
just rcmdinstall        # Build and install
```

### Running Tests

```bash
# Rust tests
just test

# R tests
just devtools-test

# Full R CMD check
just r-cmd-check
```

## Rustdoc Maintenance

Public and internal APIs should stay documented as they evolve.

- Run a targeted doc lint snapshot when touching API-heavy modules:

```bash
RUSTFLAGS='-Wmissing-docs' cargo check -p miniextendr-api --lib
```

- Prefer documenting internals in-place with rustdoc comments (`///`) near trait
  constants, enum variants, and error fields so generated docs remain useful.
- For raw header-mirror FFI declarations, document key types and safety model;
  avoid duplicating full upstream header docs verbatim.

## CI Workflow

The GitHub Actions workflow (`.github/workflows/ci.yml`) runs:

| Job | Description |
|-----|-------------|
| `version-check` | Verifies Cargo.toml and DESCRIPTION versions match |
| `fmt` | Checks Rust formatting |
| `clippy` | Runs clippy lints |
| `rust-test` | Tests on Linux/macOS/Windows, x86_64/arm64, stable/MSRV/nightly |
| `rust-features` | Tests feature combinations |
| `docs` | Builds documentation |
| `r-check-*` | R CMD check on all platforms |
| `r-tests` | R test suite |
| `cran-check` | Strict CRAN-like check |
| `msrv` | Minimum supported Rust version (1.85) |

## Release Checklist

1. **Update version**

   ```bash
   ./scripts/bump-version.sh X.Y.Z
   ```

2. **Update changelog** (if applicable)

3. **Regenerate configure**

   ```bash
   cd rpkg && autoreconf -vif
   ```

4. **Run full test suite**

   ```bash
   just check
   just test
   just r-cmd-check
   ```

5. **Commit and tag**

   ```bash
   git add -A
   git commit -m "Release vX.Y.Z"
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin main --tags
   ```

6. **Publish to crates.io** (if applicable)

   ```bash
   cargo publish -p miniextendr-macros
   cargo publish -p miniextendr-api
   ```

## Vendoring for CRAN

R packages submitted to CRAN must be self-contained. The `configure` script handles vendoring:

```bash
just configure
```

This:

1. Syncs `miniextendr-api` and `miniextendr-macros` to `rpkg/src/vendor/`
2. Vendors crates.io dependencies (proc-macro2, quote, syn, etc.)
3. Generates `.cargo/config.toml` for offline builds

## Useful Commands

```bash
# List all just recipes
just

# Clean build artifacts
just clean

# Check all crates compile
just check

# Format code
just fmt

# Run clippy
just clippy

# Build documentation
just doc

# Expand macros (requires cargo-expand)
just expand

# Run benchmarks
just bench
```

## File Locations

| Purpose | Location |
|---------|----------|
| Workspace config | `Cargo.toml` |
| R package | `rpkg/` |
| Configure script | `rpkg/configure.ac` |
| Makefile template | `rpkg/src/Makevars.in` |
| Vendored crates | `rpkg/src/vendor/` |
| CI workflow | `.github/workflows/ci.yml` |
| Version bump script | `scripts/bump-version.sh` |
