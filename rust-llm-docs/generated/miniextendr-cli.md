# miniextendr v0.1.0

`miniextendr` — maintainer CLI for miniextendr-based R packages.

Wraps the configure/install/document/vendor development loop in one
binary (`miniextendr <command>`). This is a binary-only crate; the
framework runtime lives in `miniextendr-api` and the scaffolding for
end-user packages in the `minirextendr` R package.

---

## Structs

### `cli::CargoBuildOpts`

Shared build options for cargo commands.

**Fields:**

- `release`: `bool`
  - Build in release mode.
- `features`: `Option<String>`
  - Comma-separated list of features to enable.
- `no_default_features`: `bool`
  - Disable default features.
- `all_features`: `bool`
  - Enable all features.
- `target`: `Option<String>`
  - Build target triple.
- `offline`: `bool`
  - Enable offline mode.

### `cli::Cli`

**Fields:**

- `path`: `String`
  - Project directory (default: current directory).
- `quiet`: `bool`
  - Suppress output.
- `json`: `bool`
  - Output in JSON format.
- `command`: `Command`

### `project::ProjectContext`

Discovered project paths.

**Fields:**

- `root`: `std::path::PathBuf`
  - The project root (where DESCRIPTION or Cargo.toml lives).
- `cargo_manifest`: `Option<std::path::PathBuf>`
  - `src/rust/Cargo.toml` if this is an R package with Rust.
- `description`: `Option<std::path::PathBuf>`
  - `DESCRIPTION` file if this is an R package.
- `configure_ac`: `Option<std::path::PathBuf>`
  - `configure.ac` if autoconf is set up.
- `configure`: `Option<std::path::PathBuf>`
  - `configure` script.

**Methods:**

#### `discover`

```rust
discover(path: &Path) -> Result<Self>
```

Discover project structure starting from `path`.

#### `has_miniextendr`

```rust
has_miniextendr(self: &Self) -> bool
```

Check if this looks like a miniextendr project.

#### `require_cargo_manifest`

```rust
require_cargo_manifest(self: &Self) -> Result<&Path>
```

Returns the cargo manifest path, or an error with guidance.

#### `require_configure_ac`

```rust
require_configure_ac(self: &Self) -> Result<&Path>
```

Returns the configure.ac path, or an error with guidance.

---

## Enums

### `cli::CargoCmd`

**Variants:**

- `Init { ... }`
  - Initialize Rust crate in src/rust.
- `New { ... }`
  - Create a new Rust crate in the workspace.
- `Add { ... }`
  - Add a Rust dependency.
- `Rm { ... }`
  - Remove a Rust dependency.
- `Update { ... }`
  - Update Cargo.lock.
- `Build { ... }`
  - Run cargo build.
- `Check { ... }`
  - Run cargo check.
- `Test { ... }`
  - Run cargo test.
- `Clippy { ... }`
  - Run cargo clippy.
- `Fmt { ... }`
  - Run cargo fmt.
- `Doc { ... }`
  - Build Rust documentation.
- `Search { ... }`
  - Search crates.io.
- `Deps { ... }`
  - Show dependency tree.
- `Clean`
  - Clean cargo build artifacts.

### `cli::Command`

**Variants:**

- `Init { ... }`
  - Create or add miniextendr to a project.
- `Workflow { ... }`
  - Build, document, check, and manage R package.
- `Status { ... }`
  - Check project status.
- `Cargo { ... }`
  - Run cargo commands in project context.
- `Vendor { ... }`
  - Manage vendored dependencies.
- `Feature { ... }`
  - Manage Cargo features and detection rules.
- `Render { ... }`
  - Rmarkdown/Quarto integration.
- `Rust { ... }`
  - Dynamic Rust compilation.
- `Config { ... }`
  - Show configuration.
- `Lint`
  - Run miniextendr-lint (checks macro/module consistency).
- `Clean`
  - Clean build artifacts.
- `Completions { ... }`
  - Generate shell completions.

### `cli::ConfigCmd`

**Variants:**

- `Show`
  - Show current miniextendr.yml config.
- `Defaults`
  - Show default config values.

### `cli::FeatureCmd`

**Variants:**

- `Enable { ... }`
  - Enable a feature: r6, s4, s7, serde, vctrs, rayon, build-rs, knitr, rmarkdown, quarto, feature-detection.
- `List`
  - List Cargo features and optional dependencies.
- `Detect { ... }`
  - Configure-time feature detection.
- `Rule { ... }`
  - Feature detection rules.

### `cli::FeatureDetectCmd`

**Variants:**

- `Init`
  - Set up configure-time feature detection infrastructure.
- `Update`
  - Update runtime feature detection after adding/removing features.

### `cli::FeatureRuleCmd`

**Variants:**

- `Add { ... }`
  - Add a feature detection rule.
- `Remove { ... }`
  - Remove a feature detection rule.
- `List`
  - List current feature detection rules.

### `cli::InitCmd`

**Variants:**

- `Package { ... }`
  - Create a new R package with miniextendr.
- `Monorepo { ... }`
  - Create a Rust workspace with embedded R package.
- `Use { ... }`
  - Add miniextendr scaffolding to an existing project.

### `cli::RenderCmd`

**Variants:**

- `KnitrSetup`
  - Set up knitr integration.
- `Rmarkdown`
  - Set up rmarkdown integration.
- `Quarto`
  - Set up Quarto integration.
- `QuartoPre`
  - Run Quarto pre-render hook.
- `Html`
  - HTML document format with miniextendr sync.
- `Pdf`
  - PDF document format with miniextendr sync.
- `Word`
  - Word document format with miniextendr sync.

### `cli::RustCmd`

**Variants:**

- `Source { ... }`
  - Source Rust code dynamically.
- `Function { ... }`
  - Define a single Rust function.
- `Clean`
  - Clean compiled Rust code.

### `cli::StatusCmd`

**Variants:**

- `Has`
  - Check if project has miniextendr setup (native, no R needed).
- `Show`
  - Show which miniextendr files are present/missing.
- `Validate`
  - Validate miniextendr configuration is ready to build.

### `cli::VendorCmd`

**Variants:**

- `Pack`
  - Create vendor.tar.xz for CRAN submission.
- `Versions`
  - List available miniextendr versions.
- `Miniextendr { ... }`
  - Download/copy miniextendr crates to vendor/.
- `CratesIo`
  - Vendor external crates.io dependencies.
- `Sync`
  - Sync vendored crates from local miniextendr source.
- `SyncCheck`
  - Verify vendored crates match workspace sources.
- `SyncDiff`
  - Show diff between workspace and vendor.
- `CacheInfo`
  - Show cache directory info and cached versions.
- `CacheClear { ... }`
  - Remove cached miniextendr archives.
- `UseLib { ... }`
  - Vendor a local path dependency for CRAN submission.

### `cli::WorkflowCmd`

**Variants:**

- `Autoconf`
  - Run autoconf to generate configure script from configure.ac.
- `Configure`
  - Run ./configure to generate Makevars and build config.
- `Document`
  - Generate R wrappers from Rust code (devtools::document).
- `Build { ... }`
  - Full two-pass build: autoconf, configure, install, document, install.
- `Install { ... }`
  - Install R package (R CMD INSTALL or devtools::install).
- `Check { ... }`
  - Run R CMD check or devtools::check.
- `Test { ... }`
  - Run R package tests (devtools::test).
- `Doctor`
  - Comprehensive project health check.
- `Upgrade`
  - Upgrade miniextendr package to latest conventions.
- `CheckRust`
  - Validate Rust toolchain is available.
- `Sync`
  - Sync project: autoconf + configure + document.
- `DevLink`
  - Link package for development (devtools::load_all).

---

## Functions

### `bridge::bash`

```rust
bash(script: &str, cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run a shell command via `bash -c`.

### `bridge::find_rscript`

```rust
find_rscript() -> anyhow::Result<std::path::PathBuf>
```

Locate the `Rscript` binary.

Search order:
1. `$R_HOME/bin/Rscript`
2. `Rscript` on `$PATH`

### `bridge::has_program`

```rust
has_program(name: &str) -> bool
```

Check if a program is available on PATH.

### `bridge::program_version`

```rust
program_version(name: &str) -> Option<String>
```

Get version output from a program.

### `bridge::rscript_eval`

```rust
rscript_eval(expr: &str, cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run `Rscript -e '<expr>'` in the given directory.

Forwards stdout/stderr directly for interactive feel.
Returns an error if the process exits non-zero.

### `bridge::run_command`

```rust
run_command(program: &str, args: &[impl AsRef<std::ffi::OsStr>], cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run an arbitrary command, forwarding stdio.

### `bridge::run_command_capture`

```rust
run_command_capture(program: &str, args: &[impl AsRef<std::ffi::OsStr>], cwd: &std::path::Path) -> anyhow::Result<String>
```

Run an arbitrary command and capture stdout.

### `commands::cargo::dispatch`

```rust
dispatch(cmd: &crate::cli::CargoCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::config::dispatch`

```rust
dispatch(cmd: &crate::cli::ConfigCmd, ctx: &crate::project::ProjectContext, _quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::dispatch`

```rust
dispatch(cmd: &crate::cli::Command, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::feature::dispatch`

```rust
dispatch(cmd: &crate::cli::FeatureCmd, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::init::dispatch`

```rust
dispatch(cmd: &crate::cli::InitCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::lint::run`

```rust
run(ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

Run miniextendr-lint via cargo check on the project's Rust crate.

The lint runs as a build script; cargo check triggers it.
Lint output appears as cargo warnings.

### `commands::render::dispatch`

```rust
dispatch(cmd: &crate::cli::RenderCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::rust::dispatch`

```rust
dispatch(cmd: &crate::cli::RustCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::status::dispatch`

```rust
dispatch(cmd: &crate::cli::StatusCmd, ctx: &crate::project::ProjectContext, _quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::vendor::dispatch`

```rust
dispatch(cmd: &crate::cli::VendorCmd, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::workflow::dispatch`

```rust
dispatch(cmd: &crate::cli::WorkflowCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `output::print_status`

```rust
print_status(msg: &str, json: bool)
```

Print a simple status message.

### `project::find_workspace_root`

```rust
find_workspace_root(start: &std::path::Path) -> Option<std::path::PathBuf>
```

Find the workspace root containing `start`.

Tries `git rev-parse --show-toplevel` first (fast, accurate when in a git repo),
then falls back to walking up to 3 parent directories looking for a `Cargo.toml`
with `[workspace]`.

---

## Constants

### `project::MINIEXTENDR_CRATES: &[&str]`

Miniextendr workspace crate names — the crates that get vendored/synced.
