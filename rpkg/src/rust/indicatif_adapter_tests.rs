//! indicatif adapter tests — R console progress bar integration.

use miniextendr_api::indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use miniextendr_api::miniextendr;
use miniextendr_api::progress::{RStream, RTerm, term_like_stderr, term_like_stdout};

/// Test RTerm construction and Debug output formatting.
#[miniextendr]
pub fn indicatif_rterm_debug() -> String {
    let term = RTerm::new(RStream::Stderr, 80);
    format!("{:?}", term)
}

/// Test that convenience factory functions produce draw targets without panicking.
#[miniextendr]
pub fn indicatif_factories_compile() -> bool {
    let _stdout = term_like_stdout(80);
    let _stderr = term_like_stderr(80);
    true
}

/// Test running a hidden progress bar (zero length) to exercise the full codepath.
#[miniextendr]
pub fn indicatif_hidden_bar() -> bool {
    let term = RTerm::new(RStream::Stderr, 80);
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
