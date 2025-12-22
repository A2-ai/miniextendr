//! with_r_unwind_protect benchmarks.
//!
//! Planned groups:
//! - `baseline` (closure with no R calls)
//! - `r_call` (closure that calls a trivial R API)
//! - `panic_path` (closure that panics, converted to R error)
//! - `r_error_path` (closure that triggers R error)
//!
//! For error paths, use a dedicated harness to avoid contaminating
//! subsequent benchmark runs (likely separate bench binary or subprocess).
