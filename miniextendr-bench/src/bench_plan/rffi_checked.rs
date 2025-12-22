//! Thread-checked wrapper benchmarks.
//!
//! Planned groups:
//! - `checked_vs_unchecked` for key FFI functions
//! - `panic_cost` when called from wrong thread (debug builds only)
//!
//! Use simple primitives like Rf_ScalarInteger and DATAPTR_RO for comparison.
