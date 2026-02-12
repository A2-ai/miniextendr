//! RNative path vs Coercion path benchmarks.
//!
//! Implemented groups (all parameterized by SIZES[0..5]):
//! - `integer_native`: &[i32] slice (zero-copy), Vec<i32> (memcpy)
//! - `integer_coerce`: Vec<i64> (widen), Vec<u32> (bounds-check)
//! - `real_native`: &[f64] slice, Vec<f64> (memcpy)
//! - `real_coerce`: Vec<f32> (narrow), Vec<i64> (truncate), Vec<i32> (truncate+narrow)
//!
//! Shows the cost gradient from zero-copy slice → memcpy → element-wise coercion.
