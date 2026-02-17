//! Submodule wiring checks.
//!
//! - MXL006: Child module with `miniextendr_module!` missing `use child;` in parent.
//! - MXL202: Orphan `use child;` with no child module macro.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    let files_with_module: HashSet<&Path> = index.files_with_module().into_iter().collect();

    // Build map: parent_path -> module_uses
    let module_uses: HashMap<&Path, &Vec<String>> = index
        .file_data
        .iter()
        .filter(|(_, data)| !data.module_uses.is_empty())
        .map(|(path, data)| (path.as_path(), &data.module_uses))
        .collect();

    // Build map: file_path -> set of module names it's known by via #[path] redirect.
    // E.g., if parent has `#[path = "child_enabled.rs"] mod child;`, then
    // child_enabled.rs -> {"child"}.
    let mut path_redirect_names: HashMap<std::path::PathBuf, HashSet<String>> = HashMap::new();
    for (parent_path, parent_data) in &index.file_data {
        let parent_dir = match parent_path.parent() {
            Some(dir) => dir,
            None => continue,
        };
        for (mod_name, file_path) in &parent_data.path_redirected_mods {
            let target = parent_dir.join(file_path);
            path_redirect_names
                .entry(target)
                .or_default()
                .insert(mod_name.clone());
        }
    }

    // MXL006: Child module with miniextendr_module! missing `use child;` in parent
    for child_path in &files_with_module {
        let child_stem = match child_path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem.to_string(),
            None => continue,
        };

        // Skip root modules
        if child_stem == "lib" || child_stem == "mod" {
            continue;
        }

        let parent_dir = match child_path.parent() {
            Some(dir) => dir,
            None => continue,
        };

        // Determine the module names this file is known by:
        // 1. Its file stem (default)
        // 2. Any #[path] redirect names from parent files
        let mut known_names: HashSet<String> = HashSet::new();
        known_names.insert(child_stem.clone());
        if let Some(redirect_names) = path_redirect_names.get(*child_path) {
            known_names.extend(redirect_names.iter().cloned());
        }

        let potential_parents = [parent_dir.join("lib.rs"), parent_dir.join("mod.rs")];

        for parent_path in &potential_parents {
            if let Some(parent_uses) = module_uses.get(parent_path.as_path()) {
                // Check if parent has `use <any_known_name>;`
                let is_referenced = known_names.iter().any(|name| parent_uses.contains(name));
                if !is_referenced {
                    // Find the expected module name for the diagnostic
                    let expected_name = known_names
                        .iter()
                        .find(|n| *n != &child_stem)
                        .unwrap_or(&child_stem);
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL006,
                            *child_path,
                            0,
                            format!(
                                "module `{}` has its own miniextendr_module! but is not \
                                 referenced via `use {};` in {}'s miniextendr_module!. \
                                 Functions in {} will be invisible to R.",
                                expected_name,
                                expected_name,
                                parent_path.display(),
                                expected_name,
                            ),
                        )
                        .with_help(format!(
                            "Add `use {};` to the miniextendr_module! in {}",
                            expected_name,
                            parent_path.display()
                        )),
                    );
                }
            }
        }
    }

    // MXL202: Orphan `use child;` with no child module macro
    for (path, data) in &index.file_data {
        let parent_dir = match path.parent() {
            Some(dir) => dir,
            None => continue,
        };

        for use_name in &data.module_uses {
            // Check direct child files
            let child_candidates = [
                parent_dir.join(format!("{}.rs", use_name)),
                parent_dir.join(use_name).join("mod.rs"),
            ];

            let child_has_module = child_candidates.iter().any(|candidate| {
                index
                    .file_data
                    .get(candidate)
                    .is_some_and(|d| d.has_module_macro())
            });

            if child_has_module {
                continue;
            }

            // Also check #[path]-redirected modules: the `use child;` might reference
            // a module name that maps to a different file via #[path = "file.rs"]
            let redirected_has_module = data
                .path_redirected_mods
                .iter()
                .filter(|(mod_name, _)| mod_name == use_name)
                .any(|(_, file_path)| {
                    let target = parent_dir.join(file_path);
                    index
                        .file_data
                        .get(&target)
                        .is_some_and(|d| d.has_module_macro())
                });

            if redirected_has_module {
                continue;
            }

            let child_exists = child_candidates.iter().any(|c| c.exists());

            let message = if child_exists {
                format!(
                    "`use {};` in miniextendr_module! but module `{}` has no \
                     miniextendr_module! of its own",
                    use_name, use_name
                )
            } else {
                format!(
                    "`use {};` in miniextendr_module! but module file `{}.rs` \
                     does not exist",
                    use_name, use_name
                )
            };

            diagnostics.push(
                Diagnostic::new(LintCode::MXL202, path, 0, message).with_help(format!(
                    "Remove `use {};` from miniextendr_module! or add a \
                     miniextendr_module! to the child module",
                    use_name
                )),
            );
        }
    }
}
