//! Vendoring: run cargo vendor, extract local crates, rewrite paths

use crate::metadata::LocalPackage;
use anyhow::{Context, Result, bail};
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

    // Add [patch.crates-io] to workspace root Cargo.toml so cargo vendor
    // can resolve the dependency graph even with unpublished local crates.
    // NOTE: [patch] only works in Cargo.toml, NOT in .cargo/config.toml.
    let ws_root =
        crate::find_workspace_root(manifest_path.parent().context("manifest has no parent")?)?;
    let ws_manifest = ws_root.join("Cargo.toml");
    let ws_original = std::fs::read_to_string(&ws_manifest)?;

    if !local_pkgs.is_empty() && !ws_original.contains("[patch.crates-io]") {
        let mut patch = String::from("\n[patch.crates-io]\n");
        for pkg in local_pkgs {
            patch.push_str(&format!(
                "{} = {{ path = \"{}\" }}\n",
                pkg.name,
                crate::path_to_toml(&pkg.path)
            ));
        }
        std::fs::write(&ws_manifest, format!("{}{}", ws_original, patch))?;
    }

    let output = Command::new("cargo")
        .arg("vendor")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg(vendor_dir)
        .output()
        .context("failed to run cargo vendor")?;

    // Restore original workspace Cargo.toml
    std::fs::write(&ws_manifest, &ws_original)?;

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

/// Strip relative path dependencies (`path = "../..."`) from all vendored crate manifests.
///
/// When `cargo vendor` vendors crates from a git workspace, the vendored Cargo.toml
/// files retain intra-workspace path deps (e.g., `path = "../sibling-crate"`). During
/// offline builds with cargo source replacement, these path deps cause cargo to resolve
/// siblings as path sources instead of through the directory source, which conflicts
/// with Cargo.lock entries that record them as git (or registry) sources. Stripping the
/// path keys forces cargo to resolve by name from the replaced source.
///
/// This runs BEFORE `rewrite_local_path_deps`, which adds back correct path deps
/// for local/workspace crates only.
pub fn strip_vendor_path_deps(vendor_dir: &Path, v: crate::Verbosity) -> Result<()> {
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

        for section in &["dependencies", "build-dependencies"] {
            if let Some(table) = doc.get_mut(section).and_then(|v| v.as_table_mut()) {
                for (_name, dep) in table.iter_mut() {
                    if remove_relative_path(dep) {
                        changed = true;
                    }
                }
            }
        }

        if changed {
            std::fs::write(&cargo_toml, doc.to_string())?;
            if v.debug() {
                eprintln!(
                    "  Stripped path deps from {}/Cargo.toml",
                    entry.file_name().to_string_lossy()
                );
            }
        }
    }

    Ok(())
}

/// Remove `path = "../..."` from a dependency entry (returns true if changed)
fn remove_relative_path(dep: &mut toml_edit::Item) -> bool {
    match dep {
        toml_edit::Item::Value(toml_edit::Value::InlineTable(table))
            if table
                .get("path")
                .and_then(|v| v.as_str())
                .is_some_and(|p| p.starts_with("../")) =>
        {
            table.remove("path");
            true
        }
        toml_edit::Item::Table(table)
            if table
                .get("path")
                .and_then(|v| v.as_str())
                .is_some_and(|p| p.starts_with("../")) =>
        {
            table.remove("path");
            true
        }
        _ => false,
    }
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
                    if let Some(dep) = table.get_mut(name)
                        && add_path_to_dep(dep, name)
                    {
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

        if changed {
            std::fs::write(&cargo_toml, doc.to_string())?;
        }
    }

    Ok(())
}

/// Set `path = "../<name>"` on a dependency entry (adds or overwrites)
fn add_path_to_dep(dep: &mut toml_edit::Item, name: &str) -> bool {
    let correct_path = format!("../{}", name);
    match dep {
        toml_edit::Item::Value(toml_edit::Value::String(version_str)) => {
            let version = version_str.value().to_string();
            let mut inline = toml_edit::InlineTable::new();
            inline.insert("version", toml_edit::value(&version).into_value().unwrap());
            inline.insert(
                "path",
                toml_edit::value(&correct_path).into_value().unwrap(),
            );
            *dep = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
            true
        }
        toml_edit::Item::Value(toml_edit::Value::InlineTable(table)) => {
            let current = table
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if current.as_deref() != Some(&correct_path) {
                table.insert(
                    "path",
                    toml_edit::value(&correct_path).into_value().unwrap(),
                );
                true
            } else {
                false
            }
        }
        toml_edit::Item::Table(table) => {
            let current = table
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if current.as_deref() != Some(&correct_path) {
                table.insert("path", toml_edit::value(&correct_path));
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
    if let Some(ws_pkg) = ws_doc.get("workspace").and_then(|w| w.get("package"))
        && let Some(pkg) = doc.get_mut("package")
    {
        resolve_table_workspace_fields(pkg, ws_pkg);
    }

    // Resolve [dependencies] workspace refs
    if let Some(ws_deps) = ws_doc.get("workspace").and_then(|w| w.get("dependencies")) {
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
                    && pkg_table.get(&format!("{}.workspace", field)).is_some();

            // Also handle the dotted key form: version.workspace = true
            // toml_edit parses this as a subtable with key "workspace"
            let is_ws_dotted = val
                .as_table()
                .map(|t| t.len() == 1 && t.contains_key("workspace"))
                .unwrap_or(false);

            if (is_ws || is_ws_dotted)
                && let Some(ws_val) = ws_table.get(field)
            {
                pkg_table.insert(field, ws_val.clone());
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

        if is_ws_ref && let Some(ws_dep) = ws_table.get(&key) {
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
            if let Some(optional) = extra_optional
                && let Some(t) = deps_table.get_mut(&key).and_then(|v| v.as_table_mut())
            {
                t.insert("optional", toml_edit::value(optional));
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

/// Generate a .cargo/config.toml for source replacement.
///
/// Returns the config content as a string. Also writes it to
/// `<vendor_dir>/../src/rust/.cargo/config.toml` if that path exists.
pub fn generate_cargo_config(
    manifest_path: &Path,
    vendor_dir: &Path,
    _local_pkgs: &[LocalPackage],
) -> Result<String> {
    let vendor_path = vendor_dir
        .canonicalize()
        .unwrap_or_else(|_| vendor_dir.to_path_buf());

    let mut config = String::new();
    config.push_str("[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n");

    // Add git source replacements for any git deps in Cargo.toml
    let manifest_content = std::fs::read_to_string(manifest_path)?;
    let mut git_urls: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for line in manifest_content.lines() {
        // Match: git = "https://..."
        if let Some(start) = line.find("git = \"https://") {
            let url_start = start + 7; // skip `git = "`
            if let Some(end) = line[url_start..].find('"') {
                git_urls.insert(line[url_start..url_start + end].to_string());
            }
        }
    }
    for url in &git_urls {
        config.push_str(&format!(
            "[source.\"git+{}\"]\ngit = \"{}\"\nreplace-with = \"vendored-sources\"\n\n",
            url, url
        ));
    }

    config.push_str(&format!(
        "[source.vendored-sources]\ndirectory = \"{}\"\n",
        crate::path_to_toml(&vendor_path)
    ));

    // Write to vendor dir for reference
    let config_path = vendor_dir.join(".cargo-config.toml");
    std::fs::write(&config_path, &config)?;

    Ok(config)
}

/// Strip checksums from Cargo.lock and copy to vendor dir.
///
/// Vendored crates have empty checksums, so the lockfile's `checksum = "..."`
/// lines need to be removed for `--locked` builds to work.
pub fn strip_lock_checksums(lockfile: &Path, vendor_dir: &Path, v: crate::Verbosity) -> Result<()> {
    if !lockfile.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(lockfile)?;
    let stripped: String = content
        .lines()
        .filter(|line| !line.starts_with("checksum = "))
        .collect::<Vec<_>>()
        .join("\n");

    // Write stripped lockfile to vendor dir
    let dest = vendor_dir.join("Cargo.lock");
    std::fs::write(&dest, &stripped)?;

    if v.debug() {
        let removed = content.lines().count() - stripped.lines().count();
        eprintln!("  Stripped {} checksum lines from Cargo.lock", removed);
    }

    Ok(())
}

/// Freeze: rewrite Cargo.toml so all sources resolve from vendor/.
///
/// 1. Rewrites git deps to vendor/ path deps
/// 2. Strips all `[patch.*]` sections (they reference sources outside vendor/)
/// 3. Adds `[patch.crates-io]` with vendor paths for transitive local deps
///
/// After freezing, the manifest is self-contained: `cargo build --offline`
/// works with only the vendor directory, no network or workspace context.
pub fn freeze_manifest(
    manifest_path: &Path,
    vendor_dir: &Path,
    local_pkgs: &[LocalPackage],
    v: crate::Verbosity,
) -> Result<()> {
    let content = std::fs::read_to_string(manifest_path)?;
    let mut doc: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let vendor_rel = pathdiff(
        vendor_dir,
        manifest_path.parent().context("manifest has no parent")?,
    );

    // Step 1: Rewrite git/version deps to vendor/ path deps
    let local_names: std::collections::HashSet<&str> =
        local_pkgs.iter().map(|p| p.name.as_str()).collect();

    for section in &["dependencies", "build-dependencies"] {
        if let Some(table) = doc.get_mut(section).and_then(|v| v.as_table_mut()) {
            for name in local_names.iter() {
                if let Some(dep) = table.get_mut(name) {
                    rewrite_dep_to_vendor(dep, name, &vendor_rel);
                }
            }
        }
    }

    // Step 2: Collect all crate names from existing [patch.*] sections,
    // then remove those sections. We need the names to re-add them as
    // vendor path deps (unpublished git crates aren't on crates.io).
    let mut patched_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (key, val) in doc.as_table().iter() {
        if key.starts_with("patch")
            && let Some(patch_table) = val.as_table()
        {
            for (_registry, registry_val) in patch_table.iter() {
                if let Some(registry_table) = registry_val.as_table() {
                    for (crate_name, _) in registry_table.iter() {
                        patched_names.insert(crate_name.to_string());
                    }
                }
            }
        }
    }
    let keys_to_remove: Vec<String> = doc
        .as_table()
        .iter()
        .filter(|(k, _)| k.starts_with("patch"))
        .map(|(k, _)| k.to_string())
        .collect();
    for key in &keys_to_remove {
        doc.remove(key);
    }

    // Step 3: Add [patch.crates-io] for all vendored crates that were
    // previously patched OR are local workspace deps. This ensures
    // unpublished crates (from git sources) remain available in the
    // crates-io namespace when resolved from vendored-sources.
    let mut patch_table = toml_edit::Table::new();
    for pkg in local_pkgs {
        patched_names.insert(pkg.name.clone());
    }
    for name in &patched_names {
        if vendor_dir.join(name).exists() {
            let rel = format!("{}/{}", vendor_rel, name);
            let mut inline = toml_edit::InlineTable::new();
            inline.insert("path", toml_edit::value(&rel).into_value().unwrap());
            patch_table.insert(
                name,
                toml_edit::Item::Value(toml_edit::Value::InlineTable(inline)),
            );
        }
    }
    if !patch_table.is_empty() {
        doc.insert("patch", toml_edit::Item::Table(toml_edit::Table::new()));
        if let Some(patch) = doc.get_mut("patch").and_then(|v| v.as_table_mut()) {
            patch.insert("crates-io", toml_edit::Item::Table(patch_table));
        }
    }

    std::fs::write(manifest_path, doc.to_string())?;

    if v.info() {
        eprintln!(
            "  Frozen: {} now resolves from vendor/ only",
            manifest_path.display()
        );
    }

    Ok(())
}

/// Rewrite a dependency entry to point at vendor/
fn rewrite_dep_to_vendor(dep: &mut toml_edit::Item, name: &str, vendor_rel: &str) {
    let path_val = format!("{}/{}", vendor_rel, name);
    match dep {
        toml_edit::Item::Value(toml_edit::Value::InlineTable(table)) => {
            table.remove("git");
            table.remove("branch");
            table.remove("tag");
            table.remove("rev");
            if !table.contains_key("version") {
                table.insert("version", toml_edit::value("*").into_value().unwrap());
            }
            table.insert("path", toml_edit::value(&path_val).into_value().unwrap());
        }
        toml_edit::Item::Table(table) => {
            table.remove("git");
            table.remove("branch");
            table.remove("tag");
            table.remove("rev");
            if !table.contains_key("version") {
                table.insert("version", toml_edit::value("*"));
            }
            table.insert("path", toml_edit::value(&path_val));
        }
        toml_edit::Item::Value(toml_edit::Value::String(_)) => {
            let mut inline = toml_edit::InlineTable::new();
            inline.insert("version", toml_edit::value("*").into_value().unwrap());
            inline.insert("path", toml_edit::value(&path_val).into_value().unwrap());
            *dep = toml_edit::Item::Value(toml_edit::Value::InlineTable(inline));
        }
        _ => {}
    }
}

/// Compute relative path from base to target
fn pathdiff(target: &Path, base: &Path) -> String {
    let target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    let base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());

    let target_parts: Vec<_> = target.components().collect();
    let base_parts: Vec<_> = base.components().collect();

    let common = target_parts
        .iter()
        .zip(base_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if common == 0 {
        return crate::path_to_toml(&target);
    }

    let mut rel = String::new();
    for _ in 0..base_parts.len() - common {
        rel.push_str("../");
    }
    for part in &target_parts[common..] {
        rel.push_str(&part.as_os_str().to_string_lossy());
        rel.push('/');
    }
    if rel.ends_with('/') {
        rel.pop();
    }
    rel
}

/// Regenerate Cargo.lock from vendored sources (offline)
///
/// Creates a temporary `.cargo/config.toml` with source replacement pointing
/// to `vendor_dir`, runs `cargo generate-lockfile --offline`, then removes
/// the temporary config (it would conflict with the configure-generated one).
pub fn regenerate_lockfile(
    manifest_path: &Path,
    vendor_dir: &Path,
    v: crate::Verbosity,
) -> Result<()> {
    let lockfile = manifest_path.with_file_name("Cargo.lock");
    if lockfile.exists() {
        std::fs::remove_file(&lockfile)?;
    }

    // Write temporary .cargo/config.toml so cargo can resolve vendored sources
    let cargo_dir = manifest_path.with_file_name(".cargo");
    std::fs::create_dir_all(&cargo_dir)?;
    let config_path = cargo_dir.join("config.toml");
    let had_config = config_path.exists();
    let old_config = if had_config {
        Some(std::fs::read_to_string(&config_path)?)
    } else {
        None
    };

    let vendor_path = vendor_dir
        .canonicalize()
        .unwrap_or_else(|_| vendor_dir.to_path_buf());
    let config_content = format!(
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n\
         [source.vendored-sources]\ndirectory = \"{}\"\n",
        crate::path_to_toml(&vendor_path)
    );
    std::fs::write(&config_path, &config_content)?;

    let output = std::process::Command::new("cargo")
        .arg("generate-lockfile")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--offline")
        .output()
        .context("failed to run cargo generate-lockfile")?;

    // Restore or remove the temporary config
    if let Some(old) = old_config {
        std::fs::write(&config_path, old)?;
    } else {
        let _ = std::fs::remove_file(&config_path);
        let _ = std::fs::remove_dir(&cargo_dir);
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "cargo generate-lockfile --offline failed:\n{}",
            stderr.trim()
        );
    }

    if v.info() {
        eprintln!("  CRAN mode: regenerated Cargo.lock from vendored sources");
    }

    Ok(())
}

/// Compress vendor/ into a .tar.xz tarball
pub fn compress_vendor(
    vendor_dir: &Path,
    tarball_path: &Path,
    blank_md: bool,
    v: crate::Verbosity,
) -> Result<()> {
    if let Some(parent) = tarball_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if blank_md {
        for entry in walkdir::WalkDir::new(vendor_dir) {
            let entry = entry?;
            if entry.file_type().is_file()
                && let Some(ext) = entry.path().extension()
                && ext == "md"
            {
                std::fs::write(entry.path(), "")?;
            }
        }
        if v.debug() {
            eprintln!("  Blanked .md files in vendor/");
        }
    }

    let vendor_name = vendor_dir
        .file_name()
        .context("vendor dir has no name")?
        .to_string_lossy();
    let parent_dir = vendor_dir.parent().context("vendor dir has no parent")?;

    // Suppress macOS xattr metadata that causes warnings on Linux GNU tar.
    // COPYFILE_DISABLE=1 prevents ._* AppleDouble files, but macOS bsdtar
    // still writes xattr PAX headers (LIBARCHIVE.xattr.*). The --no-xattrs
    // flag (supported by both bsdtar and GNU tar) prevents those too.
    let mut cmd = std::process::Command::new("tar");
    cmd.env("COPYFILE_DISABLE", "1");
    // Detect if tar supports --no-xattrs (bsdtar on macOS and GNU tar do)
    let has_no_xattrs = std::process::Command::new("tar")
        .arg("--no-xattrs")
        .arg("-cf")
        .arg("/dev/null")
        .arg("--files-from")
        .arg("/dev/null")
        .output()
        .is_ok_and(|o| o.status.success());
    if has_no_xattrs {
        cmd.arg("--no-xattrs");
    }
    cmd.arg("-cJf")
        .arg(tarball_path)
        .arg("-C")
        .arg(parent_dir)
        .arg(vendor_name.as_ref());
    let output = cmd.output().context("failed to run tar")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("tar compression failed:\n{}", stderr.trim());
    }

    if v.info() {
        let size = std::fs::metadata(tarball_path)
            .map(|m| m.len())
            .unwrap_or(0);
        eprintln!(
            "  Compressed vendor/ to {} ({:.1} MB)",
            tarball_path.display(),
            size as f64 / 1_048_576.0
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Verbosity;

    #[test]
    fn strip_vendor_path_deps_removes_relative_paths() {
        let dir = tempfile::tempdir().unwrap();
        let vendor = dir.path().join("vendor");
        std::fs::create_dir_all(vendor.join("crate-a")).unwrap();
        std::fs::create_dir_all(vendor.join("crate-b")).unwrap();

        // crate-a has a relative path dep to crate-b
        std::fs::write(
            vendor.join("crate-a/Cargo.toml"),
            r#"[package]
name = "crate-a"
version = "0.1.0"

[dependencies.crate-b]
version = "*"
path = "../crate-b"
"#,
        )
        .unwrap();

        // crate-b has no path deps
        std::fs::write(
            vendor.join("crate-b/Cargo.toml"),
            r#"[package]
name = "crate-b"
version = "0.1.0"
"#,
        )
        .unwrap();

        strip_vendor_path_deps(&vendor, Verbosity(0)).unwrap();

        let result = std::fs::read_to_string(vendor.join("crate-a/Cargo.toml")).unwrap();
        assert!(result.contains("crate-b"));
        assert!(result.contains("version"));
        assert!(!result.contains("path"));
    }

    #[test]
    fn strip_vendor_path_deps_keeps_internal_paths() {
        let dir = tempfile::tempdir().unwrap();
        let vendor = dir.path().join("vendor");
        std::fs::create_dir_all(vendor.join("mycrate")).unwrap();

        // path = "src/lib.rs" should NOT be stripped (it's internal, not ../...)
        std::fs::write(
            vendor.join("mycrate/Cargo.toml"),
            r#"[package]
name = "mycrate"
version = "0.1.0"

[lib]
path = "src/lib.rs"

[dependencies.sibling]
version = "*"
path = "../sibling"
"#,
        )
        .unwrap();

        strip_vendor_path_deps(&vendor, Verbosity(0)).unwrap();

        let result = std::fs::read_to_string(vendor.join("mycrate/Cargo.toml")).unwrap();
        // [lib] path is preserved (not a relative ../ path, and not in dependencies)
        assert!(result.contains("src/lib.rs"));
        // dependency path is stripped
        assert!(!result.contains("../sibling"));
    }
}
