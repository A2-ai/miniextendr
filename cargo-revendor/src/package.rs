//! Package local crates via `cargo package`
//!
//! `cargo package` resolves workspace inheritance (version.workspace = true),
//! producing standalone Cargo.toml files that work outside the workspace.
//!
//! By default `cargo package` also writes a Cargo.lock into the package, which
//! it resolves as if the crate came from crates.io. For a crate that depends on
//! an unpublished local sibling that resolution fails ("no matching package
//! named ..."), and a `[patch]` table cannot help, because packaging drops it.
//! A vendored crate needs no lockfile of its own, so the package is built with
//! `--exclude-lockfile` (Cargo 1.87 and later), which skips that resolution.
//! With an older Cargo such a crate falls back to a direct copy of its
//! directory, whose workspace inheritance the caller resolves itself.

use crate::metadata::LocalPackage;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Package each local crate, returning (name, crate_archive_path) pairs
///
/// `local_pkgs` — crates to actually package
/// `all_local_pkgs` — ALL local crates, so a path dependency on any of them
/// gets the `version` key `cargo package` requires
///
/// A crate `cargo package` cannot package comes back as its source directory.
pub fn package_local_crates(
    local_pkgs: &[LocalPackage],
    all_local_pkgs: &[LocalPackage],
    _target_manifest: &Path,
    staging_dir: &Path,
    allow_dirty: bool,
    v: crate::Verbosity,
) -> Result<Vec<(String, PathBuf)>> {
    let mut results = Vec::new();

    // Build a set of all local package names for path dep detection
    let local_names: std::collections::HashSet<&str> =
        all_local_pkgs.iter().map(|p| p.name.as_str()).collect();

    let exclude_lockfile = supports_exclude_lockfile();

    for pkg in local_pkgs {
        if v.info() {
            eprintln!("  Packaging {} v{} ...", pkg.name, pkg.version);
        }

        let target_dir = staging_dir.join("package-target");

        // Snapshot the crate manifest, rewritten below. The guards restore
        // their files on drop (scope exits at the end of this iteration, or
        // earlier via `?` / panic unwind).
        let _inner_guard = crate::manifest_guard::ManifestGuard::snapshot(&pkg.manifest_path)?;
        // Cargo package loads the workspace and can rewrite an existing
        // source-workspace lockfile. Restore that file too.
        let ws_lock = crate::find_workspace_root(&pkg.path)?.join("Cargo.lock");
        let _ws_lock_guard = ws_lock
            .is_file()
            .then(|| crate::manifest_guard::ManifestGuard::snapshot(&ws_lock))
            .transpose()?;

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

        // Unset CARGO_TARGET_DIR so cargo package uses its own target directory
        let mut cmd = Command::new("cargo");
        cmd.arg("package")
            .arg("--manifest-path")
            .arg(&pkg.manifest_path)
            .arg("--no-verify")
            .arg("--target-dir")
            .arg(&target_dir)
            .env_remove("CARGO_TARGET_DIR");

        if exclude_lockfile {
            cmd.arg("--exclude-lockfile");
        }
        if allow_dirty {
            cmd.arg("--allow-dirty");
        }

        let output = cmd
            .output()
            .with_context(|| format!("failed to run cargo package for {}", pkg.name))?;

        // Guards (_inner_guard, _ws_lock_guard) restore their files on drop —
        // no explicit restore needed. Drops run at end of this iteration.

        if !output.status.success() {
            // Fallback: copy the crate directory; the caller resolves its
            // workspace inheritance.
            if v.info() {
                eprintln!(
                    "  cargo package failed for {} ({}); copying the crate directory instead",
                    pkg.name,
                    cargo_error_summary(&String::from_utf8_lossy(&output.stderr))
                );
            }
            if v.debug() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("    cargo package stderr: {}", stderr.trim());
            }
            results.push((pkg.name.clone(), pkg.path.clone()));
            continue;
        }

        // The archive cargo just wrote (other crates share the directory)
        let crate_file = target_dir
            .join("package")
            .join(format!("{}-{}.crate", pkg.name, pkg.version));
        if !crate_file.is_file() {
            bail!(
                "cargo package succeeded for {} but wrote no {}",
                pkg.name,
                crate_file.display()
            );
        }

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
            for (alias, dep) in table.iter_mut() {
                if local_names.contains(crate::vendor::dependency_package_name(alias.get(), dep))
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

/// Whether this Cargo has `cargo package --exclude-lockfile` (Cargo 1.87+).
fn supports_exclude_lockfile() -> bool {
    Command::new("cargo")
        .args(["package", "--help"])
        .output()
        .is_ok_and(|out| String::from_utf8_lossy(&out.stdout).contains("--exclude-lockfile"))
}

/// Condense cargo's stderr to its error and first cause, for a one-line message.
fn cargo_error_summary(stderr: &str) -> String {
    let mut lines = stderr.lines().map(str::trim);
    let Some(error) = lines.find_map(|line| line.strip_prefix("error: ")) else {
        return stderr.trim().to_string();
    };
    let cause = lines
        .skip_while(|line| *line != "Caused by:")
        .nth(1)
        .filter(|line| !line.is_empty());
    match cause {
        Some(cause) => format!("{error}: {cause}"),
        None => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packaging_preserves_existing_source_lockfile() {
        let dir = tempfile::tempdir().unwrap();
        let mut packages = Vec::new();
        for name in ["core", "unused-helper"] {
            let path = dir.path().join(name);
            std::fs::create_dir_all(path.join("src")).unwrap();
            std::fs::write(path.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
            let manifest_path = path.join("Cargo.toml");
            std::fs::write(
                &manifest_path,
                format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
            )
            .unwrap();
            packages.push(LocalPackage {
                name: name.into(),
                version: "0.1.0".into(),
                path,
                manifest_path,
            });
        }
        let output = Command::new("cargo")
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(&packages[0].manifest_path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let lock = packages[0].path.join("Cargo.lock");
        let before = std::fs::read(&lock).unwrap();
        let staging = dir.path().join("staging");
        std::fs::create_dir(&staging).unwrap();
        let archives = package_local_crates(
            &packages[..1],
            &packages,
            &packages[0].manifest_path,
            &staging,
            true,
            crate::Verbosity(0),
        )
        .unwrap();
        assert!(
            archives[0].1.is_file(),
            "expected cargo package to produce an archive"
        );
        assert_eq!(std::fs::read(&lock).unwrap(), before);
    }

    #[test]
    fn packaging_archives_a_crate_with_an_unpublished_local_dependency() {
        let dir = tempfile::tempdir().unwrap();
        let mut packages = Vec::new();
        for (name, deps) in [("core", ""), ("app", "core = { path = \"../core\" }\n")] {
            let path = dir.path().join(name);
            std::fs::create_dir_all(path.join("src")).unwrap();
            std::fs::write(path.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
            let manifest_path = path.join("Cargo.toml");
            std::fs::write(
                &manifest_path,
                format!(
                    "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{deps}"
                ),
            )
            .unwrap();
            packages.push(LocalPackage {
                name: name.into(),
                version: "0.1.0".into(),
                path,
                manifest_path,
            });
        }
        let manifest_before = std::fs::read(&packages[1].manifest_path).unwrap();
        let staging = dir.path().join("staging");
        std::fs::create_dir(&staging).unwrap();
        let archives = package_local_crates(
            &packages[1..],
            &packages,
            &packages[1].manifest_path,
            &staging,
            true,
            crate::Verbosity(0),
        )
        .unwrap();
        // `core` is on no registry. Resolving it for the package's own
        // Cargo.lock used to fail, sending `app` to the direct-copy fallback
        // (its directory instead of an archive).
        assert_eq!(archives[0].1.is_file(), supports_exclude_lockfile());
        assert_eq!(
            std::fs::read(&packages[1].manifest_path).unwrap(),
            manifest_before
        );
    }

    #[test]
    fn cargo_error_summary_joins_the_error_and_its_first_cause() {
        let stderr = "   Packaging app v0.1.0 (/tmp/app)\n    Updating crates.io index\n\
            error: failed to prepare local package for uploading\n\nCaused by:\n  \
            no matching package named `core` found\n  location searched: crates.io index\n";
        assert_eq!(
            cargo_error_summary(stderr),
            "failed to prepare local package for uploading: no matching package named `core` found"
        );
        assert_eq!(
            cargo_error_summary("error: unexpected argument '--frobnicate' found\n"),
            "unexpected argument '--frobnicate' found"
        );
    }

    #[test]
    fn packaging_adds_versions_to_renamed_local_dependencies() {
        let manifest = r#"[dependencies]
core_library = { package = "core", path = "../core" }
[build-dependencies.build_core]
package = "core"
path = "../core"
"#;
        let names = std::collections::HashSet::from(["core"]);
        let rewritten: toml_edit::DocumentMut =
            add_versions_to_path_deps(manifest, &names).parse().unwrap();
        assert_eq!(
            rewritten["dependencies"]["core_library"]["version"].as_str(),
            Some("*")
        );
        assert_eq!(
            rewritten["build-dependencies"]["build_core"]["version"].as_str(),
            Some("*")
        );
        assert_eq!(
            rewritten["dependencies"]["core_library"]["package"].as_str(),
            Some("core")
        );
    }
}
