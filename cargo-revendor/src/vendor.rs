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
        // Resolve workspace inheritance in the copied Cargo.toml
        resolve_workspace_inheritance(&dest, crate_path, v)?;
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

/// Resolve `*.workspace = true` fields in a directly-copied crate's Cargo.toml
///
/// When cargo package can't run (unpublished deps), we copy the crate directly.
/// But workspace inheritance (`version.workspace = true`, etc.) won't resolve
/// outside the workspace. This function reads the workspace root's
/// `[workspace.package]` and replaces the inherited fields.
fn resolve_workspace_inheritance(
    vendor_crate_dir: &Path,
    original_crate_dir: &Path,
    v: crate::Verbosity,
) -> Result<()> {
    let cargo_toml = vendor_crate_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&cargo_toml)?;
    if !content.contains("workspace = true") && !content.contains("workspace=true") {
        return Ok(()); // no workspace inheritance to resolve
    }

    // Find workspace root
    let ws_root = match crate::find_workspace_root(original_crate_dir) {
        Ok(r) => r,
        Err(_) => return Ok(()), // not in a workspace, nothing to resolve
    };

    let ws_manifest = ws_root.join("Cargo.toml");
    if !ws_manifest.exists() {
        return Ok(());
    }

    let ws_content = std::fs::read_to_string(&ws_manifest)?;
    let ws_doc: toml_edit::DocumentMut = ws_content.parse().unwrap_or_default();
    let mut doc: toml_edit::DocumentMut = content.parse().unwrap_or_default();

    // Resolve [package] fields: version, edition, authors, etc.
    if let Some(ws_pkg) = ws_doc.get("workspace").and_then(|w| w.get("package")) {
        if let Some(pkg) = doc.get_mut("package") {
            resolve_table_workspace_fields(pkg, ws_pkg);
        }
    }

    // Resolve [dependencies] workspace refs
    if let Some(ws_deps) = ws_doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
    {
        for section in &["dependencies", "build-dependencies", "dev-dependencies"] {
            if let Some(deps) = doc.get_mut(section) {
                resolve_dep_workspace_fields(deps, ws_deps);
            }
        }
    }

    let new_content = doc.to_string();
    if new_content != content {
        std::fs::write(&cargo_toml, &new_content)?;
        if v.debug() {
            eprintln!(
                "    Resolved workspace inheritance in {}/Cargo.toml",
                vendor_crate_dir.file_name().unwrap().to_string_lossy()
            );
        }
    }

    Ok(())
}

/// Replace `field.workspace = true` with the actual value from workspace package
fn resolve_table_workspace_fields(pkg: &mut toml_edit::Item, ws_pkg: &toml_edit::Item) {
    let Some(pkg_table) = pkg.as_table_mut() else {
        return;
    };
    let Some(ws_table) = ws_pkg.as_table() else {
        return;
    };

    let fields = [
        "version",
        "edition",
        "authors",
        "description",
        "documentation",
        "readme",
        "homepage",
        "repository",
        "license",
        "license-file",
        "keywords",
        "categories",
        "rust-version",
        "exclude",
        "include",
        "publish",
    ];

    for field in fields {
        if let Some(val) = pkg_table.get(field) {
            // Check if it's { workspace = true }
            let is_ws = val
                .as_table()
                .and_then(|t| t.get("workspace"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                || val
                    .as_inline_table()
                    .and_then(|t| t.get("workspace"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                || val.as_bool().unwrap_or(false)
                    && pkg_table
                        .get(&format!("{}.workspace", field))
                        .is_some();

            // Also handle the dotted key form: version.workspace = true
            // toml_edit parses this as a subtable with key "workspace"
            let is_ws_dotted = val
                .as_table()
                .map(|t| t.len() == 1 && t.contains_key("workspace"))
                .unwrap_or(false);

            if is_ws || is_ws_dotted {
                if let Some(ws_val) = ws_table.get(field) {
                    pkg_table.insert(field, ws_val.clone());
                }
            }
        }
    }
}

/// Replace dependency `dep.workspace = true` with the workspace dependency definition
fn resolve_dep_workspace_fields(deps: &mut toml_edit::Item, ws_deps: &toml_edit::Item) {
    let Some(deps_table) = deps.as_table_mut() else {
        return;
    };
    let Some(ws_table) = ws_deps.as_table() else {
        return;
    };

    let keys: Vec<String> = deps_table.iter().map(|(k, _)| k.to_string()).collect();
    for key in keys {
        let is_ws_ref = deps_table
            .get(&key)
            .and_then(|v| {
                v.as_table()
                    .and_then(|t| t.get("workspace"))
                    .and_then(|v| v.as_bool())
                    .or_else(|| {
                        v.as_inline_table()
                            .and_then(|t| t.get("workspace"))
                            .and_then(|v| v.as_bool())
                    })
            })
            .unwrap_or(false);

        if is_ws_ref {
            if let Some(ws_dep) = ws_table.get(&key) {
                // Get extra fields from the crate's dep (like features, optional)
                let extra_features: Option<toml_edit::Array> = deps_table
                    .get(&key)
                    .and_then(|v| {
                        v.as_table()
                            .and_then(|t| t.get("features"))
                            .and_then(|f| f.as_array())
                            .or_else(|| {
                                v.as_inline_table()
                                    .and_then(|t| t.get("features"))
                                    .and_then(|f| f.as_array())
                            })
                    })
                    .cloned();

                let extra_optional: Option<bool> = deps_table.get(&key).and_then(|v| {
                    v.as_table()
                        .and_then(|t| t.get("optional"))
                        .and_then(|f| f.as_bool())
                        .or_else(|| {
                            v.as_inline_table()
                                .and_then(|t| t.get("optional"))
                                .and_then(|f| f.as_bool())
                        })
                });

                // Replace with workspace definition
                deps_table.insert(&key, ws_dep.clone());

                // Re-add extra fields
                if let Some(features) = extra_features {
                    let val = toml_edit::Value::Array(features);
                    if let Some(t) = deps_table.get_mut(&key).and_then(|v| v.as_table_mut()) {
                        t.insert("features", toml_edit::value(val.clone()));
                    } else if let Some(t) = deps_table
                        .get_mut(&key)
                        .and_then(|v| v.as_inline_table_mut())
                    {
                        t.insert("features", val);
                    }
                }
                if let Some(optional) = extra_optional {
                    if let Some(t) = deps_table.get_mut(&key).and_then(|v| v.as_table_mut()) {
                        t.insert("optional", toml_edit::value(optional));
                    }
                }

                // Remove workspace = true from the resolved dep
                if let Some(t) = deps_table.get_mut(&key).and_then(|v| v.as_table_mut()) {
                    t.remove("workspace");
                } else if let Some(t) = deps_table
                    .get_mut(&key)
                    .and_then(|v| v.as_inline_table_mut())
                {
                    t.remove("workspace");
                }
            }
        }
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
