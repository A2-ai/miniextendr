use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::bridge::{has_program, program_version};
use crate::cli::StatusCmd;
use crate::project::{MINIEXTENDR_CRATES, ProjectContext};

#[derive(Serialize)]
struct HasResult {
    has_miniextendr: bool,
    has_cargo_manifest: bool,
    has_description: bool,
    has_configure_ac: bool,
    has_configure: bool,
}

impl std::fmt::Display for HasResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "miniextendr: {}", yn(self.has_miniextendr))?;
        writeln!(f, "  src/rust/Cargo.toml: {}", yn(self.has_cargo_manifest))?;
        writeln!(f, "  DESCRIPTION: {}", yn(self.has_description))?;
        writeln!(f, "  configure.ac: {}", yn(self.has_configure_ac))?;
        write!(f, "  configure: {}", yn(self.has_configure))
    }
}

fn yn(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

pub fn dispatch(cmd: &StatusCmd, ctx: &ProjectContext, _quiet: bool, json: bool) -> Result<()> {
    match cmd {
        StatusCmd::Has => status_has(ctx, json),
        StatusCmd::Show => status_show(ctx, json),
        StatusCmd::Validate => status_validate(ctx, json),
    }
}

/// Native — check if project has miniextendr.
fn status_has(ctx: &ProjectContext, json: bool) -> Result<()> {
    let result = HasResult {
        has_miniextendr: ctx.has_miniextendr(),
        has_cargo_manifest: ctx.cargo_manifest.is_some(),
        has_description: ctx.description.is_some(),
        has_configure_ac: ctx.configure_ac.is_some(),
        has_configure: ctx.configure.is_some(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("{result}");
    }

    if !result.has_miniextendr {
        std::process::exit(1);
    }
    Ok(())
}

/// Native — show which miniextendr files are present/missing.
fn status_show(ctx: &ProjectContext, json: bool) -> Result<()> {
    let root = &ctx.root;

    // Derive wrapper filename from DESCRIPTION
    let pkg_name = read_description_field(root, "Package").unwrap_or_else(|| "miniextendr".into());
    let wrapper_file = format!("R/{pkg_name}-wrappers.R");

    let categories: &[(&str, &[&str])] = &[
        (
            "Build System",
            &[
                "configure.ac",
                "configure",
                "bootstrap.R",
                "cleanup",
                "cleanup.win",
                "cleanup.ucrt",
                "configure.win",
                "configure.ucrt",
                "tools/config.guess",
                "tools/config.sub",
            ],
        ),
        (
            "Rust Project",
            &[
                "src/rust/Cargo.toml",
                "src/rust/lib.rs",
                "src/rust/build.rs",
                "src/rust/cargo-config.toml.in",
            ],
        ),
        ("Source Templates", &["src/Makevars.in", "src/stub.c"]),
    ];

    // Build vendored crate paths dynamically from constant
    let vendor_paths: Vec<String> = MINIEXTENDR_CRATES
        .iter()
        .map(|c| format!("vendor/{c}"))
        .collect();
    let vendor_refs: Vec<&str> = vendor_paths.iter().map(|s| s.as_str()).collect();

    let categories: Vec<(&str, Vec<&str>)> = categories
        .iter()
        .map(|(name, files)| (*name, files.to_vec()))
        .chain(std::iter::once(("Vendored Crates", vendor_refs)))
        .collect();

    #[derive(Serialize)]
    struct StatusReport {
        present: Vec<String>,
        missing: Vec<String>,
        stale: Vec<String>,
    }

    let mut all_present = Vec::new();
    let mut all_missing = Vec::new();

    if !json {
        println!("=== miniextendr status ===\n");
    }

    for (category, files) in &categories {
        if !json {
            println!("-- {category} --");
        }
        for file in files {
            let path = root.join(file);
            let exists = path.exists();
            if exists {
                if !json {
                    println!("  [ok] {file}");
                }
                all_present.push(file.to_string());
            } else {
                if !json {
                    println!("  [--] {file}");
                }
                all_missing.push(file.to_string());
            }
        }
        if !json {
            println!();
        }
    }

    // Generated files (including dynamic wrapper)
    let generated = ["src/Makevars", &wrapper_file];
    if !json {
        println!("-- Generated Files --");
    }
    for file in &generated {
        let path = root.join(file);
        let exists = path.exists();
        if exists {
            if !json {
                println!("  [ok] {file}");
            }
            all_present.push(file.to_string());
        } else {
            if !json {
                println!("  [--] {file}");
            }
            all_missing.push(file.to_string());
        }
    }
    if !json {
        println!();
    }

    // Staleness check
    let template_pairs = [
        ("src/Makevars.in", "src/Makevars"),
        (
            "src/rust/cargo-config.toml.in",
            "src/rust/.cargo/config.toml",
        ),
    ];

    let mut stale = Vec::new();
    if !json {
        println!("-- Staleness --");
    }
    for (tmpl, generated) in &template_pairs {
        let tmpl_path = root.join(tmpl);
        let gen_path = root.join(generated);
        if tmpl_path.exists()
            && gen_path.exists()
            && let (Ok(tmpl_meta), Ok(gen_meta)) = (tmpl_path.metadata(), gen_path.metadata())
            && let (Ok(tmpl_time), Ok(gen_time)) = (tmpl_meta.modified(), gen_meta.modified())
            && tmpl_time > gen_time
        {
            if !json {
                println!("  [!!] {generated} is stale (template {tmpl} is newer)");
            }
            stale.push(generated.to_string());
        }
    }
    if stale.is_empty() && !json {
        println!("  All generated files up to date");
    }

    // Summary
    let total = all_present.len() + all_missing.len();
    if !json {
        println!("\n-- Summary --");
        println!("  {}/{total} files present", all_present.len());
        if !all_missing.is_empty() {
            println!("  {} files missing", all_missing.len());
        }
        if !stale.is_empty() {
            println!("  {} stale generated file(s)", stale.len());
        }
    } else {
        let report = StatusReport {
            present: all_present,
            missing: all_missing,
            stale,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

/// Native — validate miniextendr configuration.
fn status_validate(ctx: &ProjectContext, json: bool) -> Result<()> {
    let root = &ctx.root;

    #[derive(Serialize)]
    struct ValidateReport {
        pass: Vec<String>,
        warn: Vec<String>,
        fail: Vec<String>,
    }

    let mut pass = Vec::new();
    let mut warn = Vec::new();
    let mut fail = Vec::new();

    if !json {
        println!("=== miniextendr validate ===\n");
    }

    // DESCRIPTION checks
    if !json {
        println!("-- DESCRIPTION --");
    }
    let desc_path = root.join("DESCRIPTION");
    if !desc_path.exists() {
        fail.push("DESCRIPTION not found".into());
        if !json {
            println!("  [FAIL] DESCRIPTION not found");
        }
    } else {
        let bootstrap = read_description_field(root, "Config/build/bootstrap").unwrap_or_default();
        if bootstrap == "TRUE" {
            pass.push("Config/build/bootstrap = TRUE".into());
            if !json {
                println!("  [ok] Config/build/bootstrap = TRUE");
            }
        } else {
            warn.push("Config/build/bootstrap should be TRUE".into());
            if !json {
                println!("  [!!] Config/build/bootstrap not set to TRUE");
            }
        }

        let sys_req = read_description_field(root, "SystemRequirements").unwrap_or_default();
        if sys_req.to_lowercase().contains("rust") {
            pass.push("SystemRequirements mentions Rust".into());
            if !json {
                println!("  [ok] SystemRequirements mentions Rust");
            }
        } else {
            warn.push("SystemRequirements should mention Rust".into());
            if !json {
                println!("  [!!] SystemRequirements doesn't mention Rust");
            }
        }
    }

    // configure.ac
    if !json {
        println!("\n-- configure.ac --");
    }
    if ctx.configure_ac.is_none() {
        fail.push("configure.ac not found".into());
        if !json {
            println!("  [FAIL] configure.ac not found");
        }
    } else {
        pass.push("configure.ac present".into());
        if !json {
            println!("  [ok] configure.ac present");
        }
    }

    // Rust toolchain
    if !json {
        println!("\n-- Rust toolchain --");
    }
    if has_program("rustc") {
        let version = program_version("rustc").unwrap_or("unknown".into());
        pass.push(format!("Rust installed: {version}"));
        if !json {
            println!("  [ok] Rust: {version}");
        }
    } else {
        fail.push("Rust not found - install from https://rustup.rs".into());
        if !json {
            println!("  [FAIL] Rust not found");
        }
    }

    if has_program("cargo") {
        pass.push("cargo available".into());
        if !json {
            println!("  [ok] cargo available");
        }
    } else {
        fail.push("cargo not found".into());
        if !json {
            println!("  [FAIL] cargo not found");
        }
    }

    // Vendored crates
    if !json {
        println!("\n-- Vendored crates --");
    }
    for krate in MINIEXTENDR_CRATES {
        if root.join("vendor").join(krate).is_dir() {
            pass.push(format!("{krate} vendored"));
            if !json {
                println!("  [ok] {krate}");
            }
        } else {
            warn.push(format!("{krate} not vendored"));
            if !json {
                println!("  [!!] {krate} not vendored");
            }
        }
    }

    // Summary
    if !json {
        println!("\n-- Result --");
        println!("  {} passed", pass.len());
        if !warn.is_empty() {
            println!("  {} warning(s)", warn.len());
        }
        if !fail.is_empty() {
            println!("  {} failure(s)", fail.len());
            for f in &fail {
                println!("  x {f}");
            }
        }
        if fail.is_empty() && warn.is_empty() {
            println!("  All checks passed!");
        }
    } else {
        let report = ValidateReport { pass, warn, fail };
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

/// Read a field from DESCRIPTION (simple DCF parser).
fn read_description_field(root: &Path, field: &str) -> Option<String> {
    let desc_path = root.join("DESCRIPTION");
    let content = std::fs::read_to_string(desc_path).ok()?;
    let prefix = format!("{field}:");

    for line in content.lines() {
        if line.starts_with(&prefix) {
            return Some(line[prefix.len()..].trim().to_string());
        }
    }
    None
}
