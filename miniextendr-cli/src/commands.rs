#[cfg(feature = "dev")]
pub mod bench;
pub mod cargo;
pub mod config;
#[cfg(feature = "dev")]
pub mod cross;
pub mod feature;
pub mod init;
pub mod lint;
pub mod render;
pub mod rust;
pub mod status;
#[cfg(feature = "dev")]
pub mod templates;
pub mod vendor;
pub mod workflow;

use anyhow::Result;

use crate::bridge::run_command;
use crate::cli::Command;
use crate::project::ProjectContext;

pub fn dispatch(cmd: &Command, ctx: &ProjectContext, quiet: bool, json: bool) -> Result<()> {
    match cmd {
        Command::Init { cmd } => init::dispatch(cmd, ctx, quiet),
        Command::Workflow { cmd } => workflow::dispatch(cmd, ctx, quiet),
        Command::Status { cmd } => status::dispatch(cmd, ctx, quiet, json),
        Command::Cargo { cmd } => cargo::dispatch(cmd, ctx, quiet),
        Command::Vendor { cmd } => vendor::dispatch(cmd, ctx, quiet, json),
        Command::Feature { cmd } => feature::dispatch(cmd, ctx, quiet, json),
        Command::Render { cmd } => render::dispatch(cmd, ctx, quiet),
        Command::Rust { cmd } => rust::dispatch(cmd, ctx, quiet),
        Command::Config { cmd } => config::dispatch(cmd, ctx, quiet, json),
        Command::Lint => lint::run(ctx, quiet),
        Command::Clean => clean(ctx, quiet),
        Command::Completions { .. } => Ok(()), // handled in main before dispatch
        #[cfg(feature = "dev")]
        Command::Dev { cmd } => dispatch_dev(cmd, ctx, quiet),
    }
}

#[cfg(feature = "dev")]
fn dispatch_dev(cmd: &crate::cli::DevCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        crate::cli::DevCmd::Bench { cmd } => bench::dispatch(cmd, ctx, quiet),
        crate::cli::DevCmd::Cross { cmd } => cross::dispatch(cmd, ctx, quiet),
        crate::cli::DevCmd::Templates { cmd } => templates::dispatch(cmd, ctx, quiet),
    }
}

/// Clean build artifacts for the current project.
fn clean(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    if !quiet {
        eprintln!("Cleaning build artifacts...");
    }

    // 1. cargo clean on the project's Rust crate
    if let Some(manifest) = &ctx.cargo_manifest {
        run_command(
            "cargo",
            &["clean", "--manifest-path", &manifest.to_string_lossy()],
            &ctx.root,
            quiet,
        )?;
    }

    // 2. Remove common build artifact directories
    for dir_name in ["rust-target", "ra-target", "src/rust/target"] {
        let d = ctx.root.join(dir_name);
        if d.is_dir() {
            let _ = std::fs::remove_dir_all(&d);
            if !quiet {
                eprintln!("  Removed {dir_name}/");
            }
        }
    }

    // 3. Run cleanup script if present (standard R package hook)
    let cleanup = ctx.root.join("cleanup");
    if cleanup.is_file() {
        let cleanup_str = cleanup.to_string_lossy().into_owned();
        let _ = run_command("bash", &[&cleanup_str], &ctx.root, quiet);
    }

    if !quiet {
        eprintln!("Done.");
    }
    Ok(())
}
