use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "miniextendr",
    about = "miniextendr CLI — build R packages with Rust",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Project directory (default: current directory).
    #[arg(long, global = true, default_value = ".")]
    pub path: String,

    /// Suppress output.
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Output in JSON format.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create or add miniextendr to a project.
    Init {
        #[command(subcommand)]
        cmd: InitCmd,
    },
    /// Build, document, check, and manage R package.
    Workflow {
        #[command(subcommand)]
        cmd: WorkflowCmd,
    },
    /// Check project status.
    Status {
        #[command(subcommand)]
        cmd: StatusCmd,
    },
    /// Run cargo commands in project context.
    Cargo {
        #[command(subcommand)]
        cmd: CargoCmd,
    },
    /// Manage vendored dependencies.
    Vendor {
        #[command(subcommand)]
        cmd: VendorCmd,
    },
    /// Manage Cargo features and detection rules.
    Feature {
        #[command(subcommand)]
        cmd: FeatureCmd,
    },
    /// Rmarkdown/Quarto integration.
    Render {
        #[command(subcommand)]
        cmd: RenderCmd,
    },
    /// Dynamic Rust compilation.
    Rust {
        #[command(subcommand)]
        cmd: RustCmd,
    },
    /// Show configuration.
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Run miniextendr-lint (checks macro/module consistency).
    Lint,
    /// Clean build artifacts.
    Clean,
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
    /// Internal tools for miniextendr framework development.
    #[cfg(feature = "dev")]
    Dev {
        #[command(subcommand)]
        cmd: DevCmd,
    },
}

// region: Dev (framework development commands — behind `dev` feature)

#[cfg(feature = "dev")]
#[derive(Subcommand)]
pub enum DevCmd {
    /// Run benchmarks (miniextendr-bench).
    Bench {
        #[command(subcommand)]
        cmd: BenchCmd,
    },
    /// Cross-package trait dispatch testing (tests/cross-package).
    Cross {
        #[command(subcommand)]
        cmd: CrossCmd,
    },
    /// Template drift checking (minirextendr/inst/templates vs rpkg).
    Templates {
        #[command(subcommand)]
        cmd: TemplatesCmd,
    },
}
// endregion

// region: Init

#[derive(Subcommand)]
pub enum InitCmd {
    /// Create a new R package with miniextendr.
    Package {
        /// Destination path for the new package.
        path: String,
    },
    /// Create a Rust workspace with embedded R package.
    Monorepo {
        /// Destination path for the monorepo.
        path: String,
        /// Package name.
        #[arg(long)]
        package: Option<String>,
        /// Rust crate name.
        #[arg(long)]
        crate_name: Option<String>,
        /// R package directory name within the monorepo.
        #[arg(long)]
        rpkg_name: Option<String>,
        /// Local path to miniextendr source (for development).
        #[arg(long)]
        local_path: Option<String>,
        /// miniextendr version/branch (default: "main").
        #[arg(long = "miniextendr-version", default_value = "main")]
        miniextendr_version: String,
    },
    /// Add miniextendr scaffolding to an existing project.
    Use {
        /// Template type: auto, rpkg, or monorepo.
        #[arg(long, default_value = "auto")]
        template_type: String,
        /// R package directory name (for monorepo template).
        #[arg(long)]
        rpkg_name: Option<String>,
        /// miniextendr version/branch.
        #[arg(long = "miniextendr-version", default_value = "main")]
        miniextendr_version: String,
        /// Local path to miniextendr source.
        #[arg(long)]
        local_path: Option<String>,
    },
}
// endregion

// region: Workflow

#[derive(Subcommand)]
pub enum WorkflowCmd {
    /// Run autoconf to generate configure script from configure.ac.
    Autoconf,
    /// Run ./configure to generate Makevars and build config.
    /// Install mode (source vs tarball) is auto-detected from
    /// `inst/vendor.tar.xz` presence — no flag to set.
    Configure,
    /// Generate R wrappers from Rust code (devtools::document).
    Document,
    /// Full two-pass build: autoconf, configure, install, document, install.
    Build {
        /// Skip installation step.
        #[arg(long)]
        no_install: bool,
    },
    /// Install R package (R CMD INSTALL or devtools::install).
    Install {
        /// Use R CMD INSTALL instead of devtools::install.
        #[arg(long)]
        r_cmd: bool,
        /// Additional args for R CMD INSTALL.
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run R CMD check or devtools::check.
    Check {
        /// Error level: error, warning, or note.
        #[arg(long, default_value = "warning")]
        error_on: String,
        /// Directory to save check output.
        #[arg(long)]
        check_dir: Option<String>,
        /// Extra args for R CMD check.
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Run R package tests (devtools::test).
    Test {
        /// Test file filter pattern.
        #[arg(long, short)]
        filter: Option<String>,
    },
    /// Comprehensive project health check.
    Doctor,
    /// Upgrade miniextendr package to latest conventions.
    Upgrade,
    /// Validate Rust toolchain is available.
    CheckRust,
    /// Sync project: autoconf + configure + document.
    Sync,
    /// Link package for development (devtools::load_all).
    DevLink,
}
// endregion

// region: Status

#[derive(Subcommand)]
pub enum StatusCmd {
    /// Check if project has miniextendr setup (native, no R needed).
    Has,
    /// Show which miniextendr files are present/missing.
    Show,
    /// Validate miniextendr configuration is ready to build.
    Validate,
}
// endregion

// region: Cargo

/// Shared build options for cargo commands.
#[derive(clap::Args, Clone, Debug)]
pub struct CargoBuildOpts {
    /// Build in release mode.
    #[arg(long)]
    pub release: bool,
    /// Comma-separated list of features to enable.
    #[arg(long)]
    pub features: Option<String>,
    /// Disable default features.
    #[arg(long)]
    pub no_default_features: bool,
    /// Enable all features.
    #[arg(long)]
    pub all_features: bool,
    /// Build target triple.
    #[arg(long)]
    pub target: Option<String>,
    /// Enable offline mode.
    #[arg(long)]
    pub offline: bool,
}

#[derive(Subcommand)]
pub enum CargoCmd {
    /// Initialize Rust crate in src/rust.
    Init {
        /// Crate name.
        #[arg(long)]
        name: Option<String>,
        /// Rust edition.
        #[arg(long, default_value = "2024")]
        edition: String,
    },
    /// Create a new Rust crate in the workspace.
    New {
        /// Crate name.
        name: String,
        /// Create a library crate (default).
        #[arg(long, default_value_t = true)]
        lib: bool,
        /// Rust edition.
        #[arg(long, default_value = "2024")]
        edition: String,
    },
    /// Add a Rust dependency.
    Add {
        /// Dependency name.
        dep: String,
        /// Features to enable.
        #[arg(long)]
        features: Option<String>,
        /// Disable default features.
        #[arg(long)]
        no_default_features: bool,
        /// Add as optional dependency.
        #[arg(long)]
        optional: bool,
        /// Rename the dependency.
        #[arg(long)]
        rename: Option<String>,
        /// Path to local crate.
        #[arg(long)]
        crate_path: Option<String>,
        /// Git repository URL.
        #[arg(long)]
        git: Option<String>,
        /// Git branch.
        #[arg(long)]
        branch: Option<String>,
        /// Git tag.
        #[arg(long)]
        tag: Option<String>,
        /// Git revision.
        #[arg(long)]
        rev: Option<String>,
        /// Add as dev dependency.
        #[arg(long)]
        dev: bool,
        /// Add as build dependency.
        #[arg(long)]
        build: bool,
        /// Dry run (don't actually add).
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove a Rust dependency.
    Rm {
        /// Dependency name.
        dep: String,
        /// Remove from dev dependencies.
        #[arg(long)]
        dev: bool,
        /// Remove from build dependencies.
        #[arg(long)]
        build: bool,
        /// Dry run.
        #[arg(long)]
        dry_run: bool,
    },
    /// Update Cargo.lock.
    Update {
        /// Specific dependency to update.
        dep: Option<String>,
        /// Exact version for update.
        #[arg(long)]
        precise: Option<String>,
        /// Dry run.
        #[arg(long)]
        dry_run: bool,
    },
    /// Run cargo build.
    Build {
        #[command(flatten)]
        opts: CargoBuildOpts,
        /// Number of parallel jobs.
        #[arg(long, short)]
        jobs: Option<u32>,
    },
    /// Run cargo check.
    Check {
        #[command(flatten)]
        opts: CargoBuildOpts,
    },
    /// Run cargo test.
    Test {
        #[command(flatten)]
        opts: CargoBuildOpts,
        /// Compile but don't run tests.
        #[arg(long)]
        no_run: bool,
        /// Additional test arguments (after --).
        #[arg(trailing_var_arg = true)]
        test_args: Vec<String>,
    },
    /// Run cargo clippy.
    Clippy {
        #[command(flatten)]
        opts: CargoBuildOpts,
        /// Check all targets.
        #[arg(long)]
        all_targets: bool,
    },
    /// Run cargo fmt.
    Fmt {
        /// Check formatting without changing files.
        #[arg(long)]
        check: bool,
    },
    /// Build Rust documentation.
    Doc {
        /// Open docs in browser.
        #[arg(long)]
        open: bool,
        /// Skip dependencies.
        #[arg(long, default_value_t = true)]
        no_deps: bool,
        #[command(flatten)]
        opts: CargoBuildOpts,
    },
    /// Search crates.io.
    Search {
        /// Search query.
        query: String,
        /// Maximum number of results.
        #[arg(long, default_value_t = 10)]
        limit: u32,
    },
    /// Show dependency tree.
    Deps {
        /// Maximum tree depth.
        #[arg(long, default_value_t = 1)]
        depth: u32,
        /// Show only duplicate dependencies.
        #[arg(long)]
        duplicates: bool,
        /// Invert the tree for a specific package.
        #[arg(long)]
        invert: Option<String>,
    },
    /// Clean cargo build artifacts.
    Clean,
}
// endregion

// region: Vendor

#[derive(Subcommand)]
pub enum VendorCmd {
    /// Create vendor.tar.xz for CRAN submission.
    Pack,
    /// List available miniextendr versions.
    Versions,
    /// Download/copy miniextendr crates to vendor/.
    Miniextendr {
        /// Version/branch to vendor.
        #[arg(long = "miniextendr-version", default_value = "main")]
        miniextendr_version: String,
        /// Destination directory.
        #[arg(long)]
        dest: Option<String>,
        /// Force re-download.
        #[arg(long)]
        refresh: bool,
        /// Local path to miniextendr source.
        #[arg(long)]
        local_path: Option<String>,
    },
    /// Vendor external crates.io dependencies.
    CratesIo,
    /// Sync vendored crates from local miniextendr source.
    Sync,
    /// Verify vendored crates match workspace sources.
    SyncCheck,
    /// Show diff between workspace and vendor.
    SyncDiff,
    /// Show cache directory info and cached versions.
    CacheInfo,
    /// Remove cached miniextendr archives.
    CacheClear {
        /// Specific version to clear (all if omitted).
        #[arg(long = "cache-version")]
        cache_version: Option<String>,
    },
    /// Vendor a local path dependency for CRAN submission.
    UseLib {
        /// Crate name to vendor.
        crate_name: String,
        /// Development path override.
        #[arg(long)]
        dev_path: Option<String>,
    },
}
// endregion

// region: Feature

#[derive(Subcommand)]
pub enum FeatureCmd {
    /// Enable a feature: r6, s4, s7, serde, vctrs, rayon, build-rs, knitr, rmarkdown, quarto, feature-detection.
    Enable {
        /// Feature name.
        name: String,
    },
    /// List Cargo features and optional dependencies.
    List,
    /// Configure-time feature detection.
    Detect {
        #[command(subcommand)]
        cmd: FeatureDetectCmd,
    },
    /// Feature detection rules.
    Rule {
        #[command(subcommand)]
        cmd: FeatureRuleCmd,
    },
}

#[derive(Subcommand)]
pub enum FeatureDetectCmd {
    /// Set up configure-time feature detection infrastructure.
    Init,
    /// Update runtime feature detection after adding/removing features.
    Update,
}

#[derive(Subcommand)]
pub enum FeatureRuleCmd {
    /// Add a feature detection rule.
    Add {
        /// Cargo feature name.
        feature: String,
        /// Detection expression (R code returning TRUE/FALSE).
        detect: String,
        /// Cargo dependency specification.
        #[arg(long)]
        cargo_spec: Option<String>,
        /// Mark as optional dependency.
        #[arg(long)]
        optional_dep: bool,
    },
    /// Remove a feature detection rule.
    Remove {
        /// Feature name to remove.
        feature: String,
    },
    /// List current feature detection rules.
    List,
}
// endregion

// region: Render

#[derive(Subcommand)]
pub enum RenderCmd {
    /// Set up knitr integration.
    KnitrSetup,
    /// Set up rmarkdown integration.
    Rmarkdown,
    /// Set up Quarto integration.
    Quarto,
    /// Run Quarto pre-render hook.
    QuartoPre,
    /// HTML document format with miniextendr sync.
    Html,
    /// PDF document format with miniextendr sync.
    Pdf,
    /// Word document format with miniextendr sync.
    Word,
}
// endregion

// region: Rust

#[derive(Subcommand)]
pub enum RustCmd {
    /// Source Rust code dynamically.
    Source {
        /// Rust source file or inline code.
        code: String,
    },
    /// Define a single Rust function.
    Function {
        /// Rust function code.
        code: String,
    },
    /// Clean compiled Rust code.
    Clean,
}
// endregion

// region: Config

#[derive(Subcommand)]
pub enum ConfigCmd {
    /// Show current miniextendr.yml config.
    Show,
    /// Show default config values.
    Defaults,
}
// endregion

// region: Bench (dev feature)

#[cfg(feature = "dev")]
#[derive(Subcommand)]
pub enum BenchCmd {
    /// Run all benchmarks.
    Run {
        /// Extra cargo flags.
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Core benchmarks (default features, high-signal targets).
    Core {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Feature-gated benchmarks (connections, rayon, etc).
    Features {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Full benchmark suite (core + feature matrix).
    Full {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// R-side benchmarks (requires rpkg installed).
    R,
    /// Save structured baseline (text + CSV + metadata).
    Save {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Show top benchmarks from a baseline.
    Compare {
        /// CSV file to compare.
        csv_file: Option<String>,
    },
    /// Check for regressions between last 2 baselines.
    Drift {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// List saved baselines with metadata.
    Info,
    /// Macro compile-time perf (synthetic crates).
    Compile {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Lint scan performance.
    LintBench {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Check benchmark crate compiles.
    Check {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}
// endregion

// region: Cross (dev feature)

#[cfg(feature = "dev")]
#[derive(Subcommand)]
pub enum CrossCmd {
    /// Configure both cross-package test packages.
    Configure,
    /// Build and install both packages.
    Install,
    /// Regenerate docs for both packages.
    Document,
    /// Run cross-package tests.
    Test,
    /// R CMD check both packages.
    Check,
    /// Clean both packages.
    Clean,
}
// endregion

// region: Templates (dev feature)

#[cfg(feature = "dev")]
#[derive(Subcommand)]
pub enum TemplatesCmd {
    /// Verify templates haven't drifted beyond approved patch.
    Check,
    /// Accept current delta as approved (regenerate patch).
    Approve,
    /// Show template source mappings.
    Sources,
}
// endregion
