//! Non-API `ATTRIB` / `SET_ATTRIB` usage lint.
//!
//! - MXL304: Warns on calls to, or declarations of, R's `ATTRIB` and
//!   `SET_ATTRIB` (and their `_unchecked` forms). Neither is part of R's API:
//!   `R CMD check` on R >= 4.6 reports a package whose shared object references
//!   them. miniextendr-api only declares them behind its `nonapi` feature, so
//!   this rule mainly catches a crate that declares the symbol itself in an
//!   `extern` block to get around that.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (fn_name, line) in &data.nonapi_attrib_calls {
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL304,
                    path,
                    *line,
                    format!(
                        "`{fn_name}()` is not part of R's API; `R CMD check` (R >= 4.6) \
                         reports packages that reference it.",
                    ),
                )
                .with_help(
                    "Use `SexpExt::has_attributes()` to test for attributes, \
                     `SexpExt::is_identical()` for R's `identical()`, \
                     `SexpExt::get_attr()` / `set_attr()` for one attribute, and \
                     `DUPLICATE_ATTRIB` / `SHALLOW_DUPLICATE_ATTRIB` to copy all of them. \
                     Suppress an intentional site with `// mxl::allow(MXL304)`.",
                ),
            );
        }
    }
}
