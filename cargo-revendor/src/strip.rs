//! Strip unnecessary directories and TOML sections from vendored crates

use crate::Verbosity;
use anyhow::{Context, Result};
use std::path::Path;

/// Configuration for what to strip from vendored crates
#[derive(Clone, Debug)]
pub struct StripConfig {
    pub tests: bool,
    pub benches: bool,
    pub examples: bool,
    pub bins: bool,
}

impl StripConfig {
    /// Strip everything
    pub fn all() -> Self {
        Self {
            tests: true,
            benches: true,
            examples: true,
            bins: true,
        }
    }

    /// Whether any stripping is enabled
    pub fn any(&self) -> bool {
        self.tests || self.benches || self.examples || self.bins
    }

    /// Get directory names to strip
    fn dirs_to_strip(&self) -> Vec<&'static str> {
        let mut dirs = vec![".github", ".circleci", "ci", "target"];
        if self.tests {
            dirs.push("tests");
        }
        if self.benches {
            dirs.push("benches");
        }
        if self.examples {
            dirs.push("examples");
        }
        // bins: no standard dir to strip (binaries are in src/bin/)
        dirs
    }

    /// Get TOML section prefixes to strip
    fn toml_sections_to_strip(&self) -> Vec<&'static str> {
        let mut sections = Vec::new();
        if self.tests {
            sections.push("[[test]]");
            sections.push("[dev-dependencies");
        }
        if self.benches {
            sections.push("[[bench]]");
        }
        if self.examples {
            sections.push("[[example]]");
        }
        if self.bins {
            sections.push("[[bin]]");
        }
        sections
    }
}

/// Strip all vendored crates in a vendor directory.
/// Returns list of stripped items for reporting.
pub fn strip_vendor_dir(
    vendor_dir: &Path,
    config: &StripConfig,
    v: Verbosity,
) -> Result<Vec<String>> {
    let mut stripped = Vec::new();
    for entry in std::fs::read_dir(vendor_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let items = strip_crate_dir(&entry.path(), config, v)?;
            stripped.extend(items);
        }
    }
    Ok(stripped)
}

/// Strip a single vendored crate directory
fn strip_crate_dir(crate_dir: &Path, config: &StripConfig, v: Verbosity) -> Result<Vec<String>> {
    let crate_name = crate_dir.file_name().unwrap().to_string_lossy().to_string();
    let mut stripped = Vec::new();

    // Remove configured directories
    for dir_name in config.dirs_to_strip() {
        let dir_path = crate_dir.join(dir_name);
        if dir_path.exists() {
            std::fs::remove_dir_all(&dir_path).with_context(|| {
                format!("failed to remove {}/{}", crate_dir.display(), dir_name)
            })?;
            if v.debug() {
                eprintln!("  Stripped {}/{}", crate_name, dir_name);
            }
            stripped.push(format!("{}/{}", crate_name, dir_name));
        }
    }

    // Remove hidden files/dirs (except .cargo-checksum.json)
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
        let sections = config.toml_sections_to_strip();
        if !sections.is_empty() {
            strip_toml_sections(&cargo_toml, &sections)?;
        }
    }

    Ok(stripped)
}

/// Remove specified TOML sections from Cargo.toml
fn strip_toml_sections(cargo_toml: &Path, sections_to_strip: &[&str]) -> Result<()> {
    let content = std::fs::read_to_string(cargo_toml)?;
    let mut output_lines = Vec::new();
    let mut in_stripped_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_stripped_section = sections_to_strip.iter().any(|s| trimmed.starts_with(s));

            if in_stripped_section {
                continue;
            }
        }

        if in_stripped_section {
            if trimmed.starts_with('[') && !sections_to_strip.iter().any(|s| trimmed.starts_with(s))
            {
                in_stripped_section = false;
                output_lines.push(line.to_string());
            }
        } else {
            output_lines.push(line.to_string());
        }
    }

    // Remove trailing blank lines
    while output_lines.last().is_some_and(|l| l.trim().is_empty()) {
        output_lines.pop();
    }
    output_lines.push(String::new());

    std::fs::write(cargo_toml, output_lines.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_toml(dir: &Path, content: &str) -> std::path::PathBuf {
        let path = dir.join("Cargo.toml");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn strip_toml_removes_bench_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[[bench]]\nname = \"my_bench\"\nharness = false\n\n[lib]\nname = \"foo\"\n",
        );
        strip_toml_sections(&path, &["[[bench]]"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[bench]]"));
        assert!(!result.contains("my_bench"));
        assert!(result.contains("[dependencies]"));
        assert!(result.contains("[lib]"));
    }

    #[test]
    fn strip_toml_removes_test_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[[test]]\nname = \"integration\"\n\n[dependencies]\nbar = \"1\"\n",
        );
        strip_toml_sections(&path, &["[[test]]"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[test]]"));
        assert!(result.contains("[dependencies]"));
    }

    #[test]
    fn strip_toml_removes_dev_dependencies() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n\n[features]\ndefault = []\n",
        );
        strip_toml_sections(&path, &["[dev-dependencies"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[dev-dependencies]"));
        assert!(!result.contains("criterion"));
        assert!(result.contains("[features]"));
    }

    #[test]
    fn strip_toml_removes_example_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[[example]]\nname = \"demo\"\n\n[dependencies]\nbar = \"1\"\n",
        );
        strip_toml_sections(&path, &["[[example]]"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[example]]"));
        assert!(result.contains("[dependencies]"));
    }

    #[test]
    fn strip_toml_preserves_regular_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[dependencies]\nbar = \"1\"\n\n[build-dependencies]\ncc = \"1\"\n\n[features]\ndefault = []\n\n[profile.release]\nopt-level = 3\n",
        );
        strip_toml_sections(&path, &["[[test]]", "[[bench]]", "[dev-dependencies"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(result.contains("[package]"));
        assert!(result.contains("[dependencies]"));
        assert!(result.contains("[build-dependencies]"));
        assert!(result.contains("[features]"));
        assert!(result.contains("[profile.release]"));
    }

    #[test]
    fn strip_toml_handles_multiple_stripped_sections() {
        let dir = TempDir::new().unwrap();
        let path = write_toml(
            dir.path(),
            "[package]\nname = \"foo\"\n\n[[test]]\nname = \"t1\"\n\n[[test]]\nname = \"t2\"\n\n[[bench]]\nname = \"b1\"\n\n[dev-dependencies]\ncriterion = \"0.5\"\n\n[features]\ndefault = []\n",
        );
        strip_toml_sections(&path, &["[[test]]", "[[bench]]", "[dev-dependencies"]).unwrap();
        let result = std::fs::read_to_string(&path).unwrap();
        assert!(!result.contains("[[test]]"));
        assert!(!result.contains("[[bench]]"));
        assert!(!result.contains("[dev-dependencies]"));
        assert!(result.contains("[features]"));
    }

    #[test]
    fn strip_crate_dir_removes_directories() {
        let dir = TempDir::new().unwrap();
        let crate_dir = dir.path().join("mycrate");
        std::fs::create_dir_all(crate_dir.join("tests")).unwrap();
        std::fs::create_dir_all(crate_dir.join("benches")).unwrap();
        std::fs::create_dir_all(crate_dir.join("examples")).unwrap();
        std::fs::create_dir_all(crate_dir.join(".github")).unwrap();
        std::fs::create_dir_all(crate_dir.join("src")).unwrap();
        std::fs::write(crate_dir.join("tests/test.rs"), "").unwrap();
        std::fs::write(crate_dir.join("src/lib.rs"), "").unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(crate_dir.join(".cargo-checksum.json"), "{\"files\":{}}").unwrap();

        strip_crate_dir(&crate_dir, &StripConfig::all(), Verbosity(0)).unwrap();

        assert!(!crate_dir.join("tests").exists());
        assert!(!crate_dir.join("benches").exists());
        assert!(!crate_dir.join("examples").exists());
        assert!(!crate_dir.join(".github").exists());
        assert!(crate_dir.join("src").exists());
        assert!(crate_dir.join(".cargo-checksum.json").exists());
    }
}
