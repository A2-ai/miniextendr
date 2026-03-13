use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::bridge::{run_command, run_command_capture};
use crate::cli::VendorCmd;
use crate::project::{MINIEXTENDR_CRATES, ProjectContext, find_workspace_root};

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    resolve: Option<MetadataResolve>,
    workspace_members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataResolve {
    root: Option<String>,
    nodes: Vec<MetadataNode>,
}

#[derive(Debug, Deserialize)]
struct MetadataNode {
    id: String,
    dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: PathBuf,
    dependencies: Vec<MetadataDependency>,
}

#[derive(Debug, Deserialize)]
struct MetadataDependency {
    name: String,
    source: Option<String>,
    path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct InternalPackage {
    name: String,
    version: String,
    manifest_path: PathBuf,
}

#[derive(Debug)]
struct PackagedCrate {
    name: String,
    version: String,
    crate_file: PathBuf,
}

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

/// Create vendor.tar.xz for CRAN.
///
/// External crates are vendored via `cargo vendor`. Local path/workspace crates are
/// turned into `.crate` archives via `cargo package --no-verify`, using a generated
/// cargo config with `[patch.crates-io]` entries for sibling local crates.
fn vendor_pack(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let vendor_dir = ctx.root.join("vendor");
    let inst_dir = ctx.root.join("inst");
    let staging_dir = ctx.root.join(".vendor-tarball-staging");
    let compress_staging = ctx.root.join(".vendor-compress-staging");

    if !quiet {
        eprintln!("=== CRAN vendor prep ===");
    }

    remove_dir_if_exists(&staging_dir)?;
    remove_dir_if_exists(&compress_staging)?;
    remove_dir_if_exists(&vendor_dir)?;
    std::fs::create_dir_all(&staging_dir)?;

    let internal_packages = discover_internal_packages(manifest)?;
    let packaged_crates =
        package_internal_crates(&ctx.root, &staging_dir, &internal_packages, quiet)?;

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

    extract_packaged_crates(&vendor_dir, &packaged_crates, quiet)?;

    strip_vendor_toml_sections(&vendor_dir)?;
    strip_lockfile_checksums(&ctx.root.join("src/rust/Cargo.lock"))?;

    if !quiet {
        eprintln!("Compressing vendor.tar.xz...");
    }
    std::fs::create_dir_all(&inst_dir)?;
    compress_vendor_tree(
        &vendor_dir,
        &compress_staging,
        &inst_dir.join("vendor.tar.xz"),
        quiet,
    )?;

    remove_dir_if_exists(&staging_dir)?;
    remove_dir_if_exists(&compress_staging)?;

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

fn discover_internal_packages(manifest: &Path) -> Result<Vec<InternalPackage>> {
    let metadata = cargo_metadata(manifest)?;
    let resolve = metadata
        .resolve
        .context("cargo metadata did not return a resolve graph")?;
    let root_id = resolve
        .root
        .or_else(|| metadata.workspace_members.first().cloned())
        .context("cargo metadata did not identify a root package")?;

    let node_map: HashMap<_, _> = resolve
        .nodes
        .into_iter()
        .map(|node| (node.id, node.dependencies))
        .collect();
    let local_name_to_id: HashMap<_, _> = metadata
        .packages
        .iter()
        .filter(|pkg| pkg.source.is_none())
        .map(|pkg| (pkg.name.clone(), pkg.id.clone()))
        .collect();
    let package_map: HashMap<_, _> = metadata
        .packages
        .into_iter()
        .map(|pkg| (pkg.id.clone(), pkg))
        .collect();

    let mut visited = HashSet::new();
    let mut stack = vec![root_id.clone()];
    let mut internal = Vec::new();

    while let Some(id) = stack.pop() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if let Some(deps) = node_map.get(&id) {
            for dep in deps.iter().rev() {
                stack.push(dep.clone());
            }
        }

        let package = package_map
            .get(&id)
            .with_context(|| format!("cargo metadata missing package entry for {id}"))?;
        for dep in &package.dependencies {
            if dep.source.is_none()
                && dep.path.is_some()
                && let Some(dep_id) = local_name_to_id.get(&dep.name)
            {
                stack.push(dep_id.clone());
            }
        }

        if id == root_id {
            continue;
        }

        if package.source.is_none() {
            internal.push(InternalPackage {
                name: package.name.clone(),
                version: package.version.clone(),
                manifest_path: package.manifest_path.clone(),
            });
        }
    }

    internal.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));

    let mut seen = HashSet::new();
    for pkg in &internal {
        if !seen.insert(pkg.name.clone()) {
            bail!(
                "Multiple local packages named '{}' were found in the dependency graph; \
                 generated [patch.crates-io] config would be ambiguous.",
                pkg.name
            );
        }
    }

    Ok(internal)
}

fn cargo_metadata(manifest: &Path) -> Result<CargoMetadata> {
    let output = run_command_capture(
        "cargo",
        &[
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            &manifest.to_string_lossy(),
        ],
        manifest.parent().unwrap_or_else(|| Path::new(".")),
    )?;
    serde_json::from_str(&output).context("failed to parse cargo metadata JSON")
}

fn package_internal_crates(
    project_root: &Path,
    staging_dir: &Path,
    internal_packages: &[InternalPackage],
    quiet: bool,
) -> Result<Vec<PackagedCrate>> {
    if internal_packages.is_empty() {
        return Ok(Vec::new());
    }

    if !quiet {
        eprintln!("Packaging local crates...");
    }

    let target_dir = staging_dir.join("target");
    let config_dir = staging_dir.join("package-config");
    std::fs::create_dir_all(&target_dir)?;
    std::fs::create_dir_all(&config_dir)?;

    let mut packaged = Vec::with_capacity(internal_packages.len());
    for pkg in internal_packages {
        if !quiet {
            eprintln!("  {}", pkg.name);
        }
        let config_path = write_package_config(&config_dir, pkg)?;
        run_cargo_package(project_root, pkg, &config_path, &target_dir, quiet)?;
        let crate_file = target_dir
            .join("package")
            .join(format!("{}-{}.crate", pkg.name, pkg.version));
        if !crate_file.is_file() {
            bail!(
                "cargo package did not produce expected archive {}",
                crate_file.display()
            );
        }
        packaged.push(PackagedCrate {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            crate_file,
        });
    }

    Ok(packaged)
}

fn write_package_config(config_dir: &Path, current: &InternalPackage) -> Result<PathBuf> {
    let mut content = String::new();
    let mut wrote_header = false;

    for pkg in discover_internal_packages(&current.manifest_path)? {
        if pkg.name == current.name {
            continue;
        }

        let crate_dir = pkg.manifest_path.parent().with_context(|| {
            format!(
                "manifest path has no parent: {}",
                pkg.manifest_path.display()
            )
        })?;
        if !wrote_header {
            content.push_str("[patch.crates-io]\n");
            wrote_header = true;
        }
        let quoted_path = toml::Value::String(crate_dir.to_string_lossy().into_owned()).to_string();
        content.push_str(&format!("{} = {{ path = {} }}\n", pkg.name, quoted_path));
    }

    let config_path = config_dir.join(format!("{}.toml", current.name));
    std::fs::write(&config_path, content)?;
    Ok(config_path)
}

fn run_cargo_package(
    project_root: &Path,
    pkg: &InternalPackage,
    config_path: &Path,
    target_dir: &Path,
    quiet: bool,
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("package")
        .arg("--manifest-path")
        .arg(&pkg.manifest_path)
        .arg("--target-dir")
        .arg(target_dir)
        .arg("--allow-dirty")
        .arg("--no-verify")
        .arg("--config")
        .arg(config_path)
        .current_dir(project_root);

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to run cargo package for {}", pkg.name))?;
    if !status.success() {
        bail!(
            "cargo package failed for {} with status {}",
            pkg.name,
            status.code().unwrap_or(-1)
        );
    }
    Ok(())
}

fn extract_packaged_crates(
    vendor_dir: &Path,
    packaged_crates: &[PackagedCrate],
    quiet: bool,
) -> Result<()> {
    if packaged_crates.is_empty() {
        return Ok(());
    }

    if !quiet {
        eprintln!("Extracting packaged local crates...");
    }

    for pkg in packaged_crates {
        let dest = vendor_dir.join(format!("{}-{}", pkg.name, pkg.version));
        remove_dir_if_exists(&dest)?;
        std::fs::create_dir_all(&dest)?;

        let mut cmd = Command::new("tar");
        if cfg!(windows) {
            cmd.arg("--force-local");
        }
        cmd.arg("-xzf")
            .arg(&pkg.crate_file)
            .arg("--strip-components=1")
            .arg("-C")
            .arg(&dest);

        if quiet {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        } else {
            cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        }

        let status = cmd
            .status()
            .with_context(|| format!("failed to extract {}", pkg.crate_file.display()))?;
        if !status.success() {
            bail!(
                "tar failed while extracting {} with status {}",
                pkg.crate_file.display(),
                status.code().unwrap_or(-1)
            );
        }

        std::fs::write(dest.join(".cargo-checksum.json"), "{\"files\":{}}")?;
    }

    Ok(())
}

fn strip_lockfile_checksums(lockfile: &Path) -> Result<()> {
    if !lockfile.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(lockfile)?;
    let filtered: String = content
        .lines()
        .filter(|line| !line.starts_with("checksum = "))
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(lockfile, filtered)?;
    Ok(())
}

fn compress_vendor_tree(
    vendor_dir: &Path,
    staging_dir: &Path,
    tarball: &Path,
    quiet: bool,
) -> Result<()> {
    remove_dir_if_exists(staging_dir)?;
    let staging_vendor = staging_dir.join("vendor");
    copy_dir_recursive(vendor_dir, &staging_vendor)?;
    prepare_vendor_tree_for_tarball(&staging_vendor)?;

    let parent = staging_dir
        .parent()
        .with_context(|| format!("staging dir has no parent: {}", staging_dir.display()))?;
    let staging_name = staging_dir
        .file_name()
        .with_context(|| format!("staging dir has no name: {}", staging_dir.display()))?;

    let relative_vendor = PathBuf::from(staging_name).join("vendor");

    let mut cmd = Command::new("tar");
    if cfg!(windows) {
        cmd.arg("--force-local");
    }
    cmd.arg("-cJf")
        .arg(tarball)
        .arg("-C")
        .arg(parent)
        .arg(relative_vendor);

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    }

    let status = cmd
        .status()
        .with_context(|| format!("failed to create {}", tarball.display()))?;
    if !status.success() {
        bail!(
            "tar failed while creating {} with status {}",
            tarball.display(),
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}

fn prepare_vendor_tree_for_tarball(vendor_root: &Path) -> Result<()> {
    reset_vendor_checksums(vendor_root)?;
    prune_vendor_tree(vendor_root)
}

fn reset_vendor_checksums(vendor_root: &Path) -> Result<()> {
    if !vendor_root.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(vendor_root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            std::fs::write(entry.path().join(".cargo-checksum.json"), "{\"files\":{}}")?;
        }
    }

    Ok(())
}

fn prune_vendor_tree(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            if matches!(
                name.as_ref(),
                "tests" | "benches" | "examples" | ".github" | "docs"
            ) {
                std::fs::remove_dir_all(&path)?;
                continue;
            }
            prune_vendor_tree(&path)?;
        } else if file_type.is_file() && name.ends_with(".md") {
            std::fs::write(&path, "")?;
        }
    }

    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

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
