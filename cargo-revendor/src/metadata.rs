//! Cargo metadata loading and dependency analysis

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use std::path::{Path, PathBuf};

/// A local (path-based) package discovered in the dependency tree
#[derive(Debug, Clone)]
pub struct LocalPackage {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
}

/// Load cargo metadata for the given manifest
pub fn load_metadata(manifest_path: &Path) -> Result<Metadata> {
    MetadataCommand::new()
        .manifest_path(manifest_path)
        .exec()
        .with_context(|| format!("failed to load metadata from {}", manifest_path.display()))
}

/// Discover all workspace members from a workspace root Cargo.toml
pub fn discover_workspace_members(workspace_root: &Path) -> Result<Vec<LocalPackage>> {
    let ws_manifest = workspace_root.join("Cargo.toml");
    if !ws_manifest.exists() {
        return Ok(Vec::new());
    }

    let meta = MetadataCommand::new()
        .manifest_path(&ws_manifest)
        .exec()
        .with_context(|| {
            format!(
                "failed to load workspace metadata from {}",
                ws_manifest.display()
            )
        })?;

    let ws_root = meta.workspace_root.as_std_path().canonicalize()?;

    let mut members = Vec::new();
    for pkg in &meta.packages {
        let pkg_dir = pkg.manifest_path.parent().map(|p| p.as_std_path().to_path_buf());

        // A workspace member has no source (local) and is under the workspace root
        if pkg.source.is_none() {
            if let Some(ref dir) = pkg_dir {
                if let Ok(canonical) = dir.canonicalize() {
                    if canonical.starts_with(&ws_root) {
                        members.push(LocalPackage {
                            name: pkg.name.clone(),
                            version: pkg.version.to_string(),
                            path: canonical,
                            manifest_path: pkg.manifest_path.clone().into(),
                        });
                    }
                }
            }
        }
    }

    Ok(members)
}

/// Partition packages into local (path deps) and external (registry/git)
///
/// Local packages are those whose source is a local path and whose
/// manifest is NOT inside the target package's src/rust directory
/// (i.e., they're workspace siblings, not the package itself).
pub fn partition_packages(
    meta: &Metadata,
    target_manifest: &Path,
) -> Result<(Vec<LocalPackage>, Vec<String>)> {
    let target_dir = target_manifest
        .parent()
        .context("manifest has no parent")?
        .canonicalize()?;

    let mut local = Vec::new();
    let mut external = Vec::new();

    // Get the resolve graph to find actual dependencies
    let resolve = meta
        .resolve
        .as_ref()
        .context("no resolve graph in metadata")?;

    // Find the root package (the one at target_manifest)
    let root_pkg = meta
        .packages
        .iter()
        .find(|p| {
            p.manifest_path
                .canonicalize()
                .map(|c| c == target_manifest.canonicalize().unwrap_or_default())
                .unwrap_or(false)
        })
        .context("root package not found in metadata")?;

    // Collect all transitive dependency package IDs
    let mut dep_ids = std::collections::HashSet::new();
    collect_deps(resolve, &root_pkg.id, &mut dep_ids);

    for pkg in &meta.packages {
        if pkg.id == root_pkg.id {
            continue; // skip the root package itself
        }

        if !dep_ids.contains(&pkg.id) {
            continue; // not a dependency of our root
        }

        let pkg_dir = pkg
            .manifest_path
            .parent()
            .map(|p| p.as_std_path().to_path_buf());

        let is_local = pkg.source.is_none() // no source = local path
            && pkg_dir.as_ref().map(|d| {
                d.canonicalize()
                    .map(|c| !c.starts_with(&target_dir))
                    .unwrap_or(true)
            }).unwrap_or(false);

        if is_local {
            local.push(LocalPackage {
                name: pkg.name.clone(),
                version: pkg.version.to_string(),
                path: pkg_dir.unwrap_or_default(),
                manifest_path: pkg.manifest_path.clone().into(),
            });
        } else if pkg.source.is_some() {
            external.push(pkg.name.clone());
        }
    }

    Ok((local, external))
}

/// Recursively collect all dependency package IDs
fn collect_deps(
    resolve: &cargo_metadata::Resolve,
    pkg_id: &cargo_metadata::PackageId,
    visited: &mut std::collections::HashSet<cargo_metadata::PackageId>,
) {
    if !visited.insert(pkg_id.clone()) {
        return;
    }
    if let Some(node) = resolve.nodes.iter().find(|n| &n.id == pkg_id) {
        for dep in &node.deps {
            collect_deps(resolve, &dep.pkg, visited);
        }
    }
}
