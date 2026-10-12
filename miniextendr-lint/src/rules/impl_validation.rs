//! Impl block validation: class system compatibility.
//!
//! - MXL008: Trait impl class system incompatible with inherent impl.

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
    }
}
