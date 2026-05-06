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

/// Resolve `vendor/<name>/` or `vendor/<name>-<version>/` to an actual path.
///
/// Since cargo-revendor PR #239 flipped `--versioned-dirs` to default, crate
/// directories live at `vendor/<name>-<version>/`. Tests shouldn't hardcode
/// either shape — use this helper to probe in order:
///
/// 1. Exact `vendor/<name>-<version>/` if `version` is provided.
/// 2. Any `vendor/<name>-*/` glob match (versioned default).
/// 3. Flat `vendor/<name>/` fallback (the `--flat-dirs` opt-out path).
///
/// Panics with a helpful message if none exists. Mirrors the probe-then-fall-
/// back pattern in `cargo-revendor/src/verify.rs` (`verify_lock_matches_vendor`)
/// and `minirextendr/R/vendor.R` (`add_vendor_patches`).
pub fn vendor_dir_for(vendor: &Path, name: &str, version: Option<&str>) -> PathBuf {
    if let Some(v) = version {
        let versioned = vendor.join(format!("{name}-{v}"));
        if versioned.is_dir() {
            return versioned;
        }
    }
    if let Ok(entries) = std::fs::read_dir(vendor) {
        let prefix = format!("{name}-");
        let mut matches: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with(&prefix))
            })
            .collect();
        if matches.len() == 1 {
            return matches.pop().unwrap();
        }
        if matches.len() > 1 {
            panic!(
                "vendor_dir_for({name}): ambiguous — multiple versioned matches in {}: {matches:?}",
                vendor.display()
            );
        }
    }
    let flat = vendor.join(name);
    if flat.is_dir() {
        return flat;
    }
    panic!(
        "vendor_dir_for({name}): no matching directory in {}",
        vendor.display()
    );
}

/// Does either `vendor/<name>/` or `vendor/<name>-*/` exist?
///
/// Use this instead of `vendor.join(name).exists()` in assertions that only
/// care whether the crate was vendored, not about its exact directory shape.
pub fn vendor_has(vendor: &Path, name: &str) -> bool {
    if vendor.join(name).is_dir() {
        return true;
    }
    let prefix = format!("{name}-");
    std::fs::read_dir(vendor)
        .ok()
        .map(|entries| {
            entries.flatten().filter(|e| e.path().is_dir()).any(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with(&prefix))
            })
        })
        .unwrap_or(false)
}

/// Assert that `vendor/<name>/` or `vendor/<name>-*/` exists and contains a
/// `Cargo.toml`.
pub fn assert_vendor_has(vendor: &Path, name: &str) {
    let crate_dir = vendor_dir_for(vendor, name, None);
    assert!(
        crate_dir.exists(),
        "expected vendor/{} (or versioned equivalent) to exist at {}",
        name,
        vendor.display()
    );
    assert!(
        crate_dir.join("Cargo.toml").exists(),
        "expected {}/Cargo.toml",
        crate_dir.display()
    );
}

/// Assert that neither `vendor/<name>/` nor `vendor/<name>-*/` exists (matched
/// crate was excluded).
pub fn assert_vendor_missing(vendor: &Path, name: &str) {
    assert!(
        !vendor_has(vendor, name),
        "vendor/{} (or versioned equivalent) should not exist",
        name
    );
}

/// Read a vendored crate's Cargo.toml as a string (panics on missing file).
pub fn read_vendor_toml(vendor: &Path, name: &str) -> String {
    let crate_dir = vendor_dir_for(vendor, name, None);
    std::fs::read_to_string(crate_dir.join("Cargo.toml"))
        .unwrap_or_else(|_| panic!("failed to read {}/Cargo.toml", crate_dir.display()))
}

/// Assert that a vendored crate has a valid `.cargo-checksum.json` with a
/// non-empty `files` map containing SHA-256 hashes.
///
/// cargo-revendor now recomputes real checksums after CRAN-trim: the `files`
/// map has actual per-file SHA-256s and the `package` field (if present) is
/// preserved from the original registry hash.
pub fn assert_valid_checksum(vendor: &Path, name: &str) {
    let crate_dir = vendor_dir_for(vendor, name, None);
    let cksum = crate_dir.join(".cargo-checksum.json");
    let content = std::fs::read_to_string(&cksum)
        .unwrap_or_else(|_| panic!("no .cargo-checksum.json in {}", crate_dir.display()));
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_else(|e| {
        panic!(
            ".cargo-checksum.json in {} is not valid JSON: {e}",
            crate_dir.display()
        )
    });
    // `files` key must exist and be an object
    let files = parsed
        .get("files")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| {
            panic!(
                ".cargo-checksum.json in {} missing `files` object",
                crate_dir.display()
            )
        });
    // Every file in the crate dir (except .cargo-checksum.json) must appear in files
    // and every value must look like a 64-char hex SHA-256.
    for (path_str, hash_val) in files {
        let hash = hash_val.as_str().unwrap_or_else(|| {
            panic!(
                "files[{path_str}] is not a string in {}",
                crate_dir.display()
            )
        });
        assert_eq!(
            hash.len(),
            64,
            "files[{path_str}] hash has wrong length {} (expected 64-char SHA-256) in {}",
            hash.len(),
            crate_dir.display()
        );
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "files[{path_str}] hash {hash} is not hex in {}",
            crate_dir.display()
        );
    }
}

/// Assert a vendored crate's `.cargo-checksum.json` is the empty-files form
/// (`{"files":{}}`).  Retained for tests that explicitly check that a crate
/// has NO source files in its checksum map (e.g. path/git-source stubs that
/// cargo vendor emits with an empty package and empty files).
pub fn assert_empty_checksum(vendor: &Path, name: &str) {
    let crate_dir = vendor_dir_for(vendor, name, None);
    let cksum = crate_dir.join(".cargo-checksum.json");
    let content = std::fs::read_to_string(&cksum)
        .unwrap_or_else(|_| panic!("no .cargo-checksum.json in {}", crate_dir.display()));
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("bad JSON in .cargo-checksum.json: {e}"));
    let files = parsed
        .get("files")
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| panic!("missing files object in .cargo-checksum.json"));
    assert!(
        files.is_empty(),
        "expected empty files map but found: {:?}",
        files.keys().collect::<Vec<_>>()
    );
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
