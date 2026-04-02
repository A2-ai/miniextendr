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
        //
        // S3 trait impls are compatible with ALL inherent styles because S3 generics
        // dispatch on the class attribute, which every class system provides.
        // S4 trait impls work on S4 inherent (needs proper S4 class with slots).
        // S7/R6/Env trait impls require matching inherent for dispatch to work.
        for ati in &data.attributed_trait_impls {
            let trait_style = ati.class_system.as_deref().unwrap_or("env");

            if let Some((inherent_style, _)) = data.inherent_impl_class_systems.get(&ati.type_name)
            {
                let inherent = if inherent_style.is_empty() {
                    "env"
                } else {
                    inherent_style.as_str()
                };

                // S3 trait dispatch works on any class system's objects
                let compatible = trait_style == inherent || trait_style == "s3";

                if !compatible {
                    diagnostics.push(Diagnostic::new(
                        LintCode::MXL008,
                        path,
                        ati.line,
                        format!(
                            "#[miniextendr] impl {} for {} uses {}-style, but the inherent \
                             impl uses {}-style. Trait and inherent impls must use the same \
                             class system (S3 traits are compatible with all inherent styles). \
                             Either change the trait impl to #[miniextendr({})] or change \
                             the inherent impl to #[miniextendr({})].",
                            ati.trait_name,
                            ati.type_name,
                            trait_style,
                            inherent,
                            inherent,
                            trait_style,
                        ),
                    ));
                }
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
