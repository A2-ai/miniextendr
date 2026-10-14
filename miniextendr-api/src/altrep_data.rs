//! High-level ALTREP data traits.
//!
//! These traits let you implement ALTREP behavior using `&self` methods instead of
//! raw `SEXP` callbacks. The library provides blanket implementations that handle
//! the SEXP extraction automatically.
//!
//! ## Quick Start
//!
//! For common types, just use them directly:
//!
//! ```ignore
//! // Vec<i32> already implements AltIntegerData
//! let altrep = create_altinteger(vec![1, 2, 3, 4, 5]);
//! ```
//!
//! For custom types, implement the relevant trait:
//!
//! ```ignore
//! struct Fibonacci { len: usize }
//!
//! impl AltrepLen for Fibonacci {
//!     fn altrep_len(&self) -> usize { self.len }
//! }
//!
//! impl AltIntegerData for Fibonacci {
//!     fn elt(&self, i: usize) -> i32 {
//!         // Compute fibonacci(i)
//!         unimplemented!()
//!     }
//! }
//! ```
//!
//! For simple field-based types, the `Altrep*` derive macros provide a shorter path:
//! they auto-implement `AltrepLen` and the matching `Alt*Data` trait, and can
//! optionally call the low-level `impl_alt*_from_data!` helpers.

mod buffer;
mod builtins;
mod core;
mod deque;
mod iter;
/// Helper macros for implementing ALTREP data traits.
pub mod macros;
mod stream;
mod traits;

pub use buffer::AltrepBuffer;
pub(crate) use core::fill_region;
pub use core::{
    AltrepDataptr, AltrepExtract, AltrepExtractSubset, AltrepLen, AltrepSerialize, InferBase,
    Logical, Sortedness, altrep_values_written, materialize_altrep_data2,
    materialized_altrep_data2, materialized_altrep_region,
};
pub use iter::*;
pub use stream::*;
pub use traits::*;

#[cfg(test)]
mod tests;
