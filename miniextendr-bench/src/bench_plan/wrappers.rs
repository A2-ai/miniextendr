//! Generated R wrapper benchmarks (optional).
//!
//! Goal: measure overhead of calling generated R wrappers vs direct `.Call`.
//! This likely requires evaluating R code via `Rf_eval` in the embedded R
//! runtime and should be isolated in its own bench binary.
//!
//! Planned groups:
//! - `wrapper_call_overhead` (R wrapper -> .Call -> Rust)
//! - `direct_call_overhead` (direct `.Call`)
//! - `argument_coercion` (R wrapper with coercion enabled)
//! - `class_methods` (S3/S4/S7/R6 method dispatch)
//!
//! Notes:
//! - Keep R expressions pre-parsed to avoid parse overhead.
//! - Use small trivial functions to focus on wrapper cost.
