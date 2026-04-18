//! Shared test harness for cargo-revendor integration tests.
//!
//! Lives in `tests/common/mod.rs` per the Rust integration-test convention —
//! files directly in `tests/` compile as their own binaries; subdirectory-backed
//! modules don't. The project's usual "no mod.rs" rule is about production
//! crate structure; integration tests have a different convention enforced by
//! cargo itself.
//!
//! Helpers split into three groups:
//! - **Project builders**: `create_simple_crate`, `create_workspace`,
//!   `create_monorepo` — spawn a cargo project in a `TempDir`.
//! - **Git sources**: `LocalGitRepo`, `create_local_git_crate` — materialize a
//!   bare git repo locally so tests can reference it via `git = "file://..."`
//!   without touching the network.
//! - **Assertions / diffing**: `assert_vendor_has`, `assert_empty_checksum`,
//!   `extract_tarball`, `diff_trees`.

#![allow(dead_code)] // individual test binaries use different subsets

use assert_cmd::Command;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// region: project builders

/// A temporary cargo project for testing.
///
/// Holds the TempDir so the project lives for the test's duration and cleans
/// up on drop.
pub struct TestProject {
    _dir: TempDir,
    pub root: PathBuf,
}

impl TestProject {
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Build a `cargo-revendor` invocation. The resulting `Command` can be
/// `.current_dir()`'d and `.arg()`'d like any `assert_cmd` command.
pub fn revendor_cmd() -> Command {
    Command::cargo_bin("cargo-revendor").expect("cargo-revendor binary not built")
}

/// Create a single-crate project: one `Cargo.toml`, one `lib.rs`, initialized
/// as a git repo so `cargo package` is happy.
pub fn create_simple_crate(cargo_toml: &str, lib_rs: &str) -> TestProject {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("project");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), cargo_toml).unwrap();
    std::fs::write(root.join("lib.rs"), lib_rs).unwrap();
    git_init(&root);
    TestProject { _dir: dir, root }
}

/// Create a workspace with the given members.
/// `members: &[(name, cargo_toml, lib_rs)]`.
pub fn create_workspace(root_toml: &str, members: &[(&str, &str, &str)]) -> TestProject {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("workspace");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), root_toml).unwrap();
    for (name, toml, rs) in members {
        let member_dir = root.join(name);
        std::fs::create_dir_all(&member_dir).unwrap();
        std::fs::write(member_dir.join("Cargo.toml"), toml).unwrap();
        std::fs::write(member_dir.join("lib.rs"), rs).unwrap();
    }
    git_init(&root);
    TestProject { _dir: dir, root }
}

/// Monorepo shape: workspace at the root + an rpkg-style subdirectory with its
/// own `[workspace]` declaration, mirroring how the miniextendr repo is laid
/// out (rpkg is a standalone workspace inside a larger monorepo).
pub fn create_monorepo(
    ws_toml: &str,
    ws_members: &[(&str, &str, &str)],
    rpkg_toml: &str,
    rpkg_rs: &str,
) -> TestProject {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("monorepo");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Cargo.toml"), ws_toml).unwrap();
    for (name, toml, rs) in ws_members {
        let member_dir = root.join(name);
        std::fs::create_dir_all(&member_dir).unwrap();
        std::fs::write(member_dir.join("Cargo.toml"), toml).unwrap();
        std::fs::write(member_dir.join("lib.rs"), rs).unwrap();
    }
    let rpkg_dir = root.join("rpkg").join("src").join("rust");
    std::fs::create_dir_all(&rpkg_dir).unwrap();
    std::fs::write(rpkg_dir.join("Cargo.toml"), rpkg_toml).unwrap();
    std::fs::write(rpkg_dir.join("lib.rs"), rpkg_rs).unwrap();
    git_init(&root);
    TestProject { _dir: dir, root }
}

/// Initialize a git repo in `dir` with a single empty-allowed commit.
/// `cargo package` refuses to operate on paths that aren't under version
/// control, so every test project needs this.
pub fn git_init(dir: &Path) {
    run_git(dir, &["init", "-q"]);
    // `-c commit.gpgsign=false` dodges interactive signing prompts if the
    // user has `commit.gpgsign = true` globally.
    run_git(dir, &["add", "."]);
    run_git_author(dir, &["commit", "-q", "-m", "init", "--allow-empty"]);
}

fn run_git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {:?} failed to spawn: {e}", args));
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_git_author(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .unwrap_or_else(|e| panic!("git {:?} failed to spawn: {e}", args));
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

// endregion

// region: local git sources

/// A bare git repo materialized on the local filesystem, referenced via
/// `git = "file:///…"` in test fixtures.
///
/// This avoids network calls while still exercising `cargo vendor`'s git
/// resolution path (which uses the `cargo fetch` machinery).
pub struct LocalGitRepo {
    /// Filesystem path to the bare repo; format for `Cargo.toml`: `file://{path}`.
    pub bare_path: PathBuf,
    /// Commit OID of the initial commit. Usable as a `rev = "…"` pin.
    pub rev: String,
    /// Branch name the initial commit lives on — `main` by default.
    pub default_branch: String,
    _work_dir: TempDir,
    _bare_dir: TempDir,
}

impl LocalGitRepo {
    /// URL to embed in `Cargo.toml`: `file:///path/to/bare.git`.
    pub fn url(&self) -> String {
        format!("file://{}", self.bare_path.display())
    }
}

/// Create a bare local git repo containing a single-crate package.
///
/// 1. Writes `Cargo.toml` + `src/lib.rs` in a work dir.
/// 2. Initializes git, commits, reads back HEAD OID.
/// 3. Clones `--bare` to a sibling dir.
/// 4. Returns the bare path + OID so callers can reference it via
///    `file:///…` in their own test `Cargo.toml`.
pub fn create_local_git_crate(name: &str, cargo_toml: &str, lib_rs: &str) -> LocalGitRepo {
    let work = TempDir::new().unwrap();
    let work_root = work.path().join(name);
    std::fs::create_dir_all(work_root.join("src")).unwrap();
    std::fs::write(work_root.join("Cargo.toml"), cargo_toml).unwrap();
    std::fs::write(work_root.join("src/lib.rs"), lib_rs).unwrap();

    run_git(&work_root, &["init", "-q", "-b", "main"]);
    run_git(&work_root, &["add", "."]);
    run_git_author(&work_root, &["commit", "-q", "-m", "init"]);

    // Read the commit OID — callers might pin the dep to this `rev`.
    let rev_out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&work_root)
        .output()
        .expect("git rev-parse failed");
    assert!(rev_out.status.success(), "git rev-parse failed");
    let rev = String::from_utf8(rev_out.stdout)
        .unwrap()
        .trim()
        .to_string();

    // Bare clone into a sibling dir so callers can reference it as a git URL.
    let bare = TempDir::new().unwrap();
    let bare_path = bare.path().join(format!("{name}.git"));
    let out = std::process::Command::new("git")
        .args([
            "clone",
            "--bare",
            "-q",
            work_root.to_str().unwrap(),
            bare_path.to_str().unwrap(),
        ])
        .output()
        .expect("git clone --bare failed to spawn");
    assert!(
        out.status.success(),
        "git clone --bare failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    LocalGitRepo {
        bare_path,
        rev,
        default_branch: "main".to_string(),
        _work_dir: work,
        _bare_dir: bare,
    }
}

// endregion

// region: vendor assertions

/// Assert that `vendor/<name>/` exists and contains a `Cargo.toml`.
pub fn assert_vendor_has(vendor: &Path, name: &str) {
    let crate_dir = vendor.join(name);
    assert!(
        crate_dir.exists(),
        "expected vendor/{} to exist at {}",
        name,
        vendor.display()
    );
    assert!(
        crate_dir.join("Cargo.toml").exists(),
        "expected vendor/{}/Cargo.toml",
        name
    );
}

/// Assert that `vendor/<name>/` does NOT exist (matched crate was excluded).
pub fn assert_vendor_missing(vendor: &Path, name: &str) {
    assert!(
        !vendor.join(name).exists(),
        "vendor/{} should not exist",
        name
    );
}

/// Read a vendored crate's Cargo.toml as a string (panics on missing file).
pub fn read_vendor_toml(vendor: &Path, name: &str) -> String {
    std::fs::read_to_string(vendor.join(name).join("Cargo.toml"))
        .unwrap_or_else(|_| panic!("failed to read vendor/{}/Cargo.toml", name))
}

/// Assert a vendored crate's `.cargo-checksum.json` is the empty-files form
/// (`{"files":{}}`) — what cargo expects from a vendored-sources tree.
pub fn assert_empty_checksum(vendor: &Path, name: &str) {
    let cksum = vendor.join(name).join(".cargo-checksum.json");
    let content = std::fs::read_to_string(&cksum)
        .unwrap_or_else(|_| panic!("no .cargo-checksum.json in vendor/{}", name));
    assert_eq!(content, "{\"files\":{}}");
}

// endregion

// region: tarball round-trip

/// Extract a `.tar.xz` archive into `into`, which must already exist.
pub fn extract_tarball(xz: &Path, into: &Path) {
    let out = std::process::Command::new("tar")
        .arg("-xJf")
        .arg(xz)
        .arg("-C")
        .arg(into)
        .output()
        .expect("tar failed to spawn");
    assert!(
        out.status.success(),
        "tar -xJf {} failed: {}",
        xz.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// One difference between two directory trees.
#[derive(Debug, Clone)]
pub enum TreeDiff {
    /// File exists only under `a`.
    OnlyInA(String),
    /// File exists only under `b`.
    OnlyInB(String),
    /// File exists in both, contents differ.
    ContentDiff(String),
}

/// Recursively compare two directory trees. Returns every file-level
/// difference; empty vec means the trees are bit-for-bit identical.
pub fn diff_trees(a: &Path, b: &Path) -> Vec<TreeDiff> {
    let files_a = collect_file_hashes(a);
    let files_b = collect_file_hashes(b);

    let mut diffs = Vec::new();

    for (path, hash_a) in &files_a {
        match files_b.get(path) {
            Some(hash_b) if hash_a == hash_b => {}
            Some(_) => diffs.push(TreeDiff::ContentDiff(path.clone())),
            None => diffs.push(TreeDiff::OnlyInA(path.clone())),
        }
    }
    for path in files_b.keys() {
        if !files_a.contains_key(path) {
            diffs.push(TreeDiff::OnlyInB(path.clone()));
        }
    }

    diffs
}

fn collect_file_hashes(root: &Path) -> BTreeMap<String, u64> {
    use std::hash::{Hash, Hasher};
    let mut out = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .into_owned();
        let bytes = std::fs::read(entry.path()).unwrap_or_default();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        out.insert(rel, hasher.finish());
    }
    out
}

// endregion
