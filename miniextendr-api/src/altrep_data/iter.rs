//! Iterator-backed ALTREP data types.
//!
//! Provides lazy ALTREP vectors backed by Rust iterators. Elements are generated
//! on-demand and cached for repeat access.
//!
//! ## Submodules
//!
//! - [`state`]: Core `IterState<I, T>` + standard wrapper types (Int, Real, Logical, Raw)
//! - [`coerce`]: Coerced variants (`IterIntCoerceData`, `IterRealCoerceData`, `IterIntFromBoolData`)
//!   plus the String/List/Complex wrapper types (`IterStringData`, `IterListData`, `IterComplexData`)
//! - [`sparse`]: Sparse iterators using `nth()` for skip-ahead (`SparseIterState`)
//! - [`windowed`]: Sliding-window iterators (`WindowedIterState`)

mod coerce;
mod sparse;
mod state;
mod windowed;

pub use coerce::*;
pub use sparse::*;
pub use state::*;
pub use windowed::*;
