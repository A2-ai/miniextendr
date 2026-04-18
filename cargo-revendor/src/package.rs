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
use anyhow::{Context, Result, bail};
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
    v: crate::Verbosity,
) -> Result<Vec<(String, PathBuf)>> {
    let mut results = Vec::new();

    // Build [patch.crates-io] config so all workspace crates can find each other
    let patch_config = build_patch_config(all_patch_pkgs);

    // Build a set of all local package names for path dep detection
    let local_names: std::collections::HashSet<&str> =
        all_patch_pkgs.iter().map(|p| p.name.as_str()).collect();

    for pkg in local_pkgs {
        if v.info() {
            eprintln!("  Packaging {} v{} ...", pkg.name, pkg.version);
        }

        let target_dir = staging_dir.join("package-target");

        // Find workspace root — needed before guards so we know which manifest
        // to snapshot.
        let ws_root = crate::find_workspace_root(&pkg.path)?;
        let ws_manifest = ws_root.join("Cargo.toml");

        // Snapshot both the inner crate manifest and the workspace manifest.
        // The guards restore both on drop (scope exits at the end of this
        // iteration, or earlier via `?` / panic unwind).
        let _inner_guard =
            crate::manifest_guard::ManifestGuard::snapshot(&pkg.manifest_path)?;
        let _ws_guard = crate::manifest_guard::ManifestGuard::snapshot(&ws_manifest)?;

        // Temporarily rewrite Cargo.toml to add version = "*" to path-only deps
        // (cargo package rejects path deps without a version)
        let manifest_content = std::fs::read_to_string(&pkg.manifest_path)?;
        let patched = add_versions_to_path_deps(&manifest_content, &local_names);
        if patched != manifest_content {
            std::fs::write(&pkg.manifest_path, &patched)?;
            if v.debug() {
                eprintln!("    Patched Cargo.toml: added version = \"*\" to path deps");
            }
        }

        // Add [patch.crates-io] to workspace root Cargo.toml
        // (cargo ignores [patch] in .cargo/config.toml — only Cargo.toml works)
        let ws_manifest_original = std::fs::read_to_string(&ws_manifest)?;
        if !ws_manifest_original.contains("[patch.crates-io]") {
            let patched_ws = format!("{}\n{}", ws_manifest_original, patch_config);
            std::fs::write(&ws_manifest, &patched_ws)?;
        }

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

        // Guards (_inner_guard, _ws_guard) restore both manifests on drop —
        // no explicit restore needed. Drops run at end of this iteration.

        if !output.status.success() {
            // Fallback: cargo package failed (likely unpublished deps).
            // Copy the crate directly and resolve workspace inheritance manually.
            if v.info() {
                eprintln!(
                    "  cargo package failed for {}, using direct copy fallback",
                    pkg.name
                );
            }
            if v.debug() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("    cargo package stderr: {}", stderr.trim());
            }
            results.push((pkg.name.clone(), pkg.path.clone()));
            continue;
        }

        // Find the .crate file
        let package_dir = target_dir.join("package");
        let crate_file = find_crate_file(&package_dir, &pkg.name)?;

        if v.info() {
            eprintln!("  Packaged: {}", crate_file.display());
        }

        results.push((pkg.name.clone(), crate_file));
    }

    Ok(results)
}

/// Add `version = "*"` to path-only dependencies so `cargo package` accepts them.
///
/// Transforms `helper = { path = "../helper" }` into
/// `helper = { version = "*", path = "../helper" }`.
/// Only modifies deps whose name matches a known local package.
fn add_versions_to_path_deps(
    manifest_content: &str,
    local_names: &std::collections::HashSet<&str>,
) -> String {
    let mut doc: toml_edit::DocumentMut = match manifest_content.parse() {
        Ok(d) => d,
        Err(_) => return manifest_content.to_string(),
    };

    let mut changed = false;

    for section in &["dependencies", "build-dependencies", "dev-dependencies"] {
        if let Some(table) = doc.get_mut(section).and_then(|v| v.as_table_mut()) {
            for name in local_names.iter() {
                if let Some(dep) = table.get_mut(name)
                    && ensure_version(dep)
                {
                    changed = true;
                }
            }
        }
    }

    if changed {
        doc.to_string()
    } else {
        manifest_content.to_string()
    }
}

/// Ensure a dependency entry has a `version` field. Returns true if modified.
fn ensure_version(dep: &mut toml_edit::Item) -> bool {
    match dep {
        // path-only inline table: { path = "../foo" } → { version = "*", path = "../foo" }
        toml_edit::Item::Value(toml_edit::Value::InlineTable(table))
            if table.contains_key("path") && !table.contains_key("version") =>
        {
            table.insert("version", toml_edit::value("*").into_value().unwrap());
            true
        }
        // path-only table section: [dependencies.foo] path = "../foo"
        toml_edit::Item::Table(table)
            if table.contains_key("path") && !table.contains_key("version") =>
        {
            table.insert("version", toml_edit::value("*"));
            true
        }
        _ => false,
    }
}

/// Build a `[patch.crates-io]` config string for all local packages
fn build_patch_config(local_pkgs: &[LocalPackage]) -> String {
    let mut lines = vec!["[patch.crates-io]".to_string()];
    for pkg in local_pkgs {
        lines.push(format!(
            "{} = {{ path = \"{}\" }}",
            pkg.name,
            crate::path_to_toml(&pkg.path)
        ));
    }
    lines.join("\n")
}

/// Find the .crate archive for a package
fn find_crate_file(package_dir: &Path, name: &str) -> Result<PathBuf> {
    if !package_dir.exists() {
        bail!("package output dir not found: {}", package_dir.display());
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

    candidates.first().map(|e| e.path()).with_context(|| {
        format!(
            "no .crate file found for {} in {}",
            name,
            package_dir.display()
        )
    })
}
