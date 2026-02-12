//! RFactor enum ↔ R factor benchmarks.
//!
//! Implemented groups:
//! - `single_value`: cached (OnceLock) vs uncached levels for single enum → factor
//! - `vector`: FactorVec of 256 elements, cached vs uncached
//!
//! Key finding: ~4x speedup for single value conversions with cached levels.
//! Vector conversions show minimal difference since allocation dominates.
