//! cargo-revendor: Vendor Rust dependencies for R packages
//!
//! Unlike `cargo vendor`, this handles:
//! - Path dependencies (workspace members) via `cargo package`
//! - Workspace inheritance resolution
//! - Stripping test/bench/example directories
//! - Cleaning TOML sections that reference stripped directories
//! - Inter-crate path dependency rewriting
//! - Empty checksum generation for vendored crates

mod metadata;
mod package;
mod strip;
mod vendor;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

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

    /// Don't strip test/bench/example directories
    #[arg(long)]
    no_strip: bool,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let manifest_path = cli
        .manifest_path
        .canonicalize()
        .with_context(|| format!("manifest not found: {}", cli.manifest_path.display()))?;

    eprintln!(
        "cargo-revendor: vendoring deps from {}",
        manifest_path.display()
    );

    // Step 1: Load cargo metadata to discover dependencies
    let meta = metadata::load_metadata(&manifest_path)?;
    let (local_pkgs, _external_pkgs) = metadata::partition_packages(&meta, &manifest_path)?;

    // Also discover ALL workspace members from the source workspace root,
    // because transitive deps (e.g. miniextendr-engine) may not appear in
    // the rpkg dep tree but are needed by local_pkgs during cargo package.
    let all_workspace_members = if let Some(ref source_root) = cli.source_root {
        metadata::discover_workspace_members(source_root)?
    } else {
        // Auto-detect: find workspace root from the first local package
        if let Some(first_local) = local_pkgs.first() {
            let ws_root = find_workspace_root(&first_local.path)?;
            metadata::discover_workspace_members(&ws_root)?
        } else {
            Vec::new()
        }
    };

    // Merge: local_pkgs (what we need to vendor) + all workspace members (for patching)
    let patch_pkgs = merge_packages(&local_pkgs, &all_workspace_members);

    if cli.verbose {
        eprintln!("  Local packages to vendor: {}", local_pkgs.len());
        for pkg in &local_pkgs {
            eprintln!("    - {} v{} ({})", pkg.name, pkg.version, pkg.path.display());
        }
        if patch_pkgs.len() > local_pkgs.len() {
            eprintln!(
                "  Additional workspace members for patching: {}",
                patch_pkgs.len() - local_pkgs.len()
            );
        }
    }

    // Step 2: Package local crates via `cargo package`
    let staging = tempfile::tempdir().context("failed to create staging dir")?;
    let packaged = package::package_local_crates(
        &local_pkgs,
        &patch_pkgs,
        &manifest_path,
        staging.path(),
        cli.allow_dirty,
        cli.verbose,
    )?;

    // Step 3: Run `cargo vendor` for external deps
    let vendor_staging = staging.path().join("vendor");
    vendor::run_cargo_vendor(&manifest_path, &vendor_staging, &patch_pkgs, cli.verbose)?;

    // Step 4: Extract packaged local crates into vendor staging
    for (pkg_name, crate_path) in &packaged {
        vendor::extract_crate_archive(crate_path, &vendor_staging, pkg_name, cli.verbose)?;
    }

    // Step 5: Strip test/bench/example dirs and clean TOML sections
    if !cli.no_strip {
        strip::strip_vendor_dir(&vendor_staging, cli.verbose)?;
    }

    // Step 6: Rewrite inter-crate path deps for local crates
    vendor::rewrite_local_path_deps(&vendor_staging, &local_pkgs, cli.verbose)?;

    // Step 7: Clear checksums (vendored sources don't need verification)
    vendor::clear_checksums(&vendor_staging)?;

    // Step 8: Move to final output directory
    let output = if cli.output.is_absolute() {
        cli.output.clone()
    } else {
        manifest_path
            .parent()
            .unwrap()
            .parent()
            .unwrap() // src/rust -> package root
            .parent()
            .unwrap() // package root -> monorepo root (or just package root)
            .join(&cli.output)
    };

    if output.exists() {
        std::fs::remove_dir_all(&output)
            .with_context(|| format!("failed to remove existing {}", output.display()))?;
    }
    std::fs::rename(&vendor_staging, &output)
        .or_else(|_| {
            // rename fails across filesystems, fall back to copy
            copy_dir_recursive(&vendor_staging, &output)
        })
        .with_context(|| format!("failed to move vendor to {}", output.display()))?;

    eprintln!(
        "cargo-revendor: vendored {} local + external deps to {}",
        packaged.len(),
        output.display()
    );

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
