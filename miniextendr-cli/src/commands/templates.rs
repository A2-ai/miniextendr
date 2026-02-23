use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::project::{ProjectContext, find_workspace_root};

/// Template source mappings: (relative path in inst/templates, source in rpkg).
const TEMPLATE_SOURCES: &[(&str, &str)] = &[
    ("rpkg/Makevars.in", "rpkg/src/Makevars.in"),
    ("rpkg/configure.ac", "rpkg/configure.ac"),
    ("rpkg/build.rs", "rpkg/src/rust/build.rs"),
    ("monorepo/rpkg/Makevars.in", "rpkg/src/Makevars.in"),
    ("monorepo/rpkg/configure.ac", "rpkg/configure.ac"),
    ("monorepo/rpkg/build.rs", "rpkg/src/rust/build.rs"),
];

pub fn dispatch(cmd: &crate::cli::TemplatesCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        crate::cli::TemplatesCmd::Check => templates_check(ctx, quiet),
        crate::cli::TemplatesCmd::Approve => templates_approve(ctx, quiet),
        crate::cli::TemplatesCmd::Sources => templates_sources(),
    }
}

/// Print template source mappings.
fn templates_sources() -> Result<()> {
    println!("# rel\tsrc");
    println!("# === R Package Template (rpkg/) ===");
    for (rel, src) in TEMPLATE_SOURCES {
        if rel.starts_with("monorepo/") {
            continue;
        }
        println!("{rel}\t{src}");
    }
    println!("# === Monorepo Template (monorepo/) ===");
    for (rel, src) in TEMPLATE_SOURCES {
        if rel.starts_with("monorepo/") {
            println!("{rel}\t{src}");
        }
    }
    Ok(())
}

/// Populate an upstream snapshot directory from rpkg source files.
fn populate_upstream(ws: &Path, dest: &Path) -> Result<()> {
    for (rel, src) in TEMPLATE_SOURCES {
        let src_path = ws.join(src);
        if !src_path.exists() {
            bail!("Template source not found: {} (for {})", src, rel);
        }
        let dst_path = dest.join(rel);
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&src_path, &dst_path)?;
    }
    Ok(())
}

/// Populate a snapshot of the actual template files (only those tracked in TEMPLATE_SOURCES).
fn populate_templates(ws: &Path, dest: &Path) -> Result<()> {
    let templates_root = ws.join("minirextendr/inst/templates");
    for (rel, _) in TEMPLATE_SOURCES {
        let template_file = templates_root.join(rel);
        if template_file.exists() {
            let dst_path = dest.join(rel);
            if let Some(parent) = dst_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(&template_file, &dst_path)?;
        }
    }
    Ok(())
}

fn patch_file(ws: &Path) -> PathBuf {
    ws.join("patches/templates.patch")
}

/// Verify: upstream snapshot + approved patch == inst/templates.
fn templates_check(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let ws = find_workspace_root(&ctx.root)
        .ok_or_else(|| anyhow::anyhow!("Workspace root not found."))?;

    let pf = patch_file(&ws);
    if !pf.exists() {
        bail!(
            "patches/templates.patch not found.\n\
             Run `miniextendr templates approve` first."
        );
    }

    let tmp = tempfile::tempdir_in(&ws)?;
    let upstream_dir = tmp.path().join("a");
    let templates_dir = tmp.path().join("b");
    std::fs::create_dir_all(&upstream_dir)?;
    std::fs::create_dir_all(&templates_dir)?;

    // Build upstream snapshot
    populate_upstream(&ws, &upstream_dir)?;

    // Apply approved patch (if non-empty)
    let patch_content = std::fs::read_to_string(&pf)?;
    if !patch_content.trim().is_empty() {
        let _ = std::process::Command::new("patch")
            .args([
                "-d",
                &upstream_dir.to_string_lossy(),
                "-p1",
                "--forward",
                "--batch",
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    let _ = stdin.write_all(patch_content.as_bytes());
                }
                child.wait()
            });
    }

    // Build template snapshot
    populate_templates(&ws, &templates_dir)?;

    // Compare
    let output = std::process::Command::new("diff")
        .args(["-ruN"])
        .arg(&upstream_dir)
        .arg(&templates_dir)
        .output()?;

    if !output.status.success() {
        let diff = String::from_utf8_lossy(&output.stdout);
        if !quiet {
            eprint!("{diff}");
            eprintln!("\nTemplates have drifted. Run `miniextendr templates approve` to accept.");
        }
        std::process::exit(1);
    }

    if !quiet {
        println!("Templates check passed: no unexpected drift.");
    }
    Ok(())
}

/// Accept current delta by regenerating patches/templates.patch.
fn templates_approve(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let ws = find_workspace_root(&ctx.root)
        .ok_or_else(|| anyhow::anyhow!("Workspace root not found."))?;

    let tmp = tempfile::tempdir_in(&ws)?;
    let upstream_dir = tmp.path().join("a");
    let templates_dir = tmp.path().join("b");
    std::fs::create_dir_all(&upstream_dir)?;
    std::fs::create_dir_all(&templates_dir)?;

    populate_upstream(&ws, &upstream_dir)?;
    populate_templates(&ws, &templates_dir)?;

    // Generate patch: diff exits 1 when differences exist (expected)
    let output = std::process::Command::new("diff")
        .args(["-ruN", "-U2"])
        .arg(&upstream_dir)
        .arg(&templates_dir)
        .output()?;

    let pf = patch_file(&ws);
    if let Some(parent) = pf.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&pf, &output.stdout)?;

    if !quiet {
        println!("Wrote {}", pf.display());
    }
    Ok(())
}

/// Temporary directory helper using the tempfile crate pattern.
mod tempfile {
    use std::path::PathBuf;

    pub struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        pub fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    pub fn tempdir_in(base: &std::path::Path) -> anyhow::Result<TempDir> {
        let name = format!("mx-templates-{}", std::process::id());
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&path)?;
        // Suppress unused warning for base — we use temp_dir for portability
        let _ = base;
        Ok(TempDir { path })
    }
}
