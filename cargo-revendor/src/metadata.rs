//! Cargo metadata loading and dependency analysis

use anyhow::{Context, Result, bail};
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
        let pkg_dir = pkg
            .manifest_path
            .parent()
            .map(|p| p.as_std_path().to_path_buf());

        // A workspace member has no source (local) and is under the workspace root
        if pkg.source.is_none()
            && let Some(ref dir) = pkg_dir
            && let Ok(canonical) = dir.canonicalize()
            && canonical.starts_with(&ws_root)
        {
            members.push(LocalPackage {
                name: pkg.name.clone(),
                version: pkg.version.to_string(),
                path: canonical,
                manifest_path: pkg.manifest_path.clone().into(),
            });
        }
    }

    Ok(members)
}

/// Partition packages into local (path deps) and external (registry/git)
///
/// Local packages are those whose source is a local path and whose
/// manifest is NOT inside the target package's src/rust directory
/// (i.e., they're workspace siblings, not the package itself).
///
/// `git_overrides` allows callers to reclassify git-sourced deps as local
/// when the same crate is available in a local source root (e.g., a monorepo
/// where `--source-root` points at the workspace containing the git dep).
/// Any git dep whose name matches an entry in `git_overrides` is treated as
/// local and vendored from the local path rather than fetched from git.
/// Pass `&[]` when `--source-root` is not in use.
///
/// Returns an error if a git dep matches a `git_overrides` entry by name but
/// the resolved git version differs from the local version — a version mismatch
/// means the local checkout is not the same code the lockfile pinned, and
/// silently vendoring the wrong source would produce broken builds.
pub fn partition_packages(
    meta: &Metadata,
    target_manifest: &Path,
    git_overrides: &[LocalPackage],
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
            // Reclassify git deps that are available in the local source root.
            // With git-dep declarations (git = "https://..."), cargo metadata
            // sets source = "git+...", but --source-root can point at the local
            // checkout. Without this override, cargo-revendor would fetch from
            // github instead of using the local edit.
            match resolve_git_override(&pkg.name, &pkg.version.to_string(), git_overrides)? {
                Some(override_pkg) => local.push(override_pkg.clone()),
                None => external.push(pkg.name.clone()),
            }
        }
    }

    Ok((local, external))
}

/// Look up a git dep in the override list, checking that versions match.
///
/// Returns `Ok(Some(pkg))` when the git dep is overridden by a local source-root
/// member of the same name **and** the same version. Returns `Ok(None)` when no
/// override exists (dep should be treated as external). Returns `Err` when a
/// name match is found but the versions differ — this indicates the local
/// checkout is not the same code the lockfile pinned, which would produce a
/// broken vendor tree.
fn resolve_git_override<'a>(
    name: &str,
    git_version: &str,
    overrides: &'a [LocalPackage],
) -> Result<Option<&'a LocalPackage>> {
    let Some(candidate) = overrides.iter().find(|o| o.name == name) else {
        return Ok(None);
    };

    if candidate.version != git_version {
        bail!(
            "git dep `{name}` resolves to v{git_version} in Cargo.lock \
             but the local source-root has v{local} — versions must match \
             for `--source-root` override to be safe. \
             Update the local crate or pin the git dep to a matching revision.",
            local = candidate.version,
        );
    }

    Ok(Some(candidate))
}

/// Error out when two different sources resolve to the same (name, version).
///
/// Mirrors upstream cargo/src/cargo/ops/vendor.rs's duplicate-source check:
/// two git repos that happen to publish the same crate name + version would
/// otherwise silently last-write-wins during extraction, making the vendored
/// contents depend on dep-graph iteration order. Upstream hard-errors; we do
/// too.
///
/// Common legitimate case this does NOT flag: the SAME (name, version) from
/// the SAME source appearing multiple times in `meta.packages` (cargo can
/// emit dupes when a package is reached via different dep paths). Only
/// DIFFERENT sources for the same (name, version) key are errors.
pub fn check_duplicate_sources(meta: &Metadata) -> Result<()> {
    // Build (name, version, Option<source>) tuples and delegate to the
    // pure helper. Keeps the cargo_metadata dependency contained to this
    // shim and makes the core logic unit-testable with plain tuples.
    let triples: Vec<(String, String, Option<String>)> = meta
        .packages
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                p.version.to_string(),
                p.source.as_ref().map(|s| s.to_string()),
            )
        })
        .collect();
    check_duplicate_sources_impl(&triples)
}

/// Core dedup logic, factored out of [`check_duplicate_sources`] so it can
/// be unit-tested without constructing a `cargo_metadata::Metadata` fixture.
///
/// Each triple is `(name, version, source)` where `None` source means a
/// local path dep (skipped — workspace semantics prevent in-workspace
/// duplicates of the same name+version).
fn check_duplicate_sources_impl(
    pkgs: &[(String, String, Option<String>)],
) -> Result<()> {
    use std::collections::BTreeMap;

    let mut seen: BTreeMap<(String, String), String> = BTreeMap::new();

    for (name, version, source) in pkgs {
        let Some(source_str) = source else {
            continue;
        };

        match seen.get(&(name.clone(), version.clone())) {
            None => {
                seen.insert((name.clone(), version.clone()), source_str.clone());
            }
            Some(prev) if prev == source_str => {
                // Same source — legitimate duplicate, skip.
            }
            Some(prev) => {
                bail!(
                    "duplicate crate `{} v{}` from different sources:\n  - {}\n  - {}\n\
                     cargo-revendor refuses to silently last-write-wins when two sources\n\
                     disagree. Pick one in your Cargo.toml / Cargo.lock.",
                    name,
                    version,
                    prev,
                    source_str
                );
            }
        }
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str, version: &str, source: Option<&str>) -> (String, String, Option<String>) {
        (name.to_string(), version.to_string(), source.map(String::from))
    }

    #[test]
    fn no_duplicates_passes() {
        let pkgs = vec![
            pkg("a", "1.0.0", Some("registry+https://crates.io")),
            pkg("b", "2.0.0", Some("registry+https://crates.io")),
        ];
        check_duplicate_sources_impl(&pkgs).unwrap();
    }

    #[test]
    fn same_source_duplicate_is_ok() {
        // cargo metadata can emit the same pkg twice if reached via
        // multiple dep paths. Not an error.
        let pkgs = vec![
            pkg("a", "1.0.0", Some("registry+https://crates.io")),
            pkg("a", "1.0.0", Some("registry+https://crates.io")),
        ];
        check_duplicate_sources_impl(&pkgs).unwrap();
    }

    #[test]
    fn different_sources_same_name_version_errors() {
        let pkgs = vec![
            pkg("foo", "1.2.3", Some("git+https://github.com/a/foo")),
            pkg("foo", "1.2.3", Some("git+https://github.com/b/foo")),
        ];
        let err = check_duplicate_sources_impl(&pkgs).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("duplicate crate"));
        assert!(msg.contains("foo v1.2.3"));
        assert!(msg.contains("github.com/a/foo"));
        assert!(msg.contains("github.com/b/foo"));
    }

    #[test]
    fn same_name_different_versions_is_fine() {
        // Common: two versions of a transitive dep coexist. That's handled
        // by vendor/<name>-<version>/ layout; duplicates here are a
        // different concern.
        let pkgs = vec![
            pkg("foo", "1.0.0", Some("registry+https://crates.io")),
            pkg("foo", "2.0.0", Some("registry+https://crates.io")),
        ];
        check_duplicate_sources_impl(&pkgs).unwrap();
    }

    #[test]
    fn local_path_deps_not_checked() {
        // source: None = local path dep. Two local deps with the same
        // (name, version) would be a workspace-level problem, detected
        // elsewhere. Don't double-report here.
        let pkgs = vec![
            pkg("local", "0.1.0", None),
            pkg("local", "0.1.0", None),
        ];
        check_duplicate_sources_impl(&pkgs).unwrap();
    }

    #[test]
    fn registry_vs_git_for_same_version_errors() {
        // Realistic scenario: a crate pinned to a git source but also
        // appearing as a registry dep (e.g., via an inherited dep). This
        // should error — upstream cargo does.
        let pkgs = vec![
            pkg("serde", "1.0.0", Some("registry+https://crates.io")),
            pkg("serde", "1.0.0", Some("git+https://github.com/serde-rs/serde")),
        ];
        let err = check_duplicate_sources_impl(&pkgs).unwrap_err();
        assert!(err.to_string().contains("serde v1.0.0"));
    }
}
