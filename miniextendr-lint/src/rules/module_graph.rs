//! Module graph analysis.
//!
//! - MXL100: Duplicate module entrypoint symbol risk.
//! - MXL105: Unreachable module file with `miniextendr_module!`.
//! - MXL204: Multiple root-level module macros in crate graph.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    // Compute reachability first — several rules need it
    let reachable = compute_reachable(index);

    check_duplicate_module_names(index, &reachable, diagnostics);
    check_unreachable_modules(index, &reachable, diagnostics);
    check_multiple_roots(index, diagnostics);
}

fn compute_reachable(index: &CrateIndex) -> HashSet<PathBuf> {
    let root = find_root_file(&index.files);
    let mut reachable = HashSet::new();
    if let Some(root) = root {
        build_reachability(root, index, &mut reachable);
    }
    reachable
}

/// MXL100: Two miniextendr_module! blocks using the same `mod <name>` can
/// generate conflicting `R_init_<name>_miniextendr` symbols.
fn check_duplicate_module_names(
    index: &CrateIndex,
    reachable: &HashSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Build reverse map: file_path → (parent_path, mod_name) for multi-declared module names
    let path_redirect_origins = build_cfg_alternative_origins(index);

    // Only check reachable files — cfg-gated alternatives share names intentionally
    let mut names: HashMap<String, Vec<&Path>> = HashMap::new();

    for (path, data) in &index.file_data {
        if !reachable.contains(path) {
            continue;
        }
        if let Some(ref name) = data.module_name {
            names.entry(name.clone()).or_default().push(path);
        }
    }

    for (name, paths) in &names {
        if paths.len() > 1 {
            // Check if all duplicates are cfg-gated #[path] alternatives from the same
            // parent module. E.g., #[cfg(feature = "X")] #[path = "a.rs"] mod foo; and
            // #[cfg(not(feature = "X"))] #[path = "b.rs"] mod foo; — only one is active.
            if are_cfg_gated_alternatives(paths, &path_redirect_origins) {
                continue;
            }

            let locations = paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            for path in paths {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL100,
                        *path,
                        0,
                        format!(
                            "module name `{}` used in multiple miniextendr_module! blocks: {}. \
                             This generates conflicting `R_init_{}_miniextendr` symbols.",
                            name, locations, name,
                        ),
                    )
                    .with_help(
                        "Use unique module names or feature-gate one of the modules.".to_string(),
                    ),
                );
            }
        }
    }
}

/// Build a map: target_file → set of (parent_path, mod_name) for module names that
/// are declared multiple times in a parent (indicating cfg-gated alternatives).
fn build_cfg_alternative_origins(index: &CrateIndex) -> HashMap<PathBuf, Vec<(PathBuf, String)>> {
    let mut origins: HashMap<PathBuf, Vec<(PathBuf, String)>> = HashMap::new();

    for (parent_path, parent_data) in &index.file_data {
        let parent_dir = match parent_path.parent() {
            Some(dir) => dir,
            None => continue,
        };

        // Count total declarations per module name (across both direct and #[path])
        let mut name_counts: HashMap<&str, usize> = HashMap::new();
        for name in &parent_data.declared_child_mods {
            *name_counts.entry(name.as_str()).or_default() += 1;
        }
        for (mod_name, _) in &parent_data.path_redirected_mods {
            *name_counts.entry(mod_name.as_str()).or_default() += 1;
        }

        // For module names with 2+ declarations, record all their target files
        for (mod_name, count) in &name_counts {
            if *count < 2 {
                continue;
            }

            // Direct child targets
            if parent_data
                .declared_child_mods
                .iter()
                .any(|n| n == mod_name)
            {
                let sibling = parent_dir.join(format!("{mod_name}.rs"));
                origins
                    .entry(sibling)
                    .or_default()
                    .push((parent_path.clone(), mod_name.to_string()));

                let subdir_mod = parent_dir.join(mod_name).join("mod.rs");
                origins
                    .entry(subdir_mod)
                    .or_default()
                    .push((parent_path.clone(), mod_name.to_string()));
            }

            // #[path] redirect targets
            for (redir_name, file_path) in &parent_data.path_redirected_mods {
                if redir_name == mod_name {
                    let target = parent_dir.join(file_path);
                    origins
                        .entry(target)
                        .or_default()
                        .push((parent_path.clone(), mod_name.to_string()));
                }
            }
        }
    }

    origins
}

/// Returns true if all files in the group are cfg-gated alternatives for the same
/// (parent, mod_name) — meaning only one is active at compile time.
fn are_cfg_gated_alternatives(
    paths: &[&Path],
    origins: &HashMap<PathBuf, Vec<(PathBuf, String)>>,
) -> bool {
    // Collect the (parent, mod_name) pairs for each file
    let mut common_pairs: Option<HashSet<(PathBuf, String)>> = None;

    for path in paths {
        let path_buf = path.to_path_buf();
        let Some(file_origins) = origins.get(&path_buf) else {
            // File isn't part of a multi-declaration module — not a cfg alternative
            return false;
        };
        let pairs: HashSet<(PathBuf, String)> = file_origins.iter().cloned().collect();
        common_pairs = Some(match common_pairs {
            None => pairs,
            Some(existing) => existing.intersection(&pairs).cloned().collect(),
        });
    }

    // All files must share at least one common (parent, mod_name) origin
    common_pairs.is_some_and(|pairs| !pairs.is_empty())
}

/// MXL105: A file with miniextendr_module! that isn't reachable from the crate root.
fn check_unreachable_modules(
    index: &CrateIndex,
    reachable: &HashSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let root = match find_root_file(&index.files) {
        Some(r) => r,
        None => return,
    };

    for (path, data) in &index.file_data {
        if data.has_module_macro() && !reachable.contains(path) {
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL105,
                    path,
                    data.module_macro_lines.first().copied().unwrap_or(0),
                    format!(
                        "file has miniextendr_module! but is not reachable from crate root ({}). \
                         Registration code will be dead.",
                        root.display()
                    ),
                )
                .with_help("Add `mod <name>;` in a parent module or remove the dead file."),
            );
        }
    }
}

/// MXL204: Multiple root-level module macros.
fn check_multiple_roots(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    // Find files that look like roots (lib.rs or mod.rs at top level)
    let root_files: Vec<&Path> = index
        .files
        .iter()
        .filter(|path| {
            let stem = path.file_stem().and_then(|s| s.to_str());
            matches!(stem, Some("lib" | "mod"))
        })
        .filter(|path| {
            index
                .file_data
                .get(*path)
                .is_some_and(|d| d.has_module_macro())
        })
        .map(|p| p.as_path())
        .collect();

    if root_files.len() > 1 {
        let locations = root_files
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        for path in &root_files {
            diagnostics.push(Diagnostic::new(
                LintCode::MXL204,
                *path,
                0,
                format!(
                    "multiple root-level module macros found: {}. \
                     Expected at most one crate entrypoint.",
                    locations
                ),
            ));
        }
    }
}

fn find_root_file(files: &[PathBuf]) -> Option<&Path> {
    // Prefer lib.rs, then mod.rs
    files
        .iter()
        .find(|p| p.file_name().is_some_and(|n| n == "lib.rs"))
        .or_else(|| {
            files
                .iter()
                .find(|p| p.file_name().is_some_and(|n| n == "mod.rs"))
        })
        .map(|p| p.as_path())
}

fn build_reachability(file: &Path, index: &CrateIndex, reachable: &mut HashSet<PathBuf>) {
    if !reachable.insert(file.to_path_buf()) {
        return; // Already visited
    }

    let Some(data) = index.file_data.get(file) else {
        return;
    };

    let parent_dir = match file.parent() {
        Some(dir) => dir,
        None => return,
    };

    // Follow simple `mod child;` declarations
    for child_mod in &data.declared_child_mods {
        // Try child.rs
        let sibling = parent_dir.join(format!("{}.rs", child_mod));
        if index.file_data.contains_key(&sibling) {
            build_reachability(&sibling, index, reachable);
        }

        // Try child/mod.rs
        let subdir_mod = parent_dir.join(child_mod).join("mod.rs");
        if index.file_data.contains_key(&subdir_mod) {
            build_reachability(&subdir_mod, index, reachable);
        }
    }

    // Follow `#[path = "file.rs"] mod name;` declarations
    for (_mod_name, file_path) in &data.path_redirected_mods {
        let target = parent_dir.join(file_path);
        if index.file_data.contains_key(&target) {
            build_reachability(&target, index, reachable);
        }
    }
}
