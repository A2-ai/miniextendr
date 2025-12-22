//! Benchmarks for Rust -> R conversions (IntoR).
//!
//! Planned groups:
//!
//! 1) `scalars`
//!    - i32, f64, bool, Rboolean, RLogical
//!    - Option<T> (Some/None)
//!
//! 2) `vectors_native`
//!    - Vec<i32>, Vec<f64>, Vec<u8>, Vec<RLogical>
//!    - &[] and &[T] slices
//!    - Include NA densities for logical/real/int options
//!
//! 3) `vectors_option`
//!    - Vec<Option<i32>>, Vec<Option<f64>>, Vec<Option<bool>>
//!    - Vec<Option<String>>
//!    - NA density matrix
//!
//! 4) `strings`
//!    - &[String], Vec<String>, &[&str]
//!    - ASCII vs UTF-8 vs Latin-1 payloads
//!
//! 5) `lists`
//!    - Vec<SEXP> (pre-allocated)
//!    - Nested lists (small depth)
//!
//! Metrics:
//! - ns/op for scalar conversions
//! - MB/s for vector conversions
//! - allocations per conversion (if possible)
