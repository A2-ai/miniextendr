//! Direct `Rf_error`/`Rf_errorcall` usage lint.
//!
//! - MXL300: Warns on direct `Rf_error`/`Rf_errorcall` calls in user code.
//!   These longjmp through Rust frames, bypassing destructors unless wrapped in
//!   `R_UnwindProtect`. Prefer `panic!()` or `Err(...)` which produce structured
//!   R condition objects via `error_in_r`.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (fn_name, line) in &data.rf_error_calls {
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL300,
                    path,
                    *line,
                    format!(
                        "Direct `{}()` call. This longjmps through Rust frames \
                         and may skip destructors.",
                        fn_name,
                    ),
                )
                .with_help(
                    "Use `panic!()` for unrecoverable errors or return `Err(...)` for \
                     Result types. These produce structured R condition objects via error_in_r.",
                ),
            );
        }
    }
}
