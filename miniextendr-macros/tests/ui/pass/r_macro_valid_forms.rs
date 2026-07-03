//! Compile-pass test: valid R forms accepted by `r!` without errors.
//!
//! These are the tricky accept-cases from the grammar validator:
//! - `;`-statement sequences (existing documented behaviour)
//! - `<-` assignment (tokenises as `<` + `-` joint punct)
//! - `%in%` operator (tokenises as `%`, ident, `%`)
//! - `x[,1]` bracket index with empty first slot
//! - tilde formula
//! - trailing semicolon
//!
//! These tests only verify that the macro *compiles*, not that R evaluates
//! correctly (that requires a live R runtime and is covered in integration
//! tests in miniextendr-api/tests/r_eval.rs).

// Note: miniextendr_macros::r is a proc-macro that emits unsafe code
// calling miniextendr_api::expression::r_eval_str. We can verify
// compilation by checking the output type is correct without running R.

use miniextendr_macros::r;

// We cannot call the macro at compile time against a live R session here —
// but we CAN verify the macro itself expands without errors by constructing
// a dead-code context that the compiler checks but the linker strips.
#[allow(dead_code)]
fn _check_r_macro_forms() {
    // Basic arithmetic
    let _: Result<miniextendr_api::SEXP, String> = r!(1L + 2L);

    // Assignment with `<-`
    let _: Result<miniextendr_api::SEXP, String> = r!(.x <- 1L);

    // Semicolon-separated statements
    let _: Result<miniextendr_api::SEXP, String> = r!(a <- 7L; a * 6L);

    // Trailing semicolon
    let _: Result<miniextendr_api::SEXP, String> = r!(x <- 1L;);

    // Nested function call with string arg
    let _: Result<miniextendr_api::SEXP, String> = r!(nchar("hello"));

    // Tilde formula
    let _: Result<miniextendr_api::SEXP, String> = r!(y ~ x);

    // Logical operators (not trailing)
    let _: Result<miniextendr_api::SEXP, String> = r!(x && y);

    // %in% operator (tokenises as `%`, ident, `%`)
    let _: Result<miniextendr_api::SEXP, String> = r!(x %in% y);

    // Bracket index with empty first slot
    let _: Result<miniextendr_api::SEXP, String> = r!(m[, 1]);

    // Empty (missing) call arguments — valid R sublist grammar
    let _: Result<miniextendr_api::SEXP, String> = r!(matrix(, 2, 2));

    // `env` followed by `::` is R namespace access, not the env head
    let _: Result<miniextendr_api::SEXP, String> = r!(env::foo());
}

fn main() {}
