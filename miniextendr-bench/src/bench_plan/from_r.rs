//! Benchmarks for R -> Rust conversions (TryFromSexp).
//!
//! Planned groups:
//!
//! 1) `scalars`
//!    - i32, f64, bool, Rboolean
//!    - `Option<T>` (NA handling)
//!
//! 2) `slices`
//!    - &'static [i32], &'static [f64], &'static [u8]
//!    - Compare to manual pointer access + slice creation
//!
//! 3) `vectors`
//!    - `Vec<String>` (NA -> empty string)
//!    - `Vec<Option<String>>` (NA -> None)
//!    - `Vec<Option<i32/f64/bool>>`
//!
//! 4) `collections`
//!    - `HashSet<T>`, `BTreeSet<T>` for native types
//!    - `HashMap<String, V>` and `BTreeMap<String, V>` from named lists
//!
//! 5) `coerced`
//!    - `Coerced<T, R>` for numeric widening/narrowing
//!    - Error path measurement (overflow, precision loss)
//!
//! Parameters:
//! - Size matrix and NA density matrix
//! - Named list sizes and key formats
//! - Encoding variants for strings
