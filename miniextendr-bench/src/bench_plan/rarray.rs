//! Bench plan: `benches/rarray.rs`
//!
//! Focus:
//! - access patterns on `RMatrix<T>` and `RArray<T, NDIM>` wrappers.
//!
//! Suggested benchmarks:
//! - `as_slice` full-buffer iteration baseline (column-major).
//! - `get_rc` nested loops to quantify per-element index overhead.
//! - `column(col)` + per-column iteration (contiguous slices).
//! - `to_vec` copy cost (main-thread copy-out for worker-friendly compute).
//!
//! Parameters:
//! - matrix sizes from `MATRIX_DIMS` (e.g. 64x64, 256x256).
//! - optionally: add 3D arrays and “stride-heavy” index patterns.

