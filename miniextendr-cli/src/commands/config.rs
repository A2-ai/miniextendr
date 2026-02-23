use std::fmt;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::cli::ConfigCmd;
use crate::output::print_status;
use crate::project::ProjectContext;

#[derive(Serialize, Clone)]
struct Config {
    class_system: String,
    strict: bool,
    coerce: bool,
    features: Vec<String>,
    rust_version: String,
    vendor: bool,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class_system: {}", self.class_system)?;
        writeln!(f, "strict: {}", self.strict)?;
        writeln!(f, "coerce: {}", self.coerce)?;
        if self.features.is_empty() {
            writeln!(f, "features: []")?;
        } else {
            writeln!(f, "features: [{}]", self.features.join(", "))?;
        }
        writeln!(f, "rust_version: {}", self.rust_version)?;
        write!(f, "vendor: {}", self.vendor)
    }
}

fn defaults() -> Config {
    Config {
        class_system: "env".into(),
        strict: false,
        coerce: false,
        features: vec![],
        rust_version: "stable".into(),
        vendor: true,
    }
}

/// Parse a simple miniextendr.yml file (subset of YAML we actually use).
fn parse_config(content: &str) -> Config {
    let mut cfg = defaults();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            match key {
                "class_system" => cfg.class_system = value.to_string(),
                "strict" => cfg.strict = value.eq_ignore_ascii_case("true"),
                "coerce" => cfg.coerce = value.eq_ignore_ascii_case("true"),
                "rust_version" => cfg.rust_version = value.to_string(),
                "vendor" => cfg.vendor = value.eq_ignore_ascii_case("true"),
                "features" => {
                    // Handle inline list: [a, b, c] or simple value
                    let inner = value.trim_start_matches('[').trim_end_matches(']');
                    if inner.is_empty() {
                        cfg.features = vec![];
                    } else {
                        cfg.features = inner
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
                _ => {} // Ignore unknown keys
            }
        }
    }
    cfg
}

fn read_config(root: &Path) -> Config {
    let config_path = root.join("miniextendr.yml");
    if config_path.is_file() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => parse_config(&content),
            Err(_) => defaults(),
        }
    } else {
        defaults()
    }
}

pub fn dispatch(cmd: &ConfigCmd, ctx: &ProjectContext, _quiet: bool, json: bool) -> Result<()> {
    match cmd {
        ConfigCmd::Show => {
            let cfg = read_config(&ctx.root);
            if json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else {
                print_status(&cfg.to_string(), false);
            }
            Ok(())
        }
        ConfigCmd::Defaults => {
            let cfg = defaults();
            if json {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else {
                print_status(&cfg.to_string(), false);
            }
            Ok(())
        }
    }
}
