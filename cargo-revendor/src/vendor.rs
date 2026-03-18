//! Vendoring: run cargo vendor, extract local crates, rewrite paths

use crate::metadata::LocalPackage;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run `cargo vendor` for external (registry/git) dependencies
pub fn run_cargo_vendor(
    manifest_path: &Path,
    vendor_dir: &Path,
    local_pkgs: &[LocalPackage],
    v: crate::Verbosity,
) -> Result<()> {
    if v.info() {
        eprintln!("  Running cargo vendor...");
    }

    std::fs::create_dir_all(vendor_dir)?;

    // Create temp .cargo/config.toml with [patch.crates-io] so cargo vendor
    // can resolve the dependency graph even with unpublished local crates
    let rust_dir = manifest_path.parent().unwrap();
    let cargo_config_dir = rust_dir.join(".cargo");
    let cargo_config_path = cargo_config_dir.join("config.toml");
    let had_existing = cargo_config_path.exists();
    let existing_content = if had_existing {
        Some(std::fs::read_to_string(&cargo_config_path)?)
    } else {
        None
    };

    if !local_pkgs.is_empty() {
        std::fs::create_dir_all(&cargo_config_dir)?;
        let mut patch = String::from("[patch.crates-io]\n");
        for pkg in local_pkgs {
            patch.push_str(&format!(
                "{} = {{ path = \"{}\" }}\n",
                pkg.name,
                pkg.path.display()
            ));
        }
        let content = if let Some(ref existing) = existing_content {
            format!("{}\n{}", existing, patch)
        } else {
            patch
        };
        std::fs::write(&cargo_config_path, &content)?;
    }

    let output = Command::new("cargo")
        .arg("vendor")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg(vendor_dir)
        .output()
        .context("failed to run cargo vendor")?;

    // Restore config
    if let Some(ref original) = existing_content {
        std::fs::write(&cargo_config_path, original)?;
    } else if cargo_config_path.exists() {
        let _ = std::fs::remove_file(&cargo_config_path);
        let _ = std::fs::remove_dir(&cargo_config_dir);
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cargo vendor failed:\n{}", stderr.trim());
    }

    Ok(())
}

/// Extract a .crate archive into the vendor directory
/// Extract a .crate archive OR copy a directory into the vendor directory
pub fn extract_crate_archive(
    crate_path: &Path,
    vendor_dir: &Path,
    pkg_name: &str,
    v: crate::Verbosity,
) -> Result<()> {
    let dest = vendor_dir.join(pkg_name);

    // Remove any existing directory (cargo vendor may have put a placeholder)
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }

    if crate_path.is_dir() {
        // Direct copy fallback (when cargo package failed)
        copy_crate_dir(crate_path, &dest)?;
        if v.info() {
            eprintln!("  Copied {} to vendor/{}", pkg_name, pkg_name);
        }
        return Ok(());
    }

    // .crate files are gzipped tar archives
    let file = std::fs::File::open(crate_path)
        .with_context(|| format!("failed to open {}", crate_path.display()))?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);

    // Extract to a temp dir first (archive contains versioned dir like name-0.1.0/)
    let extract_tmp = vendor_dir.join(format!(".{}-extract", pkg_name));
    if extract_tmp.exists() {
        std::fs::remove_dir_all(&extract_tmp)?;
    }
    std::fs::create_dir_all(&extract_tmp)?;
    archive.unpack(&extract_tmp)?;

    // Find the extracted directory (name-version/)
    let extracted = find_single_subdir(&extract_tmp)?;

    // Move to final destination (just the crate name, no version)
    std::fs::rename(&extracted, &dest).with_context(|| {
        format!(
            "failed to move {} to {}",
            extracted.display(),
            dest.display()
        )
    })?;

    // Clean up temp dir
    let _ = std::fs::remove_dir_all(&extract_tmp);

    if v.info() {
        eprintln!("  Extracted {} to vendor/{}", pkg_name, pkg_name);
    }

    Ok(())
}

/// Rewrite inter-crate path dependencies so local crates reference each other in vendor/
pub fn rewrite_local_path_deps(
    vendor_dir: &Path,
    local_pkgs: &[LocalPackage],
    v: crate::Verbosity,
) -> Result<()> {
    let local_names: std::collections::HashSet<&str> =
        local_pkgs.iter().map(|p| p.name.as_str()).collect();

    for entry in std::fs::read_dir(vendor_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&cargo_toml)?;
        let mut doc: toml_edit::DocumentMut = content
            .parse()
            .with_context(|| format!("failed to parse {}", cargo_toml.display()))?;

        let mut changed = false;

        // Check [dependencies], [build-dependencies], [dev-dependencies]
        for section in &["dependencies", "build-dependencies", "dev-dependencies"] {
            if let Some(table) = doc.get_mut(section).and_then(|v| v.as_table_mut()) {
                for name in local_names.iter() {
                    if let Some(dep) = table.get_mut(*name) {
                        if add_path_to_dep(dep, name) {
                            changed = true;
                            if v.info() {
                                eprintln!(
                                    "  Rewrote {}.{} in {}/Cargo.toml",
                                    section,
                                    name,
                                    entry.file_name().to_string_lossy()
                                );
                            }
                        }
                    }
                }
            }
        }

        if changed {
            std::fs::write(&cargo_toml, doc.to_string())?;
        }
    }

    Ok(())
}

/// Add `path = "../<name>"` to a dependency entry if not already present
fn add_path_to_dep(dep: &mut toml_edit::Item, name: &str) -> bool {
    match dep {
        toml_edit::Item::Value(toml_edit::Value::String(version_str)) => {
            // Simple: name = "0.1.0" → name = { version = "0.1.0", path = "../name" }
            let version = version_str.value().to_string();
            let mut inline = toml_edit::InlineTable::new();
            inline.insert("version", toml_edit::value(&version).into_value().unwrap());
            inline.insert(
                "path",
                toml_edit::value(format!("../{}", name))
                    .into_value()
                    .unwrap(),
            );
            *dep = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
            true
        }
        toml_edit::Item::Value(toml_edit::Value::InlineTable(table)) => {
            if !table.contains_key("path") {
                table.insert(
                    "path",
                    toml_edit::value(format!("../{}", name))
                        .into_value()
                        .unwrap(),
                );
                true
            } else {
                false
            }
        }
        toml_edit::Item::Table(table) => {
            if !table.contains_key("path") {
                table.insert("path", toml_edit::value(format!("../{}", name)));
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Copy a crate directory to vendor/ (fallback when cargo package fails)
/// Copies source files, excluding target/ and other build artifacts
fn copy_crate_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();

        // Skip build artifacts and VCS dirs
        let first_component = relative
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_default();
        if matches!(
            first_component.as_str(),
            "target" | ".git" | ".cargo" | "ra_target" | "ra-target"
        ) {
            continue;
        }

        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Clear all .cargo-checksum.json files (vendored sources don't need verification)
pub fn clear_checksums(vendor_dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(vendor_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let cksum = entry.path().join(".cargo-checksum.json");
            std::fs::write(&cksum, "{\"files\":{}}")?;
        }
    }
    Ok(())
}

/// Find the single subdirectory in a directory (from tar extraction)
fn find_single_subdir(dir: &Path) -> Result<PathBuf> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();

    if entries.len() != 1 {
        bail!(
            "expected exactly 1 subdirectory in {}, found {}",
            dir.display(),
            entries.len()
        );
    }

    Ok(entries.remove(0).path())
}
