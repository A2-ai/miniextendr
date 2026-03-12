//! Impl block validation: class system compatibility and label uniqueness.
//!
//! - MXL008: Trait impl class system incompatible with inherent impl.
//! - MXL009: Multiple impl blocks for one type without labels.
//! - MXL010: Duplicate labels on impl blocks for one type.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        // MXL008: Class system compatibility
        for ati in &data.attributed_trait_impls {
            let trait_style = ati.class_system.as_deref().unwrap_or("env");

            if trait_style == "env"
                && let Some((inherent_style, _)) =
                    data.inherent_impl_class_systems.get(&ati.type_name)
                && !inherent_style.is_empty()
                && inherent_style != "env"
            {
                diagnostics.push(Diagnostic::new(
                    LintCode::MXL008,
                    path,
                    ati.line,
                    format!(
                        "#[miniextendr] impl {} for {} uses Env-style (default) which \
                         requires Env-style inherent impl, but {} uses \
                         #[miniextendr({})]. Env-style trait impls generate \
                         Type$Trait$method() patterns that need the type to be an \
                         environment. Either change the trait impl to use \
                         #[miniextendr({})] or change the inherent impl to \
                         #[miniextendr].",
                        ati.trait_name,
                        ati.type_name,
                        ati.type_name,
                        inherent_style,
                        inherent_style
                    ),
                ));
            }
        }

        // MXL009 + MXL010: Multiple impl blocks
        for (type_name, impl_blocks) in &data.impl_blocks_per_type {
            if impl_blocks.len() <= 1 {
                continue;
            }

            // MXL009: Missing labels
            let missing_labels: Vec<_> = impl_blocks
                .iter()
                .filter(|(label, _)| label.is_none())
                .map(|(_, line)| *line)
                .collect();

            if !missing_labels.is_empty() {
                diagnostics.push(Diagnostic::new(
                    LintCode::MXL009,
                    path,
                    impl_blocks[0].1,
                    format!(
                        "type `{}` has {} impl blocks but some are missing labels. \
                         When a type has multiple #[miniextendr] impl blocks, all must have \
                         distinct labels using #[miniextendr(label = \"...\")]. \
                         Unlabeled impl blocks at lines: {}",
                        type_name,
                        impl_blocks.len(),
                        missing_labels
                            .iter()
                            .map(|l| l.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                ));
            }

            // MXL010: Duplicate labels
            let mut seen_labels: std::collections::HashMap<&str, usize> =
                std::collections::HashMap::new();
            for (label, line) in impl_blocks {
                if let Some(label) = label {
                    if let Some(first_line) = seen_labels.get(label.as_str()) {
                        diagnostics.push(Diagnostic::new(
                            LintCode::MXL010,
                            path,
                            *line,
                            format!(
                                "duplicate label \"{}\" for type `{}`. \
                                 First occurrence at line {}. Each impl block must have \
                                 a unique label.",
                                label, type_name, first_line
                            ),
                        ));
                    } else {
                        seen_labels.insert(label.as_str(), *line);
                    }
                }
            }
        }
    }
}
