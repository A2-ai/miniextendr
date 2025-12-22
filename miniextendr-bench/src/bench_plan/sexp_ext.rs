//! Benchmarks for SexpExt helpers vs raw access.
//!
//! Planned cases:
//! - `type_of` vs `TYPEOF` direct
//! - `len` vs `Rf_xlength` direct
//! - `as_slice` vs manual pointer + slice creation
//! - unchecked variants where available
//!
//! Parameters:
//! - Types: int, real, logical, raw
//! - Sizes: tiny -> large
//! - Alignment: compare contiguous vectors vs ALTREP data (if possible)
