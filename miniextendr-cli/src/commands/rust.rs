use anyhow::Result;

use crate::bridge::run_command;
use crate::cli::RustCmd;
use crate::project::ProjectContext;

pub fn dispatch(cmd: &RustCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        RustCmd::Source { code } => rust_source(ctx, code, quiet),
        RustCmd::Function { code } => rust_function(ctx, code, quiet),
        RustCmd::Clean => rust_clean(ctx, quiet),
    }
}

/// Compile and run a Rust source file or inline code.
///
/// Creates a temporary crate, compiles it, and makes the shared library
/// available for R to load.
fn rust_source(ctx: &ProjectContext, code: &str, quiet: bool) -> Result<()> {
    let tmp = tempdir()?;
    let src_dir = tmp.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Write lib.rs
    std::fs::write(src_dir.join("lib.rs"), code)?;

    // Write Cargo.toml
    let cargo_toml = format!(
        "[package]\nname = \"mx_temp\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n\
         [lib]\ncrate-type = [\"cdylib\"]\n\n\
         [dependencies]\nminiextendr-api = {{ path = \"{}\" }}\n",
        find_miniextendr_api(ctx)
    );
    std::fs::write(tmp.join("Cargo.toml"), cargo_toml)?;

    // Build
    run_command("cargo", &["build", "--release"], &tmp, quiet)?;

    if !quiet {
        println!("Built: {}", tmp.join("target/release").display());
    }

    Ok(())
}

/// Compile a single Rust function.
fn rust_function(ctx: &ProjectContext, code: &str, quiet: bool) -> Result<()> {
    let wrapped = format!(
        "use miniextendr_api::miniextendr;\n\n\
         {code}\n"
    );
    rust_source(ctx, &wrapped, quiet)
}

/// Clean temporary compiled code.
fn rust_clean(_ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let cache = rust_cache_dir();
    if cache.is_dir() {
        std::fs::remove_dir_all(&cache)?;
        if !quiet {
            println!("Cleaned Rust compilation cache.");
        }
    } else if !quiet {
        println!("No compilation cache found.");
    }
    Ok(())
}

fn tempdir() -> Result<std::path::PathBuf> {
    let cache = rust_cache_dir();
    std::fs::create_dir_all(&cache)?;
    Ok(cache)
}

fn rust_cache_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("mx-rust-source")
}

fn find_miniextendr_api(ctx: &ProjectContext) -> String {
    // Try vendor first
    let vendor = ctx.root.join("vendor/miniextendr-api");
    if vendor.is_dir() {
        return vendor.to_string_lossy().replace('\\', "/");
    }

    // Try workspace
    for dir in [&ctx.root, &ctx.root.join(".."), &ctx.root.join("../..")] {
        let candidate = dir.join("miniextendr-api");
        if candidate.is_dir()
            && let Ok(canonical) = std::fs::canonicalize(&candidate)
        {
            return canonical.to_string_lossy().replace('\\', "/");
        }
    }

    // Fallback to crates.io
    "miniextendr-api".into()
}
