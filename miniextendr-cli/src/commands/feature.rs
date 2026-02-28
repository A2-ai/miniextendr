use anyhow::{Result, bail};

use crate::bridge::{rscript_eval, run_command};
use crate::cli::{FeatureCmd, FeatureDetectCmd, FeatureRuleCmd};
use crate::project::ProjectContext;

pub fn dispatch(cmd: &FeatureCmd, ctx: &ProjectContext, quiet: bool, json: bool) -> Result<()> {
    match cmd {
        FeatureCmd::Enable { name } => feature_enable(ctx, name, quiet),
        FeatureCmd::List => feature_list(ctx, json),
        FeatureCmd::Detect { cmd: detect_cmd } => match detect_cmd {
            FeatureDetectCmd::Init => feature_detect_init(ctx, quiet),
            FeatureDetectCmd::Update => feature_detect_update(ctx, quiet),
        },
        FeatureCmd::Rule { cmd: rule_cmd } => match rule_cmd {
            FeatureRuleCmd::Add {
                feature,
                detect,
                cargo_spec,
                optional_dep,
            } => feature_rule_add(
                ctx,
                feature,
                detect,
                cargo_spec.as_deref(),
                *optional_dep,
                quiet,
            ),
            FeatureRuleCmd::Remove { feature } => feature_rule_remove(ctx, feature, quiet),
            FeatureRuleCmd::List => feature_rule_list(ctx, json),
        },
    }
}

/// Enable a named feature by adding the cargo dependency/feature.
fn feature_enable(ctx: &ProjectContext, name: &str, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let manifest_str = manifest.to_string_lossy().to_string();

    match name {
        "r6" => {
            // R6 is an R-side class system, just need to suggest R6 in DESCRIPTION
            add_r_suggests(ctx, "R6", quiet)?;
            if !quiet {
                println!("Enabled R6 class system. Add `#[miniextendr(r6)]` to impl blocks.");
            }
        }
        "s4" => {
            add_r_depends(ctx, "methods", quiet)?;
            if !quiet {
                println!("Enabled S4 class system. Add `#[miniextendr(s4)]` to impl blocks.");
            }
        }
        "s7" => {
            add_r_suggests(ctx, "S7", quiet)?;
            if !quiet {
                println!("Enabled S7 class system. Add `#[miniextendr(s7)]` to impl blocks.");
            }
        }
        "serde" => {
            run_command(
                "cargo",
                &[
                    "add",
                    "--manifest-path",
                    &manifest_str,
                    "serde",
                    "--features",
                    "derive",
                ],
                &ctx.root,
                quiet,
            )?;
            // Enable miniextendr-api serde feature
            enable_cargo_feature(ctx, "serde", quiet)?;
            if !quiet {
                println!("Enabled serde. Use `#[derive(Serialize, Deserialize)]` on structs.");
            }
        }
        "vctrs" => {
            add_r_suggests(ctx, "vctrs", quiet)?;
            enable_cargo_feature(ctx, "vctrs", quiet)?;
            if !quiet {
                println!("Enabled vctrs integration.");
            }
        }
        "rayon" => {
            run_command(
                "cargo",
                &["add", "--manifest-path", &manifest_str, "rayon"],
                &ctx.root,
                quiet,
            )?;
            enable_cargo_feature(ctx, "rayon", quiet)?;
            if !quiet {
                println!("Enabled rayon parallelism.");
            }
        }
        "build-rs" => {
            let build_rs = ctx.root.join("src/rust/build.rs");
            if !build_rs.exists() {
                std::fs::write(
                    &build_rs,
                    "fn main() {\n    println!(\"cargo::rerun-if-changed=lib.rs\");\n}\n",
                )?;
            }
            if !quiet {
                println!("Created src/rust/build.rs");
            }
        }
        "knitr" | "rmarkdown" | "quarto" | "feature-detection" => {
            if !quiet {
                println!(
                    "Feature '{name}' requires R-side setup.\n\
                     Use `Rscript -e 'minirextendr::use_miniextendr_{name}()'` or set up manually."
                );
            }
        }
        other => {
            // Try as a cargo feature name
            enable_cargo_feature(ctx, other, quiet)?;
        }
    }
    Ok(())
}

/// List cargo features from Cargo.toml.
fn feature_list(ctx: &ProjectContext, json: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let content = std::fs::read_to_string(manifest)?;
    let toml: toml::Value = content.parse()?;

    let features = toml
        .get("features")
        .and_then(|f| f.as_table())
        .cloned()
        .unwrap_or_default();

    // Also gather optional deps
    let optional_deps: Vec<String> = toml
        .get("dependencies")
        .and_then(|d| d.as_table())
        .map(|deps| {
            deps.iter()
                .filter(|(_, v)| v.get("optional").and_then(|o| o.as_bool()).unwrap_or(false))
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default();

    if json {
        let mut map = serde_json::Map::new();
        for (name, deps) in &features {
            let deps_vec: Vec<String> = deps
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            map.insert(name.clone(), serde_json::json!(deps_vec));
        }
        if !optional_deps.is_empty() {
            map.insert("_optional_deps".into(), serde_json::json!(optional_deps));
        }
        println!("{}", serde_json::to_string_pretty(&map)?);
    } else {
        println!("Cargo features:");
        for (name, deps) in &features {
            let deps_str = deps
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            if deps_str.is_empty() {
                println!("  {name}");
            } else {
                println!("  {name} = [{deps_str}]");
            }
        }
        if !optional_deps.is_empty() {
            println!("\nOptional dependencies:");
            for dep in &optional_deps {
                println!("  {dep}");
            }
        }
    }
    Ok(())
}

/// Create detect-features.R infrastructure.
fn feature_detect_init(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    let tools_dir = ctx.root.join("tools");
    std::fs::create_dir_all(&tools_dir)?;

    let detect_file = tools_dir.join("detect-features.R");
    if detect_file.exists() {
        if !quiet {
            println!("tools/detect-features.R already exists.");
        }
        return Ok(());
    }

    let content = r#"# Feature detection rules for configure-time feature gating.
# Each rule maps a Cargo feature to a detection expression.
# Called by configure to determine which features to enable.

detect_features <- function() {
  features <- character()
  # Add rules below with add_rule():
  # features <- add_rule(features, "feature_name", detect_expr = TRUE)
  features
}

add_rule <- function(features, name, detect_expr) {
  if (isTRUE(detect_expr)) {
    features <- c(features, name)
  }
  features
}
"#;
    std::fs::write(&detect_file, content)?;
    if !quiet {
        println!("Created tools/detect-features.R");
    }
    Ok(())
}

/// Update feature detection by calling the R helper.
fn feature_detect_update(ctx: &ProjectContext, quiet: bool) -> Result<()> {
    rscript_eval("minirextendr::update_feature_detection()", &ctx.root, quiet)?;
    if !quiet {
        println!("Updated feature detection helpers.");
    }
    Ok(())
}

/// Add a feature detection rule to tools/detect-features.R.
fn feature_rule_add(
    ctx: &ProjectContext,
    feature: &str,
    detect: &str,
    _cargo_spec: Option<&str>,
    _optional_dep: bool,
    quiet: bool,
) -> Result<()> {
    let detect_file = ctx.root.join("tools/detect-features.R");
    if !detect_file.exists() {
        feature_detect_init(ctx, true)?;
    }

    let mut content = std::fs::read_to_string(&detect_file)?;

    // Add rule before the closing of detect_features function
    let rule_line =
        format!("  features <- add_rule(features, \"{feature}\", detect_expr = {detect})\n");

    // Insert before "  features\n}" (the return + closing brace)
    if let Some(pos) = content.rfind("  features\n}") {
        content.insert_str(pos, &rule_line);
    } else {
        // Fallback: append before last }
        if let Some(pos) = content.rfind('}') {
            content.insert_str(pos, &rule_line);
        }
    }

    std::fs::write(&detect_file, content)?;
    if !quiet {
        println!("Added feature rule: {feature}");
    }
    Ok(())
}

/// Remove a feature detection rule.
fn feature_rule_remove(ctx: &ProjectContext, feature: &str, quiet: bool) -> Result<()> {
    let detect_file = ctx.root.join("tools/detect-features.R");
    if !detect_file.exists() {
        bail!("tools/detect-features.R not found. Run `miniextendr feature detect init` first.");
    }

    let content = std::fs::read_to_string(&detect_file)?;
    let pattern = format!("\"{}\"", feature);
    let filtered: String = content
        .lines()
        .filter(|line| !line.contains(&pattern) || !line.contains("add_rule"))
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(&detect_file, filtered)?;

    if !quiet {
        println!("Removed feature rule: {feature}");
    }
    Ok(())
}

/// List feature detection rules.
fn feature_rule_list(ctx: &ProjectContext, json: bool) -> Result<()> {
    let detect_file = ctx.root.join("tools/detect-features.R");
    if !detect_file.exists() {
        if json {
            println!("[]");
        } else {
            println!("No feature detection rules (tools/detect-features.R not found).");
        }
        return Ok(());
    }

    let content = std::fs::read_to_string(&detect_file)?;
    let rules: Vec<String> = content
        .lines()
        .filter(|line| line.contains("add_rule"))
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.trim().to_string())
        .collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&rules)?);
    } else if rules.is_empty() {
        println!("No feature detection rules defined.");
    } else {
        println!("Feature detection rules:");
        for rule in &rules {
            println!("  {rule}");
        }
    }
    Ok(())
}

// --- Helpers ---

/// Enable a feature in the [features] section by adding it to default or as standalone.
fn enable_cargo_feature(ctx: &ProjectContext, feature: &str, quiet: bool) -> Result<()> {
    let manifest = ctx.require_cargo_manifest()?;
    let content = std::fs::read_to_string(manifest)?;
    let mut toml: toml::Value = content.parse()?;

    // Check if feature already exists
    let has_feature = toml
        .get("features")
        .and_then(|f| f.as_table())
        .is_some_and(|t| t.contains_key(feature));

    if has_feature {
        if !quiet {
            println!("Feature '{feature}' already defined in Cargo.toml");
        }
        return Ok(());
    }

    // Add feature if it maps to miniextendr-api
    let feature_def = format!("miniextendr-api/{feature}");
    if let Some(features) = toml.get_mut("features").and_then(|f| f.as_table_mut()) {
        features.insert(
            feature.to_string(),
            toml::Value::Array(vec![toml::Value::String(feature_def)]),
        );
    }

    std::fs::write(manifest, toml.to_string())?;
    if !quiet {
        println!("Added feature '{feature}' to Cargo.toml");
    }
    Ok(())
}

/// Add a package to Suggests in DESCRIPTION.
fn add_r_suggests(ctx: &ProjectContext, pkg: &str, quiet: bool) -> Result<()> {
    add_desc_field(ctx, "Suggests", pkg, quiet)
}

/// Add a package to Depends in DESCRIPTION.
fn add_r_depends(ctx: &ProjectContext, pkg: &str, quiet: bool) -> Result<()> {
    add_desc_field(ctx, "Depends", pkg, quiet)
}

fn add_desc_field(ctx: &ProjectContext, field: &str, pkg: &str, quiet: bool) -> Result<()> {
    let desc_path = ctx.root.join("DESCRIPTION");
    if !desc_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&desc_path)?;
    let prefix = format!("{field}:");

    // Check if field exists and if pkg is already listed
    let mut found_field = false;
    for line in content.lines() {
        if line.starts_with(&prefix) {
            found_field = true;
            if line.contains(pkg) {
                if !quiet {
                    println!("{pkg} already in {field}");
                }
                return Ok(());
            }
        }
    }

    let new_content = if found_field {
        // Append to existing field
        content.replace(&prefix.to_string(), &format!("{prefix} {pkg},"))
    } else {
        // Add new field
        format!("{content}{field}: {pkg}\n")
    };

    std::fs::write(&desc_path, new_content)?;
    if !quiet {
        println!("Added {pkg} to {field} in DESCRIPTION");
    }
    Ok(())
}
