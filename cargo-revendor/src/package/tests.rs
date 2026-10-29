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
