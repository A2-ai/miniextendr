//! Trait tag collision preflight.
//!
//! - MXL200: Two trait impl entries produce the same sanitized vtable name.

use std::collections::HashMap;

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        // Check for sanitized name collisions among trait impl entries.
        // The vtable static name is `__VTABLE_{TRAIT}_FOR_{TYPE_SANITIZED}`,
        // so two entries with the same (trait, sanitized_type) would collide.
        let mut seen: HashMap<(String, String), usize> = HashMap::new();

        for entry in &data.trait_impl_entries {
            let key = (
                entry.trait_name.to_uppercase(),
                entry.type_name_sanitized.to_uppercase(),
            );

            if let Some(first_line) = seen.get(&key) {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL200,
                        path,
                        entry.line,
                        format!(
                            "trait impl `{} for {}` produces the same sanitized vtable name \
                             `__VTABLE_{}_FOR_{}` as entry at line {}. \
                             These will collide at link time.",
                            entry.trait_name, entry.type_name, key.0, key.1, first_line,
                        ),
                    )
                    .with_help(
                        "Rename one of the types or use a newtype wrapper to avoid collision.",
                    ),
                );
            } else {
                seen.insert(key, entry.line);
            }
        }
    }
}
