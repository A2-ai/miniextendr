//! Function visibility checks.
//!
//! - MXL106: Non-pub function has `/// @export` (contradictory).

use crate::crate_index::{CrateIndex, LintKind};
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for item in &data.module_items {
            if item.kind != LintKind::Function {
                continue;
            }

            let is_pub = data.fn_visibility.get(&item.name).copied().unwrap_or(false);
            let has_export_tag = data
                .fn_doc_tags
                .get(&item.name)
                .is_some_and(|tags| tags.iter().any(|t| t == "export"));

            // MXL106: Non-pub function with @export is contradictory
            if !is_pub && has_export_tag {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL106,
                        path,
                        item.line,
                        format!(
                            "function `{}` has `/// @export` but is not `pub`. \
                             The @export tag has no effect without `pub fn`.",
                            item.name,
                        ),
                    )
                    .with_help("Make the function `pub fn` to enable R export."),
                );
            }
        }
    }
}
