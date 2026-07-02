//! Console-output fixtures: `r_print!`, `r_println!`, `r_str!`, and the
//! low-level `r_warning` function (audit A7 — these prelude exports had no
//! exemplar coverage; `warning!()` the condition macro is covered elsewhere).
//!
//! R-side assertions live in `rpkg/tests/testthat/test-console-output.R`.

use miniextendr_api::error::r_warning;
use miniextendr_api::prelude::TryFromSexp;
use miniextendr_api::{miniextendr, r_print, r_println, r_str};

/// Print a message to R's console via `r_print!` (no trailing newline).
/// @param msg Message to print.
#[miniextendr]
pub fn console_r_print(msg: &str) {
    r_print!("{msg}");
}

/// Print a formatted message via `r_print!` with format arguments.
/// @param label Label to print.
/// @param value Integer interpolated into the message.
#[miniextendr]
pub fn console_r_print_formatted(label: &str, value: i32) {
    r_print!("{label}={value}");
}

/// Print a message plus newline via `r_println!`.
/// @param msg Message to print.
#[miniextendr]
pub fn console_r_println(msg: &str) {
    r_println!("{msg}");
}

/// Print just a newline via the zero-argument `r_println!()` arm.
#[miniextendr]
pub fn console_r_println_empty() {
    r_println!();
}

/// Raise an R warning via the low-level `r_warning` function (not the
/// `warning!()` condition macro) and return normally afterwards.
/// @param msg Warning message.
#[miniextendr]
pub fn console_r_warning(msg: &str) -> i32 {
    r_warning(msg);
    42
}

/// Evaluate dynamically built R source via `r_str!` and return the result.
///
/// The code string is built with `format!` — the documented use case for
/// `r_str!` over the compile-time-checked `r!`. The unprotected result SEXP is
/// converted immediately, before any further allocation.
/// @param n Upper bound passed to `seq_len` inside the evaluated code.
#[miniextendr]
pub fn console_r_str_sum_seq(n: i32) -> Result<i32, String> {
    let code = format!("sum(seq_len({n}))");
    let sexp = r_str!(&code)?;
    i32::try_from_sexp(sexp).map_err(|e| e.to_string())
}

/// Evaluate a syntactically invalid string via `r_str!`; the parse error must
/// surface as an `Err` (mapped to an R condition), never a crash.
#[miniextendr]
pub fn console_r_str_parse_error() -> Result<i32, String> {
    let sexp = r_str!("1 +")?;
    i32::try_from_sexp(sexp).map_err(|e| e.to_string())
}
