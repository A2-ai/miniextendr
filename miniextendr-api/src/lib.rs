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
pub use miniextendr_macros::typed_list;
// Note: RFactor derive macro is re-exported - it shares the name with the RFactor trait
// but they're in different namespaces (derive macros vs types/traits)
#[doc(inline)]
pub use miniextendr_macros::{
    AltrepComplex, AltrepInteger, AltrepList, AltrepLogical, AltrepRaw, AltrepReal, AltrepString,
    IntoList, PreferExternalPtr, PreferList, PreferRNativeType, RFactor, TryFromList,
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
#[cfg(feature = "rayon")]
pub use rayon_bridge::{RParallelExtend, RParallelIterator};
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

pub mod convert;
pub mod dots;
pub mod list;
pub mod typed_list;
pub use convert::{AsExternalPtr, AsExternalPtrExt, AsList, AsListExt, AsRNative, AsRNativeExt};
pub use list::{IntoList, List, TryFromList};
pub use typed_list::{
    TypedEntry, TypedList, TypedListError, TypedListSpec, TypeSpec, actual_type_string,
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
pub use ndarray_impl::{
    // Shared ownership
    ArcArray1,
    ArcArray2,
    // Owned arrays
    Array0,
    Array1,
    Array2,
    Array3,
    Array4,
    Array5,
    Array6,
    ArrayD,
    // Read-only views
    ArrayView0,
    ArrayView1,
    ArrayView2,
    ArrayView3,
    ArrayView4,
    ArrayView5,
    ArrayView6,
    ArrayViewD,
    // Mutable views
    ArrayViewMut0,
    ArrayViewMut1,
    ArrayViewMut2,
    ArrayViewMut3,
    ArrayViewMut4,
    ArrayViewMut5,
    ArrayViewMut6,
    ArrayViewMutD,
    // Index types
    Ix0,
    Ix1,
    Ix2,
    Ix3,
    Ix4,
    Ix5,
    Ix6,
    IxDyn,
    // Adapter traits
    RNdArrayOps,
    RNdIndex,
    RNdSlice,
    RNdSlice2D,
    // Shape builder
    ShapeBuilder,
};

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
pub use serde_impl::{
    JsonValue, RDeserialize, RJsonValueOps, RSerialize, json_from_sexp, json_from_sexp_permissive,
    json_from_sexp_strict, json_into_sexp,
};

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

/// Integration with the `bytes` crate for byte buffer operations.
///
/// Provides adapter traits for byte buffer types:
/// - [`RBuf`][bytes_impl::RBuf] - Read operations (get_u8, get_i32, etc.)
/// - [`RBufMut`][bytes_impl::RBufMut] - Write operations (put_u8, put_slice, etc.)
///
/// Also re-exports the core `bytes` types: `Bytes`, `BytesMut`, `Buf`, `BufMut`.
///
/// Since `Buf` and `BufMut` traits require `&mut self`, implementations must use
/// interior mutability (e.g., `RefCell`). There are no blanket implementations.
///
/// Enable with `features = ["bytes"]`.
///
/// ```ignore
/// use miniextendr_api::bytes_impl::{RBuf, RBufMut, Bytes, BytesMut};
/// use std::cell::RefCell;
///
/// #[derive(ExternalPtr)]
/// struct MyBuffer {
///     data: RefCell<BytesMut>,
/// }
///
/// // Implement RBuf or RBufMut, then register with miniextendr_module!
/// #[miniextendr]
/// impl RBuf for MyBuffer { /* ... */ }
///
/// miniextendr_module! {
///     mod mybuffer;
///     impl RBuf for MyBuffer;
/// }
/// ```
#[cfg(feature = "bytes")]
pub mod bytes_impl;
#[cfg(feature = "bytes")]
pub use bytes_impl::{Buf, BufMut, Bytes, BytesMut, RBuf, RBufMut};

/// Integration with the `num-complex` crate for complex number operations.
///
/// Provides conversions between R complex vectors (`CPLXSXP`) and `Complex<f64>`:
/// - `Complex<f64>` ⇄ `complex(1)` (single complex)
/// - `Vec<Complex<f64>>` ⇄ `complex` vector
/// - `Option<Complex<f64>>` for NA support
///
/// Also provides the [`RComplexOps`][num_complex_impl::RComplexOps] adapter trait
/// for exposing complex number inspection methods to R.
///
/// Enable with `features = ["num-complex"]`.
///
/// ```ignore
/// use num_complex::Complex;
///
/// #[miniextendr]
/// fn add_complex(a: Complex<f64>, b: Complex<f64>) -> Complex<f64> {
///     a + b
/// }
/// ```
#[cfg(feature = "num-complex")]
pub mod num_complex_impl;
#[cfg(feature = "num-complex")]
pub use num_complex_impl::{Complex, RComplexOps};

/// Integration with the `url` crate for URL parsing and validation.
///
/// Provides conversions between R character vectors and `Url` types:
/// - `Url` ⇄ `character(1)` (validated URL)
/// - `Vec<Url>` ⇄ `character` vector
/// - `Option<Url>` for NA support
///
/// Also provides the [`RUrlOps`][url_impl::RUrlOps] adapter trait
/// for exposing URL inspection methods to R (scheme, host, port, path, etc.).
///
/// Enable with `features = ["url"]`.
///
/// ```ignore
/// use url::Url;
///
/// #[miniextendr]
/// fn get_domain(url: Url) -> Option<String> {
///     url.host_str().map(|s| s.to_string())
/// }
/// ```
#[cfg(feature = "url")]
pub mod url_impl;
#[cfg(feature = "url")]
pub use url_impl::{RUrlOps, Url, url_helpers};

/// Integration with the `sha2` crate for cryptographic hashing.
///
/// Provides SHA-256 and SHA-512 hashing helpers:
/// - `sha256_str(s)` / `sha256_bytes(data)` - 64-char hex hash
/// - `sha512_str(s)` / `sha512_bytes(data)` - 128-char hex hash
/// - Vector variants for batch hashing
///
/// Enable with `features = ["sha2"]`.
///
/// ```ignore
/// use miniextendr_api::sha2_impl::sha256_str;
///
/// #[miniextendr]
/// fn hash_input(s: &str) -> String {
///     sha256_str(s)
/// }
/// ```
#[cfg(feature = "sha2")]
pub mod sha2_impl;
#[cfg(feature = "sha2")]
pub use sha2_impl::{sha256_bytes, sha256_str, sha512_bytes, sha512_str};

/// Integration with the `bitflags` crate.
///
/// Provides [`RFlags<T>`][bitflags_impl::RFlags] wrapper for bitflags ↔ integer conversions:
/// - `RFlags<T>` - Wrapper implementing `TryFromSexp` and `IntoR`
/// - `flags_from_i32_strict(v)` - Strict conversion (rejects unknown bits)
/// - `flags_from_i32_truncate(v)` - Truncating conversion (ignores unknown bits)
/// - `flags_to_i32(flags)` - Convert flags to integer
///
/// Enable with `features = ["bitflags"]`.
///
/// ```ignore
/// use bitflags::bitflags;
/// use miniextendr_api::bitflags_impl::RFlags;
///
/// bitflags! {
///     #[derive(Clone, Copy, Debug)]
///     pub struct Mode: u8 {
///         const READ = 0b01;
///         const WRITE = 0b10;
///     }
/// }
///
/// #[miniextendr]
/// fn check_read(mode: RFlags<Mode>) -> bool {
///     mode.contains(Mode::READ)
/// }
/// ```
#[cfg(feature = "bitflags")]
pub mod bitflags_impl;
#[cfg(feature = "bitflags")]
pub use bitflags_impl::{Flags, RFlags};

/// Integration with the `bitvec` crate.
///
/// Provides conversions between R logical vectors and `BitVec` types:
/// - `RBitVec` (type alias for `BitVec<u8, Lsb0>`) ↔ logical vector
/// - NA values cause error (no NA representation in BitVec)
///
/// Enable with `features = ["bitvec"]`.
///
/// ```ignore
/// use miniextendr_api::bitvec_impl::RBitVec;
///
/// #[miniextendr]
/// fn count_true(bits: RBitVec) -> i32 {
///     bits.count_ones() as i32
/// }
/// ```
#[cfg(feature = "bitvec")]
pub mod bitvec_impl;
#[cfg(feature = "bitvec")]
pub use bitvec_impl::{BitVec, Lsb0, Msb0, RBitVec};

/// Integration with the `aho-corasick` crate for multi-pattern string search.
///
/// Provides fast multi-pattern search using the Aho-Corasick algorithm:
/// - `AhoCorasick` - Compiled automaton from pattern list
/// - `aho_compile(patterns)` - Build automaton from patterns
/// - `aho_find_all(ac, haystack)` - Find all matches
/// - `RAhoCorasickOps` - Adapter trait for R interop
///
/// Enable with `features = ["aho-corasick"]`.
///
/// ```ignore
/// use miniextendr_api::aho_corasick_impl::{aho_compile, aho_find_all};
///
/// #[miniextendr]
/// fn search_patterns(patterns: Vec<String>, text: &str) -> Vec<i32> {
///     let ac = aho_compile(&patterns).unwrap();
///     aho_find_all_flat(&ac, text)
/// }
/// ```
#[cfg(feature = "aho-corasick")]
pub mod aho_corasick_impl;
#[cfg(feature = "aho-corasick")]
pub use aho_corasick_impl::{
    AhoCorasick, RAhoCorasickOps, aho_compile, aho_count_matches, aho_find_all, aho_find_all_flat,
    aho_find_first, aho_is_match, aho_replace_all,
};

/// Integration with the `toml` crate for TOML value conversions.
///
/// Provides conversions between TOML values and R types:
/// - `TomlValue` ⇄ R lists/vectors
/// - `toml_from_str(s)` - Parse TOML string to value
/// - `toml_to_string(v)` - Serialize value to TOML string
/// - `RTomlOps` - Adapter trait for TOML value inspection
///
/// Enable with `features = ["toml"]`.
///
/// ```ignore
/// use miniextendr_api::toml_impl::{toml_from_str, TomlValue};
///
/// #[miniextendr]
/// fn parse_config(text: &str) -> TomlValue {
///     toml_from_str(text).unwrap()
/// }
/// ```
#[cfg(feature = "toml")]
pub mod toml_impl;
#[cfg(feature = "toml")]
pub use toml_impl::{RTomlOps, TomlValue, toml_from_str, toml_to_string, toml_to_string_pretty};

/// Integration with the `tabled` crate for table formatting.
///
/// Provides helpers for formatting data as ASCII/Unicode tables:
/// - `table_to_string(rows)` - Format rows as table string
/// - `table_to_string_opts(rows, max_width, align, trim)` - With options
/// - `table_from_vecs(headers, rows)` - Build from vectors
/// - `builder_to_string(builder)` - Dynamic table building
///
/// Enable with `features = ["tabled"]`.
///
/// ```ignore
/// use tabled::Tabled;
/// use miniextendr_api::tabled_impl::table_to_string;
///
/// #[derive(Tabled)]
/// struct Item { name: String, count: i32 }
///
/// #[miniextendr]
/// fn format_items(items: Vec<Item>) -> String {
///     table_to_string(&items)
/// }
/// ```
#[cfg(feature = "tabled")]
pub mod tabled_impl;
#[cfg(feature = "tabled")]
pub use tabled_impl::{
    Builder, Table, Tabled, builder_to_string, table_from_vecs, table_to_string,
    table_to_string_opts, table_to_string_styled,
};

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
