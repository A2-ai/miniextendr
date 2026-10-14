//! Raw R evaluation in a `#[miniextendr]` body.
//!
//! - MXL305: Warns on `Rf_eval` / `R_forceAndCall` / `Rf_applyClosure` /
//!   `R_CheckUserInterrupt` (or their `_unchecked` forms) inside a `#[miniextendr]`
//!   fn, a method of a `#[miniextendr]` impl block, or a default method of a
//!   `#[miniextendr]` trait.
//!   An R error or a pending interrupt there longjmps over the body's Rust frames:
//!   their destructors don't run, and a receiver the generated code borrowed stays
//!   borrowed for the rest of the session (#2093).

use crate::crate_index::CrateIndex;
use crate::diagnostic::Diagnostic;
use crate::lint_code::LintCode;

pub fn check(index: &CrateIndex, diagnostics: &mut Vec<Diagnostic>) {
    for (path, data) in &index.file_data {
        for (fn_name, line) in &data.raw_r_eval_calls {
            let interrupt = fn_name.starts_with("R_CheckUserInterrupt");
            let what = if interrupt {
                "a pending interrupt"
            } else {
                "an R error in the code it runs"
            };
            let help = if interrupt {
                "Use `#[miniextendr(check_interrupt)]` to check once, before the arguments \
                 are converted and the receiver is borrowed. A check inside a long loop has \
                 no safe helper yet (A2-ai/miniextendr#2101). Suppress an intentional site \
                 with `// mxl::allow(MXL305)`."
            } else {
                "Run R code through the `miniextendr_api::expression` evaluators, which stop \
                 an R error before it crosses a Rust frame: `RCall::eval` / `eval_as` and \
                 `r_eval_str` return it as `Err` (the caller's condition handlers don't see \
                 the evaluation); `eval_with_handlers` keeps the caller's handlers and \
                 carries an R error through the Rust frames as an unwind; \
                 `try_eval_with_handlers` keeps the handlers and returns the error as `Err`. \
                 `Quoted::eval()` evaluates an unevaluated argument. For `R_forceAndCall` / \
                 `Rf_applyClosure`, build the call with `RCall::from_sexp(f).arg(..)` and \
                 evaluate it the same way. Suppress an intentional site with \
                 `// mxl::allow(MXL305)`."
            };
            diagnostics.push(
                Diagnostic::new(
                    LintCode::MXL305,
                    path,
                    *line,
                    format!(
                        "`{fn_name}()` in a `#[miniextendr]` body: {what} longjmps over the \
                         body's Rust frames, so their destructors don't run and a receiver \
                         the generated code borrowed stays borrowed \
                         (`miniextendr_borrow_conflict`) for the rest of the R session.",
                    ),
                )
                .with_help(help),
            );
        }
    }
}
