use anyhow::Result;

use crate::bridge::{bash, has_program, rscript_eval, run_command};
use crate::cli::CrossCmd;
use crate::project::ProjectContext;

pub fn dispatch(cmd: &CrossCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        CrossCmd::Configure => cross_configure(ctx, quiet),
        CrossCmd::Install => cross_install(ctx, quiet),
        CrossCmd::Document => cross_document(ctx, quiet),
        CrossCmd::Test => cross_test(ctx, quiet),
        CrossCmd::Check => cross_check(ctx, quiet),
        CrossCmd::Clean => cross_clean(ctx, quiet),
    }
}

/// Find the cross-package test directory.
fn cross_dir(ctx: &ProjectContext) -> Result<std::path::PathBuf> {
    for dir in [&ctx.root, &ctx.root.join(".."), &ctx.root.join("../..")] {
        let candidate = dir.join("tests/cross-package");
        if candidate.is_dir() {
            return Ok(std::fs::canonicalize(candidate)?);
        }
    }
    anyhow::bail!(
        "tests/cross-package/ not found.\n\
         Run this command from the miniextendr workspace root."
    );
}

/// Configure a single cross-package (autoconf + configure).
fn configure_pkg(pkg_dir: &std::path::Path, quiet: bool) -> Result<()> {
    if has_program("autoconf") {
        let _ = run_command("autoconf", &["-vif"], pkg_dir, true);
    }
    bash("NOT_CRAN=true bash ./configure", pkg_dir, quiet)?;
    Ok(())
}

fn cross_configure(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let dir = cross_dir(ctx)?;
    if !quiet {
        eprintln!("=== Configuring producer.pkg ===");
    }
    configure_pkg(&dir.join("producer.pkg"), quiet)?;
    if !quiet {
        eprintln!("=== Configuring consumer.pkg ===");
    }
    configure_pkg(&dir.join("consumer.pkg"), quiet)?;
    Ok(())
}

fn cross_install(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let dir = cross_dir(ctx)?;

    // Configure + install producer first (consumer depends on it)
    if !quiet {
        eprintln!("=== Building producer.pkg ===");
    }
    configure_pkg(&dir.join("producer.pkg"), quiet)?;
    let expr = "devtools::install(\".\", upgrade=\"never\", quick=TRUE)";
    let script = format!("NOT_CRAN=true Rscript -e '{expr}'");
    bash(&script, &dir.join("producer.pkg"), quiet)?;

    // Then configure + install consumer
    if !quiet {
        eprintln!("=== Building consumer.pkg ===");
    }
    configure_pkg(&dir.join("consumer.pkg"), quiet)?;
    bash(&script, &dir.join("consumer.pkg"), quiet)?;

    Ok(())
}

fn cross_document(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let dir = cross_dir(ctx)?;
    if !quiet {
        eprintln!("=== Documenting producer.pkg ===");
    }
    configure_pkg(&dir.join("producer.pkg"), true)?;
    rscript_eval("devtools::document()", &dir.join("producer.pkg"), quiet)?;
    if !quiet {
        eprintln!("=== Documenting consumer.pkg ===");
    }
    configure_pkg(&dir.join("consumer.pkg"), true)?;
    rscript_eval("devtools::document()", &dir.join("consumer.pkg"), quiet)?;
    Ok(())
}

fn cross_test(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let dir = cross_dir(ctx)?;
    if !quiet {
        eprintln!("=== Testing producer.pkg ===");
    }
    rscript_eval("devtools::test()", &dir.join("producer.pkg"), quiet)?;
    if !quiet {
        eprintln!("=== Testing consumer.pkg ===");
    }
    rscript_eval("devtools::test()", &dir.join("consumer.pkg"), quiet)?;
    Ok(())
}

fn cross_check(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let dir = cross_dir(ctx)?;
    let expr = "devtools::check()";
    let script = format!("NOT_CRAN=true Rscript -e '{expr}'");
    if !quiet {
        eprintln!("=== Checking producer.pkg ===");
    }
    configure_pkg(&dir.join("producer.pkg"), true)?;
    bash(&script, &dir.join("producer.pkg"), quiet)?;
    if !quiet {
        eprintln!("=== Checking consumer.pkg ===");
    }
    configure_pkg(&dir.join("consumer.pkg"), true)?;
    bash(&script, &dir.join("consumer.pkg"), quiet)?;
    Ok(())
}

fn cross_clean(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let dir = cross_dir(ctx)?;
    for pkg in &["producer.pkg", "consumer.pkg"] {
        let pkg_dir = dir.join(pkg);
        let cleanup = pkg_dir.join("cleanup");
        if cleanup.is_file() {
            let _ = run_command("bash", &["cleanup"], &pkg_dir, true);
        }
        // Remove target directories
        for target in &["src/rust/target", "rust-target"] {
            let t = pkg_dir.join(target);
            if t.is_dir() {
                std::fs::remove_dir_all(&t)?;
            }
        }
    }
    if !quiet {
        println!("Cleaned cross-package test artifacts.");
    }
    Ok(())
}
