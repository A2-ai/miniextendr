//! Portable path dependencies for development R package artifacts.

use crate::metadata::LocalPackage;
use anyhow::{Context, Result, bail};
use cargo_metadata::{DependencyKind, Metadata, Package, PackageId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub const MANIFEST: &str = ".Cargo.toml.dev";

pub fn prepare(
    manifest: &Path,
    output: &Path,
    allow_dirty: bool,
    v: crate::Verbosity,
) -> Result<()> {
    let base = manifest.parent().unwrap().canonicalize()?;
    let canonical_output = output
        .parent()
        .context("--dev output has no parent")?
        .canonicalize()?
        .join(
            output
                .file_name()
                .context("--dev output has no directory name")?,
        );
    let output = canonical_output.as_path();
    let relative = output
        .strip_prefix(&base)
        .context("--dev output must be inside the source crate")?;
    if relative.as_os_str().is_empty() {
        bail!("--dev output cannot replace the source crate");
    }
    let meta = crate::metadata::load_metadata(manifest)?;
    let root = meta
        .root_package()
        .context("development bootstrap requires a package manifest")?;
    let mut selected = BTreeSet::new();
    collect_paths(root, &meta, &mut selected)?;
    selected.remove(&root.id);
    let mut packages: Vec<&Package> = meta
        .packages
        .iter()
        .filter(|pkg| selected.contains(&pkg.id))
        .collect();
    // Package the root to obtain Cargo's normalized manifest, but ship only
    // its manifest: R's builder already copies the root source files.
    packages.push(root);
    let locals: Vec<LocalPackage> = packages
        .iter()
        .map(|pkg| LocalPackage {
            name: pkg.name.clone(),
            version: pkg.version.to_string(),
            path: pkg
                .manifest_path
                .parent()
                .unwrap()
                .as_std_path()
                .to_path_buf(),
            manifest_path: pkg.manifest_path.as_std_path().to_path_buf(),
        })
        .collect();
    let mut slots = BTreeMap::new();
    for pkg in &packages {
        let slot = format!("{}-{}", pkg.name, pkg.version);
        if slots.values().any(|existing| existing == &slot) {
            bail!("development path dependencies have duplicate package slot {slot}");
        }
        slots.insert(pkg.id.clone(), slot);
    }
    let stage = tempfile::tempdir()?;
    let archives = crate::package::package_local_crates(
        &locals,
        &locals,
        manifest,
        stage.path(),
        allow_dirty,
        v,
    )?;
    let tree = stage.path().join("vendor");
    std::fs::create_dir(&tree)?;
    let mut root_manifest = None;
    for (pkg, (_, archive)) in packages.iter().zip(archives) {
        let unpack = stage.path().join(format!("unpack-{}", slots[&pkg.id]));
        std::fs::create_dir(&unpack)?;
        let copied = unpack.join(&pkg.name);
        if pkg.id == root.id {
            std::fs::create_dir(&copied)?;
            if archive.is_dir() {
                std::fs::copy(archive.join("Cargo.toml"), copied.join("Cargo.toml"))?;
                crate::vendor::resolve_workspace_inheritance(&copied, &archive, v)?;
            } else {
                let decoder = flate2::read::GzDecoder::new(std::fs::File::open(&archive)?);
                let mut tar = tar::Archive::new(decoder);
                for entry in tar.entries()? {
                    let mut entry = entry?;
                    if entry.path()?.components().count() == 2
                        && entry
                            .path()?
                            .file_name()
                            .is_some_and(|name| name == "Cargo.toml")
                    {
                        entry.unpack(copied.join("Cargo.toml"))?;
                        break;
                    }
                }
            }
        } else {
            crate::vendor::extract_crate_archive(&archive, &unpack, &pkg.name, None, v)?;
        }
        let path = copied.join("Cargo.toml");
        let mut doc: toml_edit::DocumentMut = std::fs::read_to_string(&path)?.parse()?;
        // Cargo package drops patches; the direct-copy fallback can retain
        // its temporary packaging table. Dev artifacts use normal sources.
        doc.remove("patch");
        doc.remove("workspace");
        if pkg.id == root.id {
            doc["workspace"] = toml_edit::Item::Table(toml_edit::Table::new());
            let mut exclude = toml_edit::Array::new();
            exclude.push(format!("{}/*", crate::path_to_toml(relative)));
            doc["workspace"]["exclude"] = toml_edit::value(exclude);
        }
        rewrite_paths(
            &mut doc,
            pkg,
            &meta,
            &slots,
            if pkg.id == root.id {
                Some(relative)
            } else {
                None
            },
        )?;
        if pkg.id == root.id {
            root_manifest = Some(doc.to_string());
        } else {
            std::fs::write(&path, doc.to_string())?;
            std::fs::rename(copied, tree.join(&slots[&pkg.id]))?;
        }
    }
    // Retain the previous output for recovery instead of permanently deleting
    // an existing directory. Scaffold ignore rules keep these backups local.
    if output.exists() {
        let backup = tempfile::Builder::new()
            .prefix(".dev-vendor-backup-")
            .tempdir_in(&base)?
            .keep();
        std::fs::rename(output, backup.join("vendor"))?;
        if v.info() {
            eprintln!("  Previous dev output retained at {}", backup.display());
        }
    }
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&tree, output).or_else(|_| crate::copy_dir_recursive(&tree, output))?;
    std::fs::write(manifest.with_file_name(MANIFEST), root_manifest.unwrap())?;
    if v.info() {
        eprintln!(
            "  Prepared {} path dependencies for development (no registry/Git vendor or xz)",
            selected.len()
        );
    }
    Ok(())
}

fn path_package<'a>(meta: &'a Metadata, dep: &cargo_metadata::Dependency) -> Result<&'a Package> {
    let path = dep
        .path
        .as_ref()
        .unwrap()
        .join("Cargo.toml")
        .canonicalize()?;
    meta.packages
        .iter()
        .find(|pkg| pkg.manifest_path.canonicalize().ok().as_ref() == Some(&path))
        .with_context(|| format!("path dependency {} is absent from Cargo metadata", dep.name))
}

fn collect_paths(pkg: &Package, meta: &Metadata, selected: &mut BTreeSet<PackageId>) -> Result<()> {
    for dep in pkg.dependencies.iter().filter(|dep| dep.path.is_some()) {
        let child = path_package(meta, dep)?;
        if selected.insert(child.id.clone()) {
            collect_paths(child, meta, selected)?;
        }
    }
    Ok(())
}

fn rewrite_paths(
    doc: &mut toml_edit::DocumentMut,
    pkg: &Package,
    meta: &Metadata,
    slots: &BTreeMap<PackageId, String>,
    root_output: Option<&Path>,
) -> Result<()> {
    for dep in pkg.dependencies.iter().filter(|dep| dep.path.is_some()) {
        let child = path_package(meta, dep)?;
        let section = match dep.kind {
            DependencyKind::Normal => "dependencies",
            DependencyKind::Build => "build-dependencies",
            DependencyKind::Development => "dev-dependencies",
            _ => unreachable!("Cargo dependency kind"),
        };
        let table = if let Some(target) = &dep.target {
            &mut doc["target"][&target.to_string()][section]
        } else {
            &mut doc[section]
        };
        let alias = dep.rename.as_deref().unwrap_or(&dep.name);
        // Cargo package can omit dev-only path declarations entirely.
        let Some(entry) = table.get_mut(alias) else {
            continue;
        };
        let relative = if let Some(output) = root_output {
            crate::path_to_toml(&output.join(&slots[&child.id]))
        } else {
            format!("../{}", slots[&child.id])
        };
        match entry {
            toml_edit::Item::Value(toml_edit::Value::String(version)) => {
                let mut table = toml_edit::InlineTable::new();
                table.insert("version", toml_edit::Value::from(version.value()));
                table.insert("path", toml_edit::Value::from(relative));
                *entry = toml_edit::value(table);
            }
            _ => {
                entry
                    .as_table_like_mut()
                    .unwrap()
                    .insert("path", toml_edit::value(relative));
            }
        }
    }
    Ok(())
}
