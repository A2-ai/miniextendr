//! Iterator-backed ALTREP data adaptors.
//!
//! Provides lazy backing data for ALTREP vectors from Rust iterators. Elements
//! are generated on-demand and cached for repeat access.
//!
//! ## Data adaptors, not R-facing vectors
//!
//! The `*Data` types in these submodules implement only the data-level traits
//! ([`AltrepLen`](crate::altrep_data::AltrepLen) + the matching `Alt*Data`
//! trait). They do **not** implement
//! [`RegisterAltrep`](crate::altrep::RegisterAltrep), so they cannot back a
//! live R SEXP by themselves. To expose one to R, wrap it in a concrete
//! struct deriving the matching `Altrep*` derive with `#[altrep(manual)]` and
//! delegate the data-trait methods to the inner adaptor (the derive cannot be
//! applied to the adaptors directly because they are generic over the
//! iterator type):
//!
//! ```ignore
//! use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen, IterIntData};
//!
//! #[derive(miniextendr_api::AltrepInteger)]
//! #[altrep(class = "MyLazyInts", manual)]
//! struct MyLazyInts {
//!     inner: IterIntData<std::ops::Range<i32>>,
//! }
//!
//! impl AltrepLen for MyLazyInts {
//!     fn len(&self) -> usize {
//!         self.inner.len()
//!     }
//! }
//!
//! impl AltIntegerData for MyLazyInts {
//!     fn elt(&self, i: usize) -> i32 {
//!         self.inner.elt(i)
//!     }
//! }
//! ```
//!
//! ## Duplication and serialization
//!
//! The standard atomic and string adapters implement `Clone` when their
//! concrete iterator does. Cloning preserves the cached prefix and remaining
//! iterator without advancing it. They also implement `AltrepSerialize`:
//! saving realizes the declared finite sequence; restoring uses a complete
//! cache and never needs to construct `I`. Delegate that trait from a concrete
//! registered class and select `#[altrep(manual, serialize, duplicate = "clone")]`.
//! The bridge preserves authoritative R writes in data2 as well as that cache.
//! List adapters need their own GC-root and serialization policy.
//!
//! ## Submodules
//!
//! - [`state`]: Core `IterState<I, T>` + standard data adaptors (Int, Real, Logical, Raw)
//! - [`coerce`]: Coerced variants (`IterIntCoerceData`, `IterRealCoerceData`, `IterIntFromBoolData`)
//!   plus the String/List/Complex data adaptors (`IterStringData`, `IterListData`, `IterComplexData`)
//! - [`sparse`]: Sparse iterators using `nth()` for skip-ahead (`SparseIterState`)
//! - [`windowed`]: Sliding-window iterators (`WindowedIterState`)

mod coerce;
mod extension;
mod macros;
mod sparse;
mod state;
mod windowed;

pub use coerce::*;
pub use extension::*;
pub use sparse::*;
pub use state::*;
pub use windowed::*;
