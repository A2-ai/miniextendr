use anyhow::Result;

use crate::bridge::run_command;
use crate::cli::{CargoBuildOpts, CargoCmd};
use crate::project::ProjectContext;

pub fn dispatch(cmd: &CargoCmd, ctx: &ProjectContext, quiet: bool) -> Result<()> {
    match cmd {
        CargoCmd::Init { name, edition } => cargo_init(ctx, name.as_deref(), edition, quiet),
        CargoCmd::New { name, lib, edition } => cargo_new(ctx, name, *lib, edition, quiet),
        CargoCmd::Add {
            dep,
            features,
            no_default_features,
            optional,
            rename,
            crate_path,
            git,
            branch,
            tag,
            rev,
            dev,
            build,
            dry_run,
        } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["add".to_string(), dep.clone()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            if let Some(f) = features {
                args.push("--features".into());
                args.push(f.clone());
            }
            if *no_default_features {
                args.push("--no-default-features".into());
            }
            if *optional {
                args.push("--optional".into());
            }
            if let Some(r) = rename {
                args.push("--rename".into());
                args.push(r.clone());
            }
            if let Some(p) = crate_path {
                args.push("--path".into());
                args.push(p.clone());
            }
            if let Some(g) = git {
                args.push("--git".into());
                args.push(g.clone());
            }
            if let Some(b) = branch {
                args.push("--branch".into());
                args.push(b.clone());
            }
            if let Some(t) = tag {
                args.push("--tag".into());
                args.push(t.clone());
            }
            if let Some(r) = rev {
                args.push("--rev".into());
                args.push(r.clone());
            }
            if *dev {
                args.push("--dev".into());
            }
            if *build {
                args.push("--build".into());
            }
            if *dry_run {
                args.push("--dry-run".into());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Rm {
            dep,
            dev,
            build,
            dry_run,
        } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["remove".to_string(), dep.clone()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            if *dev {
                args.push("--dev".into());
            }
            if *build {
                args.push("--build".into());
            }
            if *dry_run {
                args.push("--dry-run".into());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Update {
            dep,
            precise,
            dry_run,
        } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["update".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            if let Some(d) = dep {
                args.push(d.clone());
            }
            if let Some(p) = precise {
                args.push("--precise".into());
                args.push(p.clone());
            }
            if *dry_run {
                args.push("--dry-run".into());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Build { opts, jobs } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["build".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            push_build_opts(&mut args, opts);
            if let Some(j) = jobs {
                args.push("--jobs".into());
                args.push(j.to_string());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Check { opts } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["check".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            push_build_opts(&mut args, opts);
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Test {
            opts,
            no_run,
            test_args,
        } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["test".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            push_build_opts(&mut args, opts);
            if *no_run {
                args.push("--no-run".into());
            }
            if !test_args.is_empty() {
                args.push("--".into());
                args.extend(test_args.iter().cloned());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Clippy { opts, all_targets } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["clippy".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            push_build_opts(&mut args, opts);
            if *all_targets {
                args.push("--all-targets".into());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Fmt { check } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["fmt".to_string(), "--all".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            if *check {
                args.push("--".into());
                args.push("--check".into());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Doc {
            open,
            no_deps,
            opts,
        } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["doc".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            push_build_opts(&mut args, opts);
            if *no_deps {
                args.push("--no-deps".into());
            }
            if *open {
                args.push("--open".into());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Search { query, limit } => {
            let args = vec![
                "search".to_string(),
                query.clone(),
                "--limit".into(),
                limit.to_string(),
            ];
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Deps {
            depth,
            duplicates,
            invert,
        } => {
            let manifest = ctx.require_cargo_manifest()?;
            let mut args = vec!["tree".to_string()];
            args.push("--manifest-path".into());
            args.push(manifest.to_string_lossy().into());
            args.push("--depth".into());
            args.push(depth.to_string());
            if *duplicates {
                args.push("--duplicates".into());
            }
            if let Some(inv) = invert {
                args.push("--invert".into());
                args.push(inv.clone());
            }
            run_cargo(&args, &ctx.root, quiet)
        }
        CargoCmd::Clean => {
            let manifest = ctx.require_cargo_manifest()?;
            let args = vec![
                "clean".to_string(),
                "--manifest-path".into(),
                manifest.to_string_lossy().into(),
            ];
            run_cargo(&args, &ctx.root, quiet)
        }
    }
}

fn push_build_opts(args: &mut Vec<String>, opts: &CargoBuildOpts) {
    if opts.release {
        args.push("--release".into());
    }
    if let Some(f) = &opts.features {
        args.push("--features".into());
        args.push(f.clone());
    }
    if opts.no_default_features {
        args.push("--no-default-features".into());
    }
    if opts.all_features {
        args.push("--all-features".into());
    }
    if let Some(t) = &opts.target {
        args.push("--target".into());
        args.push(t.clone());
    }
    if opts.offline {
        args.push("--offline".into());
    }
}

fn run_cargo(args: &[String], cwd: &std::path::Path, quiet: bool) -> Result<()> {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_command("cargo", &str_args, cwd, quiet)?;
    Ok(())
}

fn cargo_init(ctx: &ProjectContext, name: Option<&str>, edition: &str, quiet: bool) -> Result<()> {
    let src_rust = ctx.root.join("src/rust");
    std::fs::create_dir_all(&src_rust)?;
    let mut args = vec!["init", "--lib", "--edition", edition];
    if let Some(n) = name {
        args.push("--name");
        args.push(n);
    }
    args.push(src_rust.to_str().unwrap_or("src/rust"));
    run_command("cargo", &args, &ctx.root, quiet)?;
    Ok(())
}

fn cargo_new(
    ctx: &ProjectContext,
    name: &str,
    lib: bool,
    edition: &str,
    quiet: bool,
) -> Result<()> {
    let mut args = vec!["new", name, "--edition", edition, "--vcs", "none"];
    if lib {
        args.push("--lib");
    }
    run_command("cargo", &args, &ctx.root, quiet)?;
    Ok(())
}
