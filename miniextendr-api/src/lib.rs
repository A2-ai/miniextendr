//! miniextendr-api: core runtime for Rust <-> R interop.
//!
//! This crate provides the FFI surface, safety wrappers, and macro re-exports
//! used by most miniextendr users. It is the primary dependency for building
//! Rust-powered R packages and exposing Rust types to R.
//!
//! At a glance:
//! - FFI bindings + checked wrappers for R's C API (`ffi`, `r_ffi_checked`).
//! - Conversions between Rust and R types (`IntoR`, `TryFromSexp`, `Coerce`).
//! - ALTREP traits, registration helpers, and iterator-backed ALTREP data types.
//! - Wrapper generation from Rust signatures (`#[miniextendr]`, `miniextendr_module!`).
//! - Worker-thread pattern for panic isolation and `Drop` safety (`worker`).
//! - Class system support (S3, S4, S7, R6, env-style impl blocks).
//! - Cross-package trait ABI for type-erased dispatch (`trait_abi`).
//!
//! Most users should depend on this crate directly. For embedding R in
//! standalone binaries or integration tests, see `miniextendr-engine`.
//!
//! ## Quick start
//!
//! ```ignore
//! use miniextendr_api::miniextendr;
//!
//! #[miniextendr]
//! fn add(a: i32, b: i32) -> i32 {
//!     a + b
//! }
//! ```
//!
//! Register exports in your package/module:
//!
//! ```ignore
//! use miniextendr_api::miniextendr_module;
//!
//! miniextendr_module! {
//!     mod mypkg;
//!     fn add;
//! }
//! ```
//!
//! ## R wrapper generation
//!
//! `#[miniextendr]` and `miniextendr_module!` generate C-ABI wrappers plus
//! R functions that call `.Call(...)` using the original argument names.
//! Wrapper R code is produced from Rust doc comments (roxygen tags are
//! extracted) by the `document` binary and committed into
//! `R/miniextendr_wrappers.R` so CRAN builds do not require codegen.
//!
//! ## GC protection and ownership
//!
//! R's garbage collector can reclaim any SEXP that isn't protected. miniextendr
//! provides three complementary protection mechanisms:
//!
//! | Strategy | Module | Lifetime | Release Order | Use Case |
//! |----------|--------|----------|---------------|----------|
//! | **PROTECT stack** | [`gc_protect`] | Within `.Call` | LIFO (stack) | Temporary allocations |
//! | **Preserve list** | [`preserve`] | Across `.Call`s | Any order | Long-lived R objects |
//! | **R ownership** | [`ExternalPtr`](struct@ExternalPtr) | Until R GCs | R decides | Rust data owned by R |
//!
//! Quick guide:
//!
//! **Temporary allocations during computation** -> [`ProtectScope`]
//! ```ignore
//! unsafe fn compute(x: SEXP) -> SEXP {
//!     let scope = ProtectScope::new();
//!     let temp = scope.protect(Rf_allocVector(REALSXP, 100));
//!     // ... work with temp ...
//!     result.into_raw()
//! } // UNPROTECT(n) called automatically
//! ```
//!
//! **R objects surviving across `.Call`s** -> [`preserve`]
//! ```ignore
//! // In RAllocator or similar long-lived context
//! let cell = unsafe { preserve::insert(backing_vec) };
//! // ... use across multiple .Calls ...
//! unsafe { preserve::release(cell) };
//! ```
//!
//! **Rust data owned by R** -> [`ExternalPtr`](struct@ExternalPtr)
//! ```ignore
//! #[miniextendr]
//! fn create_model() -> ExternalPtr<MyModel> {
//!     ExternalPtr::new(MyModel::new())
//! } // R owns it; Drop runs when R GCs
//! ```
//!
//! Note: ALTREP trait methods receive raw SEXP pointers from R's runtime.
//! These are safe to dereference because R guarantees valid SEXPs in ALTREP callbacks.
//!
//! ## Threading and safety
//!
//! R uses `longjmp` for errors, which can bypass Rust destructors. The default
//! pattern is to run Rust logic on a worker thread and marshal R API calls back
//! to the main R thread via `with_r_thread`. Most FFI wrappers are
//! main-thread routed via `#[r_ffi_checked]`. Use unchecked variants only when
//! you have arranged a safe context.
//!
//! With the `nonapi` feature, miniextendr can disable R's stack checking to allow
//! calls from other threads. R is still not thread-safe; serialize all R API use.
//!
//! ## Feature Flags
//!
//! ### Core Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `nonapi` | Non-API R symbols (stack controls, mutable `DATAPTR`). May break with R updates. |
//! | `rayon` | Parallel iterators via Rayon. Adds `RParallelIterator`, `RParallelExtend`. |
//! | `connections` | Experimental R connection framework. **Unstable R API.** |
//! | `indicatif` | Progress bars via R console. Requires `nonapi`. |
//! | `vctrs` | Access to vctrs C API (`obj_is_vector`, `short_vec_size`, `short_vec_recycle`). |
//!
//! ### Type Conversions (Scalars & Vectors)
//!
//! | Feature | Rust Type | R Type | Notes |
//! |---------|-----------|--------|-------|
//! | `either` | `Either<L, R>` | Tries L then R | Union-like dispatch |
//! | `uuid` | `Uuid`, `Vec<Uuid>` | `character` | UUID ↔ string |
//! | `regex` | `Regex` | `character(1)` | Compiles pattern from R |
//! | `url` | `Url`, `Vec<Url>` | `character` | Validated URLs |
//! | `time` | `OffsetDateTime`, `Date` | `POSIXct`, `Date` | Date/time conversions |
//! | `ordered-float` | `OrderedFloat<f64>` | `numeric` | NaN-orderable floats |
//! | `num-bigint` | `BigInt`, `BigUint` | `character` | Arbitrary precision via strings |
//! | `rust_decimal` | `Decimal` | `character` | Fixed-point decimals |
//! | `num-complex` | `Complex<f64>` | `complex` | Native R complex support |
//! | `indexmap` | `IndexMap<String, T>` | named `list` | Preserves insertion order |
//! | `bitflags` | `RFlags<T>` | `integer` | Bitflags ↔ integer |
//! | `bitvec` | `RBitVec` | `logical` | Bit vectors ↔ logical |
//!
//! ### Matrix & Array Libraries
//!
//! | Feature | Types | Conversions |
//! |---------|-------|-------------|
//! | `ndarray` | `Array1`–`Array6`, `ArrayD`, views | R vectors/matrices ↔ ndarray |
//! | `nalgebra` | `DVector`, `DMatrix` | R vectors/matrices ↔ nalgebra |
//!
//! ### Serialization
//!
//! | Feature | Traits/Modules | Description |
//! |---------|----------------|-------------|
//! | `serde` | `RSerialize`, `RDeserialize` | JSON serialization via serde_json |
//! | `serde_r` | `RSerializeNative`, `RDeserializeNative` | Direct Rust ↔ R (no JSON) |
//! | `serde_full` | Both above | Enables `serde` + `serde_r` |
//!
//! ### Adapter Traits (Generic Operations)
//!
//! | Feature | Traits | Use Case |
//! |---------|--------|----------|
//! | `num-traits` | `RNum`, `RSigned`, `RFloat` | Generic numeric operations |
//! | `bytes` | `RBuf`, `RBufMut` | Byte buffer operations |
//!
//! ### Text & Data Processing
//!
//! | Feature | Types/Functions | Description |
//! |---------|-----------------|-------------|
//! | `aho-corasick` | `AhoCorasick`, `aho_compile` | Fast multi-pattern string search |
//! | `toml` | `TomlValue`, `toml_from_str` | TOML parsing and serialization |
//! | `tabled` | `table_to_string` | ASCII/Unicode table formatting |
//! | `sha2` | `sha256_str`, `sha512_bytes` | Cryptographic hashing |
//!
//! ### Random Number Generation
//!
//! | Feature | Types | Description |
//! |---------|-------|-------------|
//! | `rand` | `RRng`, `RDistributions` | Wraps R's RNG with `rand` traits |
//! | `rand_distr` | Re-exports `rand_distr` | Additional distributions (Normal, Exp, etc.) |
//!
//! ### Binary Data
//!
//! | Feature | Types | Description |
//! |---------|-------|-------------|
//! | `raw_conversions` | `Raw<T>`, `RawSlice<T>` | POD types ↔ raw vectors via bytemuck |
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
pub use miniextendr_macros::typed_list;
// Note: RFactor derive macro is re-exported - it shares the name with the RFactor trait
// but they're in different namespaces (derive macros vs types/traits)
#[doc(inline)]
pub use miniextendr_macros::{
    AltrepComplex, AltrepInteger, AltrepList, AltrepLogical, AltrepRaw, AltrepReal, AltrepString,
    IntoList, PreferExternalPtr, PreferList, PreferRNativeType, RFactor, TryFromList,
};
#[cfg(feature = "vctrs")]
#[doc(inline)]
pub use miniextendr_macros::Vctrs;

pub mod altrep;
pub mod altrep_bridge;
pub mod altrep_data;
pub mod altrep_impl;
pub mod altrep_traits;

// Re-export for backward compatibility - RegisterAltrep was moved from altrep_registration to altrep
#[doc(hidden)]
pub mod altrep_registration {
    pub use crate::altrep::RegisterAltrep;
}
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

// ALTREP package name global - set by C entrypoint before ALTREP registration
// This is a pointer to a null-terminated C string provided by C code.
// Default: c"unknown" for safety if not set.
use std::sync::atomic::{AtomicPtr, Ordering};
static ALTREP_PKG_NAME_PTR: AtomicPtr<std::ffi::c_char> =
    AtomicPtr::new(c"unknown".as_ptr() as *mut _);

/// Returns the current ALTREP package name as a C string pointer.
/// This is set by the C entrypoint before ALTREP registration.
#[doc(hidden)]
pub struct AltrepPkgName;

impl AltrepPkgName {
    /// Get the package name pointer.
    #[inline]
    pub fn as_ptr() -> *const std::ffi::c_char {
        ALTREP_PKG_NAME_PTR.load(Ordering::Acquire)
    }
}

/// Opaque handle for ALTREP package name.
/// Use `ALTREP_PKG_NAME.as_ptr()` to get the C string pointer.
#[doc(hidden)]
pub static ALTREP_PKG_NAME: AltrepPkgName = AltrepPkgName;

/// Set the ALTREP package name. Called from C entrypoint.
/// # Safety
/// The provided pointer must point to a valid null-terminated C string
/// that lives for the duration of the R session.
#[doc(hidden)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn miniextendr_set_altrep_pkg_name(name: *const std::ffi::c_char) {
    ALTREP_PKG_NAME_PTR.store(name as *mut _, Ordering::Release);
}

// Note: SexpExt is pub(crate), imported directly in modules that need it
pub mod from_r;
pub mod into_r;
pub use into_r::{Altrep, IntoR};
pub mod into_r_as;
pub use into_r_as::{IntoRAs, StorageCoerceError};
pub mod unwind_protect;
pub mod worker;

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
pub use from_r::{SexpError, SexpLengthError, SexpNaError, SexpTypeError, TryFromSexp};

// Encoding / locale probing (mainly for debugging; some parts require `nonapi`)
// NOTE: Disabled because it references non-exported symbols from R's Defn.h
// (e.g., known_to_be_utf8, utf8locale) that cause dlopen failures at runtime.
// #[cfg(feature = "nonapi")]
// pub mod encoding;

// Note: RNativeType is pub(crate), imported directly in modules that need it

pub mod backtrace;

pub mod coerce;
pub use coerce::{Coerce, CoerceError, Coerced, TryCoerce};

/// Traits for R's `as.<class>()` coercion functions.
///
/// This module provides traits for implementing R's generic coercion methods
/// (`as.data.frame()`, `as.list()`, `as.character()`, etc.) for Rust types
/// wrapped in [`ExternalPtr`].
///
/// See the [`as_coerce`] module documentation for usage examples.
pub mod as_coerce;
pub use as_coerce::{
    // Error type
    AsCoerceError,
    // Marker trait
    AsCoercible,
    // Core coercion traits
    AsCharacter, AsComplex, AsDataFrame, AsDate, AsEnvironment, AsFactor, AsFunction, AsInteger,
    AsList as AsListCoerce, AsLogical, AsMatrix, AsNumeric, AsPOSIXct, AsRaw, AsVector,
    // Helpers
    SUPPORTED_AS_GENERICS, is_supported_as_generic,
};

pub mod convert;
pub mod dots;
pub mod list;
pub mod strvec;
pub mod typed_list;
pub use convert::{AsExternalPtr, AsExternalPtrExt, AsList, AsListExt, AsRNative, AsRNativeExt};
pub use list::{IntoList, List, ListAccumulator, ListBuilder, ListMut, TryFromList, collect_list};
pub use strvec::{StrVec, StrVecBuilder};
pub use typed_list::{
    TypeSpec, TypedEntry, TypedList, TypedListError, TypedListSpec, actual_type_string,
    sexptype_name, validate_list,
};

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

// Reference-counted GC protection (BTreeMap + VECSXP backing)
pub mod refcount_protect;
pub use refcount_protect::{
    Arena, ArenaGuard, HashMapArena, MapStorage, RefCountedArena, RefCountedGuard,
    ThreadLocalArena, ThreadLocalHashArena,
};

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

/// Optional vctrs C API support.
///
/// Provides access to vctrs' maturing C API functions for vector operations.
/// This is an optional dependency - if vctrs is not available at runtime,
/// [`init_vctrs`] will return an error.
///
/// Available functions:
/// - [`obj_is_vector`](vctrs::obj_is_vector) - Check if object is a vector
/// - [`short_vec_size`](vctrs::short_vec_size) - Get vector size
/// - [`short_vec_recycle`](vctrs::short_vec_recycle) - Recycle to target size
///
/// Enable with `features = ["vctrs"]`.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::vctrs::{init_vctrs, obj_is_vector, short_vec_size};
///
/// // In R_init_<pkg>:
/// if init_vctrs().is_ok() {
///     // vctrs support enabled
/// }
///
/// // Later:
/// if obj_is_vector(x)? {
///     let n = short_vec_size(x)?;
/// }
/// ```
#[cfg(feature = "vctrs")]
pub mod vctrs;
#[cfg(feature = "vctrs")]
pub use vctrs::{
    // Phase C traits
    IntoVctrs,
    // Error types
    VctrsBuildError,
    VctrsClass,
    VctrsError,
    VctrsKind,
    VctrsListOf,
    VctrsRecord,
    // Extension trait
    VctrsSexpExt,
    // Initialization
    init_vctrs,
    // Construction helpers
    new_list_of,
    new_rcrd,
    new_vctr,
};

// Stub for miniextendr_init_vctrs when vctrs feature is disabled.
// Always returns 1 (NotAvailable) so C code can call it unconditionally.
#[cfg(not(feature = "vctrs"))]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn miniextendr_init_vctrs() -> i32 {
    1 // NotAvailable
}

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
    RIterator, RMakeIter, ROrd, RPartialOrd, RToVec,
};

/// This is used to ensure the macros of `miniextendr-macros` treat this crate as a "user crate"
/// atleast in the `macro_coverage`
#[doc(hidden)]
extern crate self as miniextendr_api;

#[doc(hidden)]
pub mod macro_coverage;

// =============================================================================
// Optional integrations with external crates (feature-gated)
// =============================================================================
//
// All optional feature integrations are organized in the `optionals` module.
// Types are re-exported at crate root for backwards compatibility.

/// Optional feature integrations with third-party crates.
///
/// This module contains all feature-gated integrations with external crates.
/// Each submodule is only compiled when its corresponding feature is enabled.
/// See the [module documentation][optionals] for a complete list of available features.
pub mod optionals;

// Re-export optional types at crate root for backwards compatibility
#[cfg(feature = "rayon")]
pub use optionals::rayon_bridge;
#[cfg(feature = "rayon")]
pub use optionals::{RParallelExtend, RParallelIterator};

#[cfg(feature = "rand_distr")]
pub use optionals::rand_distr;
#[cfg(feature = "rand")]
pub use optionals::rand_impl;
#[cfg(feature = "rand")]
pub use optionals::{RDistributionOps, RDistributions, RRng, RRngOps};

#[cfg(feature = "either")]
pub use optionals::either_impl;
#[cfg(feature = "either")]
pub use optionals::{Either, Left, Right};

#[cfg(feature = "ndarray")]
pub use optionals::ndarray_impl;
#[cfg(feature = "ndarray")]
pub use optionals::{
    ArcArray1, ArcArray2, Array0, Array1, Array2, Array3, Array4, Array5, Array6, ArrayD,
    ArrayView0, ArrayView1, ArrayView2, ArrayView3, ArrayView4, ArrayView5, ArrayView6, ArrayViewD,
    ArrayViewMut0, ArrayViewMut1, ArrayViewMut2, ArrayViewMut3, ArrayViewMut4, ArrayViewMut5,
    ArrayViewMut6, ArrayViewMutD, Ix0, Ix1, Ix2, Ix3, Ix4, Ix5, Ix6, IxDyn, RNdArrayOps, RNdIndex,
    RNdSlice, RNdSlice2D, ShapeBuilder,
};

#[cfg(feature = "nalgebra")]
pub use optionals::nalgebra_impl;
#[cfg(feature = "nalgebra")]
pub use optionals::{DMatrix, DVector, RMatrixOps, RVectorOps};

#[cfg(feature = "num-bigint")]
pub use optionals::num_bigint_impl;
#[cfg(feature = "num-bigint")]
pub use optionals::{BigInt, BigUint, RBigIntBitOps, RBigIntOps, RBigUintBitOps, RBigUintOps};

#[cfg(feature = "rust_decimal")]
pub use optionals::rust_decimal_impl;
#[cfg(feature = "rust_decimal")]
pub use optionals::{Decimal, RDecimalOps};

#[cfg(feature = "ordered-float")]
pub use optionals::ordered_float_impl;
#[cfg(feature = "ordered-float")]
pub use optionals::{OrderedFloat, ROrderedFloatOps};

#[cfg(feature = "num-complex")]
pub use optionals::num_complex_impl;
#[cfg(feature = "num-complex")]
pub use optionals::{Complex, RComplexOps};

#[cfg(feature = "num-traits")]
pub use optionals::num_traits_impl;
#[cfg(feature = "num-traits")]
pub use optionals::{RFloat, RNum, RSigned};

#[cfg(feature = "uuid")]
pub use optionals::uuid_impl;
#[cfg(feature = "uuid")]
pub use optionals::{RUuidOps, Uuid, uuid_helpers};

#[cfg(feature = "regex")]
pub use optionals::regex_impl;
#[cfg(feature = "regex")]
pub use optionals::{CaptureGroups, RCaptureGroups, RRegexOps, Regex};

#[cfg(feature = "url")]
pub use optionals::url_impl;
#[cfg(feature = "url")]
pub use optionals::{RUrlOps, Url, url_helpers};

#[cfg(feature = "aho-corasick")]
pub use optionals::aho_corasick_impl;
#[cfg(feature = "aho-corasick")]
pub use optionals::{
    AhoCorasick, RAhoCorasickOps, aho_compile, aho_count_matches, aho_find_all, aho_find_all_flat,
    aho_find_first, aho_is_match, aho_replace_all,
};

#[cfg(feature = "indexmap")]
pub use optionals::indexmap_impl;
#[cfg(feature = "indexmap")]
pub use optionals::{IndexMap, RIndexMapOps};

#[cfg(feature = "time")]
pub use optionals::time_impl;
#[cfg(feature = "time")]
pub use optionals::{Date, Duration, OffsetDateTime, RDateTimeFormat, RDuration};

#[cfg(feature = "serde")]
pub use optionals::serde_impl;
#[cfg(feature = "serde")]
pub use optionals::{
    JsonValue, RDeserialize, RJsonValueOps, RSerialize, json_from_sexp, json_from_sexp_permissive,
    json_from_sexp_strict, json_into_sexp,
};
#[cfg(feature = "serde")]
pub use serde;

#[cfg(feature = "toml")]
pub use optionals::toml_impl;
#[cfg(feature = "toml")]
pub use optionals::{RTomlOps, TomlValue, toml_from_str, toml_to_string, toml_to_string_pretty};

#[cfg(feature = "bytes")]
pub use optionals::bytes_impl;
#[cfg(feature = "bytes")]
pub use optionals::{Buf, BufMut, Bytes, BytesMut, RBuf, RBufMut};

#[cfg(feature = "sha2")]
pub use optionals::sha2_impl;
#[cfg(feature = "sha2")]
pub use optionals::{sha256_bytes, sha256_str, sha512_bytes, sha512_str};

#[cfg(feature = "bitflags")]
pub use optionals::bitflags_impl;
#[cfg(feature = "bitflags")]
pub use optionals::{Flags, RFlags};

#[cfg(feature = "bitvec")]
pub use optionals::bitvec_impl;
#[cfg(feature = "bitvec")]
pub use optionals::{BitVec, Lsb0, Msb0, RBitVec};

#[cfg(feature = "tabled")]
pub use optionals::tabled_impl;
#[cfg(feature = "tabled")]
pub use optionals::{
    Builder, Table, Tabled, builder_to_string, table_from_vecs, table_to_string,
    table_to_string_opts, table_to_string_styled,
};

/// N-dimensional R arrays with const generic dimension count.
pub mod rarray;
pub use rarray::{RArray, RArray3D, RMatrix, RVector};

/// Direct R serialization via serde (no JSON intermediate).
///
/// Provides efficient type-preserving conversions between Rust types and native R objects:
/// - [`RSerializeNative`][serde_r::RSerializeNative] - Convert Rust → R (struct → named list)
/// - [`RDeserializeNative`][serde_r::RDeserializeNative] - Convert R → Rust (named list → struct)
///
/// Enable with `features = ["serde_r"]`.
///
/// See the [`serde_r`] module documentation for type mappings and examples.
#[cfg(feature = "serde_r")]
pub mod serde_r;
#[cfg(feature = "serde_r")]
pub use serde_r::{RDeserializeNative, RDeserializer, RSerdeError, RSerializeNative, RSerializer};

/// Integration with the `bytemuck` crate for POD type conversions.
///
/// Provides explicit, safe conversions between Rust POD (Plain Old Data) types
/// and R raw vectors:
/// - `Raw<T>` - Single POD value (headerless, native layout)
/// - `RawSlice<T>` - Sequence of POD values (headerless)
/// - `RawTagged<T>` / `RawSliceTagged<T>` - With header metadata
///
/// Enable with `features = ["raw_conversions"]`.
///
/// ```ignore
/// use bytemuck::{Pod, Zeroable};
/// use miniextendr_api::raw_conversions::{Raw, RawSlice};
///
/// #[derive(Copy, Clone, Pod, Zeroable)]
/// #[repr(C)]
/// struct Vec3 { x: f32, y: f32, z: f32 }
///
/// #[miniextendr]
/// fn encode(x: f64, y: f64, z: f64) -> Raw<Vec3> {
///     Raw(Vec3 { x: x as f32, y: y as f32, z: z as f32 })
/// }
/// ```
#[cfg(feature = "raw_conversions")]
pub mod raw_conversions;
#[cfg(feature = "raw_conversions")]
pub use raw_conversions::{
    Pod, Raw, RawError, RawHeader, RawSlice, RawSliceTagged, RawTagged, Zeroable, raw_from_bytes,
    raw_slice_from_bytes, raw_slice_to_bytes, raw_to_bytes,
};

/// Factor support for enum ↔ R factor conversions.
///
/// Provides the [`RFactor`] trait for converting Rust enums to/from R factors.
/// Use `#[derive(RFactor)]` on C-style enums to auto-generate the implementation.
///
/// # Example
///
/// ```ignore
/// use miniextendr_api::RFactor;
///
/// #[derive(Copy, Clone, RFactor)]
/// enum Color { Red, Green, Blue }
///
/// #[miniextendr]
/// fn color_name(c: Color) -> &'static str {
///     match c {
///         Color::Red => "red",
///         Color::Green => "green",
///         Color::Blue => "blue",
///     }
/// }
/// ```
pub mod factor;
pub use factor::{
    Factor, FactorMut, FactorOptionVec, FactorVec, RFactor, build_factor, build_levels_sexp,
    build_levels_sexp_cached, factor_from_sexp,
};
