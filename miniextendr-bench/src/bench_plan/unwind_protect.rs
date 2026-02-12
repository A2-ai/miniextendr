//! with_r_unwind_protect benchmarks.
//!
//! Implemented groups:
//! - `baseline`: direct noop vs unwind_protect noop
//! - `r_call`: closure that calls a trivial R API inside unwind protection
//!
//! Deferred (requires subprocess isolation):
//! - `panic_path`: closure that panics, converted to R error
//! - `r_error_path`: closure that triggers R error via longjmp
//!
//! Error paths contaminate process state and cannot run in the same
//! bench process as normal benchmarks.
