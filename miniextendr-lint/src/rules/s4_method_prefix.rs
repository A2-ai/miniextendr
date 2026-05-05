//! MXL111: `s4_*` method name on `#[miniextendr(s4)]` impl.
//!
//! S4 codegen auto-prepends `s4_` to every instance method name when generating
//! the R generic. A Rust method named `s4_foo` on an `#[miniextendr(s4)]` impl
//! produces an R generic `s4_s4_foo`, making it unreachable via the expected
//! `s4_foo(obj, ...)` call.

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (impl_type, methods) in &data.impl_methods {
            for (method_name, line, class_system) in methods {
                if class_system != "s4" {
                    continue;
                }
                // Constructors (`new`) are not auto-prefixed — they become the
                // class constructor function, named after the type, not `s4_new`.
                if method_name == "new" {
                    continue;
                }
                if let Some(rest) = method_name.strip_prefix("s4_") {
                    diagnostics.push(
                        Diagnostic::new(
                            LintCode::MXL111,
                            path,
                            *line,
                            format!(
                                "method `{impl_type}::{method_name}` will produce R generic \
                                 `s4_{method_name}` (s4 codegen auto-prepends `s4_`); \
                                 rename to `{rest}` to avoid the double prefix",
                            ),
                        )
                        .with_help(
                            "the macro prepends `s4_` to every S4 method name; \
                             keep the Rust method name unprefixed",
                        ),
                    );
                }
            }
        }
    }
}
