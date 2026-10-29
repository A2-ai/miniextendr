use super::*;

/// `minirextendr/inst/` in the repo checkout — the disk source the
/// embedded manifest must mirror.
fn inst_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../minirextendr/inst")
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("readable template dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Drift guard: every file under `minirextendr/inst/templates/` and
/// `minirextendr/inst/scripts/` must be embedded with identical content,
/// and every embedded key must still exist on disk. A template added,
/// renamed, or removed without updating [`EMBEDDED`] fails here.
#[test]
fn embedded_covers_disk() {
    let inst = inst_root();
    let mut disk_files = Vec::new();
    walk_files(&inst.join("templates"), &mut disk_files);
    walk_files(&inst.join("scripts"), &mut disk_files);

    let mut problems = Vec::new();
    let mut seen = Vec::new();
    for path in &disk_files {
        let rel = path
            .strip_prefix(&inst)
            .expect("under inst root")
            .to_string_lossy()
            .replace('\\', "/");
        seen.push(rel.clone());
        match EMBEDDED.iter().find(|(p, _)| *p == rel) {
            None => problems.push(format!(
                "{rel}: on disk but not embedded — add tpl!(\"{rel}\") to EMBEDDED"
            )),
            Some((_, embedded_content)) => {
                let disk_content = std::fs::read_to_string(path).expect("readable template");
                if disk_content != *embedded_content {
                    problems.push(format!(
                        "{rel}: embedded copy differs from disk (stale build?)"
                    ));
                }
            }
        }
    }
    for (rel, _) in EMBEDDED {
        if !seen.contains(&(*rel).to_string()) {
            problems.push(format!(
                "{rel}: embedded but missing on disk — template removed or renamed?"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn mustache_substitutes_triple_then_double() {
    let data = TemplateData::new("my.pkg").with_rpkg("rp");
    let out = render(
        "t",
        "x {{{rpkg_name}}} y {{package}} z {{package_rs}}",
        Render::Mustache,
        &data,
    )
    .unwrap();
    assert_eq!(out, "x rp y my.pkg z my_pkg");
}

#[test]
fn mustache_rejects_unknown_placeholders_batched() {
    let data = TemplateData::new("p");
    let err = render(
        "t",
        "{{unknown_a}} and {{unknown_b}} and {{unknown_a}}",
        Render::Mustache,
        &data,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("{{unknown_a}}"), "{err}");
    assert!(err.contains("{{unknown_b}}"), "{err}");
}

#[test]
fn mustache_ignores_non_placeholder_braces() {
    let data = TemplateData::new("p");
    // Rust format strings, shell ${...}, R code — none are placeholders.
    let content = "format!(\"Hello, {}!\", name); ${srcdir}; if (x) {{ }}";
    assert_eq!(
        render("t", content, Render::Mustache, &data).unwrap(),
        content
    );
}

#[test]
fn triple_only_preserves_double_braces() {
    let data = TemplateData::new("p").with_rpkg("rp");
    let out = render(
        "t",
        "{{{rpkg_name}}} keeps {{rpkg_name}}",
        Render::TripleOnly,
        &data,
    )
    .unwrap();
    assert_eq!(out, "rp keeps {{rpkg_name}}");
}

#[test]
fn ignore_filter_drops_blanks_and_comments() {
    let data = TemplateData::new("p");
    let out = render(
        "t",
        "# comment\n\npattern1\npattern2\n",
        Render::IgnoreFilter,
        &data,
    )
    .unwrap();
    assert_eq!(out, "pattern1\npattern2\n");
}

#[test]
fn merge_ignore_appends_missing_only() {
    let merged = merge_ignore_lines("existing\npattern1\n", &["pattern1", "pattern2"]);
    assert_eq!(merged, "existing\npattern1\npattern2\n");
}

/// Cross-language drift guard for the R version floor (#1366): the Rust
/// mirror, the R-side `MX_R_FLOOR` constant, and `rpkg/DESCRIPTION`'s
/// `Depends` line (the exemplar the floor is derived from) must all agree.
#[test]
fn r_floor_matches_minirextendr_and_rpkg() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");

    let utils_r = std::fs::read_to_string(repo_root.join("minirextendr/R/utils.R"))
        .expect("readable minirextendr/R/utils.R");
    let r_line = utils_r
        .lines()
        .find(|line| line.starts_with("MX_R_FLOOR"))
        .expect("MX_R_FLOOR defined in minirextendr/R/utils.R");
    assert_eq!(
        r_line.trim(),
        format!("MX_R_FLOOR <- \"{R_VERSION_FLOOR}\""),
        "R_VERSION_FLOOR drifted from minirextendr's MX_R_FLOOR"
    );

    let rpkg_desc = std::fs::read_to_string(repo_root.join("rpkg/DESCRIPTION"))
        .expect("readable rpkg/DESCRIPTION");
    assert!(
        rpkg_desc
            .lines()
            .any(|line| line.trim() == format!("Depends: {}", r_depends_entry())),
        "rpkg/DESCRIPTION's Depends floor drifted from R_VERSION_FLOOR"
    );
}

#[test]
fn desc_ensure_r_floor_adds_depends_when_absent() {
    let content = "Package: p\nVersion: 1.0.0\n";
    let out = desc_ensure_r_floor(content);
    assert!(
        out.contains(&format!("Depends: {}\n", r_depends_entry())),
        "{out}"
    );
}

#[test]
fn desc_ensure_r_floor_prepends_to_foreign_depends() {
    let content = "Package: p\nDepends: methods, utils\n";
    let out = desc_ensure_r_floor(content);
    assert!(
        out.contains(&format!("Depends: {}, methods, utils\n", r_depends_entry())),
        "{out}"
    );
}

#[test]
fn desc_ensure_r_floor_raises_lower_floor() {
    for lower in ["R (>= 4.3)", "R (>=4.3)", "R (> 4.3)", "R"] {
        let content = format!("Package: p\nDepends: {lower}, methods\n");
        let out = desc_ensure_r_floor(&content);
        assert!(
            out.contains(&format!("Depends: {}, methods\n", r_depends_entry())),
            "{lower}: {out}"
        );
    }
}

#[test]
fn desc_ensure_r_floor_keeps_equal_or_higher_floor() {
    for kept in [
        "R (>= 4.4)",
        "R (>= 4.4.0)",
        "R (>= 4.5)",
        "R (>= 4.6)",
        "R (== 4.4)",
    ] {
        let content = format!("Package: p\nDepends: {kept}, methods\n");
        assert_eq!(desc_ensure_r_floor(&content), content, "{kept}");
    }
}

#[test]
fn version_less_than_matches_compare_version() {
    assert!(version_less_than("4.4", "4.5"));
    assert!(version_less_than("4.4.9", "4.5"));
    assert!(version_less_than("4.5", "4.5.0"));
    assert!(!version_less_than("4.5", "4.5"));
    assert!(!version_less_than("4.10", "4.5"));
    assert!(!version_less_than("4.5.1", "4.5"));
}

#[test]
fn desc_set_field_replaces_with_continuations() {
    let content = "Package: p\nAuthors@R:\n    person(\"A\")\nLicense: old\n";
    let out = desc_set_field(content, "License", "MIT + file LICENSE");
    assert_eq!(
        out,
        "Package: p\nAuthors@R:\n    person(\"A\")\nLicense: MIT + file LICENSE\n"
    );
    let out = desc_set_field(&out, "NeedsCompilation", "yes");
    assert!(out.ends_with("NeedsCompilation: yes\n"));
    // Replacing a field with continuation lines drops the continuation.
    let out = desc_set_field(content, "Authors@R", "person(\"B\")");
    assert!(out.contains("Authors@R: person(\"B\")\n"));
    assert!(!out.contains("person(\"A\")"));
}

#[test]
fn year_is_plausible() {
    let year: i64 = current_year().parse().unwrap();
    assert!((2026..2200).contains(&year), "year = {year}");
}

#[test]
fn to_rust_name_replaces_dots_and_hyphens() {
    assert_eq!(to_rust_name("my.pkg-x"), "my_pkg_x");
}
