//! Rayon integration benchmarks (feature = "rayon").
//!
//! Planned groups:
//! - `Vec<T>` parallel collection via `par_iter().collect()`
//! - `with_r_vec<T>` zero-copy fill vs `Vec<T>` + IntoR
//! - `reduce::sum` / `reduce::mean` vs sequential reductions
//! - scaling across Rayon thread counts
//!
//! Parameters:
//! - vector size matrix
//! - parallelism level (rayon thread count)
