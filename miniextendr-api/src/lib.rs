//! miniextendr-api: core runtime, FFI, ALTREP, and macros
//!
//! Note: ALTREP trait methods receive raw SEXP pointers from R's runtime.
//! These are safe to dereference because R guarantees valid SEXPs in ALTREP callbacks.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

// Procedural macros (re-exported from miniextendr-macros)
#[doc(inline)]
pub use miniextendr_macros::ExternalPtr;
#[doc(inline)]
pub use miniextendr_macros::RNativeType;
#[doc(inline)]
pub use miniextendr_macros::miniextendr;
#[doc(inline)]
pub use miniextendr_macros::miniextendr_module;
#[doc(inline)]
pub use miniextendr_macros::r_ffi_checked;
#[doc(inline)]
pub use miniextendr_macros::{
    AltrepComplex, AltrepInteger, AltrepList, AltrepLogical, AltrepRaw, AltrepReal, AltrepString,
};

pub mod altrep;
pub mod altrep_bridge;
pub mod altrep_data;
pub mod altrep_impl;
pub mod altrep_registration;
pub mod altrep_traits;
pub mod ffi;

// Re-export high-level ALTREP data traits
pub use altrep_data::{
    AltComplexData,
    AltIntegerData,
    AltListData,
    AltLogicalData,
    AltRawData,
    AltRealData,
    AltStringData,
    AltrepDataptr,
    AltrepLen,
    // Iterator-backed ALTREP types (R-native)
    IterComplexData,
    // Iterator-backed ALTREP types (with Coerce support)
    IterIntCoerceData,
    IterIntData,
    IterIntFromBoolData,
    IterListData,
    IterLogicalData,
    IterRawData,
    IterRealCoerceData,
    IterRealData,
    IterState,
    IterStringData,
    Logical,
    Sortedness,
};
// Re-export RBase enum
pub use altrep::RBase;
// Note: SexpExt is pub(crate), imported directly in modules that need it
pub mod from_r;
pub mod into_r;
pub use into_r::IntoR;
pub mod unwind_protect;
pub mod worker;

// Rayon integration (parallel computation with R interop)
#[cfg(feature = "rayon")]
pub mod rayon_bridge;
pub use worker::*;

// Thread safety utilities for calling R from non-main threads
pub mod thread;

// Stack size constants and builder (always available)
#[cfg(windows)]
pub use thread::WINDOWS_R_STACK_SIZE;
pub use thread::{DEFAULT_R_STACK_SIZE, RThreadBuilder};

// Stack checking control (requires nonapi feature)
#[cfg(feature = "nonapi")]
pub use thread::{StackCheckGuard, scope_with_r, spawn_with_r, with_stack_checking_disabled};

// Error handling helpers (r_stop, r_warning, r_print, r_println, r_error! macro)
pub mod error;
pub use error::{r_print, r_println, r_stop, r_warning};

// Re-export from_r
pub use from_r::{
    CoercedSexpError, SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp,
};

// Encoding / locale probing (mainly for debugging; some parts require `nonapi`)
// NOTE: Disabled because it references non-exported symbols from R's Defn.h
// (e.g., known_to_be_utf8, utf8locale) that cause dlopen failures at runtime.
// #[cfg(feature = "nonapi")]
// pub mod encoding;

// Note: RNativeType is pub(crate), imported directly in modules that need it

pub mod backtrace;

pub mod coerce;
pub use coerce::{Coerce, CoerceError, Coerced, TryCoerce};

pub mod dots;

// External pointer module - Box-like owned pointer wrapping R's EXTPTRSXP
pub mod externalptr;

// Connection framework (unstable R API - use with caution)
#[cfg(feature = "connections")]
pub mod connection;
pub use externalptr::{
    ErasedExternalPtr,
    ExternalPtr,
    ExternalSlice,
    TypedExternal,
    // ALTREP helpers (checked)
    altrep_data1_as,
    // ALTREP helpers (unchecked - for performance-critical callbacks)
    altrep_data1_as_unchecked,
    altrep_data1_mut,
    altrep_data1_mut_unchecked,
    altrep_data2_as,
    altrep_data2_as_unchecked,
};

// TypedExternal implementations for std types
pub mod externalptr_std;

// R object preservation and allocator
pub mod preserve;

pub mod allocator;
pub use allocator::RAllocator;

// =============================================================================
// Trait ABI Support
// =============================================================================
//
// Cross-package trait dispatch using a stable C ABI.
// See `trait_abi` module docs for details.

/// ABI types for cross-package trait dispatch.
///
/// This module defines the stable, C-compatible types used for runtime trait
/// dispatch across R package boundaries.
pub mod abi;

/// Runtime support for trait ABI operations.
///
/// Provides C-callable loading and type conversion helpers for trait ABI support.
pub mod trait_abi;

// Re-export key ABI types at crate root for convenience
pub use abi::{mx_base_vtable, mx_erased, mx_meth, mx_tag};

/// This is used to ensure the macros of `miniextendr-macros` treat this crate as a "user crate"
/// atleast in the `macro_coverage`
#[doc(hidden)]
extern crate self as miniextendr_api;

#[doc(hidden)]
pub mod macro_coverage;
