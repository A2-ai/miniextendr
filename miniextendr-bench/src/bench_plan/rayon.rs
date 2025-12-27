//! Rayon integration benchmarks (feature = "rayon").
//!
//! Planned groups:
//! - `RVec<T>` parallel collection vs sequential `Vec<T>` collect
//! - `with_r_vec<T>` zero-copy fill vs `Vec<T>` + IntoR
//! - `reduce::sum` / `reduce::mean` vs sequential reductions
//! - scaling across Rayon thread counts
//!
//! Parameters:
//! - vector size matrix
//! - parallelism level (rayon thread count)
