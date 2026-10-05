//! Doc links into R packages the package does not declare.
//!
//! - MXL204: roxygen2 reads ``[`Type::method`]``, `[Type::method]` and
//!   `[text][pkg::topic]` as a link into the R package named before the last
//!   `::`, and warns "refers to un-installed package" when there is none. The
//!   link was usually written for rustdoc. This rule warns on each `pkg::`
//!   link in `#[miniextendr]` docs whose `pkg` is not the package itself, not
//!   in `Depends` / `Imports` / `Suggests` / `Enhances` / `LinkingTo`, and not
//!   shipped with R. Leading prose counts only under
//!   `roxygen_prose_links = "keep"`: the default strips its links. The rule
//!   reports and never rewrites. It skips when there is no `DESCRIPTION`.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    let Some(description) = &index.description else {
        return;
    };
    for (path, data) in &index.file_data {
        for link in &data.doc_pkg_links {
            if (link.leading_prose && !index.prose_links_keep) || description.resolves(&link.pkg) {
                continue;
            }
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL204,
                    path,
                    link.line,
                    format!(
                        "`{}` reaches roxygen2 as a link into the R package `{}`, which is not \
                         this package, not in DESCRIPTION (Depends, Imports, Suggests, Enhances, \
                         LinkingTo) and not shipped with R.",
                        link.written, link.pkg,
                    ),
                )
                .with_help(
                    "If it names a Rust item, root the target at the crate with the \
                     `[text][crate::path]` spelling (`[`Type::method`][crate::Type::method]`); \
                     it is a link in rustdoc and plain text on the R page. If it is an R \
                     package, add it to DESCRIPTION.",
                ),
            );
        }
    }
}
