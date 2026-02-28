//! Convenience re-exports for common miniextendr items.
//!
//! A single `use miniextendr_api::prelude::*;` brings into scope the most
//! commonly used macros, traits, types, and helpers so user crates can avoid
//! a long list of individual imports.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::prelude::*;
//!
//! #[miniextendr]
//! fn add(a: i32, b: i32) -> i32 {
//!     a + b
//! }
//!
//! miniextendr_module! {
//!     mod mypkg;
//!     fn add;
//! }
//! ```

// ---------------------------------------------------------------------------
// Proc-macro re-exports
// ---------------------------------------------------------------------------
pub use crate::{
    miniextendr, miniextendr_module, typed_list, list,
    // Derive macros
    Altrep, ExternalPtr, MatchArg, RFactor,
};

// ---------------------------------------------------------------------------
// Core traits
// ---------------------------------------------------------------------------
pub use crate::{IntoR, TryFromSexp, Coerce, TryCoerce};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------
pub use crate::{
    Missing,
    List, ListBuilder, ListMut,
    StrVec, StrVecBuilder, NamedVector,
    OwnedProtect, ProtectScope,
};

// ---------------------------------------------------------------------------
// Worker thread
// ---------------------------------------------------------------------------
pub use crate::{with_r_thread, Sendable};

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------
pub use crate::{r_stop, r_warning};

// ---------------------------------------------------------------------------
// FFI (SEXP is needed in almost every crate)
// ---------------------------------------------------------------------------
pub use crate::ffi::SEXP;
