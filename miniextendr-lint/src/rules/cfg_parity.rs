//! cfg attribute parity between items and module entries.
//!
//! - MXL104: `#[cfg(...)]` mismatch between `#[miniextendr]` item and module entry.
//! - MXL109: `#[cfg(...)]` mismatch between `mod` declaration and `use` entry.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    check_item_vs_module_entry(index, diagnostics);
    check_mod_vs_use(index, diagnostics);
}

/// MXL104: cfg parity between #[miniextendr] items and their module entries.
fn check_item_vs_module_entry(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    // Files in cfg-gated modules already have module-level feature gating,
    // so function-level cfg without matching module entry cfg is expected.
    let cfg_gated = index.cfg_gated_module_files();

    for (path, data) in &index.file_data {
        if cfg_gated.contains(path) {
            continue;
        }
        // Compare cfg attrs for each key that exists on both sides
        for (key, item_cfgs) in &data.item_cfgs {
            if let Some(module_cfgs) = data.module_entry_cfgs.get(key) {
                // Normalize and compare
                let mut item_sorted = item_cfgs.clone();
                item_sorted.sort();
                let mut module_sorted = module_cfgs.clone();
                module_sorted.sort();

                if item_sorted != module_sorted {
                    let item_str = if item_cfgs.is_empty() {
                        "(none)".to_string()
                    } else {
                        item_cfgs.join(", ")
                    };
                    let module_str = if module_cfgs.is_empty() {
                        "(none)".to_string()
                    } else {
                        module_cfgs.join(", ")
                    };

                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL104,
                            path,
                            0,
                            format!(
                                "cfg mismatch for `{}`: item has [{}] but module entry has [{}]",
                                key, item_str, module_str,
                            ),
                        )
                        .with_help(
                            "Mirror #[cfg(...)] attributes on both the item and its module entry.",
                        ),
                    );
                }
            } else {
                // Item has cfg but module entry doesn't
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL104,
                        path,
                        0,
                        format!(
                            "cfg mismatch for `{}`: item has cfg attributes [{}] but module \
                             entry has none",
                            key,
                            item_cfgs.join(", "),
                        ),
                    )
                    .with_help("Add matching #[cfg(...)] to the module entry."),
                );
            }
        }

        // Module entry has cfg but item doesn't
        for (key, module_cfgs) in &data.module_entry_cfgs {
            if !data.item_cfgs.contains_key(key) {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL104,
                        path,
                        0,
                        format!(
                            "cfg mismatch for `{}`: module entry has cfg attributes [{}] but \
                             item has none",
                            key,
                            module_cfgs.join(", "),
                        ),
                    )
                    .with_help("Add matching #[cfg(...)] to the #[miniextendr] item."),
                );
            }
        }
    }
}

/// MXL109: cfg parity between `mod` declarations and `use` entries in miniextendr_module!.
///
/// If `#[cfg(feature = "X")] mod child;` appears, then `use child;` in the same file's
/// miniextendr_module! should also have `#[cfg(feature = "X")]`, and vice versa.
fn check_mod_vs_use(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for use_name in &data.module_uses {
            let mod_cfgs = data.mod_decl_cfgs.get(use_name);
            let use_cfgs = data.use_entry_cfgs.get(use_name);

            match (mod_cfgs, use_cfgs) {
                (Some(mod_c), None) => {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL109,
                            path,
                            0,
                            format!(
                                "`mod {};` has cfg [{}] but `use {};` in miniextendr_module! has none",
                                use_name,
                                mod_c.join(", "),
                                use_name,
                            ),
                        )
                        .with_help(format!(
                            "Add matching #[cfg(...)] before `use {};` in miniextendr_module!.",
                            use_name,
                        )),
                    );
                }
                (None, Some(use_c)) => {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL109,
                            path,
                            0,
                            format!(
                                "`use {};` in miniextendr_module! has cfg [{}] but `mod {};` has none",
                                use_name,
                                use_c.join(", "),
                                use_name,
                            ),
                        )
                        .with_help(format!(
                            "Add matching #[cfg(...)] to the `mod {};` declaration.",
                            use_name,
                        )),
                    );
                }
                (Some(mod_c), Some(use_c)) => {
                    let mut mod_sorted = mod_c.clone();
                    mod_sorted.sort();
                    let mut use_sorted = use_c.clone();
                    use_sorted.sort();
                    if mod_sorted != use_sorted {
                        diagnostics.push(
                            Diagnostic::new(
                                LintCode::MXL109,
                                path,
                                0,
                                format!(
                                    "cfg mismatch for module `{}`: `mod` has [{}] but `use` has [{}]",
                                    use_name,
                                    mod_c.join(", "),
                                    use_c.join(", "),
                                ),
                            )
                            .with_help("Ensure #[cfg(...)] on `mod` and `use` entries match."),
                        );
                    }
                }
                (None, None) => {} // No cfg on either — fine
            }
        }
    }
}
