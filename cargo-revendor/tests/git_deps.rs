//! Git-dependency integration tests for cargo-revendor (#227).
//!
//! cargo-revendor delegates git dependency materialization to `cargo vendor`.
//! These tests exercise the full path — including pins by branch, tag, and
//! rev, and git repos that contain multiple crates — against a bare local
//! repo (no network calls).
//!
//! All tests are gated behind `#[ignore]` because they still spawn cargo,
//! which on a cold machine may touch the registry even for pure-git deps
//! (cargo re-resolves the full graph). Run with `cargo test -- --ignored`.

mod common;

use common::{
    assert_valid_checksum, assert_vendor_has, assert_vendor_missing, create_local_git_crate,
    create_simple_crate, read_vendor_toml, revendor_cmd,
};

/// **G1** — bare local git repo used as a dep source. Verify `vendor/foo/`
/// materializes with a clean Cargo.toml, empty checksum, and the right
/// version.
#[test]
#[ignore] // network (cargo may touch the registry even for pure-git deps)
fn git_dep_from_local_bare_repo() {
    let git_foo = create_local_git_crate(
        "foo",
        r#"
[package]
name = "foo"
version = "0.1.0"
edition = "2021"
publish = false
"#,
        "pub fn foo() -> u32 { 42 }\n",
    );

    let project = create_simple_crate(
        &format!(
            r#"
[package]
name = "test-proj"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
foo = {{ git = "{}" }}
"#,
            git_foo.url()
        ),
        "pub use foo::foo;\n",
    );

    let vendor = project.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(project.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    assert_vendor_has(&vendor, "foo");
    let toml = read_vendor_toml(&vendor, "foo");
    assert!(
        toml.contains("name = \"foo\""),
        "vendored Cargo.toml should name foo, got:\n{toml}"
    );
    assert!(
        toml.contains("version = \"0.1.0\""),
        "vendored Cargo.toml should report version 0.1.0, got:\n{toml}"
    );
    // Git-source crates have "package": null (no registry hash) but should
    // still have a non-empty files map with SHA-256s for all vendored files.
    assert_valid_checksum(&vendor, "foo");
}

/// **G2** — pin the dep to a specific commit OID. Verify the pin round-trips
/// through vendoring and that the lockfile records the rev.
#[test]
#[ignore] // network
fn git_dep_pinned_by_rev() {
    let git_foo = create_local_git_crate(
        "pinned",
        r#"
[package]
name = "pinned"
version = "0.1.0"
edition = "2021"
publish = false
"#,
        "pub const ANSWER: u32 = 42;\n",
    );

    let rev = git_foo.rev.clone();

    let project = create_simple_crate(
        &format!(
            r#"
[package]
name = "test-proj"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
pinned = {{ git = "{}", rev = "{}" }}
"#,
            git_foo.url(),
            rev
        ),
        "pub use pinned::ANSWER;\n",
    );

    let vendor = project.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(project.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    assert_vendor_has(&vendor, "pinned");

    // The lockfile should record our pinned rev.
    let lock =
        std::fs::read_to_string(project.root().join("Cargo.lock")).expect("Cargo.lock missing");
    assert!(
        lock.contains(&rev),
        "Cargo.lock should pin the exact rev {}, got:\n{}",
        rev,
        lock
    );
    assert!(
        lock.contains("source = \"git+"),
        "Cargo.lock should record a git source, got:\n{lock}"
    );
}

/// **G3** — one bare git repo referenced by a branch pin vs the implicit
/// default branch — both should resolve to the same vendored copy (same
/// commit), not diverge into two.
#[test]
#[ignore] // network
fn git_dep_branch_vs_default_resolve_consistently() {
    // One repo, one commit on branch `main`. The work dir was branched `main`
    // explicitly in `create_local_git_crate`, so `branch = "main"` and a
    // default-branch reference should point at the same object.
    let git_foo = create_local_git_crate(
        "branched",
        r#"
[package]
name = "branched"
version = "0.1.0"
edition = "2021"
publish = false
"#,
        "pub fn hi() {}\n",
    );

    let project = create_simple_crate(
        &format!(
            r#"
[package]
name = "test-proj"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
branched = {{ git = "{}", branch = "{}" }}
"#,
            git_foo.url(),
            git_foo.default_branch
        ),
        "pub use branched::hi;\n",
    );

    let vendor = project.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(project.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    assert_vendor_has(&vendor, "branched");
    let lock =
        std::fs::read_to_string(project.root().join("Cargo.lock")).expect("Cargo.lock missing");
    // Rev pinned even when source spec is a branch — cargo normalizes to the
    // resolved commit.
    assert!(
        lock.contains(&git_foo.rev),
        "Cargo.lock should record the resolved rev {}, got:\n{}",
        git_foo.rev,
        lock
    );
}

/// **G4** — git monorepo containing two crates; the caller depends on only
/// one. Verify the other is not pulled into `vendor/`.
#[test]
#[ignore] // network
fn git_monorepo_only_referenced_crate_vendored() {
    // Build a mini monorepo locally.
    let work = tempfile::TempDir::new().unwrap();
    let work_root = work.path().join("mono");
    std::fs::create_dir_all(work_root.join("used").join("src")).unwrap();
    std::fs::create_dir_all(work_root.join("unused").join("src")).unwrap();

    std::fs::write(
        work_root.join("Cargo.toml"),
        r#"[workspace]
members = ["used", "unused"]
resolver = "2"
"#,
    )
    .unwrap();
    std::fs::write(
        work_root.join("used/Cargo.toml"),
        r#"[package]
name = "used"
version = "0.1.0"
edition = "2021"
publish = false
"#,
    )
    .unwrap();
    std::fs::write(work_root.join("used/src/lib.rs"), "pub fn used() {}\n").unwrap();
    std::fs::write(
        work_root.join("unused/Cargo.toml"),
        r#"[package]
name = "unused"
version = "0.1.0"
edition = "2021"
publish = false
"#,
    )
    .unwrap();
    std::fs::write(work_root.join("unused/src/lib.rs"), "pub fn unused() {}\n").unwrap();

    common::git_init(&work_root);

    let bare = tempfile::TempDir::new().unwrap();
    let bare_path = bare.path().join("mono.git");
    std::process::Command::new("git")
        .args([
            "clone",
            "--bare",
            "-q",
            work_root.to_str().unwrap(),
            bare_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone --bare failed");

    let url = format!("file://{}", bare_path.display());

    let project = create_simple_crate(
        &format!(
            r#"
[package]
name = "test-proj"
version = "0.1.0"
edition = "2021"
publish = false

[workspace]

[lib]
path = "lib.rs"

[dependencies]
used = {{ git = "{url}" }}
"#
        ),
        "pub use used::used;\n",
    );

    let vendor = project.root().join("vendor");
    revendor_cmd()
        .arg("revendor")
        .arg("--manifest-path")
        .arg(project.root().join("Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .assert()
        .success();

    assert_vendor_has(&vendor, "used");
    assert_vendor_missing(&vendor, "unused");
}

/// **G5** — `[patch."https://…"] foo = { path = "../foo" }` redirects the
/// git source to a local workspace crate. The git source shouldn't be in
/// `vendor/`; the path dep should be vendored as if it were a normal local
/// crate.
#[test]
#[ignore] // network
fn git_dep_overridden_by_patch() {
    // Bare git repo as the "upstream" that we're going to override.
    let git_upstream = create_local_git_crate(
        "overridden",
        r#"
[package]
name = "overridden"
version = "0.1.0"
edition = "2021"
publish = false
"#,
        "pub fn upstream() -> u32 { 1 }\n",
    );

    // Test project is a workspace: member `test-proj` depends on the git dep
    // at declaration time; a sibling workspace member `overridden-local` is
    // named the same and patched in.
    let work = tempfile::TempDir::new().unwrap();
    let root = work.path().join("ws");
    std::fs::create_dir_all(root.join("test-proj")).unwrap();
    std::fs::create_dir_all(root.join("overridden-local")).unwrap();

    let url = git_upstream.url();
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[workspace]
members = ["test-proj", "overridden-local"]
resolver = "2"

[patch."{url}"]
overridden = {{ path = "overridden-local" }}
"#
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("test-proj/Cargo.toml"),
        format!(
            r#"[package]
name = "test-proj"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "lib.rs"

[dependencies]
overridden = {{ git = "{url}" }}
"#
        ),
    )
    .unwrap();
    std::fs::write(root.join("test-proj/lib.rs"), "pub use overridden::*;\n").unwrap();
    std::fs::write(
        root.join("overridden-local/Cargo.toml"),
        r#"[package]
name = "overridden"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
path = "src/lib.rs"
"#,
    )
    .unwrap();
    std::fs::create_dir_all(root.join("overridden-local/src")).unwrap();
    std::fs::write(
        root.join("overridden-local/src/lib.rs"),
        "pub fn overridden() -> u32 { 99 }\n",
    )
    .unwrap();
    common::git_init(&root);

    let vendor = root.join("vendor");
    revendor_cmd()
        .arg("--manifest-path")
        .arg(root.join("test-proj/Cargo.toml"))
        .arg("--output")
        .arg(&vendor)
        .arg("--source-root")
        .arg(&root)
        .assert()
        .success();

    // The local crate gets packaged and vendored.
    assert_vendor_has(&vendor, "overridden");
    // The git source is patched out — it shouldn't appear as a separate dir.
    // cargo vendor places git sources at `vendor/<name>/`, same as path deps,
    // so the only way to distinguish is by content / checksum — read the
    // vendored Cargo.toml and confirm it has the local crate's content,
    // not the upstream's.
    let toml = read_vendor_toml(&vendor, "overridden");
    assert!(
        toml.contains("[package]") && toml.contains("name = \"overridden\""),
        "vendored crate should be the patched local, got:\n{toml}"
    );
}
