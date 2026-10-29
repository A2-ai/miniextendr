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

// region: resolve_source_override tests

#[test]
fn git_override_no_match_returns_none() {
    // Dep is not in overrides — should stay external.
    let overrides = vec![local_pkg("miniextendr-api", "0.5.0")];
    let result = resolve_source_override("serde", "1.0.0", &overrides).unwrap();
    assert!(
        result.is_none(),
        "unrelated dep should not match any override"
    );
}

#[test]
fn git_override_empty_list_returns_none() {
    let result = resolve_source_override("miniextendr-api", "0.5.0", &[]).unwrap();
    assert!(result.is_none(), "empty override list should never match");
}

#[test]
fn git_override_name_and_version_match_returns_pkg() {
    let overrides = vec![
        local_pkg("miniextendr-api", "0.5.0"),
        local_pkg("miniextendr-macros", "0.5.0"),
    ];
    let result = resolve_source_override("miniextendr-api", "0.5.0", &overrides)
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
    let err = resolve_source_override("miniextendr-api", "0.5.0", &overrides).unwrap_err();
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
    let result = resolve_source_override("miniextendr-api", "0.5.0", &overrides)
        .unwrap()
        .expect("should match first entry");
    assert_eq!(result.version, "0.5.0");
}

#[test]
fn git_override_unrelated_crate_same_version_not_matched() {
    // A dep named "serde" shouldn't match an override for "miniextendr-api"
    // even if versions happen to be equal.
    let overrides = vec![local_pkg("miniextendr-api", "1.0.0")];
    let result = resolve_source_override("serde", "1.0.0", &overrides).unwrap();
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

// region: discover_from_patch_config tests

/// Build a monorepo-shaped fixture rooted at a TempDir:
///   <root>/miniextendr-api/Cargo.toml         (framework crate source)
///   <root>/rpkg/src/rust/Cargo.toml           (target manifest)
///   <root>/rpkg/src/rust/.cargo/config.toml    (with the given body)
/// Returns (TempDir guard, target manifest path).
fn patch_config_fixture(config_body: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    // Framework crate at the workspace root (the [patch] path target).
    let api_dir = root.join("miniextendr-api");
    std::fs::create_dir_all(&api_dir).unwrap();
    std::fs::write(
        api_dir.join("Cargo.toml"),
        "[package]\nname = \"miniextendr-api\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    // Target manifest, nested rpkg-style, with its own .cargo/config.toml.
    let rust_dir = root.join("rpkg").join("src").join("rust");
    std::fs::create_dir_all(rust_dir.join(".cargo")).unwrap();
    let manifest = rust_dir.join("Cargo.toml");
    std::fs::write(
        &manifest,
        "[package]\nname = \"rpkg\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(rust_dir.join(".cargo").join("config.toml"), config_body).unwrap();

    (dir, manifest)
}

#[test]
fn patch_config_resolves_existing_path() {
    // A [patch."git+url"] entry whose path resolves to a real Cargo.toml
    // is returned as a local override.
    let body = "[patch.\"https://github.com/A2-ai/miniextendr\"]\n\
                    miniextendr-api = { path = \"../../../miniextendr-api\" }\n";
    let (_guard, manifest) = patch_config_fixture(body);
    let found = discover_from_patch_config(&manifest).unwrap();
    assert_eq!(found.len(), 1, "the resolvable entry should be discovered");
    assert_eq!(found[0].name, "miniextendr-api");
    assert_eq!(found[0].version, "0.1.0");
}

#[test]
fn patch_config_skips_missing_path() {
    // A [patch] entry pointing at a path with NO Cargo.toml is skipped
    // (and warned about). This is the latch-leak shape (#865/#876): the
    // override silently doesn't apply, so the framework crate is NOT
    // reported as a local override and falls back to git vendoring.
    let body = "[patch.\"https://github.com/A2-ai/miniextendr\"]\n\
                    miniextendr-api = { path = \"../../../does-not-exist\" }\n";
    let (_guard, manifest) = patch_config_fixture(body);
    let found = discover_from_patch_config(&manifest).unwrap();
    assert!(
        found.is_empty(),
        "an unresolvable [patch] path must NOT be reported as a local override, \
             so the loud-fail assertion can fire; got {found:?}"
    );
}

#[test]
fn patch_config_no_patch_table_is_empty() {
    // No [patch] table at all — the dev-tree-without-overrides case. No
    // local crates discovered; the caller's assertion catches the leak.
    let body = "[build]\nrustflags = []\n";
    let (_guard, manifest) = patch_config_fixture(body);
    let found = discover_from_patch_config(&manifest).unwrap();
    assert!(found.is_empty(), "no [patch] table → no local overrides");
}

// endregion

// region: discover_patch_url_map tests

#[test]
fn patch_url_map_records_url_for_resolvable_entry() {
    // The resolvable [patch."<url>"] entry is mapped crate -> url.
    let body = "[patch.\"https://github.com/A2-ai/miniextendr\"]\n\
                    miniextendr-api = { path = \"../../../miniextendr-api\" }\n";
    let (_guard, manifest) = patch_config_fixture(body);
    let map = discover_patch_url_map(&manifest).unwrap();
    assert_eq!(
        map.get("miniextendr-api").map(String::as_str),
        Some("https://github.com/A2-ai/miniextendr")
    );
}

#[test]
fn patch_url_map_strips_git_plus_prefix() {
    // cargo accepts [patch."git+https://…"]; the lockfile/source-replacement
    // form is git+<url>, so the stored url must be the bare URL (the stamper
    // re-adds git+). Verify the prefix is stripped.
    let body = "[patch.\"git+https://github.com/A2-ai/miniextendr\"]\n\
                    miniextendr-api = { path = \"../../../miniextendr-api\" }\n";
    let (_guard, manifest) = patch_config_fixture(body);
    let map = discover_patch_url_map(&manifest).unwrap();
    assert_eq!(
        map.get("miniextendr-api").map(String::as_str),
        Some("https://github.com/A2-ai/miniextendr")
    );
}

#[test]
fn patch_url_map_skips_unresolvable_path() {
    // An unresolvable [patch] path means the crate is vendored from its real
    // git source (already lock-correct) — it must NOT be stamped, so it is
    // absent from the map.
    let body = "[patch.\"https://github.com/A2-ai/miniextendr\"]\n\
                    miniextendr-api = { path = \"../../../does-not-exist\" }\n";
    let (_guard, manifest) = patch_config_fixture(body);
    let map = discover_patch_url_map(&manifest).unwrap();
    assert!(
        map.is_empty(),
        "unresolvable patch path must not be mapped; got {map:?}"
    );
}

// endregion
