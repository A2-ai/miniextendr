//! Export attribute redundancy checks.
//!
//! - MXL203: `internal` + `noexport` redundancy.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (name, (has_internal, has_noexport, line)) in &data.export_control {
            if *has_internal && *has_noexport {
                diagnostics.push(
                    Diagnostic::new(
                        LintCode::MXL203,
                        path,
                        *line,
                        format!(
                            "`{}` has both `internal` and `noexport`. \
                             `internal` already suppresses @export (and adds @keywords internal), \
                             making `noexport` redundant.",
                            name,
                        ),
                    )
                    .with_help("Remove `noexport` and keep only `internal`."),
                );
            }
        }
    }
}
