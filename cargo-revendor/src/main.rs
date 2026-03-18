//! cargo-revendor: Vendor Rust dependencies for R packages
//!
//! Unlike `cargo vendor`, this handles:
//! - Path dependencies (workspace members) via `cargo package`
//! - Workspace inheritance resolution
//! - Opt-in stripping of test/bench/example/bin directories
//! - Cleaning TOML sections that reference stripped directories
//! - Inter-crate path dependency rewriting
//! - Empty checksum generation for vendored crates
//! - Caching via Cargo.lock hash (skip re-vendoring when deps unchanged)
//! - JSON output for machine consumption

mod cache;
mod metadata;
mod package;
mod strip;
mod vendor;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// Verbosity level (0=quiet, 1=-v, 2=-vv, 3=-vvv)
#[derive(Clone, Copy, Debug)]
pub struct Verbosity(pub u8);

impl Verbosity {
    pub fn info(self) -> bool {
        self.0 >= 1
    }
    pub fn debug(self) -> bool {
        self.0 >= 2
    }
    pub fn trace(self) -> bool {
        self.0 >= 3
    }
}

#[derive(Parser)]
#[command(
    name = "cargo-revendor",
    about = "Vendor Rust dependencies for R packages, handling workspace/path deps"
)]
struct Cli {
    /// When invoked as `cargo revendor`, cargo passes "revendor" as first arg
    #[arg(hide = true, default_value = "revendor")]
    _subcommand: String,

    /// Path to the Cargo.toml of the R package's Rust crate
    #[arg(long, default_value = "src/rust/Cargo.toml")]
    manifest_path: PathBuf,

    /// Output directory for vendored crates
    #[arg(long, short, default_value = "vendor")]
    output: PathBuf,

    /// Root of the monorepo/workspace containing path dependencies.
    /// If not set, auto-detected from workspace metadata.
    #[arg(long)]
    source_root: Option<PathBuf>,

    /// Allow dirty working directory when running cargo package
    #[arg(long, default_value_t = true)]
    allow_dirty: bool,

    /// Strip test directories from vendored crates
    #[arg(long)]
    strip_tests: bool,

    /// Strip bench directories from vendored crates
    #[arg(long)]
    strip_benches: bool,

    /// Strip example directories from vendored crates
    #[arg(long)]
    strip_examples: bool,

    /// Strip binary directories from vendored crates
    #[arg(long)]
    strip_bins: bool,

    /// Strip all non-essential directories (tests, benches, examples, bins)
    #[arg(long)]
    strip_all: bool,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Force re-vendoring even if Cargo.lock hasn't changed
    #[arg(long)]
    force: bool,

}

impl Cli {
    fn verbosity(&self) -> Verbosity {
        Verbosity(self.verbose)
    }

    fn strip_config(&self) -> strip::StripConfig {
        if self.strip_all {
            strip::StripConfig::all()
        } else {
            strip::StripConfig {
                tests: self.strip_tests,
                benches: self.strip_benches,
                examples: self.strip_examples,
                bins: self.strip_bins,
            }
        }
    }
}

/// JSON output structure
#[derive(serde::Serialize)]
struct JsonOutput {
    vendor_dir: String,
    local_crates: Vec<String>,
    external_crates: usize,
    total_crates: usize,
    cached: bool,
    stripped: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let v = cli.verbosity();

    let manifest_path = cli
        .manifest_path
        .canonicalize()
        .with_context(|| format!("manifest not found: {}", cli.manifest_path.display()))?;

    if v.info() {
        eprintln!(
            "cargo-revendor: vendoring deps from {}",
            manifest_path.display()
        );
    }

    // Resolve output path
    let output = if cli.output.is_absolute() {
        cli.output.clone()
    } else {
        // Relative to the manifest's package root (parent of src/rust/)
        let pkg_root = manifest_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .unwrap_or(
                manifest_path
                    .parent()
                    .context("manifest path has no parent")?,
            );
        pkg_root.join(&cli.output)
    };

    // Step 0: Check cache — skip if Cargo.lock unchanged
    let lockfile = manifest_path.with_file_name("Cargo.lock");
    if !cli.force && cache::is_cached(&lockfile, &output)? {
        if v.info() {
            eprintln!("cargo-revendor: vendor/ is up to date (Cargo.lock unchanged)");
        }
        if cli.json {
            let count = std::fs::read_dir(&output)
                .map(|d| d.filter_map(|e| e.ok()).filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false)).count())
                .unwrap_or(0);
            let json = JsonOutput {
                vendor_dir: output.display().to_string(),
                local_crates: vec![],
                external_crates: count,
                total_crates: count,
                cached: true,
                stripped: vec![],
            };
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        return Ok(());
    }

    // Step 1: Load cargo metadata to discover dependencies
    let meta = metadata::load_metadata(&manifest_path)?;
    let (local_pkgs, _external_pkgs) = metadata::partition_packages(&meta, &manifest_path)?;

    // Also discover ALL workspace members from the source workspace root
    let all_workspace_members = if let Some(ref source_root) = cli.source_root {
        metadata::discover_workspace_members(source_root)?
    } else if let Some(first_local) = local_pkgs.first() {
        let ws_root = find_workspace_root(&first_local.path)?;
        metadata::discover_workspace_members(&ws_root)?
    } else {
        Vec::new()
    };

    let patch_pkgs = merge_packages(&local_pkgs, &all_workspace_members);

    if v.info() {
        eprintln!("  Local packages to vendor: {}", local_pkgs.len());
        for pkg in &local_pkgs {
            eprintln!("    - {} v{} ({})", pkg.name, pkg.version, pkg.path.display());
        }
        if v.debug() && patch_pkgs.len() > local_pkgs.len() {
            eprintln!(
                "  Additional workspace members for patching: {}",
                patch_pkgs.len() - local_pkgs.len()
            );
        }
    }

    // Step 2: Package local crates via `cargo package`
    let staging = tempfile::tempdir().context("failed to create staging dir")?;
    let vendor_staging = staging.path().join("vendor");

    let packaged = package::package_local_crates(
        &local_pkgs,
        &patch_pkgs,
        &manifest_path,
        staging.path(),
        cli.allow_dirty,
        v,
    )?;

    // Step 3: Run `cargo vendor` for external deps
    vendor::run_cargo_vendor(&manifest_path, &vendor_staging, &patch_pkgs, v)?;

    // Step 4: Extract packaged local crates into vendor staging
    for (pkg_name, crate_path) in &packaged {
        vendor::extract_crate_archive(crate_path, &vendor_staging, pkg_name, v)?;
    }

    // Step 5: Strip directories (opt-in)
    let strip_cfg = cli.strip_config();
    let stripped = if strip_cfg.any() {
        strip::strip_vendor_dir(&vendor_staging, &strip_cfg, v)?
    } else {
        vec![]
    };

    // Step 6: Rewrite inter-crate path deps for local crates
    vendor::rewrite_local_path_deps(&vendor_staging, &local_pkgs, v)?;

    // Step 7: Clear checksums
    vendor::clear_checksums(&vendor_staging)?;

    // Step 8: Move to final output directory
    if output.exists() {
        std::fs::remove_dir_all(&output)
            .with_context(|| format!("failed to remove existing {}", output.display()))?;
    }
    std::fs::rename(&vendor_staging, &output)
        .or_else(|_| copy_dir_recursive(&vendor_staging, &output))
        .with_context(|| format!("failed to move vendor to {}", output.display()))?;

    // Step 9: Generate .cargo/config.toml for source replacement
    let config_toml = vendor::generate_cargo_config(&manifest_path, &output, &local_pkgs)?;
    if v.info() {
        eprintln!("  Generated .cargo/config.toml for source replacement");
    }
    if v.debug() {
        eprintln!("{}", config_toml);
    }

    // Step 10: Strip checksums from Cargo.lock (vendored crates have empty checksums)
    vendor::strip_lock_checksums(&lockfile, &output, v)?;

    // Step 11: Save cache
    cache::save_cache(&lockfile, &output)?;

    // Count total crates
    let total = std::fs::read_dir(&output)
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    if cli.json {
        let json = JsonOutput {
            vendor_dir: output.display().to_string(),
            local_crates: packaged.iter().map(|(n, _)| n.clone()).collect(),
            external_crates: total - packaged.len(),
            total_crates: total,
            cached: false,
            stripped,
        };
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else if v.info() {
        eprintln!(
            "cargo-revendor: vendored {} local + {} external deps to {}",
            packaged.len(),
            total - packaged.len(),
            output.display()
        );
    }

    Ok(())
}

/// Find workspace root by walking up from a directory
pub fn find_workspace_root(dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let mut dir = dir.canonicalize()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            anyhow::bail!("no workspace root found");
        }
    }
}

/// Merge two package lists, deduplicating by name
fn merge_packages(
    a: &[metadata::LocalPackage],
    b: &[metadata::LocalPackage],
) -> Vec<metadata::LocalPackage> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for pkg in a.iter().chain(b.iter()) {
        if seen.insert(pkg.name.clone()) {
            result.push(pkg.clone());
        }
    }
    result
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
