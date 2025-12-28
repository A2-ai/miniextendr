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
//!     fn len(&self) -> usize { self.len }
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

mod core;
mod traits;
mod iter;
mod builtins;
pub mod macros;

pub use core::{AltrepDataptr, AltrepExtractSubset, AltrepLen, AltrepSerialize, InferBase, Logical, Sortedness};
pub(crate) use core::fill_region;
pub use traits::*;
pub use iter::*;

#[cfg(test)]
mod tests;
