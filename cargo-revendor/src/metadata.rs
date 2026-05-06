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
fn check_duplicate_sources_impl(pkgs: &[(String, String, Option<String>)]) -> Result<()> {
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

/// Discover local package overrides from `[patch."<url>"]` tables in
/// `.cargo/config.toml`.
///
/// Cargo's config search order walks up from the manifest directory, checking
/// `<dir>/.cargo/config.toml` at each level, then falls back to
/// `$HOME/.cargo/config.toml`. This function mirrors that walk.
///
/// For each `[patch."<url>"]` table (the URL may have or lack the `git+`
/// scheme prefix — both forms are accepted), entries of the form
/// `<crate-name> = { path = "<path>" }` are collected. The `path` is resolved
/// relative to the config file that declares it. For each entry, the target
/// crate's `Cargo.toml` is read to extract the version, and a `LocalPackage`
/// is returned.
///
/// Entries where the `path` does not contain a readable `Cargo.toml` are
/// silently skipped (the dep may not exist yet on this machine).
///
/// On TOML parse errors in a config file, returns an error with the file path
/// and position so the caller can report it loudly.
pub fn discover_from_patch_config(manifest_path: &Path) -> Result<Vec<LocalPackage>> {
    let config_files = cargo_config_search_paths(manifest_path);
    let mut results: Vec<LocalPackage> = Vec::new();
    // Track by crate name: first config file (closest to manifest) wins.
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for config_path in &config_files {
        if !config_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;

        let doc: toml_edit::DocumentMut = content
            .parse()
            .with_context(|| format!("failed to parse TOML in {}", config_path.display()))?;

        // .cargo/config.toml lives in `<config_dir>/.cargo/config.toml`; the
        // paths declared inside it resolve relative to `<config_dir>`.
        let config_dir = config_path
            .parent() // .cargo/
            .and_then(|p| p.parent()) // <config_dir>
            .unwrap_or(config_path.parent().unwrap_or(config_path));

        // Walk every top-level table key that looks like `patch."<url>"`.
        let Some(patch_tbl) = doc.get("patch").and_then(|v| v.as_table()) else {
            continue;
        };

        for (_url_key, url_entries) in patch_tbl.iter() {
            let Some(entries) = url_entries.as_table() else {
                continue;
            };
            for (crate_name, crate_spec) in entries.iter() {
                if seen.contains(crate_name) {
                    continue; // earlier config file already provided this entry
                }

                // Extract the `path` field from either an inline table
                // (`{ path = "..." }`) or a regular table (`[patch.url.name]`).
                let path_val = crate_spec
                    .as_inline_table()
                    .and_then(|t| t.get("path"))
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        crate_spec
                            .as_table()
                            .and_then(|t| t.get("path"))
                            .and_then(|v| v.as_str())
                    });

                let Some(path_val) = path_val else {
                    continue; // not a path-dep override
                };

                let crate_path = if Path::new(path_val).is_absolute() {
                    PathBuf::from(path_val)
                } else {
                    config_dir.join(path_val)
                };

                let crate_manifest = crate_path.join("Cargo.toml");
                if !crate_manifest.exists() {
                    continue; // path doesn't resolve on this machine — skip
                }

                // Read the version from the crate's own Cargo.toml.
                let version = read_package_version(&crate_manifest).with_context(|| {
                    format!(
                        "failed to read version from {} (patch entry for `{crate_name}` in {})",
                        crate_manifest.display(),
                        config_path.display()
                    )
                })?;

                let canonical = crate_path
                    .canonicalize()
                    .unwrap_or_else(|_| crate_path.clone());

                seen.insert(crate_name.to_string());
                results.push(LocalPackage {
                    name: crate_name.to_string(),
                    version,
                    path: canonical.clone(),
                    manifest_path: canonical.join("Cargo.toml"),
                });
            }
        }
    }

    Ok(results)
}

/// Return the cargo config search path for a given manifest path, in
/// priority order (highest priority first). Mirrors cargo's own search:
/// starting from the manifest's directory, walk up the filesystem checking
/// `<dir>/.cargo/config.toml` (and legacy `<dir>/.cargo/config`) at each
/// level, stopping at a filesystem root. `$HOME/.cargo/config.toml` is
/// appended last.
fn cargo_config_search_paths(manifest_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Walk from manifest dir upward.
    let mut dir = manifest_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    loop {
        let cargo_dir = dir.join(".cargo");
        // Prefer config.toml; fall back to legacy `config`.
        let toml_path = cargo_dir.join("config.toml");
        let plain_path = cargo_dir.join("config");
        if toml_path.exists() {
            paths.push(toml_path);
        } else if plain_path.exists() {
            paths.push(plain_path);
        }

        // Stop at filesystem root.
        if !dir.pop() {
            break;
        }
    }

    // $HOME/.cargo/config.toml as the global fallback.
    if let Some(home) = home_dir() {
        let global = home.join(".cargo").join("config.toml");
        if global.exists() && !paths.contains(&global) {
            paths.push(global);
        }
    }

    paths
}

/// Read `[package] version` from a `Cargo.toml`, resolving `version.workspace = true`
/// by walking up to the nearest workspace-root manifest's `[workspace.package]`.
fn read_package_version(manifest: &Path) -> Result<String> {
    let content = std::fs::read_to_string(manifest)
        .with_context(|| format!("failed to read {}", manifest.display()))?;
    let doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("failed to parse TOML in {}", manifest.display()))?;
    let version_item = doc
        .get("package")
        .and_then(|p| p.as_table())
        .and_then(|t| t.get("version"))
        .with_context(|| format!("no `[package] version` found in {}", manifest.display()))?;

    if let Some(s) = version_item.as_str() {
        return Ok(s.to_string());
    }

    // `version.workspace = true` — either inline (`version = { workspace = true }`)
    // or dotted-key form (`version.workspace = true`, parsed as a sub-table).
    let inherits_workspace = version_item
        .as_inline_table()
        .and_then(|t| t.get("workspace"))
        .and_then(|v| v.as_bool())
        == Some(true)
        || version_item
            .as_table()
            .and_then(|t| t.get("workspace"))
            .and_then(|v| v.as_bool())
            == Some(true);

    if inherits_workspace {
        return read_workspace_package_version(manifest);
    }

    anyhow::bail!(
        "could not resolve `[package] version` in {} (not a string and not workspace-inherited)",
        manifest.display()
    )
}

/// Walk up from `manifest`'s parent looking for the workspace-root `Cargo.toml`,
/// then read `[workspace.package].version`.
fn read_workspace_package_version(manifest: &Path) -> Result<String> {
    let start = manifest
        .parent()
        .with_context(|| format!("manifest {} has no parent", manifest.display()))?;
    let mut dir: Option<&Path> = start.parent();
    while let Some(d) = dir {
        let candidate = d.join("Cargo.toml");
        if candidate.exists() {
            let content = std::fs::read_to_string(&candidate)
                .with_context(|| format!("failed to read {}", candidate.display()))?;
            if let Ok(doc) = content.parse::<toml_edit::DocumentMut>()
                && let Some(ws) = doc.get("workspace").and_then(|v| v.as_table())
            {
                if let Some(version) = ws
                    .get("package")
                    .and_then(|p| p.as_table())
                    .and_then(|t| t.get("version"))
                    .and_then(|v| v.as_str())
                {
                    return Ok(version.to_string());
                }
                anyhow::bail!(
                    "workspace at {} has no `[workspace.package] version`; \
                     crate {} declares `version.workspace = true`",
                    candidate.display(),
                    manifest.display()
                );
            }
        }
        dir = d.parent();
    }
    anyhow::bail!(
        "no workspace `Cargo.toml` found above {}; cannot resolve `version.workspace = true`",
        manifest.display()
    )
}

/// Cross-platform home directory. Tries `$HOME` (Unix), `$USERPROFILE` (Windows).
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
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
        (
            name.to_string(),
            version.to_string(),
            source.map(String::from),
        )
    }

    fn local_pkg(name: &str, version: &str) -> LocalPackage {
        LocalPackage {
            name: name.to_string(),
            version: version.to_string(),
            path: PathBuf::from("/workspace").join(name),
            manifest_path: PathBuf::from("/workspace").join(name).join("Cargo.toml"),
        }
    }

    // region: check_duplicate_sources tests

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
        let pkgs = vec![pkg("local", "0.1.0", None), pkg("local", "0.1.0", None)];
        check_duplicate_sources_impl(&pkgs).unwrap();
    }

    #[test]
    fn registry_vs_git_for_same_version_errors() {
        // Realistic scenario: a crate pinned to a git source but also
        // appearing as a registry dep (e.g., via an inherited dep). This
        // should error — upstream cargo does.
        let pkgs = vec![
            pkg("serde", "1.0.0", Some("registry+https://crates.io")),
            pkg(
                "serde",
                "1.0.0",
                Some("git+https://github.com/serde-rs/serde"),
            ),
        ];
        let err = check_duplicate_sources_impl(&pkgs).unwrap_err();
        assert!(err.to_string().contains("serde v1.0.0"));
    }

    // endregion

    // region: resolve_git_override tests

    #[test]
    fn git_override_no_match_returns_none() {
        // Dep is not in overrides — should stay external.
        let overrides = vec![local_pkg("miniextendr-api", "0.5.0")];
        let result = resolve_git_override("serde", "1.0.0", &overrides).unwrap();
        assert!(
            result.is_none(),
            "unrelated dep should not match any override"
        );
    }

    #[test]
    fn git_override_empty_list_returns_none() {
        let result = resolve_git_override("miniextendr-api", "0.5.0", &[]).unwrap();
        assert!(result.is_none(), "empty override list should never match");
    }

    #[test]
    fn git_override_name_and_version_match_returns_pkg() {
        let overrides = vec![
            local_pkg("miniextendr-api", "0.5.0"),
            local_pkg("miniextendr-macros", "0.5.0"),
        ];
        let result = resolve_git_override("miniextendr-api", "0.5.0", &overrides)
            .unwrap()
            .expect("should match");
        assert_eq!(result.name, "miniextendr-api");
        assert_eq!(result.version, "0.5.0");
    }

    #[test]
    fn git_override_name_match_version_mismatch_errors() {
        // Git dep is pinned to 0.5.0 but local checkout is on 0.6.0.
        // Silently vendoring would produce a build that doesn't match
        // the lockfile — cargo-revendor must refuse.
        let overrides = vec![local_pkg("miniextendr-api", "0.6.0")];
        let err = resolve_git_override("miniextendr-api", "0.5.0", &overrides).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("miniextendr-api"),
            "error should name the crate"
        );
        assert!(
            msg.contains("0.5.0"),
            "error should mention the git version"
        );
        assert!(
            msg.contains("0.6.0"),
            "error should mention the local version"
        );
    }

    #[test]
    fn git_override_picks_first_name_match() {
        // Degenerate case: two entries with the same name but different versions.
        // Only the first match is considered; version check applies to it.
        let overrides = vec![
            local_pkg("miniextendr-api", "0.5.0"),
            local_pkg("miniextendr-api", "0.6.0"),
        ];
        // Matches first entry — version must equal the git dep's version.
        let result = resolve_git_override("miniextendr-api", "0.5.0", &overrides)
            .unwrap()
            .expect("should match first entry");
        assert_eq!(result.version, "0.5.0");
    }

    #[test]
    fn git_override_unrelated_crate_same_version_not_matched() {
        // A dep named "serde" shouldn't match an override for "miniextendr-api"
        // even if versions happen to be equal.
        let overrides = vec![local_pkg("miniextendr-api", "1.0.0")];
        let result = resolve_git_override("serde", "1.0.0", &overrides).unwrap();
        assert!(result.is_none());
    }

    // endregion

    // region: read_package_version tests

    #[test]
    fn read_package_version_string_literal() {
        let dir = tempfile::TempDir::new().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"foo\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        assert_eq!(read_package_version(&manifest).unwrap(), "1.2.3");
    }

    #[test]
    fn read_package_version_workspace_inheritance_inline() {
        // Workspace root with [workspace.package] version, member declares
        // `version = { workspace = true }` (inline-table form).
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\n\
             [workspace.package]\nversion = \"4.5.6\"\n",
        )
        .unwrap();
        let member = dir.path().join("member");
        std::fs::create_dir(&member).unwrap();
        let manifest = member.join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"bar\"\nversion = { workspace = true }\n",
        )
        .unwrap();
        assert_eq!(read_package_version(&manifest).unwrap(), "4.5.6");
    }

    #[test]
    fn read_package_version_workspace_inheritance_dotted() {
        // Same as above but using the dotted-key form
        // (`version.workspace = true`), which is the canonical style used by
        // the miniextendr workspace and most cargo workspaces in the wild.
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\n\
             [workspace.package]\nversion = \"7.8.9\"\n",
        )
        .unwrap();
        let member = dir.path().join("member");
        std::fs::create_dir(&member).unwrap();
        let manifest = member.join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"bar\"\nversion.workspace = true\n",
        )
        .unwrap();
        assert_eq!(read_package_version(&manifest).unwrap(), "7.8.9");
    }

    #[test]
    fn read_package_version_workspace_missing_root_errors() {
        // Member declares `version.workspace = true` but no ancestor
        // Cargo.toml has [workspace] — bail with a clear error.
        let dir = tempfile::TempDir::new().unwrap();
        let manifest = dir.path().join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[package]\nname = \"bar\"\nversion.workspace = true\n",
        )
        .unwrap();
        let err = read_package_version(&manifest).unwrap_err();
        assert!(
            err.to_string()
                .contains("no workspace `Cargo.toml` found above"),
            "unexpected error message: {err}"
        );
    }

    // endregion
}
