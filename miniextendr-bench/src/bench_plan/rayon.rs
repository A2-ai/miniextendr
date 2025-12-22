//! Rayon integration benchmarks (feature = "rayon").
//!
//! Planned groups:
//! - `collect_r` vs sequential collect
//! - `with_r_real_vec` zero-copy fill
//! - `RVecBuilder` par_fill_with vs par_fill_from_slice
//! - `run_r` overhead from Rayon threads
//!
//! Parameters:
//! - vector size matrix
//! - parallelism level (rayon thread count)
