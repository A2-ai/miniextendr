# miniextendr v0.1.0

---

## Structs

### `CargoBuildOpts`

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

### `Cli`

**Fields:**

- `path`: `String`
  - Project directory (default: current directory).
- `quiet`: `bool`
  - Suppress output.
- `json`: `bool`
  - Output in JSON format.
- `command`: `Command`

### `ProjectContext`

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

### `CargoCmd`

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

### `Command`

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

### `ConfigCmd`

**Variants:**

- `Show`
  - Show current miniextendr.yml config.
- `Defaults`
  - Show default config values.

### `FeatureCmd`

**Variants:**

- `Enable { ... }`
  - Enable a feature: r6, s4, s7, serde, vctrs, rayon, build-rs, knitr, rmarkdown, quarto, feature-detection.
- `List`
  - List Cargo features and optional dependencies.
- `Detect { ... }`
  - Configure-time feature detection.
- `Rule { ... }`
  - Feature detection rules.

### `FeatureDetectCmd`

**Variants:**

- `Init`
  - Set up configure-time feature detection infrastructure.
- `Update`
  - Update runtime feature detection after adding/removing features.

### `FeatureRuleCmd`

**Variants:**

- `Add { ... }`
  - Add a feature detection rule.
- `Remove { ... }`
  - Remove a feature detection rule.
- `List`
  - List current feature detection rules.

### `InitCmd`

**Variants:**

- `Package { ... }`
  - Create a new R package with miniextendr.
- `Monorepo { ... }`
  - Create a Rust workspace with embedded R package.
- `Use { ... }`
  - Add miniextendr scaffolding to an existing project.

### `RenderCmd`

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

### `RustCmd`

**Variants:**

- `Source { ... }`
  - Source Rust code dynamically.
- `Function { ... }`
  - Define a single Rust function.
- `Clean`
  - Clean compiled Rust code.

### `StatusCmd`

**Variants:**

- `Has`
  - Check if project has miniextendr setup (native, no R needed).
- `Show`
  - Show which miniextendr files are present/missing.
- `Validate`
  - Validate miniextendr configuration is ready to build.

### `VendorCmd`

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

### `WorkflowCmd`

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
