//! Benchmarks for SexpExt helpers vs raw access.
//!
//! Planned cases:
//! - `is_integer` vs `type_of() == INTSXP`
//! - `len` vs `Rf_xlength` direct
//! - `as_slice` vs manual pointer + slice creation
//! - unchecked variants where available
//!
//! Parameters:
//! - Types: int, real, logical, raw
//! - Sizes: tiny -> large
//! - Alignment: compare contiguous vectors vs ALTREP data (if possible)
