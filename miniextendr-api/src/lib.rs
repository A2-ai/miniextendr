//! miniextendr-api: core runtime, FFI, ALTREP, and macros
//!
//! Note: ALTREP trait methods receive raw SEXP pointers from R's runtime.
//! These are safe to dereference because R guarantees valid SEXPs in ALTREP callbacks.
//!
//! # GC Protection Strategies
//!
//! R's garbage collector can reclaim any SEXP that isn't protected. miniextendr
//! provides three complementary protection mechanisms for different scenarios:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`] | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **Preserve list** | [`preserve`] | Across `.Call`s | Any order | Long-lived R objects |
//! | **R ownership** | [`ExternalPtr`](struct@ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! ## Quick Guide
//!
//! **Temporary allocations during computation** → [`ProtectScope`]
//! ```ignore
//! unsafe fn compute(x: SEXP) -> SEXP {
//!     let scope = ProtectScope::new();
//!     let temp = scope.protect(Rf_allocVector(REALSXP, 100));
//!     // ... work with temp ...
//!     result.into_raw()
//! } // UNPROTECT(n) called automatically
//! ```
//!
//! **R objects surviving across `.Call`s** → [`preserve`]
//! ```ignore
//! // In RAllocator or similar long-lived context
//! let cell = unsafe { preserve::insert(backing_vec) };
//! // ... use across multiple .Calls ...
//! unsafe { preserve::release(cell) };
//! ```
//!
//! **Rust data owned by R** → [`ExternalPtr`](struct@ExternalPtr)
//! ```ignore
//! #[miniextendr]
//! fn create_model() -> ExternalPtr<MyModel> {
//!     ExternalPtr::new(MyModel::new())
//! } // R owns it; Drop runs when R GCs
//! ```
//!
//! See each module's documentation for detailed usage and safety requirements.
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
    IntoList, PreferExternalPtr, PreferList, PreferRNativeType, TryFromList,
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

// `indicatif` progress integration (R console)
#[cfg(feature = "indicatif")]
pub mod progress;

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

// RNG (random number generation) utilities
pub mod rng;
pub use rng::{RngGuard, with_rng};

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

pub mod convert;
pub mod dots;
pub mod list;
pub use convert::{AsExternalPtr, AsExternalPtrExt, AsList, AsListExt, AsRNative, AsRNativeExt};
pub use list::{IntoList, List, TryFromList};

// External pointer module - Box-like owned pointer wrapping R's EXTPTRSXP
pub mod externalptr;

// Connection framework (unstable R API - use with caution)
#[cfg(feature = "connections")]
pub mod connection;
pub use externalptr::{
    ErasedExternalPtr,
    ExternalPtr,
    ExternalSlice,
    IntoExternalPtr,
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

// GC protection toolkit (PROTECT stack RAII wrappers)
pub mod gc_protect;
pub use gc_protect::{OwnedProtect, ProtectIndex, ProtectScope, ReprotectSlot, Root};

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
pub use trait_abi::TraitView;

// =============================================================================
// Marker Traits
// =============================================================================
//
// Marker traits for types derived with proc-macros.
// These enable compile-time identification and blanket implementations.

/// Marker traits for proc-macro derived types.
pub mod markers;
pub use markers::{
    IsAltrepComplexData, IsAltrepIntegerData, IsAltrepListData, IsAltrepLogicalData,
    IsAltrepRawData, IsAltrepRealData, IsAltrepStringData, IsRNativeType,
};

// =============================================================================
// Adapter Traits
// =============================================================================
//
// Built-in adapter traits with blanket implementations for standard library traits.
// These allow any Rust type implementing Debug, Display, Hash, Ord, etc. to be
// exposed to R without boilerplate.

/// Built-in adapter traits for std library traits.
///
/// Provides [`RDebug`], [`RDisplay`], [`RHash`], [`ROrd`], [`RPartialOrd`],
/// [`RError`], [`RFromStr`], [`RClone`], [`RCopy`], [`RDefault`], [`RIterator`],
/// [`RExtend`], and [`RFromIter`] with blanket implementations where possible.
/// See module docs for usage.
///
/// [`RDebug`]: adapter_traits::RDebug
/// [`RDisplay`]: adapter_traits::RDisplay
/// [`RHash`]: adapter_traits::RHash
/// [`ROrd`]: adapter_traits::ROrd
/// [`RPartialOrd`]: adapter_traits::RPartialOrd
/// [`RError`]: adapter_traits::RError
/// [`RFromStr`]: adapter_traits::RFromStr
/// [`RClone`]: adapter_traits::RClone
/// [`RCopy`]: adapter_traits::RCopy
/// [`RDefault`]: adapter_traits::RDefault
/// [`RIterator`]: adapter_traits::RIterator
/// [`RExtend`]: adapter_traits::RExtend
/// [`RFromIter`]: adapter_traits::RFromIter
pub mod adapter_traits;
pub use adapter_traits::{
    RClone, RCopy, RDebug, RDefault, RDisplay, RError, RExtend, RFromIter, RFromStr, RHash,
    RIterator, ROrd, RPartialOrd,
};

/// This is used to ensure the macros of `miniextendr-macros` treat this crate as a "user crate"
/// atleast in the `macro_coverage`
#[doc(hidden)]
extern crate self as miniextendr_api;

#[doc(hidden)]
pub mod macro_coverage;

// =============================================================================
// Optional integrations with external crates
// =============================================================================

/// Integration with the `rand` crate for R's RNG.
///
/// Provides:
/// - [`RRng`][rand_impl::RRng] - Wraps R's RNG, implements `rand::RngCore`
/// - [`RDistributions`][rand_impl::RDistributions] - Direct access to R's native distributions
/// - [`RRngOps`][rand_impl::RRngOps] - Adapter trait for exposing custom RNGs to R
///
/// Enable with `features = ["rand"]`.
#[cfg(feature = "rand")]
pub mod rand_impl;
#[cfg(feature = "rand")]
pub use rand_impl::{RDistributionOps, RDistributions, RRng, RRngOps};

/// Re-export of `rand_distr` for probability distributions.
///
/// Provides distributions like `Normal`, `Exp`, `Uniform`, etc. that work
/// with [`RRng`]. Enable with `features = ["rand_distr"]`.
///
/// ```ignore
/// use miniextendr_api::{RRng, rand_distr::Normal};
/// use rand::distr::Distribution;
///
/// #[miniextendr(rng)]
/// fn sample_normal(n: i32, mean: f64, sd: f64) -> Vec<f64> {
///     let mut rng = RRng::new();
///     let normal = Normal::new(mean, sd).unwrap();
///     (0..n).map(|_| normal.sample(&mut rng)).collect()
/// }
/// ```
///
/// **Note:** For standard normal/exponential, [`RDistributions::standard_normal`]
/// and [`RDistributions::standard_exp`] are faster as they use R's native functions.
#[cfg(feature = "rand_distr")]
pub use rand_distr;

/// Integration with the `either` crate.
///
/// Provides [`TryFromSexp`] and [`IntoR`] for [`Either<L, R>`][either::Either].
///
/// Enable with `features = ["either"]`.
#[cfg(feature = "either")]
pub mod either_impl;
#[cfg(feature = "either")]
pub use either_impl::{Either, Left, Right};

/// Integration with the `ndarray` crate.
///
/// Provides conversions between R vectors/matrices and ndarray types
/// (`Array1`, `Array2`, `ArrayView1`, `ArrayView2`).
///
/// Enable with `features = ["ndarray"]`.
#[cfg(feature = "ndarray")]
pub mod ndarray_impl;
#[cfg(feature = "ndarray")]
pub use ndarray_impl::{Array1, Array2, ArrayView1, ArrayView2, RNdArrayOps};

/// Integration with the `nalgebra` crate.
///
/// Provides conversions between R vectors/matrices and nalgebra types
/// (`DVector`, `DMatrix`).
///
/// Enable with `features = ["nalgebra"]`.
#[cfg(feature = "nalgebra")]
pub mod nalgebra_impl;
#[cfg(feature = "nalgebra")]
pub use nalgebra_impl::{DMatrix, DVector, RMatrixOps, RVectorOps};

/// Integration with the `num-bigint` crate.
///
/// Provides conversions for `BigInt` and `BigUint` via R character vectors.
///
/// Enable with `features = ["num-bigint"]`.
#[cfg(feature = "num-bigint")]
pub mod num_bigint_impl;
#[cfg(feature = "num-bigint")]
pub use num_bigint_impl::{
    BigInt, BigUint, RBigIntBitOps, RBigIntOps, RBigUintBitOps, RBigUintOps,
};

/// Integration with the `rust_decimal` crate.
///
/// Provides conversions for `Decimal` via R character vectors.
///
/// Enable with `features = ["rust_decimal"]`.
#[cfg(feature = "rust_decimal")]
pub mod rust_decimal_impl;
#[cfg(feature = "rust_decimal")]
pub use rust_decimal_impl::{Decimal, RDecimalOps};

/// Integration with the `ordered-float` crate.
///
/// Provides conversions for `OrderedFloat<f64>` and `OrderedFloat<f32>`.
///
/// Enable with `features = ["ordered-float"]`.
#[cfg(feature = "ordered-float")]
pub mod ordered_float_impl;
#[cfg(feature = "ordered-float")]
pub use ordered_float_impl::{OrderedFloat, ROrderedFloatOps};

/// UUID support via the `uuid` crate.
///
/// Provides conversions between R character vectors and `Uuid` types:
/// - `Uuid` ⇄ `character(1)`
/// - `Vec<Uuid>` ⇄ `character`
/// - `Option<Uuid>` for NA support
///
/// Enable with `features = ["uuid"]`.
#[cfg(feature = "uuid")]
pub mod uuid_impl;
#[cfg(feature = "uuid")]
pub use uuid_impl::{RUuidOps, Uuid, uuid_helpers};

/// Regex support via the `regex` crate.
///
/// Provides compiled regular expressions from R character patterns:
/// - `Regex` from `character(1)` (compiles pattern)
/// - `Option<Regex>` for NA support
///
/// Note: `Regex` does not convert back to R (use original pattern string).
///
/// Enable with `features = ["regex"]`.
#[cfg(feature = "regex")]
pub mod regex_impl;
#[cfg(feature = "regex")]
pub use regex_impl::{CaptureGroups, RCaptureGroups, RRegexOps, Regex};

/// IndexMap support via the `indexmap` crate.
///
/// Provides conversions between R named lists and `IndexMap<String, T>`:
/// - `IndexMap<String, T>` ⇄ named `list()`
/// - Preserves insertion order in both directions
/// - Auto-names unnamed elements ("V1", "V2", ...)
///
/// Enable with `features = ["indexmap"]`.
#[cfg(feature = "indexmap")]
pub mod indexmap_impl;
#[cfg(feature = "indexmap")]
pub use indexmap_impl::{IndexMap, RIndexMapOps};

/// Time and date support via the `time` crate.
///
/// Provides conversions between R date/time types and `time` crate types:
/// - `OffsetDateTime` ⇄ `POSIXct` (seconds since epoch + timezone)
/// - `Date` ⇄ R `Date` (days since 1970-01-01)
/// - Vector and Option variants for all types
///
/// Enable with `features = ["time"]`.
#[cfg(feature = "time")]
pub mod time_impl;
#[cfg(feature = "time")]
pub use time_impl::{Date, Duration, OffsetDateTime, RDateTimeFormat, RDuration};

/// N-dimensional R arrays with const generic dimension count.
pub mod rarray;
pub use rarray::{RArray, RArray3D, RMatrix, RVector};

/// Integration with the `serde` crate for JSON serialization.
///
/// Provides adapter traits for serializing/deserializing Rust types to/from JSON:
/// - `RSerialize` - Serialize to JSON string
/// - `RDeserialize` - Parse JSON string
///
/// Also re-exports `serde` with derive macros enabled for `#[derive(Serialize, Deserialize)]`.
///
/// Enable with `features = ["serde"]`.
///
/// ```ignore
/// use miniextendr_api::serde::{Serialize, Deserialize};
/// use miniextendr_api::serde_impl::{RSerialize, RDeserialize};
///
/// #[derive(Serialize, Deserialize, ExternalPtr)]
/// struct Config {
///     name: String,
///     count: i32,
/// }
///
/// #[miniextendr]
/// impl RSerialize for Config {}
///
/// #[miniextendr]
/// impl RDeserialize for Config {}
/// ```
#[cfg(feature = "serde")]
pub mod serde_impl;
#[cfg(feature = "serde")]
pub use serde;
#[cfg(feature = "serde")]
pub use serde_impl::{RDeserialize, RSerialize};

/// Integration with the `num-traits` crate for generic numeric operations.
///
/// Provides adapter traits for generic numeric types:
/// - [`RNum`][num_traits_impl::RNum] - Basic numeric operations (zero, one, is_zero)
/// - [`RSigned`][num_traits_impl::RSigned] - Signed number operations (abs, signum)
/// - [`RFloat`][num_traits_impl::RFloat] - Floating-point operations (floor, ceil, sqrt, trig)
///
/// All traits have blanket implementations for types implementing the corresponding
/// `num-traits` trait.
///
/// Enable with `features = ["num-traits"]`.
///
/// ```ignore
/// use miniextendr_api::num_traits_impl::{RNum, RFloat};
///
/// #[derive(ExternalPtr)]
/// struct MyFloat(f64);
///
/// #[miniextendr]
/// impl RNum for MyFloat {}
///
/// #[miniextendr]
/// impl RFloat for MyFloat {}
/// ```
#[cfg(feature = "num-traits")]
pub mod num_traits_impl;
#[cfg(feature = "num-traits")]
pub use num_traits_impl::{RFloat, RNum, RSigned};
