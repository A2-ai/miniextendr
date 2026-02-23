use std::path::Path;

use anyhow::{Result, bail};

use crate::bridge::{bash, run_command, run_command_capture};
use crate::cli::VendorCmd;
use crate::project::{MINIEXTENDR_CRATES, ProjectContext, find_workspace_root};

pub fn dispatch(cmd: &VendorCmd, ctx: &ProjectContext, quiet: bool, json: bool) -> Result<()> {
    match cmd {
        VendorCmd::Pack => vendor_pack(ctx, quiet),
        VendorCmd::Versions => vendor_versions(quiet, json),
        VendorCmd::Miniextendr {
            miniextendr_version: _,
            dest: _,
            refresh: _,
            local_path,
        } => vendor_miniextendr(ctx, local_path.as_deref(), quiet),
        VendorCmd::CratesIo => vendor_crates_io(ctx, quiet),
        VendorCmd::Sync => vendor_sync(ctx, quiet),
        VendorCmd::SyncCheck => vendor_sync_check(ctx, quiet),
        VendorCmd::SyncDiff => vendor_sync_diff(ctx),
        VendorCmd::CacheInfo => vendor_cache_info(json),
        VendorCmd::CacheClear { cache_version } => {
            vendor_cache_clear(cache_version.as_deref(), quiet)
        }
        VendorCmd::UseLib {
            crate_name,
            dev_path,
        } => vendor_use_lib(ctx, crate_name, dev_path.as_deref(), quiet),
    }
}

/// Create vendor.tar.xz for CRAN. Runs `cargo vendor` + packages workspace crates.
fn vendor_pack(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let vendor_dir = ctx.root.join("vendor");
    let inst_dir = ctx.root.join("inst");

    if !quiet {
        eprintln!("=== CRAN vendor prep ===");
    }

    // 1. Run cargo vendor
    if !quiet {
        eprintln!("Vendoring external dependencies...");
    }
    run_command(
        "cargo",
        &[
            "vendor",
            "--manifest-path",
            &manifest.to_string_lossy(),
            &vendor_dir.to_string_lossy(),
        ],
        &ctx.root,
        quiet,
    )?;

    // 2. Copy workspace crates on top (if in monorepo)
    let workspace_root = find_workspace_root(&ctx.root);
    if let Some(ws) = &workspace_root {
        if !quiet {
            eprintln!("Copying workspace crates...");
        }
        for krate in MINIEXTENDR_CRATES {
            let src = ws.join(krate);
            let dst = vendor_dir.join(krate);
            if src.is_dir() {
                if dst.is_dir() {
                    std::fs::remove_dir_all(&dst)?;
                }
                copy_dir_recursive(&src, &dst)?;
                // Write .cargo-checksum.json
                std::fs::write(dst.join(".cargo-checksum.json"), "{\"files\":{}}")?;
                if !quiet {
                    eprintln!("  {krate}");
                }
            }
        }
    }

    // 3. Strip [[bench]], [[test]], and [dev-dependencies] from vendored Cargo.toml
    //    (cargo fails with "can't find bench at benches/foo.rs" if these sections
    //    reference files that don't exist in the vendored copy)
    strip_vendor_toml_sections(&vendor_dir)?;

    // 4. Strip checksums from Cargo.lock
    let lockfile = ctx.root.join("src/rust/Cargo.lock");
    if lockfile.exists() {
        let content = std::fs::read_to_string(&lockfile)?;
        let filtered: String = content
            .lines()
            .filter(|line| !line.starts_with("checksum = "))
            .map(|line| format!("{line}\n"))
            .collect();
        std::fs::write(&lockfile, filtered)?;
    }

    // 5. Compress into inst/vendor.tar.xz
    if !quiet {
        eprintln!("Compressing vendor.tar.xz...");
    }
    std::fs::create_dir_all(&inst_dir)?;
    let tarball = inst_dir.join("vendor.tar.xz");
    bash(
        &format!(
            "tar -cJf '{}' -C '{}' vendor",
            tarball.to_string_lossy(),
            ctx.root.to_string_lossy()
        ),
        &ctx.root,
        quiet,
    )?;

    if !quiet {
        eprintln!("=== Done: inst/vendor.tar.xz ready for CRAN ===");
    }
    Ok(())
}

/// List available miniextendr versions via GitHub API.
fn vendor_versions(quiet: bool, json: bool) -> Result<()> {
    if !crate::bridge::has_program("gh") {
        bail!("gh (GitHub CLI) not found. Install it to list versions.");
    }
    let output = run_command_capture(
        "gh",
        &[
            "api",
            "repos/CGMossa/miniextendr/branches",
            "--jq",
            ".[].name",
        ],
        Path::new("."),
    )?;

    if json {
        let versions: Vec<&str> = output.lines().collect();
        println!("{}", serde_json::to_string_pretty(&versions)?);
    } else if !quiet {
        println!("Available miniextendr versions:");
        for line in output.lines() {
            println!("  {line}");
        }
    }
    Ok(())
}

/// Copy miniextendr crates to vendor/ from local path.
fn vendor_miniextendr(ctx: &ProjectContext, local_path: Option<&str>, quiet: bool) -> Result<()> {
    let vendor_dir = ctx.root.join("vendor");
    std::fs::create_dir_all(&vendor_dir)?;

    let source = match local_path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            // Try to find monorepo root
            match find_workspace_root(&ctx.root) {
                Some(ws) => ws,
                None => bail!(
                    "No local miniextendr source found. Use --local-path or run from monorepo."
                ),
            }
        }
    };

    for krate in MINIEXTENDR_CRATES {
        let src = source.join(krate);
        let dst = vendor_dir.join(krate);
        if src.is_dir() {
            if dst.is_dir() {
                std::fs::remove_dir_all(&dst)?;
            }
            copy_dir_recursive(&src, &dst)?;
            std::fs::write(dst.join(".cargo-checksum.json"), "{\"files\":{}}")?;
            if !quiet {
                eprintln!("  Vendored {krate}");
            }
        } else if !quiet {
            eprintln!("  Skipping {krate} (not found at {})", src.display());
        }
    }

    Ok(())
}

/// Run `cargo vendor` for external crates.io deps.
fn vendor_crates_io(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let vendor_dir = ctx.root.join("vendor");
    run_command(
        "cargo",
        &[
            "vendor",
            "--manifest-path",
            &manifest.to_string_lossy(),
            &vendor_dir.to_string_lossy(),
        ],
        &ctx.root,
        quiet,
    )?;
    Ok(())
}

/// Sync vendored crates from workspace source.
fn vendor_sync(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    vendor_miniextendr(ctx, None, quiet)
}

/// Verify vendored crates match workspace sources.
fn vendor_sync_check(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let vendor_dir = ctx.root.join("vendor");
    if !vendor_dir.is_dir() {
        if !quiet {
            eprintln!("No vendor/ directory found. Run configure or vendor first.");
        }
        std::process::exit(1);
    }

    let workspace_root = find_workspace_root(&ctx.root);
    let ws = match &workspace_root {
        Some(ws) => ws,
        None => {
            if !quiet {
                eprintln!("Not in a monorepo. Cannot check vendor sync.");
            }
            return Ok(());
        }
    };

    let mut drift_found = false;

    for krate in MINIEXTENDR_CRATES {
        let vendor_crate = vendor_dir.join(krate);
        let workspace_crate = ws.join(krate);

        if !vendor_crate.is_dir() {
            if !quiet {
                eprintln!("WARNING: vendor/{krate} not found");
            }
            continue;
        }
        if !workspace_crate.is_dir() {
            continue;
        }

        let result = std::process::Command::new("diff")
            .args(["-rq"])
            .arg(workspace_crate.join("src"))
            .arg(vendor_crate.join("src"))
            .output();

        if let Ok(output) = result
            && !output.status.success()
        {
            eprintln!("DRIFT: {krate}/src differs from vendor");
            drift_found = true;
        }
    }

    if drift_found {
        eprintln!(
            "\nVendored crates have drifted. Run `miniextendr workflow configure` to refresh."
        );
        std::process::exit(1);
    } else if !quiet {
        println!("Vendor sync check passed: all miniextendr crates match.");
    }
    Ok(())
}

/// Show diff between workspace and vendored crates.
fn vendor_sync_diff(ctx: &ProjectContext) -> Result<()> {
    let vendor_dir = ctx.root.join("vendor");
    let workspace_root = find_workspace_root(&ctx.root);

    let ws = match &workspace_root {
        Some(ws) => ws,
        None => {
            eprintln!("Not in a monorepo.");
            return Ok(());
        }
    };

    for krate in MINIEXTENDR_CRATES {
        let vendor_src = vendor_dir.join(krate).join("src");
        let workspace_src = ws.join(krate).join("src");
        if vendor_src.is_dir() && workspace_src.is_dir() {
            println!("=== {krate} ===");
            let _ = run_command(
                "diff",
                &[
                    "-ruN",
                    &workspace_src.to_string_lossy(),
                    &vendor_src.to_string_lossy(),
                ],
                &ctx.root,
                false,
            );
            println!();
        }
    }
    Ok(())
}

/// Show cache info (uses rappdirs-like logic for cache directory).
fn vendor_cache_info(json: bool) -> Result<()> {
    let cache_dir = cache_directory();
    if json {
        let info = serde_json::json!({
            "cache_dir": cache_dir.to_string_lossy(),
            "exists": cache_dir.exists(),
        });
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Cache directory: {}", cache_dir.display());
        if cache_dir.exists() {
            let entries: Vec<_> = std::fs::read_dir(&cache_dir)?
                .filter_map(|e| e.ok())
                .collect();
            if entries.is_empty() {
                println!("  (empty)");
            } else {
                for entry in entries {
                    println!("  {}", entry.file_name().to_string_lossy());
                }
            }
        } else {
            println!("  (does not exist)");
        }
    }
    Ok(())
}

/// Clear cached archives.
fn vendor_cache_clear(version: Option<&str>, quiet: bool) -> Result<()> {
    let cache_dir = cache_directory();
    if !cache_dir.exists() {
        if !quiet {
            println!("Cache directory does not exist.");
        }
        return Ok(());
    }

    match version {
        Some(v) => {
            // Remove specific version
            for entry in std::fs::read_dir(&cache_dir)? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().contains(v) {
                    std::fs::remove_file(entry.path())?;
                    if !quiet {
                        println!("Removed: {}", entry.file_name().to_string_lossy());
                    }
                }
            }
        }
        None => {
            std::fs::remove_dir_all(&cache_dir)?;
            if !quiet {
                println!("Cache cleared.");
            }
        }
    }
    Ok(())
}

/// Vendor a local path dependency.
fn vendor_use_lib(
    ctx: &ProjectContext,
    crate_name: &str,
    dev_path: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let vendor_dir = ctx.root.join("vendor");
    std::fs::create_dir_all(&vendor_dir)?;

    let source = match dev_path {
        Some(p) => std::path::PathBuf::from(p),
        None => bail!("--dev-path is required for vendor use-lib"),
    };

    if !source.is_dir() {
        bail!("Source path does not exist: {}", source.display());
    }

    let dst = vendor_dir.join(crate_name);
    if dst.is_dir() {
        std::fs::remove_dir_all(&dst)?;
    }
    copy_dir_recursive(&source, &dst)?;
    std::fs::write(dst.join(".cargo-checksum.json"), "{\"files\":{}}")?;

    if !quiet {
        println!("Vendored {crate_name} from {}", source.display());
    }
    Ok(())
}

// --- Helpers ---

/// Strip `[[bench]]`, `[[test]]`, and `[dev-dependencies]` sections from
/// all `Cargo.toml` files in `vendor/`. These sections reference files
/// (benches/, tests/) that don't exist in vendored copies, causing cargo to
/// fail with "can't find bench at benches/foo.rs".
fn strip_vendor_toml_sections(vendor_dir: &Path) -> Result<()> {
    if !vendor_dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(vendor_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let cargo_toml = entry.path().join("Cargo.toml");
        if !cargo_toml.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&cargo_toml)?;
        // Check if stripping is needed
        if !content.contains("[[bench]]")
            && !content.contains("[[test]]")
            && !content.contains("[dev-dependencies]")
        {
            continue;
        }
        let stripped = strip_toml_sections(&content);
        std::fs::write(&cargo_toml, stripped)?;
    }
    Ok(())
}

/// Remove `[[bench]]`, `[[test]]`, and `[dev-dependencies]` sections from TOML text.
/// Each section runs from its header until the next TOML table/array header.
fn strip_toml_sections(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_strip_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Detect section headers
        if trimmed.starts_with("[[bench]]")
            || trimmed.starts_with("[[test]]")
            || trimmed.starts_with("[dev-dependencies]")
        {
            in_strip_section = true;
            continue;
        }

        // Any new section header ends a strip section
        if in_strip_section && (trimmed.starts_with('[') && !trimmed.is_empty()) {
            in_strip_section = false;
        }

        if !in_strip_section {
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn cache_directory() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        return std::path::PathBuf::from(xdg).join("minirextendr");
    }
    if let Ok(home) = std::env::var("HOME") {
        #[cfg(target_os = "macos")]
        return std::path::PathBuf::from(&home).join("Library/Caches/minirextendr");
        #[cfg(not(target_os = "macos"))]
        return std::path::PathBuf::from(&home).join(".cache/minirextendr");
    }
    std::path::PathBuf::from("/tmp/minirextendr-cache")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Skip target dirs, .git, etc.
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "target" || name_str == ".git" || name_str == "vendor" {
            continue;
        }

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
