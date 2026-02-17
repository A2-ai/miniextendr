//! Per-file module ↔ item consistency checks.
//!
//! - MXL001: Multiple `miniextendr_module!` macros in one file.
//! - MXL002: `#[miniextendr]` items found but no `miniextendr_module!`.
//! - MXL003: `miniextendr_module!` present but no `#[miniextendr]` items.
//! - MXL004: `#[miniextendr]` item not listed in `miniextendr_module!`.
//! - MXL005: Item in `miniextendr_module!` has no matching `#[miniextendr]`.
//! - MXL101: Duplicate entries in one `miniextendr_module!`.

use std::collections::HashMap;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        // MXL001: Multiple miniextendr_module! macros
        if data.module_macro_lines.len() > 1 {
            let lines = data
                .module_macro_lines
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            diagnostics.push(Diagnostic::new(
                LintCode::MXL001,
                path,
                data.module_macro_lines[0],
                format!(
                    "multiple miniextendr_module! macros found (at most 1 allowed per file). \
                     Found at lines: {}",
                    lines
                ),
            ));
        }

        let has_miniextendr = !data.miniextendr_items.is_empty();
        let has_module = !data.module_items.is_empty();

        // MXL002: #[miniextendr] items but no module
        if has_miniextendr && !has_module && data.module_macro_lines.is_empty() {
            diagnostics.push(Diagnostic::new(
                LintCode::MXL002,
                path,
                data.miniextendr_items[0].line,
                "#[miniextendr] items found but no miniextendr_module! in file".to_string(),
            ));
        }

        // MXL003: Module but no #[miniextendr] items
        if !has_miniextendr && has_module {
            diagnostics.push(Diagnostic::new(
                LintCode::MXL003,
                path,
                data.module_macro_lines.first().copied().unwrap_or(0),
                "miniextendr_module! present but no #[miniextendr] items in file".to_string(),
            ));
        }

        let miniextendr_set = data.miniextendr_items_set();
        let module_set = data.module_items_set();

        // MXL004: #[miniextendr] items not in module
        for item in &data.miniextendr_items {
            if !module_set.contains(item) {
                diagnostics.push(Diagnostic::new(
                    LintCode::MXL004,
                    path,
                    item.line,
                    format!(
                        "#[miniextendr] {} not listed in miniextendr_module!",
                        item.display()
                    ),
                ));
            }
        }

        // MXL005: Module items without #[miniextendr]
        for item in &data.module_items {
            if !miniextendr_set.contains(item) {
                diagnostics.push(Diagnostic::new(
                    LintCode::MXL005,
                    path,
                    item.line,
                    format!(
                        "{} listed in miniextendr_module! but has no #[miniextendr] attribute",
                        item.display()
                    ),
                ));
            }
        }

        // MXL101: Duplicate entries in one miniextendr_module!
        let mut seen: HashMap<String, usize> = HashMap::new();
        for item in &data.module_items {
            let key = item.dedup_key();
            if let Some(first_line) = seen.get(&key) {
                diagnostics.push(Diagnostic::new(
                    LintCode::MXL101,
                    path,
                    item.line,
                    format!(
                        "duplicate entry `{}` in miniextendr_module! (first at line {})",
                        item.display(),
                        first_line
                    ),
                ));
            } else {
                seen.insert(key, item.line);
            }
        }
    }
}
