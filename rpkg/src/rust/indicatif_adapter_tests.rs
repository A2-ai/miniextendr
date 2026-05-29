//! indicatif adapter tests — R connection-routed progress bar integration.

use miniextendr_api::connection::{RNullConnection, RStderr};
use miniextendr_api::indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use miniextendr_api::miniextendr;
use miniextendr_api::prelude::SEXP;
use miniextendr_api::progress::{
    RTerm, term_like_connection, term_like_connection_with_hz, term_like_stderr,
    term_like_stderr_with_hz, term_like_stdout, term_like_stdout_with_hz,
};

/// Test RTerm construction and Debug output formatting.
#[miniextendr]
pub fn indicatif_rterm_debug() -> String {
    let term = RTerm::new(RStderr, 80);
    format!("{:?}", term)
}

/// Test that convenience factory functions produce draw targets without panicking.
#[miniextendr]
pub fn indicatif_factories_compile() -> bool {
    let _stdout = term_like_stdout(80);
    let _stderr = term_like_stderr(80);
    let _stdout_hz = term_like_stdout_with_hz(80, 5);
    let _stderr_hz = term_like_stderr_with_hz(80, 5);
    let null = RNullConnection::new();
    let _conn_hz = term_like_connection_with_hz(null, 80, 5);
    true
}

/// Test running a hidden progress bar (zero length) to exercise the full codepath.
#[miniextendr]
pub fn indicatif_hidden_bar() -> bool {
    let term = RTerm::new(RStderr, 80);
    let target = ProgressDrawTarget::term_like(Box::new(term));
    let pb = ProgressBar::with_draw_target(Some(0), target);
    pb.finish_and_clear();
    true
}

/// Test running a short progress bar that renders a few ticks to R stderr.
#[miniextendr]
pub fn indicatif_short_bar() -> String {
    let target = term_like_stderr(60);
    let pb = ProgressBar::with_draw_target(Some(5), target);
    pb.set_style(
        ProgressStyle::with_template("{bar:20} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    for i in 0..5 {
        pb.set_message(format!("step {}", i));
        pb.inc(1);
    }
    pb.finish_with_message("done");
    "done".to_string()
}

// region: Upstream example-derived fixtures

/// Test a spinner (no length) progress indicator.
#[miniextendr]
pub fn indicatif_spinner_demo() -> String {
    let target = term_like_stderr(60);
    let pb = ProgressBar::with_draw_target(None, target);
    pb.set_style(ProgressStyle::with_template("{spinner} {msg}").unwrap());
    pb.set_message("working...");
    for _ in 0..3 {
        pb.tick();
    }
    pb.finish_with_message("done");
    "done".to_string()
}

/// Test a progress bar with a custom download-style template.
/// @param total Total number of steps.
#[miniextendr]
pub fn indicatif_download_style(total: i32) -> String {
    let target = term_like_stderr(80);
    let pb = ProgressBar::with_draw_target(Some(total as u64), target);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] [{bar:40}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    for _ in 0..total {
        pb.inc(1);
    }
    pb.finish_with_message("complete");
    "complete".to_string()
}

/// Test a progress bar with dynamic message updates at each step.
/// @param steps Character vector of step messages.
#[miniextendr]
pub fn indicatif_with_messages(steps: Vec<String>) -> String {
    let target = term_like_stderr(60);
    let pb = ProgressBar::with_draw_target(Some(steps.len() as u64), target);
    for step in &steps {
        pb.set_message(step.clone());
        pb.inc(1);
    }
    let last = steps.last().cloned().unwrap_or_else(|| "done".to_string());
    pb.finish_with_message(last.clone());
    last
}

/// Test a progress bar with elapsed time display.
#[miniextendr]
pub fn indicatif_elapsed_demo() -> String {
    let target = term_like_stderr(60);
    let pb = ProgressBar::with_draw_target(Some(3), target);
    pb.set_style(ProgressStyle::with_template("{elapsed} {bar:20} {pos}/{len}").unwrap());
    for _ in 0..3 {
        pb.inc(1);
    }
    pb.finish();
    "finished".to_string()
}

// endregion

// region: Connection-target fixtures (issue #178)

/// Drive a short progress bar against an arbitrary R connection SEXP. The
/// R-side test creates a `textConnection(out, open = "w")` / `file()` /
/// custom connection, passes it in here, and inspects the result after the
/// bar finishes.
///
/// Returns the literal string `"ok"` after the bar finishes — the caller is
/// expected to read the underlying buffer (via the textConnection variable,
/// file contents, etc.) to assert that bytes actually reached the connection.
///
/// @param conn An open writable R connection.
/// @param ticks Number of bar steps.
#[miniextendr]
pub fn indicatif_drive_connection(conn: SEXP, ticks: i32) -> String {
    let target = term_like_connection(conn, 40);
    let pb = ProgressBar::with_draw_target(Some(ticks as u64), target);
    pb.set_style(ProgressStyle::with_template("{pos}/{len}").unwrap());
    for _ in 0..ticks {
        pb.inc(1);
    }
    pb.finish();
    "ok".to_string()
}

/// Smoke-test routing a bar at [`RNullConnection`]. Must not panic — bytes are
/// discarded by the OS null device, but the codepath must be exercised
/// without longjmping. Returns `"ok"` on success.
#[miniextendr]
pub fn indicatif_to_null_connection() -> String {
    let null = RNullConnection::new();
    // RNullConnection: IntoR returns the wrapped SEXP (disarming Drop).
    let target = term_like_connection(null, 40);
    let pb = ProgressBar::with_draw_target(Some(2), target);
    pb.set_style(ProgressStyle::with_template("{pos}/{len}").unwrap());
    for _ in 0..2 {
        pb.inc(1);
    }
    pb.finish();
    "ok".to_string()
}

// endregion
