//! Development bootstrap bundles path dependencies without freezing sources.
mod common;

#[test]
fn dev_bundle_relocates_path_chain_and_preserves_source_manifest() {
    let git = common::create_local_git_crate(
        "remote-value",
        "[package]\nname = \"remote-value\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn value() -> i32 { 1 }\n",
    );
    let project = common::create_workspace(
        "[workspace]\nmembers = [\"app\", \"core\", \"leaf\"]\nexclude = [\"unused-test-helper\"]\nresolver = \"2\"\n[workspace.package]\nversion = \"0.2.0\"\nedition = \"2021\"\n",
        &[
            (
                "app",
                &format!(
                    "[package]\nname = \"dev-probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"lib.rs\"\n[dependencies]\nengine = {{ package = \"core-value\", path = \"../core\", features = [\"answer\"] }}\nremote-value = {{ git = {:?} }}\n",
                    git.url()
                ),
                "pub fn value() -> i32 { engine::value() + remote_value::value() }\n",
            ),
            (
                "core",
                "[package]\nname = \"core-value\"\nversion.workspace = true\nedition.workspace = true\n[lib]\npath = \"lib.rs\"\n[features]\nanswer = []\n[dependencies]\nleaf-value = { path = \"../leaf\" }\n",
                "pub fn value() -> i32 { leaf_value::value() }\n",
            ),
            (
                "leaf",
                "[package]\nname = \"leaf-value\"\nversion.workspace = true\nedition.workspace = true\n[lib]\npath = \"lib.rs\"\n",
                "pub fn value() -> i32 { 6 }\n",
            ),
        ],
    );
    let core_manifest = project.root().join("core/Cargo.toml");
    let mut core = std::fs::read_to_string(&core_manifest).unwrap();
    core.push_str("\n[dev-dependencies]\nunused-test-helper = { path = \"../unused-test-helper\" }\n\n[build-dependencies]\nleaf-value = { path = \"../leaf\" }\n");
    core.push_str(&format!(
        "\n[dependencies.remote-value]\ngit = {:?}\n",
        git.url()
    ));
    std::fs::write(core_manifest, core).unwrap();
    let unused = project.root().join("unused-test-helper");
    std::fs::create_dir(&unused).unwrap();
    std::fs::write(unused.join("Cargo.toml"), "[package]\nname = \"unused-test-helper\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\npath = \"lib.rs\"\n[workspace]\n").unwrap();
    std::fs::write(unused.join("lib.rs"), "pub fn unused() {}\n").unwrap();
    std::fs::write(
        project.root().join("core/build.rs"),
        "fn main() { println!(\"cargo:rustc-env=LEAF_VALUE={}\", leaf_value::value()); }\n",
    )
    .unwrap();
    std::fs::write(project.root().join("core/fixture.txt"), "included payload").unwrap();
    std::fs::write(project.root().join("core/lib.rs"),
        "const _: &str = include_str!(\"fixture.txt\");\npub fn value() -> i32 { leaf_value::value() }\n").unwrap();
    std::fs::write(project.root().join("core/.gitignore"), "ignored-output/\n").unwrap();
    std::fs::create_dir(project.root().join("core/ignored-output")).unwrap();
    std::fs::write(
        project.root().join("core/ignored-output/artifact"),
        "do not bundle",
    )
    .unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(project.root())
            .status()
            .unwrap()
            .success()
    );
    let app = project.root().join("app");
    let manifest = app.join("Cargo.toml");
    let before = std::fs::read(&manifest).unwrap();
    let vendor = app.join("vendor");
    let invoke = || {
        common::revendor_cmd()
            .args(["revendor", "--dev", "--manifest-path"])
            .arg(&manifest)
            .arg("--output")
            .arg(&vendor)
            .arg("-v")
            .assert()
            .success();
    };
    invoke();
    assert_eq!(std::fs::read(&manifest).unwrap(), before);
    assert!(!app.join(".Cargo.toml.prefreeze").exists());
    assert!(!vendor.join(".cargo-config.toml").exists());
    assert!(!vendor.join("core-value-0.2.0/ignored-output").exists());
    let names: std::collections::BTreeSet<_> = std::fs::read_dir(&vendor)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        [
            "core-value-0.2.0",
            "leaf-value-0.2.0",
            "unused-test-helper-0.1.0"
        ]
        .map(String::from)
        .into()
    );
    std::fs::write(vendor.join("caller-owned"), "recover me").unwrap();
    invoke();
    assert_eq!(std::fs::read(&manifest).unwrap(), before);
    let backup = std::fs::read_dir(&app)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".dev-vendor-backup-")
        })
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(backup.join("vendor/caller-owned")).unwrap(),
        "recover me"
    );
    let portable = std::fs::read_to_string(app.join(".Cargo.toml.dev")).unwrap();
    assert!(!portable.contains("[patch"), "{portable}");
    assert!(
        portable.contains(&git.url()),
        "Git source must stay normal: {portable}"
    );
    assert!(portable.contains("vendor/core-value-0.2.0"), "{portable}");
    let relocated = tempfile::tempdir().unwrap();
    std::fs::write(relocated.path().join("Cargo.toml"), portable).unwrap();
    std::fs::copy(app.join("lib.rs"), relocated.path().join("lib.rs")).unwrap();
    std::fs::rename(&vendor, relocated.path().join("vendor")).unwrap();
    let cargo_home = tempfile::tempdir().unwrap();
    let check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(relocated.path())
        .env("CARGO_HOME", cargo_home.path())
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
}
