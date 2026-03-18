//! Strip unnecessary directories and TOML sections from vendored crates

use anyhow::{Context, Result};
use std::path::Path;

/// Directories to remove from vendored crates
const STRIP_DIRS: &[&str] = &[
    "tests", "benches", "examples", ".github", ".circleci", "docs", "ci", "target",
];

/// TOML sections to remove (they reference stripped directories)
const STRIP_TOML_SECTIONS: &[&str] = &["[[bench]]", "[[test]]", "[[example]]", "[dev-dependencies"];

/// Strip all vendored crates in a vendor directory
pub fn strip_vendor_dir(vendor_dir: &Path, verbose: bool) -> Result<()> {
    for entry in std::fs::read_dir(vendor_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            strip_crate_dir(&entry.path(), verbose)?;
        }
    }
    Ok(())
}

/// Strip a single vendored crate directory
fn strip_crate_dir(crate_dir: &Path, verbose: bool) -> Result<()> {
    // Remove unwanted directories
    for dir_name in STRIP_DIRS {
        let dir_path = crate_dir.join(dir_name);
        if dir_path.exists() {
            std::fs::remove_dir_all(&dir_path).with_context(|| {
                format!(
                    "failed to remove {}/{}",
                    crate_dir.display(),
                    dir_name
                )
            })?;
            if verbose {
                eprintln!(
                    "  Stripped {}/{}",
                    crate_dir.file_name().unwrap().to_string_lossy(),
                    dir_name
                );
            }
        }
    }

    // Remove hidden files (except .cargo-checksum.json)
    for entry in std::fs::read_dir(crate_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') && name_str != ".cargo-checksum.json" {
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(entry.path())?;
            } else {
                std::fs::remove_file(entry.path())?;
            }
        }
    }

    // Clean TOML sections that reference stripped directories
    let cargo_toml = crate_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        strip_toml_sections(&cargo_toml)?;
    }

    Ok(())
}

/// Remove [[bench]], [[test]], [[example]], and [dev-dependencies] sections from Cargo.toml
fn strip_toml_sections(cargo_toml: &Path) -> Result<()> {
    let content = std::fs::read_to_string(cargo_toml)?;
    let mut output_lines = Vec::new();
    let mut in_stripped_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if this line starts a section header
        if trimmed.starts_with('[') {
            // Check if it's a section we want to strip
            in_stripped_section = STRIP_TOML_SECTIONS
                .iter()
                .any(|s| trimmed.starts_with(s));

            if in_stripped_section {
                continue;
            }
        }

        if in_stripped_section {
            // Check if we've hit a new non-stripped section
            if trimmed.starts_with('[')
                && !STRIP_TOML_SECTIONS
                    .iter()
                    .any(|s| trimmed.starts_with(s))
            {
                in_stripped_section = false;
                output_lines.push(line.to_string());
            }
            // Otherwise skip the line (still in stripped section)
        } else {
            output_lines.push(line.to_string());
        }
    }

    // Remove trailing blank lines
    while output_lines.last().is_some_and(|l| l.trim().is_empty()) {
        output_lines.pop();
    }
    output_lines.push(String::new()); // single trailing newline

    std::fs::write(cargo_toml, output_lines.join("\n"))?;
    Ok(())
}
