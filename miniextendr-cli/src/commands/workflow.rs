use anyhow::{Result, bail};

use crate::bridge::{bash, has_program, program_version, rscript_eval, run_command};
use crate::cli::WorkflowCmd;
use crate::project::{MINIEXTENDR_CRATES, ProjectContext};

pub fn dispatch(cmd: &WorkflowCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        WorkflowCmd::Autoconf => workflow_autoconf(ctx, quiet),
        WorkflowCmd::Configure { cran } => workflow_configure(ctx, *cran, quiet),
        WorkflowCmd::Document => workflow_document(ctx, quiet),
        WorkflowCmd::Build { no_install } => workflow_build(ctx, *no_install, quiet),
        WorkflowCmd::Install { r_cmd, args } => workflow_install(ctx, *r_cmd, args, quiet),
        WorkflowCmd::Check {
            error_on,
            check_dir,
            args: _,
        } => workflow_check(ctx, error_on, check_dir.as_deref(), quiet),
        WorkflowCmd::Test { filter } => workflow_test(ctx, filter.as_deref(), quiet),
        WorkflowCmd::Doctor => workflow_doctor(ctx, quiet),
        WorkflowCmd::Upgrade => workflow_upgrade(ctx, quiet),
        WorkflowCmd::CheckRust => workflow_check_rust(quiet),
        WorkflowCmd::Sync => workflow_sync(ctx, quiet),
        WorkflowCmd::DevLink => workflow_dev_link(ctx, quiet),
    }
}

/// Native — runs autoconf.
fn workflow_autoconf(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    ctx.require_configure_ac()?;
    run_command("autoconf", &["-vif"], &ctx.root, quiet)?;
    Ok(())
}

/// Native — runs bash ./configure.
fn workflow_configure(ctx: &ProjectContext, cran: bool, quiet: bool) -> Result<()> {
    // Run autoconf first if available
    if ctx.configure_ac.is_some() && has_program("autoconf") {
        let _ = run_command("autoconf", &["-vif"], &ctx.root, true);
    }

    if ctx.configure.is_none() && ctx.configure_ac.is_none() {
        bail!(
            "No configure or configure.ac found.\n\
             Run `miniextendr init use` to set up miniextendr scaffolding."
        );
    }

    let script = if cran {
        "PREPARE_CRAN=true bash ./configure"
    } else {
        "NOT_CRAN=true bash ./configure"
    };
    bash(script, &ctx.root, quiet)?;
    Ok(())
}

/// Calls `devtools::document()` directly — requires devtools, not minirextendr.
fn workflow_document(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    // Configure first
    let _ = workflow_configure(ctx, false, true);

    let root = ctx.root.to_string_lossy().replace('\\', "/");
    let expr = format!("devtools::document(\"{root}\")");
    let script = format!("NOT_CRAN=true Rscript -e '{}'", expr.replace('\'', "'\\''"));
    bash(&script, &ctx.root, quiet)?;
    Ok(())
}

/// Full two-pass build: configure, install, document, install.
fn workflow_build(ctx: &ProjectContext, no_install: bool, quiet: bool) -> Result<()> {
    if !quiet {
        eprintln!("=== configure ===");
    }
    workflow_configure(ctx, false, quiet)?;

    if !no_install {
        if !quiet {
            eprintln!("=== install (pass 1) ===");
        }
        workflow_install(ctx, true, &[], quiet)?;

        if !quiet {
            eprintln!("=== document ===");
        }
        workflow_document(ctx, quiet)?;

        if !quiet {
            eprintln!("=== install (pass 2) ===");
        }
        workflow_install(ctx, true, &[], quiet)?;
    }

    Ok(())
}

/// Install via R CMD INSTALL or devtools::install.
fn workflow_install(ctx: &ProjectContext, r_cmd: bool, args: &[String], quiet: bool) -> Result<()> {
    let root = ctx.root.to_string_lossy();
    if r_cmd {
        let extra = args.join(" ");
        let script = format!("NOT_CRAN=true R CMD INSTALL {extra} {root}");
        bash(&script, &ctx.root, quiet)?;
    } else {
        let root_escaped = root.replace('\\', "/");
        let expr = format!("devtools::install(\"{root_escaped}\")");
        rscript_eval(&expr, &ctx.root, quiet)?;
    }
    Ok(())
}

/// Run devtools::check or rcmdcheck directly.
fn workflow_check(
    ctx: &ProjectContext,
    error_on: &str,
    check_dir: Option<&str>,
    quiet: bool,
) -> Result<()> {
    let root = ctx.root.to_string_lossy().replace('\\', "/");
    let check_dir_r = match check_dir {
        Some(d) => format!("\"{}\"", d.replace('\\', "/")),
        None => "NULL".to_string(),
    };
    let expr = format!(
        "devtools::check(\"{root}\", error_on = \"{error_on}\", check_dir = {check_dir_r})"
    );
    let script = format!("NOT_CRAN=true Rscript -e '{}'", expr.replace('\'', "'\\''"));
    bash(&script, &ctx.root, quiet)?;
    Ok(())
}

/// Run devtools::test directly.
fn workflow_test(ctx: &ProjectContext, filter: Option<&str>, quiet: bool) -> Result<()> {
    let root = ctx.root.to_string_lossy().replace('\\', "/");
    let expr = match filter {
        Some(f) => {
            format!("testthat::set_max_fails(Inf); devtools::test(\"{root}\", filter = \"{f}\")")
        }
        None => format!("testthat::set_max_fails(Inf); devtools::test(\"{root}\")"),
    };
    rscript_eval(&expr, &ctx.root, quiet)?;
    Ok(())
}

/// Native — comprehensive project health check.
fn workflow_doctor(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let root = &ctx.root;

    if !quiet {
        println!("=== miniextendr doctor ===\n");
    }

    let mut pass = 0u32;
    let mut warn = 0u32;
    let mut fail = 0u32;

    // Toolchain
    if !quiet {
        println!("-- Toolchain --");
    }
    if let Some(v) = program_version("rustc") {
        pass += 1;
        if !quiet {
            println!("  [ok] Rust: {v}");
        }
    } else {
        fail += 1;
        if !quiet {
            println!("  [FAIL] Rust not found - install from https://rustup.rs");
        }
    }

    if has_program("cargo") {
        pass += 1;
        if !quiet {
            println!("  [ok] cargo available");
        }
    } else {
        fail += 1;
        if !quiet {
            println!("  [FAIL] cargo not found");
        }
    }

    if has_program("autoconf") {
        pass += 1;
        if !quiet {
            println!("  [ok] autoconf available");
        }
    } else {
        warn += 1;
        if !quiet {
            println!("  [!!] autoconf not found (needed for configure.ac changes)");
        }
    }

    // Vendored crates
    if !quiet {
        println!("\n-- Vendored crates --");
    }
    for krate in MINIEXTENDR_CRATES {
        if root.join("vendor").join(krate).is_dir() {
            pass += 1;
            if !quiet {
                println!("  [ok] {krate}");
            }
        } else {
            fail += 1;
            if !quiet {
                println!("  [FAIL] {krate} not vendored");
            }
        }
    }

    // Generated file freshness
    if !quiet {
        println!("\n-- Generated files --");
    }
    let template_pairs = [("src/Makevars.in", "src/Makevars")];

    for (tmpl, generated) in &template_pairs {
        let tmpl_path = root.join(tmpl);
        let gen_path = root.join(generated);

        if !tmpl_path.exists() {
            continue;
        }

        if !gen_path.exists() {
            warn += 1;
            if !quiet {
                println!("  [!!] {generated} missing (run `miniextendr workflow configure`)");
            }
        } else if let (Ok(tm), Ok(gm)) = (tmpl_path.metadata(), gen_path.metadata())
            && let (Ok(tt), Ok(gt)) = (tm.modified(), gm.modified())
        {
            if tt > gt {
                warn += 1;
                if !quiet {
                    println!("  [!!] {generated} is stale (template is newer)");
                }
            } else {
                pass += 1;
                if !quiet {
                    println!("  [ok] {generated} up to date");
                }
            }
        }
    }

    // NAMESPACE check
    if !quiet {
        println!("\n-- NAMESPACE --");
    }
    let ns_path = root.join("NAMESPACE");
    if ns_path.exists()
        && let Ok(content) = std::fs::read_to_string(&ns_path)
    {
        if content.contains("useDynLib") {
            pass += 1;
            if !quiet {
                println!("  [ok] NAMESPACE contains useDynLib");
            }
        } else {
            fail += 1;
            if !quiet {
                println!("  [FAIL] NAMESPACE missing useDynLib directive");
            }
        }
    }

    // Summary
    if !quiet {
        println!("\n-- Summary --");
        println!("  {pass} passed");
        if warn > 0 {
            println!("  {warn} warning(s)");
        }
        if fail > 0 {
            println!("  {fail} failure(s)");
        }
        if fail == 0 && warn == 0 {
            println!("  All checks passed!");
        }
    }

    if fail > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Upgrade — calls Rscript with devtools/usethis directly.
fn workflow_upgrade(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    // For upgrade, we do need the minirextendr package since the logic is complex.
    // But per the user's request, let's keep it simple: re-run autoconf + configure.
    if !quiet {
        eprintln!("Upgrading: re-running autoconf + configure...");
    }
    workflow_autoconf(ctx, true).ok();
    workflow_configure(ctx, false, quiet)?;
    if !quiet {
        eprintln!("Upgrade complete. Run `miniextendr workflow build` for a full rebuild.");
    }
    Ok(())
}

/// Native — check Rust toolchain.
fn workflow_check_rust(quiet: bool) -> Result<()> {
    if let Some(v) = program_version("rustc") {
        if !quiet {
            println!("rustc: {v}");
        }
    } else {
        bail!("Rust not found. Install from https://rustup.rs");
    }

    if let Some(v) = program_version("cargo") {
        if !quiet {
            println!("cargo: {v}");
        }
    } else {
        bail!("cargo not found. Install from https://rustup.rs");
    }

    Ok(())
}

/// Combined autoconf + configure + document.
fn workflow_sync(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    if !quiet {
        eprintln!("=== autoconf ===");
    }
    workflow_autoconf(ctx, quiet).ok();

    if !quiet {
        eprintln!("=== configure ===");
    }
    workflow_configure(ctx, false, quiet)?;

    if !quiet {
        eprintln!("=== document ===");
    }
    workflow_document(ctx, quiet)?;

    Ok(())
}

/// devtools::load_all — requires R but not minirextendr.
fn workflow_dev_link(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let root = ctx.root.to_string_lossy().replace('\\', "/");
    let expr = format!("devtools::load_all(\"{root}\")");
    rscript_eval(&expr, &ctx.root, quiet)?;
    Ok(())
}
