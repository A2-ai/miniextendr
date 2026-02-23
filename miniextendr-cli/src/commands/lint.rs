use anyhow::{Result, bail};

use crate::project::ProjectContext;

/// Run miniextendr-lint via cargo check on the project's Rust crate.
///
/// The lint runs as a build script; cargo check triggers it.
/// Lint output appears as cargo warnings.
pub fn run(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;

    // Run cargo check and capture output to filter for lint issues
    let output = std::process::Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(manifest)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        // Print full output on failure
        if !quiet {
            eprint!("{stderr}");
        }
        bail!("cargo check failed (see output above)");
    }

    // Filter for lint-specific warnings
    let lint_issues: Vec<&str> = stderr
        .lines()
        .filter(|line| {
            line.contains("miniextendr")
                && (line.contains("[MXL") || line.contains("miniextendr-lint"))
        })
        .collect();

    if lint_issues.is_empty() {
        if !quiet {
            println!("miniextendr-lint: no issues found");
        }
    } else {
        for line in &lint_issues {
            eprintln!("{line}");
        }
        eprintln!();
        bail!("miniextendr-lint found issues (see above)");
    }

    Ok(())
}
