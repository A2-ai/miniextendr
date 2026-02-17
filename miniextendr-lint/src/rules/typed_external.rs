//! ExternalPtr / TypedExternal contract checks.
//!
//! - MXL007: `impl Type;` entry requires ExternalPtr derive or TypedExternal impl.
//! - MXL102: `impl Trait for Type;` also requires TypedExternal for cross-package dispatch.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    let typed_external_types = index.all_typed_external_types();

    for (path, data) in &index.file_data {
        // MXL007: impl Type; without ExternalPtr/TypedExternal
        for (type_name, line) in &data.impl_type_entries {
            if !typed_external_types.contains(type_name) {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL007,
                        path,
                        *line,
                        format!(
                            "struct `{}` is used in `impl {};` but does not derive \
                             ExternalPtr or implement TypedExternal.",
                            type_name, type_name,
                        ),
                    )
                    .with_help(format!(
                        "Add `#[derive(ExternalPtr)]` to the `{}` struct definition.",
                        type_name
                    )),
                );
            }
        }

        // MXL102: impl Trait for Type; also needs TypedExternal
        for entry in &data.trait_impl_entries {
            // Only check non-generic simple types (generic types can't derive ExternalPtr)
            if entry.is_generic {
                continue;
            }
            if !typed_external_types.contains(&entry.type_name) {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL102,
                        path,
                        entry.line,
                        format!(
                            "`impl {} for {};` requires `{}` to have ExternalPtr derive or \
                             TypedExternal impl for type-safe cross-package dispatch.",
                            entry.trait_name, entry.type_name, entry.type_name,
                        ),
                    )
                    .with_help(format!(
                        "Add `#[derive(ExternalPtr)]` to `{}`.",
                        entry.type_name
                    )),
                );
            }
        }
    }
}
