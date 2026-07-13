# miniextendr v0.1.0

`miniextendr` — maintainer CLI for miniextendr-based R packages.

Wraps the configure/install/document/vendor development loop in one
binary (`miniextendr <command>`). This is a binary-only crate; the
framework runtime lives in `miniextendr-api` and the scaffolding for
end-user packages in the `minirextendr` R package.

---

## Modules

### `commands::cargo`

`pub mod cargo;`

### `commands::config`

`pub mod config;`

### `commands::feature`

`pub mod feature;`

### `commands::init`

`pub mod init;`

### `commands::lint`

`pub mod lint;`

### `commands::render`

`pub mod render;`

### `commands::rust`

`pub mod rust;`

### `commands::status`

`pub mod status;`

### `commands::vendor`

`pub mod vendor;`

### `commands::workflow`

`pub mod workflow;`

---

## Structs

### `cli::CargoBuildOpts`

```rust
pub struct CargoBuildOpts
```

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

```rust
pub struct Cli
```

**Fields:**

- `path`: `String`
  - Project directory (default: current directory).
- `quiet`: `bool`
  - Suppress output.
- `json`: `bool`
  - Output in JSON format.
- `command`: `Command`

### `project::ProjectContext`

```rust
pub struct ProjectContext
```

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

**Inherent associated items:**

#### `description_field`

```rust
fn description_field(self: &Self, field: &str) -> Option<String>
```

Read a field's value from `DESCRIPTION`, if present.

DCF (Debian Control File, the format `DESCRIPTION` uses) allows a
field's value to continue onto following lines: a continuation line
starts with whitespace and is joined onto the value of the field it
extends, separated by a single space.

#### `discover`

```rust
fn discover(path: &Path) -> Result<Self>
```

Discover project structure starting from `path`.

#### `has_miniextendr`

```rust
fn has_miniextendr(self: &Self) -> bool
```

Check if this looks like a miniextendr project.

#### `package_name`

```rust
fn package_name(self: &Self) -> Option<String>
```

The `Package` field from `DESCRIPTION`, if present.

#### `require_cargo_manifest`

```rust
fn require_cargo_manifest(self: &Self) -> Result<&Path>
```

Returns the cargo manifest path, or an error with guidance.

#### `require_configure_ac`

```rust
fn require_configure_ac(self: &Self) -> Result<&Path>
```

Returns the configure.ac path, or an error with guidance.

---

## Enums

### `cli::CargoCmd`

```rust
pub enum CargoCmd
```

**Variants:**

- `Init { name: Option<String>, edition: String }`
  - Initialize Rust crate in src/rust.
- `New { name: String, lib: bool, edition: String }`
  - Create a new Rust crate in the workspace.
- `Add { dep: String, features: Option<String>, no_default_features: bool, optional: bool, rename: Option<String>, crate_path: Option<String>, git: Option<String>, branch: Option<String>, tag: Option<String>, rev: Option<String>, dev: bool, build: bool, dry_run: bool }`
  - Add a Rust dependency.
- `Rm { dep: String, dev: bool, build: bool, dry_run: bool }`
  - Remove a Rust dependency.
- `Update { dep: Option<String>, precise: Option<String>, dry_run: bool }`
  - Update Cargo.lock.
- `Build { opts: CargoBuildOpts, jobs: Option<u32> }`
  - Run cargo build.
- `Check { opts: CargoBuildOpts }`
  - Run cargo check.
- `Test { opts: CargoBuildOpts, no_run: bool, test_args: Vec<String> }`
  - Run cargo test.
- `Clippy { opts: CargoBuildOpts, all_targets: bool }`
  - Run cargo clippy.
- `Fmt { check: bool }`
  - Run cargo fmt.
- `Doc { open: bool, no_deps: bool, opts: CargoBuildOpts }`
  - Build Rust documentation.
- `Search { query: String, limit: u32 }`
  - Search crates.io.
- `Deps { depth: u32, duplicates: bool, invert: Option<String> }`
  - Show dependency tree.
- `Clean`
  - Clean cargo build artifacts.

### `cli::Command`

```rust
pub enum Command
```

**Variants:**

- `Init { cmd: InitCmd }`
  - Create or add miniextendr to a project.
- `Workflow { cmd: WorkflowCmd }`
  - Build, document, check, and manage R package.
- `Status { cmd: StatusCmd }`
  - Check project status.
- `Cargo { cmd: CargoCmd }`
  - Run cargo commands in project context.
- `Vendor { cmd: VendorCmd }`
  - Manage vendored dependencies.
- `Feature { cmd: FeatureCmd }`
  - Manage Cargo features and detection rules.
- `Render { cmd: RenderCmd }`
  - Rmarkdown/Quarto integration.
- `Rust { cmd: RustCmd }`
  - Dynamic Rust compilation.
- `Config { cmd: ConfigCmd }`
  - Show configuration.
- `Lint`
  - Run miniextendr-lint (checks macro/module consistency).
- `Clean`
  - Clean build artifacts.
- `Completions { shell: clap_complete::Shell }`
  - Generate shell completions.

### `cli::ConfigCmd`

```rust
pub enum ConfigCmd
```

**Variants:**

- `Show`
  - Show current miniextendr.yml config.
- `Defaults`
  - Show default config values.

### `cli::FeatureCmd`

```rust
pub enum FeatureCmd
```

**Variants:**

- `Enable { name: String }`
  - Enable a feature: r6, s4, s7, serde, vctrs, rayon, build-rs, knitr, rmarkdown, quarto, feature-detection.
- `List`
  - List Cargo features and optional dependencies.
- `Detect { cmd: FeatureDetectCmd }`
  - Configure-time feature detection.
- `Rule { cmd: FeatureRuleCmd }`
  - Feature detection rules.

### `cli::FeatureDetectCmd`

```rust
pub enum FeatureDetectCmd
```

**Variants:**

- `Init`
  - Set up configure-time feature detection infrastructure.
- `Update`
  - Update runtime feature detection after adding/removing features.

### `cli::FeatureRuleCmd`

```rust
pub enum FeatureRuleCmd
```

**Variants:**

- `Add { feature: String, detect: String, cargo_spec: Option<String>, optional_dep: bool }`
  - Add a feature detection rule.
- `Remove { feature: String }`
  - Remove a feature detection rule.
- `List`
  - List current feature detection rules.

### `cli::InitCmd`

```rust
pub enum InitCmd
```

**Variants:**

- `Package { path: String }`
  - Create a new R package with miniextendr.
- `Monorepo { path: String, package: Option<String>, crate_name: Option<String>, rpkg_name: Option<String>, local_path: Option<String>, miniextendr_version: String }`
  - Create a Rust workspace with embedded R package.
- `Use { template_type: String, rpkg_name: Option<String>, miniextendr_version: String, local_path: Option<String> }`
  - Add miniextendr scaffolding to an existing project.

### `cli::RenderCmd`

```rust
pub enum RenderCmd
```

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

```rust
pub enum RustCmd
```

**Variants:**

- `Source { code: String }`
  - Source Rust code dynamically.
- `Function { code: String }`
  - Define a single Rust function.
- `Clean`
  - Clean compiled Rust code.

### `cli::StatusCmd`

```rust
pub enum StatusCmd
```

**Variants:**

- `Has`
  - Check if project has miniextendr setup (native, no R needed).
- `Show`
  - Show which miniextendr files are present/missing.
- `Validate`
  - Validate miniextendr configuration is ready to build.

### `cli::VendorCmd`

```rust
pub enum VendorCmd
```

**Variants:**

- `Pack`
  - Create vendor.tar.xz for CRAN submission.
- `Versions`
  - List available miniextendr versions.
- `Miniextendr { miniextendr_version: String, dest: Option<String>, refresh: bool, local_path: Option<String> }`
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
- `CacheClear { cache_version: Option<String> }`
  - Remove cached miniextendr archives.
- `UseLib { crate_name: String, dev_path: Option<String> }`
  - Vendor a local path dependency for CRAN submission.

### `cli::WorkflowCmd`

```rust
pub enum WorkflowCmd
```

**Variants:**

- `Autoconf`
  - Run autoconf to generate configure script from configure.ac.
- `Configure`
  - Run ./configure to generate Makevars and build config.
- `Document`
  - Generate R wrappers from Rust code (devtools::document).
- `Build { no_install: bool }`
  - Full two-pass build: autoconf, configure, install, document, install.
- `Install { r_cmd: bool, args: Vec<String> }`
  - Install R package (R CMD INSTALL or devtools::install).
- `Check { error_on: String, check_dir: Option<String>, args: Vec<String> }`
  - Run R CMD check or devtools::check.
- `Test { filter: Option<String> }`
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
fn bash(script: &str, cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run a shell command via `bash -c`.

### `bridge::find_rscript`

```rust
fn find_rscript() -> anyhow::Result<std::path::PathBuf>
```

Locate the `Rscript` binary.

Search order:
1. `$R_HOME/bin/Rscript`
2. `Rscript` on `$PATH`

### `bridge::has_program`

```rust
fn has_program(name: &str) -> bool
```

Check if a program is available on PATH.

### `bridge::program_version`

```rust
fn program_version(name: &str) -> Option<String>
```

Get version output from a program.

### `bridge::rscript_eval`

```rust
fn rscript_eval(expr: &str, cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run `Rscript -e '<expr>'` in the given directory.

Forwards stdout/stderr directly for interactive feel.
Returns an error if the process exits non-zero.

### `bridge::run_command`

```rust
fn run_command(program: &str, args: &[impl AsRef<std::ffi::OsStr>], cwd: &std::path::Path, quiet: bool) -> anyhow::Result<std::process::ExitStatus>
```

Run an arbitrary command, forwarding stdio.

### `bridge::run_command_capture`

```rust
fn run_command_capture(program: &str, args: &[impl AsRef<std::ffi::OsStr>], cwd: &std::path::Path) -> anyhow::Result<String>
```

Run an arbitrary command and capture stdout.

### `commands::cargo::dispatch`

```rust
fn dispatch(cmd: &crate::cli::CargoCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::config::dispatch`

```rust
fn dispatch(cmd: &crate::cli::ConfigCmd, ctx: &crate::project::ProjectContext, _quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::dispatch`

```rust
fn dispatch(cmd: &crate::cli::Command, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::feature::dispatch`

```rust
fn dispatch(cmd: &crate::cli::FeatureCmd, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::init::dispatch`

```rust
fn dispatch(cmd: &crate::cli::InitCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::lint::run`

```rust
fn run(ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

Run miniextendr-lint via cargo check on the project's Rust crate.

The lint runs as a build script; cargo check triggers it.
Lint output appears as cargo warnings.

### `commands::render::dispatch`

```rust
fn dispatch(cmd: &crate::cli::RenderCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::rust::dispatch`

```rust
fn dispatch(cmd: &crate::cli::RustCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `commands::status::dispatch`

```rust
fn dispatch(cmd: &crate::cli::StatusCmd, ctx: &crate::project::ProjectContext, _quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::vendor::dispatch`

```rust
fn dispatch(cmd: &crate::cli::VendorCmd, ctx: &crate::project::ProjectContext, quiet: bool, json: bool) -> anyhow::Result<()>
```

### `commands::workflow::dispatch`

```rust
fn dispatch(cmd: &crate::cli::WorkflowCmd, ctx: &crate::project::ProjectContext, quiet: bool) -> anyhow::Result<()>
```

### `output::print_json`

```rust
fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()>
```

Serialize `value` as pretty JSON and print it to stdout.

### `output::print_status`

```rust
fn print_status(msg: &str)
```

Print a simple status message.

### `project::find_workspace_root`

```rust
fn find_workspace_root(start: &std::path::Path) -> Option<std::path::PathBuf>
```

Find the workspace root containing `start`.

Tries `git rev-parse --show-toplevel` first (fast, accurate when in a git repo),
then falls back to walking up to 3 parent directories looking for a `Cargo.toml`
with `[workspace]`.

---

## Constants

### `project::MINIEXTENDR_CRATES`

```rust
pub const MINIEXTENDR_CRATES: &[&str] = _;
```

Miniextendr workspace crate names — the crates that get vendored/synced.
