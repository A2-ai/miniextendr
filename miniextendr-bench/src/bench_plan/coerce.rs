//! Coerce / TryCoerce benchmarks.
//!
//! Planned groups:
//! - `infallible_scalar` (i8/i16/u16/bool -> i32 or f64)
//! - `fallible_scalar` (f64 -> i32, u64 -> i32) with overflow/precision cases
//! - `slice_coerce` (`Vec<T>` -> `Vec<R>`) for large sizes
//! - `option_coerce` (`Option<T>` -> NA mapping)
//! - `coerced_wrapper` (`Coerced<T, R>` creation and access)
//!
//! Track:
//! - success vs error path costs
//! - scaling with vector length
