use super::*;
use tempfile::TempDir;

#[test]
fn workspace_root_accepts_standalone_packages_and_virtual_members() {
    let root = TempDir::new().unwrap();
    let member = root.path().join("core");
    std::fs::create_dir_all(member.join("src")).unwrap();
    std::fs::write(member.join("src/lib.rs"), "pub fn hello() {}\n").unwrap();
    std::fs::write(
        member.join("Cargo.toml"),
        r#"[package]
name = "core"
version = "0.1.0"
"#,
    )
    .unwrap();
    assert_eq!(
        find_workspace_root(&member).unwrap(),
        member.canonicalize().unwrap()
    );
    std::fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"core\"]\n",
    )
    .unwrap();
    assert_eq!(
        find_workspace_root(&member).unwrap(),
        root.path().canonicalize().unwrap()
    );
}

/// Build a minimal Cli with all flags at their defaults. We parse with a
/// dummy manifest path so clap doesn't run auto-discovery.
fn base_cli() -> Cli {
    Cli::parse_from(["cargo-revendor", "revendor", "--manifest-path", "/dev/null"])
}

#[test]
fn compression_level_requires_archive_and_accepts_only_xz_presets() {
    assert_eq!(base_cli().compression_level, None);
    for level in ["0", "1", "9"] {
        let cli = Cli::try_parse_from([
            "cargo-revendor",
            "revendor",
            "--compress",
            "vendor.tar.xz",
            "--compression-level",
            level,
        ])
        .unwrap();
        assert_eq!(cli.compression_level, Some(level.parse().unwrap()));
    }
    for args in [
        vec!["--compression-level", "1"],
        vec![
            "--compress",
            "vendor.tar.xz",
            "--compression-level",
            "1",
            "--verify",
        ],
        vec![
            "--compress",
            "vendor.tar.xz",
            "--compression-level",
            "1",
            "--local-only",
        ],
        vec!["--compress", "vendor.tar.xz", "--compression-level", "10"],
        vec!["--compress", "vendor.tar.xz", "--compression-level", "-1"],
        vec!["--compress", "vendor.tar.xz", "--compression-level", "fast"],
    ] {
        assert!(
            Cli::try_parse_from(["cargo-revendor", "revendor"].into_iter().chain(args)).is_err()
        );
    }
}

fn tmp_output() -> (TempDir, std::path::PathBuf) {
    let d = TempDir::new().unwrap();
    let p = d.path().join("vendor");
    std::fs::create_dir_all(&p).unwrap();
    (d, p)
}

// region: validate_flag_compatibility

#[test]
fn validate_external_only_freeze_errors() {
    let (_d, output) = tmp_output();
    let mut cli = base_cli();
    cli.freeze = true;
    let err = validate_flag_compatibility(&cli, Mode::ExternalOnly, &output).unwrap_err();
    assert!(
        err.to_string()
            .contains("--external-only is incompatible with --freeze")
    );
}

#[test]
fn validate_external_only_compress_errors() {
    let (_d, output) = tmp_output();
    let mut cli = base_cli();
    cli.compress = Some(std::path::PathBuf::from("vendor.tar.xz"));
    let err = validate_flag_compatibility(&cli, Mode::ExternalOnly, &output).unwrap_err();
    assert!(
        err.to_string()
            .contains("--external-only is incompatible with --compress")
    );
}

#[test]
fn validate_external_only_source_marker_errors() {
    let (_d, output) = tmp_output();
    let mut cli = base_cli();
    cli.source_marker = true;
    let err = validate_flag_compatibility(&cli, Mode::ExternalOnly, &output).unwrap_err();
    assert!(
        err.to_string()
            .contains("--external-only is incompatible with --source-marker")
    );
}

#[test]
fn validate_external_only_blank_md_errors() {
    let (_d, output) = tmp_output();
    let mut cli = base_cli();
    cli.blank_md = true;
    let err = validate_flag_compatibility(&cli, Mode::ExternalOnly, &output).unwrap_err();
    assert!(
        err.to_string()
            .contains("--external-only is incompatible with --blank-md")
    );
}

#[test]
fn validate_local_only_compress_without_externals_errors() {
    let (_d, output) = tmp_output();
    // No .revendor-cache-external present → should fail
    let mut cli = base_cli();
    cli.compress = Some(std::path::PathBuf::from("vendor.tar.xz"));
    let err = validate_flag_compatibility(&cli, Mode::LocalOnly, &output).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("--local-only with --freeze/--compress"),
        "unexpected error: {msg}"
    );
    assert!(
        msg.contains(".revendor-cache-external not found"),
        "unexpected error: {msg}"
    );
}

#[test]
fn validate_local_only_compress_with_externals_ok() {
    let (_d, output) = tmp_output();
    // Write the sentinel file — externals were previously vendored.
    std::fs::write(output.join(cache::CACHE_FILE_EXTERNAL), "abcd1234").unwrap();
    let mut cli = base_cli();
    cli.compress = Some(std::path::PathBuf::from("vendor.tar.xz"));
    validate_flag_compatibility(&cli, Mode::LocalOnly, &output).unwrap();
}

#[test]
fn validate_full_mode_accepts_all_flags() {
    let (_d, output) = tmp_output();
    // Full mode should never error in validate_flag_compatibility.
    let mut cli = base_cli();
    cli.freeze = true;
    cli.compress = Some(std::path::PathBuf::from("v.tar.xz"));
    cli.source_marker = true;
    cli.blank_md = true;
    validate_flag_compatibility(&cli, Mode::Full, &output).unwrap();
}

#[test]
fn validate_local_only_no_flags_always_ok() {
    let (_d, output) = tmp_output();
    // No externals present, but also no flags that require them.
    let cli = base_cli();
    validate_flag_compatibility(&cli, Mode::LocalOnly, &output).unwrap();
}

// endregion
