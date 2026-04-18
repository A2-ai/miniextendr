//! cargo-revendor: Vendor Rust dependencies for R packages
//!
//! Unlike `cargo vendor`, this handles:
//! - Path dependencies (workspace members) via `cargo package`
//! - Workspace inheritance resolution
//! - Opt-in stripping of test/bench/example/bin directories
//! - Cleaning TOML sections that reference stripped directories
//! - Inter-crate path dependency rewriting
//! - Empty checksum generation for vendored crates
//! - Caching via Cargo.lock hash (skip re-vendoring when deps unchanged)
//! - JSON output for machine consumption

mod cache;
mod manifest_guard;
mod metadata;
mod package;
mod verify;

/// Convert a path to a TOML-safe string (forward slashes, no \\?\ prefix)
pub fn path_to_toml(path: &std::path::Path) -> String {
    let s = path.display().to_string();
    // Strip Windows extended-length path prefix (\\?\) that canonicalize() adds
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
mod strip;
mod vendor;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

/// Verbosity level (0=quiet, 1=-v, 2=-vv, 3=-vvv)
#[derive(Clone, Copy, Debug)]
pub struct Verbosity(pub u8);

impl Verbosity {
    pub fn info(self) -> bool {
        self.0 >= 1
    }
    pub fn debug(self) -> bool {
        self.0 >= 2
    }
    pub fn trace(self) -> bool {
        self.0 >= 3
    }
}

#[derive(Parser)]
#[command(
    name = "cargo-revendor",
    about = "Vendor Rust dependencies for R packages, handling workspace/path deps"
)]
struct Cli {
    /// When invoked as `cargo revendor`, cargo passes "revendor" as first arg
    #[arg(hide = true, default_value = "revendor")]
    _subcommand: String,

    /// Path to the Cargo.toml of the R package's Rust crate
    #[arg(long, default_value = "src/rust/Cargo.toml")]
    manifest_path: PathBuf,

    /// Output directory for vendored crates
    #[arg(long, short, default_value = "vendor")]
    output: PathBuf,

    /// Root of the monorepo/workspace containing path dependencies.
    /// If not set, auto-detected from workspace metadata.
    #[arg(long)]
    source_root: Option<PathBuf>,

    /// Allow dirty working directory when running cargo package
    #[arg(long, default_value_t = true)]
    allow_dirty: bool,

    /// Strip test directories from vendored crates
    #[arg(long)]
    strip_tests: bool,

    /// Strip bench directories from vendored crates
    #[arg(long)]
    strip_benches: bool,

    /// Strip example directories from vendored crates
    #[arg(long)]
    strip_examples: bool,

    /// Strip binary directories from vendored crates
    #[arg(long)]
    strip_bins: bool,

    /// Strip all non-essential directories (tests, benches, examples, bins)
    #[arg(long)]
    strip_all: bool,

    /// Output results as JSON
    #[arg(long)]
    json: bool,

    /// Increase verbosity (-v info, -vv debug, -vvv trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Force re-vendoring even if Cargo.lock hasn't changed
    #[arg(long)]
    force: bool,

    /// Compress vendor/ into a tarball (e.g., vendor.tar.xz)
    #[arg(long)]
    compress: Option<PathBuf>,

    /// Blank .md files in vendor/ before compression
    #[arg(long)]
    blank_md: bool,

    /// Freeze: rewrite Cargo.toml so all sources resolve from vendor/.
    /// Rewrites git deps to vendor/ path deps, strips [patch.*] sections,
    /// adds [patch.crates-io] for transitive local deps, and regenerates
    /// Cargo.lock offline. Makes the manifest self-contained for hermetic
    /// offline builds with no network, git, or workspace context.
    #[arg(long)]
    freeze: bool,

    /// Fail fast on any external `git = "..."` dependency that `--freeze`
    /// cannot rewrite to a vendor path. Requires `--freeze`.
    ///
    /// Without this flag, external git deps remain as `git =` entries in
    /// the frozen manifest and rely on `.cargo/config.toml` source
    /// replacement (which `cargo revendor` writes to
    /// `vendor/.cargo-config.toml`) for offline builds. With this flag,
    /// cargo-revendor exits non-zero if the frozen manifest would still
    /// contain `git =` entries — useful for CI guards that must guarantee
    /// the manifest alone is buildable offline.
    #[arg(long, requires = "freeze")]
    strict_freeze: bool,

    /// Write .vendor-source marker file recording provenance
    #[arg(long)]
    source_marker: bool,

    /// Verify-only: check Cargo.lock against the already-populated vendor/
    /// directory (and, if --compress is given, the tarball against vendor/)
    /// without re-vendoring. Exits non-zero if any drift is detected.
    ///
    /// Use in CI or pre-release checks to guarantee the committed
    /// vendor.tar.xz matches Cargo.lock.
    #[arg(long)]
    verify: bool,

    /// Additional manifests to include in the vendor graph — mirrors
    /// `cargo vendor --sync <extra.toml>`. Each path points at the
    /// `Cargo.toml` of a disjoint workspace whose dep graph should be
    /// unioned into a single shared `vendor/` tree.
    ///
    /// Use case: one R package (`rpkg/src/rust/Cargo.toml`) and a
    /// separate benchmarks workspace (`miniextendr-bench/Cargo.toml`)
    /// that want to share one offline artifact. Each --sync manifest's
    /// Cargo.lock is also checked by --verify. See #229.
    #[arg(long)]
    sync: Vec<PathBuf>,

    /// Use flat directory names (`vendor/<name>/`) for ALL vendored crates,
    /// reverting to the old cargo vendor default layout.
    ///
    /// By default (without this flag), `cargo revendor` uses versioned
    /// directory names (`vendor/<name>-<version>/`) for every crate, ensuring
    /// the layout is stable and unambiguous across regenerations.
    ///
    /// Use this flag only if you need compatibility with tools that hardcode
    /// flat vendor paths.
    #[arg(long)]
    flat_dirs: bool,
}

impl Cli {
    fn verbosity(&self) -> Verbosity {
        Verbosity(self.verbose)
    }

    fn strip_config(&self) -> strip::StripConfig {
        if self.strip_all {
            strip::StripConfig::all()
        } else {
            strip::StripConfig {
                tests: self.strip_tests,
                benches: self.strip_benches,
                examples: self.strip_examples,
                bins: self.strip_bins,
            }
        }
    }
}

/// JSON output structure
#[derive(serde::Serialize)]
struct JsonOutput {
    vendor_dir: String,
    local_crates: Vec<String>,
    external_crates: usize,
    total_crates: usize,
    cached: bool,
    stripped: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let v = cli.verbosity();
    // versioned_dirs is default-on; only disable when --flat-dirs is passed
    let versioned_dirs = !cli.flat_dirs;

    let manifest_path = cli
        .manifest_path
        .canonicalize()
        .with_context(|| format!("manifest not found: {}", cli.manifest_path.display()))?;

    if v.info() && !cli.verify {
        eprintln!(
            "cargo-revendor: vendoring deps from {}",
            manifest_path.display()
        );
    }

    // Resolve output path (relative to CWD)
    let output = if cli.output.is_absolute() {
        cli.output.clone()
    } else {
        std::env::current_dir()?.join(&cli.output)
    };

    let lockfile = manifest_path.with_file_name("Cargo.lock");

    // Canonicalize each --sync manifest path once. Used by both the verify
    // shortcut above and the vendor flow below (plus cache hashing).
    let sync_manifests: Vec<std::path::PathBuf> = cli
        .sync
        .iter()
        .map(|p| {
            p.canonicalize()
                .unwrap_or_else(|_| std::env::current_dir().unwrap().join(p))
        })
        .collect();

    // Verify-only: don't vendor; just assert existing artifacts are in sync.
    if cli.verify {
        let sync_lockfiles: Vec<std::path::PathBuf> = sync_manifests
            .iter()
            .map(|m| m.with_file_name("Cargo.lock"))
            .collect();
        return run_verify(
            &lockfile,
            &sync_lockfiles,
            &output,
            cli.compress.as_deref(),
            v,
        );
    }

    // Step 1: Load cargo metadata to discover dependencies
    let meta = metadata::load_metadata(&manifest_path)?;

    // Mirror upstream cargo's duplicate-source check: error out if two
    // different git sources resolve to the same crate name+version. Without
    // this, cargo-revendor silently last-write-wins during extraction, so
    // the vendored contents depend on dep-graph iteration order.
    metadata::check_duplicate_sources(&meta)?;

    let (mut local_pkgs, _external_pkgs) = metadata::partition_packages(&meta, &manifest_path)?;

    // Also discover ALL workspace members from the source workspace root
    let all_workspace_members = if let Some(ref source_root) = cli.source_root {
        metadata::discover_workspace_members(source_root)?
    } else if let Some(first_local) = local_pkgs.first() {
        let ws_root = find_workspace_root(&first_local.path)?;
        metadata::discover_workspace_members(&ws_root)?
    } else {
        Vec::new()
    };

    // Fix paths in local_pkgs: when .cargo/config.toml has source replacement
    // (e.g., [source.vendored-sources] directory = "vendor"), cargo metadata
    // resolves local workspace crate paths to the vendor directory instead of the
    // real workspace source. Detect this and replace with the real workspace path.
    let canonical_output = output.canonicalize().unwrap_or_else(|_| output.clone());
    for pkg in &mut local_pkgs {
        let canonical_pkg = pkg.path.canonicalize().unwrap_or_else(|_| pkg.path.clone());
        if canonical_pkg.starts_with(&canonical_output) {
            // This path is inside the output vendor directory — find the real source
            if let Some(ws_pkg) = all_workspace_members.iter().find(|ws| ws.name == pkg.name) {
                if v.debug() {
                    eprintln!(
                        "  Fixed {}: {} -> {}",
                        pkg.name,
                        pkg.path.display(),
                        ws_pkg.path.display()
                    );
                }
                pkg.path = ws_pkg.path.clone();
                pkg.manifest_path = ws_pkg.manifest_path.clone();
            }
        }
    }

    let patch_pkgs = merge_packages(&local_pkgs, &all_workspace_members);

    if v.info() {
        eprintln!("  Local packages to vendor: {}", local_pkgs.len());
        for pkg in &local_pkgs {
            eprintln!(
                "    - {} v{} ({})",
                pkg.name,
                pkg.version,
                pkg.path.display()
            );
        }
        if v.debug() && patch_pkgs.len() > local_pkgs.len() {
            eprintln!(
                "  Additional workspace members for patching: {}",
                patch_pkgs.len() - local_pkgs.len()
            );
        }
    }

    // Local crate source trees participate in the cache key — pure source
    // edits to workspace crates leave Cargo.lock untouched (#150), so hashing
    // only the lockfile would silently serve a stale vendor/ copy.
    let local_crate_paths: Vec<std::path::PathBuf> =
        local_pkgs.iter().map(|p| p.path.clone()).collect();

    // Step 0: Check cache — skip if all inputs are unchanged
    if !cli.force && cache::is_cached(&lockfile, &sync_manifests, &output, &local_crate_paths)? {
        if v.info() {
            eprintln!("cargo-revendor: vendor/ is up to date (inputs unchanged)");
        }
        if cli.json {
            let count = std::fs::read_dir(&output)
                .map(|d| {
                    d.filter_map(|e| e.ok())
                        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                        .count()
                })
                .unwrap_or(0);
            let json = JsonOutput {
                vendor_dir: output.display().to_string(),
                local_crates: local_pkgs.iter().map(|p| p.name.clone()).collect(),
                external_crates: count.saturating_sub(local_pkgs.len()),
                total_crates: count,
                cached: true,
                stripped: vec![],
            };
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        return Ok(());
    }

    // Step 2: Package local crates via `cargo package`
    let staging = tempfile::tempdir().context("failed to create staging dir")?;
    let vendor_staging = staging.path().join("vendor");

    let packaged = package::package_local_crates(
        &local_pkgs,
        &patch_pkgs,
        &manifest_path,
        staging.path(),
        cli.allow_dirty,
        v,
    )?;

    // Step 3: Run `cargo vendor` for external deps
    vendor::run_cargo_vendor(
        &manifest_path,
        &vendor_staging,
        &patch_pkgs,
        &sync_manifests,
        versioned_dirs,
        v,
    )?;

    // Step 4: Extract packaged local crates into vendor staging
    for (pkg_name, crate_path) in &packaged {
        let pkg_version = if versioned_dirs {
            local_pkgs.iter().find(|p| &p.name == pkg_name).map(|p| p.version.as_str())
        } else {
            None
        };
        vendor::extract_crate_archive(crate_path, &vendor_staging, pkg_name, pkg_version, v)?;
    }

    // Step 5: Strip directories (opt-in)
    let strip_cfg = cli.strip_config();
    let stripped = if strip_cfg.any() {
        strip::strip_vendor_dir(&vendor_staging, &strip_cfg, v)?
    } else {
        vec![]
    };

    // Step 5.5: Strip relative path deps from all vendored crates
    // (cargo vendor preserves intra-workspace path deps from git sources;
    // these conflict with source replacement during offline builds)
    vendor::strip_vendor_path_deps(&vendor_staging, v)?;

    // Step 6: Rewrite inter-crate path deps for local crates
    vendor::rewrite_local_path_deps(&vendor_staging, &local_pkgs, versioned_dirs, v)?;

    // Step 7: Clear checksums
    vendor::clear_checksums(&vendor_staging)?;

    // Step 8: Move to final output directory
    if output.exists() {
        std::fs::remove_dir_all(&output)
            .with_context(|| format!("failed to remove existing {}", output.display()))?;
    }
    std::fs::rename(&vendor_staging, &output)
        .or_else(|_| copy_dir_recursive(&vendor_staging, &output))
        .with_context(|| format!("failed to move vendor to {}", output.display()))?;

    // Step 9: Generate .cargo/config.toml for source replacement
    let config_toml = vendor::generate_cargo_config(&manifest_path, &output, &local_pkgs)?;
    if v.info() {
        eprintln!("  Generated .cargo/config.toml for source replacement");
    }
    if v.debug() {
        eprintln!("{}", config_toml);
    }

    // Step 10: Strip checksums from Cargo.lock (vendored crates have empty checksums)
    vendor::strip_lock_checksums(&lockfile, &output, v)?;

    // Step 11: Write source marker
    if cli.source_marker {
        let source_info = cli
            .source_root
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "auto-detected".to_string());
        std::fs::write(output.join(".vendor-source"), &source_info)?;
        if v.info() {
            eprintln!("  Wrote .vendor-source marker: {}", source_info);
        }
    }

    // Step 12: Freeze — rewrite manifest so all sources resolve from vendor/
    if cli.freeze {
        vendor::freeze_manifest(
            &manifest_path,
            &output,
            &local_pkgs,
            versioned_dirs,
            cli.strict_freeze,
            v,
        )?;
        vendor::regenerate_lockfile(&manifest_path, &output, v)?;
    }

    // Step 13: Compress to tarball (relative paths resolve from CWD)
    if let Some(ref tarball_path) = cli.compress {
        let tarball = if tarball_path.is_absolute() {
            tarball_path.clone()
        } else {
            std::env::current_dir()?.join(tarball_path)
        };
        vendor::compress_vendor(&output, &tarball, cli.blank_md, v)?;
    }

    // Step 14: Save cache
    cache::save_cache(&lockfile, &sync_manifests, &output, &local_crate_paths)?;

    // Count total crates
    let total = std::fs::read_dir(&output)
        .map(|d| {
            d.filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    if cli.json {
        let json = JsonOutput {
            vendor_dir: output.display().to_string(),
            local_crates: packaged.iter().map(|(n, _)| n.clone()).collect(),
            external_crates: total - packaged.len(),
            total_crates: total,
            cached: false,
            stripped,
        };
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else if v.info() {
        eprintln!(
            "cargo-revendor: vendored {} local + {} external deps to {}",
            packaged.len(),
            total - packaged.len(),
            output.display()
        );
    }

    Ok(())
}

/// Verify that Cargo.lock, vendor/, and (optionally) the tarball agree.
fn run_verify(
    lockfile: &std::path::Path,
    sync_lockfiles: &[std::path::PathBuf],
    vendor_dir: &std::path::Path,
    tarball: Option<&std::path::Path>,
    v: Verbosity,
) -> Result<()> {
    if v.info() {
        eprintln!("cargo-revendor: verifying Cargo.lock ↔ {}", vendor_dir.display());
    }
    verify::verify_lock_matches_vendor(lockfile, vendor_dir)?;
    if v.info() {
        eprintln!("  Cargo.lock ↔ vendor/: OK");
    }

    // Every --sync manifest carries its own Cargo.lock; each must agree with
    // the shared vendor/ too.
    for sync_lock in sync_lockfiles {
        if v.info() {
            eprintln!(
                "cargo-revendor: verifying {} ↔ {}",
                sync_lock.display(),
                vendor_dir.display()
            );
        }
        verify::verify_lock_matches_vendor(sync_lock, vendor_dir)?;
        if v.info() {
            eprintln!("  {} ↔ vendor/: OK", sync_lock.display());
        }
    }

    if let Some(tarball) = tarball {
        let tarball_abs = if tarball.is_absolute() {
            tarball.to_path_buf()
        } else {
            std::env::current_dir()?.join(tarball)
        };
        if v.info() {
            eprintln!(
                "cargo-revendor: verifying {} ↔ {}",
                tarball_abs.display(),
                vendor_dir.display()
            );
        }
        verify::verify_tarball_matches_vendor(&tarball_abs, vendor_dir)?;
        if v.info() {
            eprintln!("  tarball ↔ vendor/: OK");
        }
    }

    Ok(())
}

/// Find workspace root by walking up from a directory
pub fn find_workspace_root(dir: &std::path::Path) -> Result<std::path::PathBuf> {
    let mut dir = dir.canonicalize()?;
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            anyhow::bail!("no workspace root found");
        }
    }
}

/// Merge two package lists, deduplicating by name
fn merge_packages(
    a: &[metadata::LocalPackage],
    b: &[metadata::LocalPackage],
) -> Vec<metadata::LocalPackage> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for pkg in a.iter().chain(b.iter()) {
        if seen.insert(pkg.name.clone()) {
            result.push(pkg.clone());
        }
    }
    result
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(src).unwrap();
        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
