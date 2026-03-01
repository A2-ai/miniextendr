//! `_unchecked` FFI call outside guard context.
//!
//! - MXL301: Warns on `ffi::*_unchecked()` calls in user code.
//!   These bypass main-thread routing and must only be called inside
//!   `with_r_unwind_protect`, `with_r_thread`, or similar guard closures.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (fn_name, line) in &data.ffi_unchecked_calls {
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL301,
                    path,
                    *line,
                    format!(
                        "`ffi::{}()` is a raw FFI call — only safe on R's main thread.",
                        fn_name
                    ),
                )
                .with_help(
                    "Use the checked wrapper (without `_unchecked` suffix), or ensure this \
                     call is inside `with_r_unwind_protect` / an `extern \"C-unwind\"` callback.",
                ),
            );
        }
    }
}
