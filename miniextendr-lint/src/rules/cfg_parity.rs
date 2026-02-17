//! cfg attribute parity between items and module entries.
//!
//! - MXL104: `#[cfg(...)]` mismatch between `#[miniextendr]` item and module entry.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
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
