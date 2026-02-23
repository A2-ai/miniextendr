# Developer Workflow

Quick reference for common development tasks. See [CLAUDE.md](../CLAUDE.md) for complete build system documentation.

## Prerequisites

- Rust toolchain (stable)
- R 4.2+ with development headers
- [just](https://github.com/casey/just) command runner (optional if using the CLI)
- autoconf (for regenerating `configure` from `configure.ac`)

> **Tip:** The `miniextendr` CLI (`miniextendr-cli/`) can replace most `just` recipes. Install with `just cli-install` or `cargo install --path miniextendr-cli --features dev`. See the [CLI README](../miniextendr-cli/README.md) for full usage.

## Quick Start

```bash
just configure          # Generate build config (REQUIRED first step)
just rcmdinstall        # Build and install the rpkg R package
just devtools-test      # Run R tests
```

## Common Workflows

### Rust-only development (fastest iteration)

```bash
just check              # Type-check all crates
just check-features     # Verify feature combinations compile
just test               # Run Rust unit tests
just clippy             # Lint
just fmt                # Format
```

### R package development

```bash
just configure          # 1. Generate Makevars, config files
just rcmdinstall        # 2. Compile Rust + install R package
just devtools-test      # 3. Run R tests
```

### After changing proc macros or `#[miniextendr]` attributes

```bash
just configure          # 1. Configure
just rcmdinstall        # 2. Build (compiles new macros)
just devtools-document  # 3. Regenerate R wrappers (runs macros)
just rcmdinstall        # 4. Rebuild with updated wrappers
```

### Adding a new `#[miniextendr]` function

1. Add `#[miniextendr]` to your `pub` function in a `.rs` file
2. Add `fn your_function;` to the `miniextendr_module!` in that file
3. If it's in a sub-module, add `use module_name;` to the parent's `miniextendr_module!`
4. Run the macro-change workflow above

### Running R CMD check

```bash
just configure          # 1. Configure
just r-cmd-build        # 2. Build tarball
just r-cmd-check        # 3. Check the tarball
```

### Cross-package tests

```bash
just cross-install      # Build and install producer.pkg + consumer.pkg
just cross-test         # Run cross-package trait ABI tests
```

### CRAN release prep

```bash
just vendor             # Package workspace crates + vendor external deps
just configure-cran     # Configure with PREPARE_CRAN=true
just r-cmd-build        # Build tarball
just r-cmd-check        # Check tarball (always check the tarball, not source dir)
```

## Troubleshooting

### "configure: command not found"

Always invoke configure with `bash`:

```bash
cd rpkg && bash ./configure     # Correct
cd rpkg && ./configure          # May fail on some systems
```

Or regenerate from `configure.ac`:

```bash
cd rpkg && autoconf && bash ./configure
```

### "Could not find tools necessary to compile a package"

If running in a sandboxed environment, compilation commands need the sandbox disabled.

### R tests fail with "could not find function"

The function exists in Rust but isn't callable from R. Check:

1. Function is `pub`
2. Function is listed in `miniextendr_module!`
3. Sub-module is `use`d in parent module's `miniextendr_module!`
4. Run `just devtools-document` to regenerate NAMESPACE

Quick fix:

```bash
just configure && just rcmdinstall && just devtools-document && just rcmdinstall
```

### Stale R wrappers after macro changes

Same fix — run the full 4-step workflow:

```bash
just configure && just rcmdinstall && just devtools-document && just rcmdinstall
```

### Permission errors during install

Use a local library path:

```bash
R_LIBS=/tmp/R_lib just rcmdinstall
```

### Lint warnings about missing module entries

Run `just lint` to check consistency between `#[miniextendr]` attributes and `miniextendr_module!` declarations. See [MACRO_ERRORS.md](MACRO_ERRORS.md) for interpreting lint output.

### Transient test failures in parallel runs

Running all test suites in parallel can cause races. Re-run failed suites individually:

```bash
just test               # If this fails, run specific crate tests
just clippy             # May fail during parallel configure
```

## Build Contexts

| Context | When | Behavior |
|---|---|---|
| `dev-monorepo` | In the monorepo (default) | Uses `[patch]` paths, no vendoring |
| `dev-detached` | No monorepo, no vendor | Uses git/network deps |
| `vendored-install` | Vendor artifacts present | Offline build |
| `prepare-cran` | `PREPARE_CRAN=true` | CRAN release prep |

## Sync Checks

```bash
just vendor-sync-check  # Verify vendored crates match workspace
just templates-check    # Verify templates haven't drifted from rpkg
just templates-approve  # Accept current template delta (after intentional changes)
```
