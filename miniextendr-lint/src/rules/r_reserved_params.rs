//! R reserved-word parameter name check.
//!
//! - MXL110: A `#[miniextendr]` function or method has a parameter whose name
//!   is an R reserved word. The proc macro forwards parameter names verbatim
//!   into the generated R wrapper, so the wrapper will be syntactically invalid.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

/// R reserved words from `?Reserved`.
///
/// Strictly reserved only — does not include quasi-reserved (`T`, `F`, `c`, `t`, `q`)
/// to avoid false positives.
const R_RESERVED: &[&str] = &[
    "if",
    "else",
    "repeat",
    "while",
    "function",
    "for",
    "in",
    "next",
    "break",
    "TRUE",
    "FALSE",
    "NULL",
    "Inf",
    "NaN",
    "NA",
    "NA_integer_",
    "NA_real_",
    "NA_complex_",
    "NA_character_",
];

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (fn_name, params) in &data.fn_param_names {
            for (param, line) in params {
                if R_RESERVED.contains(&param.as_str()) {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL110,
                            path,
                            *line,
                            format!(
                                "parameter `{param}` of `{fn_name}` is an R reserved word; \
                                 the generated R wrapper will be syntactically invalid",
                            ),
                        )
                        .with_help("rename the parameter (e.g., `n_chunks` instead of `repeat`)"),
                    );
                }
            }
        }
    }
}
