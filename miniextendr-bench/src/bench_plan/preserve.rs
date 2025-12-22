//! Preserve-list benchmarks.
//!
//! Planned groups:
//! - `insert_release` (preserve::insert/release)
//! - `unchecked_insert_release` (unchecked variants)
//! - `compare_protect` (Rf_protect/Rf_unprotect baseline)
//! - `scale` (batch insert/release N objects)
//!
//! Metrics:
//! - ns/op for single insert/release
//! - throughput for batch operations
