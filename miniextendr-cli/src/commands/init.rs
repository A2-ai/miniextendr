//! `miniextendr init` — scaffold packages from the canonical minirextendr
//! templates.
//!
//! Every file comes from the embedded copy of `minirextendr/inst/templates/`
//! (see [`crate::scaffold`]), so a CLI-generated package uses exactly the
//! same build system as `minirextendr::create_miniextendr_package()` /
//! `create_miniextendr_monorepo()` / `use_miniextendr()`: the two-mode
//! install latch keyed on `inst/vendor.tar.xz` presence, configure-written
//! `.cargo/config.toml`, wrapper + wasm-registry generation, and no `just`
//! requirement (#1351).

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::bridge::{has_program, run_command};
use crate::cli::InitCmd;
use crate::project::{ProjectContext, parse_description_field};
use crate::scaffold::{self, MONOREPO_ROOT_PLAN, RPKG_PLAN, RUST_SYSTEM_REQUIREMENT, TemplateData};

pub fn dispatch(cmd: &InitCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        InitCmd::Package { dest } => init_package(dest, quiet),
        InitCmd::Monorepo {
            dest,
            package,
            crate_name,
            rpkg_name,
            local_path: _,
            miniextendr_version: _,
        } => init_monorepo(
            dest,
            package.as_deref(),
            crate_name.as_deref(),
            rpkg_name.as_deref(),
            quiet,
        ),
        InitCmd::Use {
            crate_name,
            template_type,
            rpkg_name,
            miniextendr_version: _,
            local_path: _,
        } => init_use(
            ctx,
            template_type,
            rpkg_name.as_deref(),
            crate_name.as_deref(),
            quiet,
        ),
    }
}

// region: init package

/// Create a new standalone R package
/// (~ `minirextendr::create_miniextendr_package()`).
fn init_package(path: &str, quiet: bool) -> Result<()> {
    let root = Path::new(path);
    if root.exists() {
        bail!("Directory already exists: {path}");
    }

    let pkg_name = package_name_from_path(root)?;
    validate_package_name(&pkg_name)?;

    if !quiet {
        eprintln!("Creating R package: {pkg_name}");
    }

    let data = TemplateData::new(&pkg_name);
    scaffold_rpkg_fresh(root, &data)?;
    run_autoconf(root, quiet);

    if !quiet {
        eprintln!("\nPackage created at: {path}");
        eprintln!("Next steps:");
        eprintln!("  cd {path}");
        eprintln!("  # edit src/rust/lib.rs, then:");
        eprintln!("  miniextendr workflow build");
    }
    Ok(())
}

// endregion

// region: init monorepo

/// Create a Rust workspace with an embedded R package
/// (~ `minirextendr::create_miniextendr_monorepo()`).
fn init_monorepo(
    path: &str,
    package: Option<&str>,
    crate_name: Option<&str>,
    rpkg_name: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let root = Path::new(path);
    if root.exists() {
        bail!("Directory already exists: {path}");
    }

    // Defaults mirror create_miniextendr_monorepo(): package from the
    // directory name, crate name = package with dots as dashes, R package
    // subdirectory named after the package.
    let pkg_name = match package {
        Some(p) => p.to_string(),
        None => package_name_from_path(root)?,
    };
    validate_package_name(&pkg_name)?;
    let crate_name = crate_name
        .map(str::to_string)
        .unwrap_or_else(|| pkg_name.replace('.', "-"));
    let rpkg_name = rpkg_name.map(str::to_string).unwrap_or_else(|| {
        // rpkg_name defaults to the package name; when that collides with the
        // crate directory (dot-less package names), fall back to "rpkg"
        // instead of erroring like R does — same layout the templates
        // document, one less required flag.
        if pkg_name == crate_name {
            "rpkg".to_string()
        } else {
            pkg_name.clone()
        }
    });
    if rpkg_name == crate_name {
        bail!(
            "--rpkg-name and --crate-name must be different (both are \"{crate_name}\" — \
             they are sibling directories under the project root).\n\
             Pass e.g. --rpkg-name {crate_name}-rpkg"
        );
    }

    if !quiet {
        eprintln!("Creating monorepo: {}", root.display());
        eprintln!("  package:   {pkg_name}");
        eprintln!("  crate:     {crate_name}/");
        eprintln!("  R package: {rpkg_name}/");
    }

    let data = TemplateData::new(&pkg_name)
        .with_crate(&crate_name, &format!("../../../{crate_name}"), true)
        .with_rpkg(&rpkg_name);

    // Workspace root: Cargo.toml, .gitignore, tools/bump-version.R, core crate.
    scaffold::apply_plan(root, "templates/monorepo", MONOREPO_ROOT_PLAN, &data, true)?;

    // Embedded R package (~ create_rpkg_subdirectory()).
    let rpkg_root = root.join(&rpkg_name);
    scaffold_rpkg_at(&rpkg_root, "templates/monorepo/rpkg", &data)?;
    run_autoconf(&rpkg_root, quiet);

    if !quiet {
        eprintln!("\nMonorepo created at: {path}");
        eprintln!("Next steps:");
        eprintln!("  1. Edit {crate_name}/src/lib.rs for your core Rust library");
        eprintln!("  2. Edit {rpkg_name}/src/rust/lib.rs for R-exposed functions");
        eprintln!("  3. cd {path}/{rpkg_name} && miniextendr workflow build");
    }
    Ok(())
}

// endregion

// region: init use

/// Add miniextendr scaffolding to an existing project
/// (~ `minirextendr::use_miniextendr()`), auto-detecting standalone-package
/// vs monorepo layout.
fn init_use(
    ctx: &ProjectContext,
    template_type: &str,
    rpkg_name: Option<&str>,
    crate_name: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let root = &ctx.root;

    let template_type = match template_type {
        "auto" => {
            // Mirror detect_project_type(): a Cargo.toml at the project root
            // means "Rust project, embed an R package"; a DESCRIPTION means
            // "standalone R package".
            if root.join("Cargo.toml").is_file() {
                "monorepo"
            } else if root.join("DESCRIPTION").is_file() {
                "rpkg"
            } else {
                bail!(
                    "Could not detect project type: no Cargo.toml or DESCRIPTION in {}.\n\
                     Use `miniextendr init package <path>` to create a new package, or pass\n\
                     --template-type rpkg|monorepo explicitly.",
                    root.display()
                );
            }
        }
        t @ ("rpkg" | "monorepo") => t,
        other => bail!("Unknown template type: {other} (expected auto, rpkg, or monorepo)"),
    };

    match template_type {
        "monorepo" => init_use_monorepo(root, rpkg_name, crate_name, quiet),
        _ => init_use_rpkg(root, quiet),
    }
}

/// `init use` in a Rust workspace: create the R package subdirectory.
fn init_use_monorepo(
    root: &Path,
    rpkg_name: Option<&str>,
    crate_name: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let core = select_workspace_library(root, crate_name)?;
    let pkg_name = core.name.replace(['-', '_'], ".");
    let rpkg_name = rpkg_name.unwrap_or("rpkg");
    let core_dir = core.manifest_path.parent().unwrap();
    let rust_dir = root.canonicalize()?.join(rpkg_name).join("src/rust");
    let crate_path = relative_path(core_dir, &rust_dir)?;

    if !quiet {
        eprintln!("Detected Rust project — creating R package in {rpkg_name}/");
        eprintln!("Using package name: {pkg_name}");
    }

    let data = TemplateData::new(&pkg_name)
        .with_crate(&core.name, &crate_path, false)
        .with_rpkg(rpkg_name);
    let rpkg_root = root.join(rpkg_name);
    scaffold_rpkg_at(&rpkg_root, "templates/monorepo/rpkg", &data)?;
    scaffold::apply_plan(
        root,
        "templates/monorepo",
        &[scaffold::GITATTRIBUTES],
        &data,
        false,
    )?;
    run_autoconf(&rpkg_root, quiet);

    if !quiet {
        eprintln!("\nminiextendr scaffolding added.");
        eprintln!("Next: cd {rpkg_name} && miniextendr workflow build");
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct WorkspaceLibrary {
    id: String,
    name: String,
    manifest_path: std::path::PathBuf,
    targets: Vec<LibraryTarget>,
}

#[derive(serde::Deserialize)]
struct LibraryTarget {
    crate_types: Vec<String>,
}

fn select_workspace_library(root: &Path, name: Option<&str>) -> Result<WorkspaceLibrary> {
    #[derive(serde::Deserialize)]
    struct Metadata {
        packages: Vec<WorkspaceLibrary>,
        workspace_members: Vec<String>,
    }
    let manifest = root.join("Cargo.toml").canonicalize()?;
    let output = std::process::Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .context("failed to run cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
    let mut packages: Vec<_> = metadata
        .packages
        .into_iter()
        .filter(|pkg| {
            metadata.workspace_members.contains(&pkg.id)
                && pkg.targets.iter().any(|target| {
                    target
                        .crate_types
                        .iter()
                        .any(|kind| matches!(kind.as_str(), "lib" | "rlib"))
                })
        })
        .collect();
    for package in &mut packages {
        package.manifest_path = package.manifest_path.canonicalize()?;
    }
    let available = packages
        .iter()
        .map(|pkg| pkg.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let selected = match name {
        Some(name) => packages.iter().position(|pkg| pkg.name == name),
        None => packages
            .iter()
            .position(|pkg| pkg.manifest_path == manifest)
            .or_else(|| (packages.len() == 1).then_some(0)),
    };
    let selected = selected.with_context(|| format!(
        "Select a workspace library with --crate-name <package-name>. Available libraries: {available}"
    ))?;
    Ok(packages.swap_remove(selected))
}

fn relative_path(target: &Path, base: &Path) -> Result<String> {
    let target = std::path::absolute(target)?;
    let base = std::path::absolute(base)?;
    let target: Vec<_> = target.components().collect();
    let base: Vec<_> = base.components().collect();
    let common = target.iter().zip(&base).take_while(|(a, b)| a == b).count();
    if common == 0 {
        bail!("The R package and Rust library must be on the same filesystem root");
    }
    let mut relative = std::path::PathBuf::new();
    for _ in common..base.len() {
        relative.push("..");
    }
    for component in &target[common..] {
        relative.push(component.as_os_str());
    }
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

/// `init use` in an R package directory: add scaffolding in place.
fn init_use_rpkg(root: &Path, quiet: bool) -> Result<()> {
    let desc_path = root.join("DESCRIPTION");
    let pkg_name = if desc_path.is_file() {
        let content = std::fs::read_to_string(&desc_path)?;
        parse_description_field(&content, "Package")
            .context("Could not read Package field from DESCRIPTION")?
    } else {
        // ~ use_miniextendr_description(): seed a minimal DESCRIPTION when
        // none exists, deriving the package name from the directory.
        let pkg_name = package_name_from_path(root)?;
        validate_package_name(&pkg_name)?;
        std::fs::write(&desc_path, scaffold::description_content(&pkg_name))?;
        if !quiet {
            eprintln!("No DESCRIPTION found; created a minimal one");
        }
        pkg_name
    };

    if !quiet {
        eprintln!("Adding miniextendr to: {pkg_name}");
    }

    update_description(root)?;
    let version = parse_description_field(&std::fs::read_to_string(&desc_path)?, "Version");

    // LICENSE if missing (required by License: MIT + file LICENSE).
    let license_path = root.join("LICENSE");
    if !license_path.is_file() {
        std::fs::write(&license_path, scaffold::license_content(&pkg_name))?;
    }

    // Minimal NAMESPACE if missing (~ use_miniextendr_namespace()): without
    // it the first build fails on the configure-time guard.
    let namespace_path = root.join("NAMESPACE");
    if !namespace_path.is_file() {
        std::fs::write(&namespace_path, scaffold::minimal_namespace(&pkg_name))?;
        if !quiet {
            eprintln!(
                "No NAMESPACE found; created a minimal one with useDynLib({pkg_name}, .registration = TRUE)"
            );
        }
    }

    let data = TemplateData::new(&pkg_name)
        .with_version(version.as_deref().unwrap_or(scaffold::FRESH_VERSION));
    scaffold::apply_plan(root, "templates/rpkg", RPKG_PLAN, &data, false)?;
    scaffold::write_config_scripts(root)?;
    std::fs::create_dir_all(root.join("vendor"))?;

    run_autoconf(root, quiet);

    if !quiet {
        eprintln!("\nminiextendr scaffolding added.");
        eprintln!("Next: edit src/rust/lib.rs, then run: miniextendr workflow build");
    }
    Ok(())
}

// endregion

// region: shared scaffold steps

/// Write the full fresh standalone-package surface into `root`.
fn scaffold_rpkg_fresh(root: &Path, data: &TemplateData) -> Result<()> {
    scaffold_rpkg_at(root, "templates/rpkg", data)
}

/// Write the fresh R-package scaffold surface: the canonical DESCRIPTION
/// literal, LICENSE, NAMESPACE, the template plan, autoconf helper scripts,
/// and the `vendor/` directory (~ `create_rpkg_subdirectory()`).
fn scaffold_rpkg_at(root: &Path, prefix: &str, data: &TemplateData) -> Result<()> {
    scaffold::write_file(
        &root.join("DESCRIPTION"),
        &scaffold::description_content(&data.package),
        false,
    )?;
    scaffold::write_file(
        &root.join("LICENSE"),
        &scaffold::license_content(&data.package),
        false,
    )?;
    scaffold::write_file(
        &root.join("NAMESPACE"),
        &scaffold::minimal_namespace(&data.package),
        false,
    )?;
    scaffold::apply_plan(root, prefix, RPKG_PLAN, data, true)?;
    scaffold::write_config_scripts(root)?;
    std::fs::create_dir_all(root.join("vendor"))?;
    Ok(())
}

/// Merge the miniextendr DESCRIPTION fields into an existing file
/// (~ `use_miniextendr_description()`).
fn update_description(root: &Path) -> Result<()> {
    let desc_path = root.join("DESCRIPTION");
    let mut content = std::fs::read_to_string(&desc_path)?;

    for (field, value) in scaffold::CONFIG_BUILD_FIELDS {
        content = scaffold::desc_set_field(&content, field, value);
    }

    // Depends: merge the miniextendr R version floor (raise a lower declared
    // floor, keep a higher one) — this path runs against pre-existing
    // DESCRIPTIONs (~ mx_desc_ensure_r_floor()).
    content = scaffold::desc_ensure_r_floor(&content);

    // License: fill in when unset or still a usethis placeholder
    // ("`use_mit_license()`, `use_gpl3_license()` or friends ...").
    let license = parse_description_field(&content, "License").unwrap_or_default();
    if license.is_empty() || license.contains("_license()") {
        content = scaffold::desc_set_field(&content, "License", "MIT + file LICENSE");
    }

    // SystemRequirements: append the Rust toolchain requirement if absent.
    let sys_req = parse_description_field(&content, "SystemRequirements").unwrap_or_default();
    if !sys_req.to_lowercase().contains("rust") {
        let merged = if sys_req.is_empty() {
            RUST_SYSTEM_REQUIREMENT.to_string()
        } else {
            format!("{sys_req}, {RUST_SYSTEM_REQUIREMENT}")
        };
        content = scaffold::desc_set_field(&content, "SystemRequirements", &merged);
    }

    std::fs::write(&desc_path, content)?;
    Ok(())
}

/// Run autoconf when available and make `configure` executable
/// (~ `miniextendr_autoconf()`). Failure is a warning, not an error — the
/// user can install autoconf and rerun `miniextendr workflow autoconf`.
fn run_autoconf(dir: &Path, quiet: bool) {
    if !has_program("autoconf") {
        if !quiet {
            eprintln!(
                "autoconf not found; run `miniextendr workflow autoconf` after installing it"
            );
        }
        return;
    }
    match run_command("autoconf", &["-v", "-i", "-f"], dir, quiet) {
        Ok(_) => {
            let configure = dir.join("configure");
            #[cfg(unix)]
            if configure.is_file() {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(&configure, std::fs::Permissions::from_mode(0o755));
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("autoconf failed: {e:#}");
                eprintln!("Run `miniextendr workflow autoconf` manually later");
            }
        }
    }
}

// endregion

// region: validation helpers

fn package_name_from_path(root: &Path) -> Result<String> {
    root.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_string)
        .with_context(|| format!("could not derive a package name from {}", root.display()))
}

/// Validate an R package name (~ `create_miniextendr_package()`): ASCII
/// letters, digits, and dots; starts with a letter; does not end with a dot;
/// at least 2 characters.
fn validate_package_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    let valid = name.len() >= 2
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '.')
        && !name.ends_with('.');
    if !valid {
        let suggestion: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '.')
            .collect();
        bail!(
            "\"{name}\" is not a valid R package name.\n\
             R package names must start with a letter, contain only ASCII letters, digits,\n\
             and dots, and not end with a dot. Try: \"{suggestion}\" \
             (or pass --package for monorepos)"
        );
    }
    Ok(())
}

// endregion

#[cfg(test)]
mod tests;
