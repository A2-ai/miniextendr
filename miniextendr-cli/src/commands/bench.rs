use anyhow::Result;

use crate::bridge::run_command;
use crate::cli::BenchCmd;
use crate::project::{ProjectContext, find_workspace_root};

pub fn dispatch(cmd: &BenchCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        BenchCmd::Run { args } => bench_run(ctx, args, quiet),
        BenchCmd::Core { args } => bench_core(ctx, args, quiet),
        BenchCmd::Features { args } => bench_features(ctx, args, quiet),
        BenchCmd::Full { args } => bench_full(ctx, args, quiet),
        BenchCmd::R => bench_r(ctx, quiet),
        BenchCmd::Save { args } => bench_save(ctx, args, quiet),
        BenchCmd::Compare { csv_file } => bench_compare(ctx, csv_file.as_deref(), quiet),
        BenchCmd::Drift { args } => bench_drift(ctx, args, quiet),
        BenchCmd::Info => bench_info(ctx, quiet),
        BenchCmd::Compile { args } => bench_compile(ctx, args, quiet),
        BenchCmd::LintBench { args } => bench_lint(ctx, args, quiet),
        BenchCmd::Check { args } => bench_check(ctx, args, quiet),
    }
}

/// Find the bench manifest. Tries `miniextendr-bench/Cargo.toml` relative to
/// the workspace root (walking up from ctx.root).
fn bench_manifest(ctx: &ProjectContext) -> Result<std::path::PathBuf> {
    // Try workspace root patterns
    for dir in [&ctx.root, &ctx.root.join(".."), &ctx.root.join("../..")] {
        let candidate = dir.join("miniextendr-bench/Cargo.toml");
        if candidate.is_file() {
            return Ok(std::fs::canonicalize(candidate)?);
        }
    }
    anyhow::bail!(
        "miniextendr-bench/Cargo.toml not found.\n\
         Run this command from the miniextendr workspace root."
    );
}

fn workspace_root(ctx: &ProjectContext) -> Result<std::path::PathBuf> {
    find_workspace_root(&ctx.root).ok_or_else(|| {
        anyhow::anyhow!("Workspace root not found. Run from the miniextendr monorepo.")
    })
}

fn bench_run(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let manifest = bench_manifest(ctx)?;
    let mut cmd_args: Vec<String> = vec![
        "bench".into(),
        "--manifest-path".into(),
        manifest.to_string_lossy().into(),
    ];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, &ctx.root, quiet)?;
    Ok(())
}

fn bench_core(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let manifest = bench_manifest(ctx)?;
    let core_benches = [
        "ffi_calls",
        "into_r",
        "from_r",
        "translate",
        "strings",
        "externalptr",
        "worker",
        "unwind_protect",
    ];
    let mut cmd_args: Vec<String> = vec![
        "bench".into(),
        "--manifest-path".into(),
        manifest.to_string_lossy().into(),
    ];
    for b in &core_benches {
        cmd_args.push("--bench".into());
        cmd_args.push((*b).into());
    }
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, &ctx.root, quiet)?;
    Ok(())
}

fn bench_features(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let manifest = bench_manifest(ctx)?;
    let mut cmd_args: Vec<String> = vec![
        "bench".into(),
        "--manifest-path".into(),
        manifest.to_string_lossy().into(),
        "--features".into(),
        "connections,rayon,refcount-fast-hash".into(),
    ];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, &ctx.root, quiet)?;
    Ok(())
}

fn bench_full(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    bench_run(ctx, args, quiet)?;
    let manifest = bench_manifest(ctx)?;
    let mut cmd_args: Vec<String> = vec![
        "bench".into(),
        "--manifest-path".into(),
        manifest.to_string_lossy().into(),
        "--features".into(),
        "connections,rayon,refcount-fast-hash".into(),
        "--bench".into(),
        "connections".into(),
        "--bench".into(),
        "rayon".into(),
        "--bench".into(),
        "refcount_protect".into(),
    ];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, &ctx.root, quiet)?;
    Ok(())
}

fn bench_r(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let script = r#"
for f in rpkg/tests/testthat/bench-*.R; do
  echo "=== Running $f ==="
  Rscript "$f"
  echo ""
done
"#;
    run_command("bash", &["-c", script], &ws, quiet)?;
    Ok(())
}

fn bench_save(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let script_path = ws.join("tests/perf/bench_baseline.sh");
    let mut cmd_args = vec![
        script_path.to_string_lossy().to_string(),
        "save".into(),
        "--".into(),
    ];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("bash", &str_args, &ws, quiet)?;
    Ok(())
}

fn bench_compare(ctx: &ProjectContext, csv_file: Option<&str>, quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let script_path = ws.join("tests/perf/bench_baseline.sh");
    let mut cmd_args = vec![script_path.to_string_lossy().to_string(), "compare".into()];
    if let Some(f) = csv_file {
        cmd_args.push(f.into());
    }
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("bash", &str_args, &ws, quiet)?;
    Ok(())
}

fn bench_drift(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let script_path = ws.join("tests/perf/bench_baseline.sh");
    let mut cmd_args = vec![script_path.to_string_lossy().to_string(), "drift".into()];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("bash", &str_args, &ws, quiet)?;
    Ok(())
}

fn bench_info(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let script_path = ws.join("tests/perf/bench_baseline.sh");
    let cmd_args = [script_path.to_string_lossy().to_string(), "info".into()];
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("bash", &str_args, &ws, quiet)?;
    Ok(())
}

fn bench_compile(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let script_path = ws.join("tests/perf/macro_compile_bench.sh");
    let mut cmd_args = vec![script_path.to_string_lossy().to_string()];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("bash", &str_args, &ws, quiet)?;
    Ok(())
}

fn bench_lint(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let ws = workspace_root(ctx)?;
    let manifest = ws.join("miniextendr-lint/Cargo.toml");
    let mut cmd_args: Vec<String> = vec![
        "bench".into(),
        "--manifest-path".into(),
        manifest.to_string_lossy().into(),
        "--bench".into(),
        "lint_scan".into(),
    ];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, &ws, quiet)?;
    Ok(())
}

fn bench_check(ctx: &ProjectContext, args: &[String], quiet: bool) -> Result<()> {
    let manifest = bench_manifest(ctx)?;
    let mut cmd_args: Vec<String> = vec![
        "check".into(),
        "--manifest-path".into(),
        manifest.to_string_lossy().into(),
        "--benches".into(),
        "--tests".into(),
        "--examples".into(),
    ];
    cmd_args.extend(args.iter().cloned());
    let str_args: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, &ctx.root, quiet)?;
    Ok(())
}
