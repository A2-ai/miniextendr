//! Direct access to the trait ABI entry points compiled into this package.
//!
//! Re-export the implementations so Rust callers use their actual `C-unwind`
//! signatures. The same functions are registered as R C-callables by package
//! initialization; calling them here needs no `R_GetCCallable` indirection.
//!
//! To construct a generated view, use [`TraitView::try_from_sexp`] instead of
//! casting the vtable returned by [`mx_query`]. A view needs both the concrete
//! data pointer and the vtable, and roots its R receiver.
//!
//! [`TraitView::try_from_sexp`]: super::TraitView::try_from_sexp

pub use crate::mx_abi::{mx_get, mx_query, mx_wrap};
