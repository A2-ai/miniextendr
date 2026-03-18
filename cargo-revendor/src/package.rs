//! Package local crates via `cargo package`
//!
//! `cargo package` resolves workspace inheritance (version.workspace = true),
//! producing standalone Cargo.toml files that work outside the workspace.
//!
//! To resolve inter-crate dependencies during packaging, we create a temporary
//! `.cargo/config.toml` with `[patch.crates-io]` entries pointing each local
//! crate to its path. This lets `cargo package` find siblings that aren't
//! published to crates.io.

use crate::metadata::LocalPackage;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Package each local crate, returning (name, crate_archive_path) pairs
///
/// `local_pkgs` — crates to actually package
/// `all_patch_pkgs` — ALL workspace crates (for [patch.crates-io] config)
pub fn package_local_crates(
    local_pkgs: &[LocalPackage],
    all_patch_pkgs: &[LocalPackage],
    _target_manifest: &Path,
    staging_dir: &Path,
    allow_dirty: bool,
    verbose: bool,
) -> Result<Vec<(String, PathBuf)>> {
    let mut results = Vec::new();

    // Build [patch.crates-io] config so all workspace crates can find each other
    let patch_config = build_patch_config(all_patch_pkgs);

    for pkg in local_pkgs {
        if verbose {
            eprintln!("  Packaging {} v{} ...", pkg.name, pkg.version);
        }

        let target_dir = staging_dir.join("package-target");

        // Find workspace root (where .cargo/config.toml will be placed)
        // cargo resolves config from CWD upward, so we put it at the workspace root
        let ws_root = crate::find_workspace_root(&pkg.path)?;
        let cargo_config_dir = ws_root.join(".cargo");
        let cargo_config_path = cargo_config_dir.join("config.toml");
        let existing_content = if cargo_config_path.exists() {
            Some(std::fs::read_to_string(&cargo_config_path)?)
        } else {
            None
        };

        // Write temporary config with patches (append to existing if present)
        std::fs::create_dir_all(&cargo_config_dir)?;
        let config_content = if let Some(ref existing) = existing_content {
            if existing.contains("[patch.crates-io]") {
                existing.clone() // already has patches, don't double up
            } else {
                format!("{}\n{}", existing, patch_config)
            }
        } else {
            patch_config.clone()
        };
        std::fs::write(&cargo_config_path, &config_content)?;

        // Unset CARGO_TARGET_DIR so cargo package uses its own target directory
        let mut cmd = Command::new("cargo");
        cmd.arg("package")
            .arg("--manifest-path")
            .arg(&pkg.manifest_path)
            .arg("--no-verify")
            .arg("--target-dir")
            .arg(&target_dir)
            .env_remove("CARGO_TARGET_DIR");

        if allow_dirty {
            cmd.arg("--allow-dirty");
        }

        let output = cmd
            .output()
            .with_context(|| format!("failed to run cargo package for {}", pkg.name))?;

        // Restore original config (or remove temporary one)
        if let Some(ref original) = existing_content {
            std::fs::write(&cargo_config_path, original)?;
        } else {
            let _ = std::fs::remove_file(&cargo_config_path);
            let _ = std::fs::remove_dir(&cargo_config_dir); // only removes if empty
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "cargo package failed for {}:\n{}",
                pkg.name,
                stderr.trim()
            );
        }

        // Find the .crate file
        let package_dir = target_dir.join("package");
        let crate_file = find_crate_file(&package_dir, &pkg.name)?;

        if verbose {
            eprintln!("  Packaged: {}", crate_file.display());
        }

        results.push((pkg.name.clone(), crate_file));
    }

    Ok(results)
}

/// Build a `[patch.crates-io]` config string for all local packages
fn build_patch_config(local_pkgs: &[LocalPackage]) -> String {
    let mut lines = vec!["[patch.crates-io]".to_string()];
    for pkg in local_pkgs {
        lines.push(format!(
            "{} = {{ path = \"{}\" }}",
            pkg.name,
            pkg.path.display()
        ));
    }
    lines.join("\n")
}

/// Find the .crate archive for a package
fn find_crate_file(package_dir: &Path, name: &str) -> Result<PathBuf> {
    if !package_dir.exists() {
        bail!(
            "package output dir not found: {}",
            package_dir.display()
        );
    }

    let prefix = format!("{}-", name);
    let mut candidates: Vec<_> = std::fs::read_dir(package_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let fname = e.file_name();
            let s = fname.to_string_lossy();
            s.starts_with(&prefix) && s.ends_with(".crate")
        })
        .collect();

    // Sort by mtime descending (newest first)
    candidates.sort_by(|a, b| {
        let ma = a.metadata().and_then(|m| m.modified()).ok();
        let mb = b.metadata().and_then(|m| m.modified()).ok();
        mb.cmp(&ma)
    });

    candidates
        .first()
        .map(|e| e.path())
        .with_context(|| format!("no .crate file found for {} in {}", name, package_dir.display()))
}
