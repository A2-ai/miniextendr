//! MXL302: non-doc attribute interrupts a doc-comment stream on a `#[miniextendr]` item.
//!
//! When `#[cfg(...)]`, `#[deprecated]`, or another non-doc attribute appears *between*
//! two `///` comment blocks on a `#[miniextendr]` item, trailing prose can be incorrectly
//! concatenated into the preceding `@examples` / `@details` / `@return` block, producing
//! corrupted `.Rd` output.
//!
//! The macro now resets multiline-continuation context at the interruption point so the
//! generated output is well-formed, but this warning guides users toward the idiomatic
//! pattern: all `///` comments placed above all non-doc attributes.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (item_name, line) in &data.interleaved_doc_attrs {
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL302,
                    path,
                    *line,
                    format!(
                        "`{item_name}`: non-doc attribute interrupts a `///` doc-comment stream; \
                         trailing doc content will not continue the preceding roxygen tag",
                    ),
                )
                .with_help(
                    "Move all `///` doc comments above non-doc attributes (`#[cfg(...)]`, \
                     `#[deprecated]`, etc.) to keep the doc-comment stream contiguous. \
                     The macro recovers automatically but tag-continuation is reset at the \
                     interruption point.",
                ),
            );
        }
    }
}
