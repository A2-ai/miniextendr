//! Trait registration checks.
//!
//! - MXL103: Generic concrete type in trait-ABI registration.
//! - MXL107: Missing `#[miniextendr] impl Trait for Type` for module entry.
//! - MXL108: Missing module entry for `#[miniextendr] impl Trait for Type`.

use std::collections::HashSet;

use crate::crate_index::{CrateIndex, LintKind};
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        // MXL103: Generic concrete type in trait-ABI registration
        for entry in &data.trait_impl_entries {
            if entry.is_generic {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL103,
                        path,
                        entry.line,
                        format!(
                            "`impl {} for {};` uses a generic type `{}`. \
                             Cross-package trait dispatch is limited for generic concrete types.",
                            entry.trait_name, entry.type_name, entry.type_name,
                        ),
                    )
                    .with_help("Consider wrapping in a named newtype and deriving ExternalPtr."),
                );
            }
        }

        // Collect trait impl keys from both sides for cross-checking
        let source_trait_impls: HashSet<String> = data
            .miniextendr_items
            .iter()
            .filter(|i| i.kind == LintKind::TraitImpl)
            .map(|i| i.name.clone())
            .collect();

        let module_trait_impls: HashSet<String> = data
            .module_items
            .iter()
            .filter(|i| i.kind == LintKind::TraitImpl)
            .map(|i| i.name.clone())
            .collect();

        // MXL107: Module has `impl Trait for Type;` but no matching attributed impl
        // (Enhanced version of MXL005 specifically for trait impls)
        for item in &data.module_items {
            if item.kind == LintKind::TraitImpl && !source_trait_impls.contains(&item.name) {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL107,
                        path,
                        item.line,
                        format!(
                            "module entry `impl {};` has no matching \
                             `#[miniextendr] impl {}` in source. The generated vtable \
                             static will be missing.",
                            item.name, item.name,
                        ),
                    )
                    .with_help(format!(
                        "Add `#[miniextendr]` to the trait impl or remove `impl {};` \
                         from miniextendr_module!.",
                        item.name
                    )),
                );
            }
        }

        // MXL108: Attributed trait impl but no module entry
        // (Enhanced version of MXL004 specifically for trait impls)
        for item in &data.miniextendr_items {
            if item.kind == LintKind::TraitImpl && !module_trait_impls.contains(&item.name) {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL108,
                        path,
                        item.line,
                        format!(
                            "`#[miniextendr] impl {}` generates wrappers but is not \
                             registered in miniextendr_module!.",
                            item.name,
                        ),
                    )
                    .with_help(format!("Add `impl {};` to miniextendr_module!.", item.name)),
                );
            }
        }
    }
}
