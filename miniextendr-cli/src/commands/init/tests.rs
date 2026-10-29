use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use super::*;
use crate::scaffold::{Dest, EMBEDDED, Render, dest_rel};

// region: test harness

/// Fresh scratch directory per test (removed by `Scratch::drop`).
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "miniextendr-cli-init-{}-{}-{tag}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        Scratch(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Independent re-implementation of the substitution used to compute the
/// EXPECTED file content from the on-disk template. Deliberately naive so
/// a bug in `scaffold::render` cannot cancel itself out in the test.
fn naive_substitute(template: &str, data: &[(&str, &str)], triple_only: bool) -> String {
    let mut out = template.to_string();
    for (key, value) in data {
        out = out.replace(&format!("{{{{{{{key}}}}}}}"), value);
        if !triple_only {
            out = out.replace(&format!("{{{{{key}}}}}"), value);
        }
    }
    out
}

/// Read a template from the repo checkout (NOT the embedded copy).
fn disk_template(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../minirextendr/inst")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel))
        .unwrap_or_else(|e| panic!("read {rel} from scaffold: {e}"))
}

fn walk_scaffold(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("readable scaffold dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            walk_scaffold(&path, out);
        } else {
            out.push(path);
        }
    }
}

// endregion

/// The #1351 regression test: the standalone scaffold's build-system
/// surface must equal the canonical minirextendr templates modulo the
/// documented substitutions, for every file in the plan — expected
/// content is computed from the templates ON DISK, so both the embedded
/// copies and the renderer are checked against the source of truth.
#[test]
fn package_scaffold_matches_canonical_templates() {
    let scratch = Scratch::new("pkg-surface");
    let root = scratch.path().join("my.pkg");
    let data = TemplateData::new("my.pkg");
    scaffold_rpkg_fresh(&root, &data).unwrap();

    let subs: Vec<(&str, &str)> = data.pairs().iter().map(|(k, v)| (*k, v.as_str())).collect();

    let mut problems = Vec::new();
    for entry in RPKG_PLAN {
        let dest = dest_rel(entry, &data).unwrap();
        let got = read(&root, &dest.to_string_lossy());
        let template = disk_template(&format!("templates/rpkg/{}", entry.template));
        let expected = match entry.render {
            Render::Mustache => naive_substitute(&template, &subs, false),
            Render::TripleOnly => naive_substitute(&template, &subs, true),
            Render::Verbatim => template,
            Render::IgnoreFilter => {
                let patterns: Vec<&str> = template
                    .lines()
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .collect();
                patterns.join("\n") + "\n"
            }
        };
        if got != expected {
            problems.push(format!(
                "{}: scaffolded content differs from canonical template {}",
                dest.display(),
                entry.template
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    // config.guess / config.sub come from the bundled scripts.
    for (src, dest) in scaffold::CONFIG_SCRIPTS {
        assert_eq!(read(&root, dest), disk_template(src), "{dest}");
    }
}

/// The obsolete four-mode build model must be gone from every scaffolded
/// file: no PREPARE_CRAN / NOT_CRAN / BUILD_CONTEXT, no
/// cargo-config.toml.in indirection, and no unresolved placeholders.
#[test]
fn no_obsolete_build_system_markers() {
    let scratch = Scratch::new("pkg-markers");
    let root = scratch.path().join("my.pkg");
    let data = TemplateData::new("my.pkg");
    scaffold_rpkg_fresh(&root, &data).unwrap();

    let mut files = Vec::new();
    walk_scaffold(&root, &mut files);
    assert!(files.len() > 20, "scaffold unexpectedly small: {files:?}");

    let mut problems = Vec::new();
    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy();
        if name == "cargo-config.toml.in" {
            problems.push(format!("{}: retired file scaffolded", file.display()));
        }
        let content = std::fs::read_to_string(file).expect("scaffolded files are UTF-8");
        for marker in ["PREPARE_CRAN", "NOT_CRAN", "BUILD_CONTEXT"] {
            if content.contains(marker) {
                problems.push(format!("{}: contains retired {marker}", file.display()));
            }
        }
        for placeholder in ["{{package", "{{crate_name", "{{rpkg_name", "{{year"] {
            if content.contains(placeholder) {
                problems.push(format!(
                    "{}: unresolved placeholder {placeholder}",
                    file.display()
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    // The canonical two-mode latch and its consumers must be present.
    let configure_ac = read(&root, "configure.ac");
    assert!(
        configure_ac.contains("inst/vendor.tar.xz"),
        "install-mode latch missing"
    );
    assert!(
        configure_ac.starts_with(&format!("AC_INIT([my.pkg], [{}])", scaffold::FRESH_VERSION)),
        "package name and version not substituted"
    );
    let cargo_toml = read(&root, "src/rust/Cargo.toml");
    assert!(cargo_toml.contains("name = \"my_pkg\""));
    assert!(
        cargo_toml.contains("miniextendr-api = { git ="),
        "source mode must declare the git dependency"
    );
    assert!(
        !cargo_toml.contains("../../vendor/miniextendr-api"),
        "retired hardcoded vendor path dependency"
    );
    let desc = read(&root, "DESCRIPTION");
    for (field, value) in scaffold::CONFIG_BUILD_FIELDS {
        assert!(
            desc.contains(&format!("{field}: {value}")),
            "{field} missing"
        );
    }
    assert!(desc.contains(&format!("SystemRequirements: {RUST_SYSTEM_REQUIREMENT}")));
    assert!(
        desc.contains(&format!("Depends: {}\n", scaffold::r_depends_entry())),
        "fresh DESCRIPTION missing the R version floor: {desc}"
    );
    assert!(read(&root, "NAMESPACE").contains("useDynLib(my.pkg, .registration = TRUE)"));
    assert!(root.join("vendor").is_dir());
}

#[cfg(unix)]
#[test]
fn build_hooks_are_executable() {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new("pkg-exec");
    let root = scratch.path().join("my.pkg");
    scaffold_rpkg_fresh(&root, &TemplateData::new("my.pkg")).unwrap();
    for rel in [
        "cleanup",
        "cleanup.win",
        "cleanup.ucrt",
        "configure.win",
        "configure.ucrt",
        "tools/config.guess",
        "tools/config.sub",
    ] {
        let mode = std::fs::metadata(root.join(rel))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o111,
            0o111,
            "{rel} must be executable (mode {mode:o})"
        );
    }
}

#[test]
fn monorepo_scaffold_matches_canonical_templates() {
    let scratch = Scratch::new("monorepo");
    let root = scratch.path().join("proj");
    init_monorepo(
        &root.to_string_lossy(),
        Some("my.proj"),
        None, // crate_name defaults to my-proj
        None, // rpkg_name defaults to my.proj
        true,
    )
    .unwrap();

    let data = TemplateData::new("my.proj")
        .with_crate("my-proj", "../../../my-proj", true)
        .with_rpkg("my.proj");
    let subs: Vec<(&str, &str)> = data.pairs().iter().map(|(k, v)| (*k, v.as_str())).collect();

    // Workspace root surface vs disk templates.
    for entry in MONOREPO_ROOT_PLAN {
        let dest = dest_rel(entry, &data).unwrap();
        let got = read(&root, &dest.to_string_lossy());
        let template = disk_template(&format!("templates/monorepo/{}", entry.template));
        let expected = match entry.render {
            Render::TripleOnly => naive_substitute(&template, &subs, true),
            _ => naive_substitute(&template, &subs, false),
        };
        assert_eq!(got, expected, "{} differs from canonical", dest.display());
    }

    assert_eq!(
        read(&root, "my.proj/.gitattributes"),
        disk_template("templates/monorepo/rpkg/gitattributes")
    );

    // Embedded R package uses the monorepo/rpkg templates (sibling path
    // dependency present, monorepo-flavored configure.ac).
    let rpkg_cargo = read(&root, "my.proj/src/rust/Cargo.toml");
    let manifest: toml::Value = toml::from_str(&rpkg_cargo).unwrap();
    assert_eq!(manifest["package"]["name"].as_str(), Some("my_proj-r"));
    assert_eq!(manifest["lib"]["name"].as_str(), Some("my_proj"));
    let core = &manifest["dependencies"]["core_library"];
    assert_eq!(core["package"].as_str(), Some("my-proj"));
    assert_eq!(core["path"].as_str(), Some("../../../my-proj"));
    let rust = read(&root, "my.proj/src/rust/lib.rs");
    assert!(rust.contains("\npub fn core_greeting()"));
    assert!(rust.contains("core_library::hello()"));
    let configure_ac = read(&root, "my.proj/configure.ac");
    assert_eq!(
        configure_ac,
        naive_substitute(
            &disk_template("templates/monorepo/rpkg/configure.ac"),
            &subs,
            false
        )
    );
    assert_eq!(
        read(&root, "my.proj/src/Makevars.win"),
        disk_template("templates/monorepo/rpkg/Makevars.win")
    );
    // bump-version.R keeps copy_template() semantics: {{{rpkg_name}}}
    // substituted, nothing else touched, no placeholder residue.
    let bump = read(&root, "tools/bump-version.R");
    assert!(bump.contains("tools/bump-version.R my.proj"));
    assert!(!bump.contains("{{{"));
}

#[test]
fn monorepo_rejects_colliding_names() {
    let scratch = Scratch::new("monorepo-collide");
    let root = scratch.path().join("proj");
    let err = init_monorepo(
        &root.to_string_lossy(),
        Some("mypkg"),
        Some("mypkg"),
        Some("mypkg"),
        true,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("must be different"), "{err}");
}

#[test]
fn init_use_merges_into_existing_package() {
    let scratch = Scratch::new("use-existing");
    let root = scratch.path().to_path_buf();
    std::fs::write(
        root.join("DESCRIPTION"),
        "Package: existing.pkg\n\
             Title: Existing\n\
             Version: 1.0.0\n\
             License: `use_mit_license()`, `use_gpl3_license()` or friends to pick a license\n\
             SystemRequirements: GNU make\n\
             Encoding: UTF-8\n",
    )
    .unwrap();
    std::fs::write(root.join("NAMESPACE"), "export(existing_fn)\n").unwrap();
    std::fs::write(root.join(".Rbuildignore"), "^custom$\n").unwrap();
    std::fs::write(
        root.join(".gitattributes"),
        "# User rules\n*.R text eol=lf\n",
    )
    .unwrap();

    init_use_rpkg(&root, true).unwrap();

    let desc = read(&root, "DESCRIPTION");
    assert!(desc.contains("Package: existing.pkg"));
    assert!(desc.contains("Version: 1.0.0"), "existing fields preserved");
    for (field, value) in scaffold::CONFIG_BUILD_FIELDS {
        assert!(desc.contains(&format!("{field}: {value}")), "{field}");
    }
    assert!(
        desc.contains("License: MIT + file LICENSE"),
        "placeholder replaced"
    );
    assert!(
        desc.contains(&format!(
            "SystemRequirements: GNU make, {RUST_SYSTEM_REQUIREMENT}"
        )),
        "Rust appended to existing SystemRequirements: {desc}"
    );
    assert!(
        desc.contains(&format!("Depends: {}\n", scaffold::r_depends_entry())),
        "R version floor added to a DESCRIPTION without Depends: {desc}"
    );

    // Existing NAMESPACE untouched; ignore file merged with dedupe.
    assert_eq!(read(&root, "NAMESPACE"), "export(existing_fn)\n");
    let rbuildignore = read(&root, ".Rbuildignore");
    assert!(
        rbuildignore.starts_with("^custom$\n"),
        "existing patterns kept"
    );
    assert!(rbuildignore.contains("^src/rust/target$"));
    let attributes = read(&root, ".gitattributes");
    assert!(attributes.starts_with("# User rules\n*.R text eol=lf\n"));
    assert!(attributes.contains("NAMESPACE -merge\n"));
    assert!(attributes.contains("R/*-wrappers.R -merge\n"));
    assert!(attributes.contains("configure -merge\n"));
    assert!(attributes.contains("man/*.Rd -merge\n"));
    // Idempotence: rerunning must not duplicate patterns.
    init_use_rpkg(&root, true).unwrap();
    let again = read(&root, ".Rbuildignore");
    assert_eq!(again.matches("^src/rust/target$").count(), 1);
    assert_eq!(read(&root, ".gitattributes"), attributes);

    // Build-system files landed and are canonical.
    // AC_INIT carries the package's own version, not the fresh one.
    assert!(read(&root, "configure.ac").starts_with("AC_INIT([existing.pkg], [1.0.0])"));
    assert!(root.join("src/rust/lib.rs").is_file());
    assert!(root.join("tools/build-html-reference.R").is_file());
    assert!(root.join("tools/lock-shape-check.R").is_file());
}

/// `init use` must never lower an existing R floor — merge semantics are
/// unit-tested on `desc_ensure_r_floor`; this checks the wiring end to end.
#[test]
fn init_use_keeps_higher_r_floor() {
    let scratch = Scratch::new("use-higher-floor");
    let root = scratch.path().to_path_buf();
    std::fs::write(
        root.join("DESCRIPTION"),
        "Package: floor.pkg\n\
             Version: 1.0.0\n\
             Depends: R (>= 4.6), methods\n\
             License: MIT + file LICENSE\n",
    )
    .unwrap();

    init_use_rpkg(&root, true).unwrap();

    let desc = read(&root, "DESCRIPTION");
    assert!(
        desc.contains("Depends: R (>= 4.6), methods\n"),
        "existing higher floor must survive init use: {desc}"
    );
}

#[test]
fn init_use_detects_monorepo_from_cargo_toml() {
    let scratch = Scratch::new("use-monorepo");
    let root = scratch.path().to_path_buf();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"my-core\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();

    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/lib.rs"), "pub fn add() {}\n").unwrap();
    init_use_monorepo(&root, None, None, true).unwrap();

    // Package name derived from the crate name (dashes -> dots), R
    // package scaffolded into a sibling subdirectory.
    let desc = read(&root, "rpkg/DESCRIPTION");
    assert!(desc.contains("Package: my.core"));
    let cargo = read(&root, "rpkg/src/rust/Cargo.toml");
    assert!(cargo.contains("core_library = { package = \"my-core\", path = \"../../..\" }"));
}

#[test]
fn init_use_virtual_workspace_selects_library_and_preserves_core() {
    let scratch = Scratch::new("virtual-use");
    let root = scratch.path();
    for name in ["first", "second"] {
        let member = root.join("crates").join(name);
        std::fs::create_dir_all(member.join("src")).unwrap();
        std::fs::write(member.join("src/lib.rs"), "pub fn add() {}\n").unwrap();
        std::fs::write(
            member.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
        )
        .unwrap();
    }
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    let error = init_use_monorepo(root, None, None, true).unwrap_err();
    assert!(error.to_string().contains("--crate-name"));
    assert!(!root.join("rpkg").exists());
    init_use_monorepo(root, None, Some("second"), true).unwrap();
    let manifest = read(root, "rpkg/src/rust/Cargo.toml");
    assert!(manifest.contains("name = \"second-r\""));
    assert!(manifest.contains("path = \"../../../crates/second\""));
    assert!(read(root, "rpkg/src/rust/lib.rs").contains("//     core_library::hello()"));
    assert_eq!(read(root, "crates/second/src/lib.rs"), "pub fn add() {}\n");
}

#[test]
fn validate_package_name_mirrors_r() {
    for good in ["my.pkg", "abc", "a2", "A.b.c"] {
        assert!(validate_package_name(good).is_ok(), "{good}");
    }
    for bad in ["a", "1pkg", "my-pkg", "pkg.", "my_pkg", ".pkg"] {
        assert!(validate_package_name(bad).is_err(), "{bad}");
    }
}

/// Guard: every plan template resolves inside the embedded manifest for
/// both template-type prefixes, and destinations are well-formed.
#[test]
fn plans_are_consistent_with_manifest() {
    for (prefix, plan) in [
        ("templates/rpkg", RPKG_PLAN),
        ("templates/monorepo/rpkg", RPKG_PLAN),
        ("templates/monorepo", MONOREPO_ROOT_PLAN),
    ] {
        for entry in plan {
            let rel = format!("{prefix}/{}", entry.template);
            assert!(
                EMBEDDED.iter().any(|(p, _)| *p == rel),
                "plan references unembedded template {rel}"
            );
            if let Dest::Path(p) = entry.dest {
                assert!(!p.starts_with('/'), "absolute dest {p}");
            }
        }
    }
}
