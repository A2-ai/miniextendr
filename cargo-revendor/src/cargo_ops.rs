//! Direct cargo library integration for vendoring and packaging.
//!
//! Uses `cargo::ops::vendor` and `cargo::ops::package` instead of shelling
//! out to cargo CLI commands. This gives us:
//! - Proper error types (no stderr parsing)
//! - Direct access to the resolver
//! - Single-process execution (no temp config file hacks)

use crate::metadata::LocalPackage;
use crate::Verbosity;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Run cargo vendor using the library API directly.
///
/// Unlike shelling out to `cargo vendor`, this:
/// - Works in a single process
/// - Returns structured errors
/// - Doesn't need temp .cargo/config.toml with [patch] hacks
pub fn vendor_via_library(
    manifest_path: &Path,
    vendor_dir: &Path,
    v: Verbosity,
) -> Result<()> {
    let gctx = cargo::GlobalContext::default()
        .context("failed to create cargo global context")?;

    // Set verbosity on the cargo shell
    if !v.info() {
        gctx.shell().set_verbosity(cargo::core::Verbosity::Quiet);
    }

    let ws = cargo::core::Workspace::new(manifest_path, &gctx)
        .with_context(|| format!("failed to load workspace from {}", manifest_path.display()))?;

    std::fs::create_dir_all(vendor_dir)?;

    let opts = cargo::ops::VendorOptions {
        no_delete: false,
        versioned_dirs: false,
        destination: vendor_dir,
        extra: vec![],
        respect_source_config: false,
    };

    cargo::ops::vendor(&ws, &opts)
        .context("cargo vendor (library) failed")?;

    if v.info() {
        eprintln!("  Vendored external deps via cargo library API");
    }

    Ok(())
}

/// Package a local crate using the cargo library API.
///
/// Returns the path to the .crate archive file.
pub fn package_via_library(
    manifest_path: &Path,
    allow_dirty: bool,
    v: Verbosity,
) -> Result<PathBuf> {
    let gctx = cargo::GlobalContext::default()
        .context("failed to create cargo global context")?;

    if !v.info() {
        gctx.shell().set_verbosity(cargo::core::Verbosity::Quiet);
    }

    let ws = cargo::core::Workspace::new(manifest_path, &gctx)
        .with_context(|| {
            format!(
                "failed to load workspace from {}",
                manifest_path.display()
            )
        })?;

    let opts = cargo::ops::PackageOpts {
        gctx: &gctx,
        list: false,
        fmt: cargo::ops::PackageMessageFormat::Human,
        check_metadata: false,
        allow_dirty,
        include_lockfile: true,
        verify: false,
        jobs: None,
        keep_going: false,
        to_package: cargo::ops::Packages::Default,
        targets: vec![],
        cli_features: cargo::core::resolver::CliFeatures::new_all(false),
        reg_or_index: None,
        dry_run: false,
    };

    let file_locks = cargo::ops::package(&ws, &opts)
        .context("cargo package (library) failed")?;

    // Return the path to the first (and usually only) .crate file
    let crate_path = file_locks
        .first()
        .context("cargo package produced no output")?
        .path()
        .to_path_buf();

    if v.info() {
        eprintln!("  Packaged via cargo library API: {}", crate_path.display());
    }

    Ok(crate_path)
}

/// Discover local path dependencies from a workspace using cargo's resolver.
///
/// This is more reliable than parsing `cargo metadata` output because it uses
/// the same resolution logic that cargo build/vendor uses.
pub fn discover_local_deps(
    manifest_path: &Path,
    v: Verbosity,
) -> Result<Vec<LocalPackage>> {
    let gctx = cargo::GlobalContext::default()
        .context("failed to create cargo global context")?;

    if !v.debug() {
        gctx.shell().set_verbosity(cargo::core::Verbosity::Quiet);
    }

    let ws = cargo::core::Workspace::new(manifest_path, &gctx)
        .with_context(|| format!("failed to load workspace from {}", manifest_path.display()))?;

    let mut local_pkgs = Vec::new();
    let target_dir = manifest_path
        .parent()
        .context("manifest has no parent")?
        .canonicalize()?;

    for member in ws.members() {
        let pkg_dir = member
            .manifest_path()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        // Skip the root package itself
        if let Ok(canonical) = pkg_dir.canonicalize() {
            if canonical == target_dir {
                continue;
            }
        }

        local_pkgs.push(LocalPackage {
            name: member.name().to_string(),
            version: member.version().to_string(),
            path: pkg_dir.clone(),
            manifest_path: member.manifest_path().to_path_buf(),
        });
    }

    if v.debug() {
        eprintln!("  Discovered {} workspace members via cargo API", local_pkgs.len());
    }

    Ok(local_pkgs)
}
