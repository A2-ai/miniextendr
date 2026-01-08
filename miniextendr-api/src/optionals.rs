//! Optional feature integrations with third-party crates.
//!
//! This module contains all feature-gated integrations with external crates.
//! Each submodule is only compiled when its corresponding feature is enabled.
//!
//! # Available Features
//!
//! | Feature | Module | Description |
//! |---------|--------|-------------|
//! | `rayon` | [`rayon_bridge`] | Parallel computation with R interop |
//! | `rand` | [`rand_impl`] | R's RNG wrapped for `rand` crate |
//! | `rand_distr` | - | Re-exports `rand_distr` distributions |
//! | `either` | [`either_impl`] | `Either<L, R>` conversions |
//! | `ndarray` | [`ndarray_impl`] | N-dimensional array conversions |
//! | `nalgebra` | [`nalgebra_impl`] | Linear algebra type conversions |
//! | `num-bigint` | [`num_bigint_impl`] | Big integer support |
//! | `rust_decimal` | [`rust_decimal_impl`] | Decimal number support |
//! | `ordered-float` | [`ordered_float_impl`] | Ordered floats for sorting |
//! | `uuid` | [`uuid_impl`] | UUID conversions |
//! | `regex` | [`regex_impl`] | Compiled regex from R strings |
//! | `indexmap` | [`indexmap_impl`] | Order-preserving maps |
//! | `time` | [`time_impl`] | Date/time conversions |
//! | `serde` | [`serde_impl`] | JSON serialization |
//! | `num-traits` | [`num_traits_impl`] | Generic numeric operations |
//! | `bytes` | [`bytes_impl`] | Byte buffer operations |
//! | `num-complex` | [`num_complex_impl`] | Complex number support |
//! | `url` | [`url_impl`] | URL parsing and validation |
//! | `sha2` | [`sha2_impl`] | Cryptographic hashing |
//! | `bitflags` | [`bitflags_impl`] | Bitflag conversions |
//! | `bitvec` | [`bitvec_impl`] | Bit vector conversions |
//! | `aho-corasick` | [`aho_corasick_impl`] | Multi-pattern string search |
//! | `toml` | [`toml_impl`] | TOML parsing |
//! | `tabled` | [`tabled_impl`] | Table formatting |

// =============================================================================
// Rayon - Parallel computation
// =============================================================================

/// Rayon integration for parallel computation with R interop.
///
/// Provides:
/// - [`with_r_vec`][rayon_bridge::with_r_vec] - Zero-copy parallel fill into R vectors
/// - [`with_r_matrix`][rayon_bridge::with_r_matrix] - Parallel matrix fill
/// - [`reduce`][rayon_bridge::reduce] - Parallel reductions returning R scalars
/// - [`RParallelIterator`][rayon_bridge::RParallelIterator] - Adapter trait for R
///
/// Enable with `features = ["rayon"]`.
#[cfg(feature = "rayon")]
pub mod rayon_bridge;
#[cfg(feature = "rayon")]
pub use rayon_bridge::{RParallelExtend, RParallelIterator};

// =============================================================================
// Rand - Random number generation
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
#[cfg(feature = "rand_distr")]
pub use rand_distr;

// =============================================================================
// Either - Sum type
// =============================================================================

/// Integration with the `either` crate.
///
/// Provides [`TryFromSexp`] and [`IntoR`] for [`Either<L, R>`][either::Either].
///
/// Enable with `features = ["either"]`.
#[cfg(feature = "either")]
pub mod either_impl;
#[cfg(feature = "either")]
pub use either_impl::{Either, Left, Right};

// =============================================================================
// Ndarray - N-dimensional arrays
// =============================================================================

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

// =============================================================================
// Nalgebra - Linear algebra
// =============================================================================

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

// =============================================================================
// Numeric types
// =============================================================================

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

/// Integration with the `num-complex` crate for complex number operations.
///
/// Provides conversions between R complex vectors (`CPLXSXP`) and `Complex<f64>`.
///
/// Enable with `features = ["num-complex"]`.
#[cfg(feature = "num-complex")]
pub mod num_complex_impl;
#[cfg(feature = "num-complex")]
pub use num_complex_impl::{Complex, RComplexOps};

/// Integration with the `num-traits` crate for generic numeric operations.
///
/// Provides adapter traits for generic numeric types:
/// - [`RNum`][num_traits_impl::RNum] - Basic numeric operations
/// - [`RSigned`][num_traits_impl::RSigned] - Signed number operations
/// - [`RFloat`][num_traits_impl::RFloat] - Floating-point operations
///
/// Enable with `features = ["num-traits"]`.
#[cfg(feature = "num-traits")]
pub mod num_traits_impl;
#[cfg(feature = "num-traits")]
pub use num_traits_impl::{RFloat, RNum, RSigned};

// =============================================================================
// String/Text types
// =============================================================================

/// UUID support via the `uuid` crate.
///
/// Provides conversions between R character vectors and `Uuid` types.
///
/// Enable with `features = ["uuid"]`.
#[cfg(feature = "uuid")]
pub mod uuid_impl;
#[cfg(feature = "uuid")]
pub use uuid_impl::{RUuidOps, Uuid, uuid_helpers};

/// Regex support via the `regex` crate.
///
/// Provides compiled regular expressions from R character patterns.
///
/// Enable with `features = ["regex"]`.
#[cfg(feature = "regex")]
pub mod regex_impl;
#[cfg(feature = "regex")]
pub use regex_impl::{CaptureGroups, RCaptureGroups, RRegexOps, Regex};

/// Integration with the `url` crate for URL parsing and validation.
///
/// Provides conversions between R character vectors and `Url` types.
///
/// Enable with `features = ["url"]`.
#[cfg(feature = "url")]
pub mod url_impl;
#[cfg(feature = "url")]
pub use url_impl::{RUrlOps, Url, url_helpers};

/// Integration with the `aho-corasick` crate for multi-pattern string search.
///
/// Provides fast multi-pattern search using the Aho-Corasick algorithm.
///
/// Enable with `features = ["aho-corasick"]`.
#[cfg(feature = "aho-corasick")]
pub mod aho_corasick_impl;
#[cfg(feature = "aho-corasick")]
pub use aho_corasick_impl::{
    AhoCorasick, RAhoCorasickOps, aho_compile, aho_count_matches, aho_find_all, aho_find_all_flat,
    aho_find_first, aho_is_match, aho_replace_all,
};

// =============================================================================
// Collections
// =============================================================================

/// IndexMap support via the `indexmap` crate.
///
/// Provides conversions between R named lists and `IndexMap<String, T>`.
///
/// Enable with `features = ["indexmap"]`.
#[cfg(feature = "indexmap")]
pub mod indexmap_impl;
#[cfg(feature = "indexmap")]
pub use indexmap_impl::{IndexMap, RIndexMapOps};

// =============================================================================
// Date/Time
// =============================================================================

/// Time and date support via the `time` crate.
///
/// Provides conversions between R date/time types and `time` crate types.
///
/// Enable with `features = ["time"]`.
#[cfg(feature = "time")]
pub mod time_impl;
#[cfg(feature = "time")]
pub use time_impl::{Date, Duration, OffsetDateTime, RDateTimeFormat, RDuration};

// =============================================================================
// Serialization
// =============================================================================

/// Integration with the `serde` crate for JSON serialization.
///
/// Provides adapter traits for serializing/deserializing Rust types to/from JSON.
///
/// Enable with `features = ["serde"]`.
#[cfg(feature = "serde")]
pub mod serde_impl;
#[cfg(feature = "serde")]
pub use serde;
#[cfg(feature = "serde")]
pub use serde_impl::{
    JsonValue, RDeserialize, RJsonValueOps, RSerialize, json_from_sexp, json_from_sexp_permissive,
    json_from_sexp_strict, json_into_sexp,
};

/// Integration with the `toml` crate for TOML value conversions.
///
/// Provides conversions between TOML values and R types.
///
/// Enable with `features = ["toml"]`.
#[cfg(feature = "toml")]
pub mod toml_impl;
#[cfg(feature = "toml")]
pub use toml_impl::{RTomlOps, TomlValue, toml_from_str, toml_to_string, toml_to_string_pretty};

// =============================================================================
// Byte/Binary handling
// =============================================================================

/// Integration with the `bytes` crate for byte buffer operations.
///
/// Provides adapter traits for byte buffer types.
///
/// Enable with `features = ["bytes"]`.
#[cfg(feature = "bytes")]
pub mod bytes_impl;
#[cfg(feature = "bytes")]
pub use bytes_impl::{Buf, BufMut, Bytes, BytesMut, RBuf, RBufMut};

/// Integration with the `sha2` crate for cryptographic hashing.
///
/// Provides SHA-256 and SHA-512 hashing helpers.
///
/// Enable with `features = ["sha2"]`.
#[cfg(feature = "sha2")]
pub mod sha2_impl;
#[cfg(feature = "sha2")]
pub use sha2_impl::{sha256_bytes, sha256_str, sha512_bytes, sha512_str};

// =============================================================================
// Bit manipulation
// =============================================================================

/// Integration with the `bitflags` crate.
///
/// Provides [`RFlags<T>`][bitflags_impl::RFlags] wrapper for bitflags ↔ integer conversions.
///
/// Enable with `features = ["bitflags"]`.
#[cfg(feature = "bitflags")]
pub mod bitflags_impl;
#[cfg(feature = "bitflags")]
pub use bitflags_impl::{Flags, RFlags};

/// Integration with the `bitvec` crate.
///
/// Provides conversions between R logical vectors and `BitVec` types.
///
/// Enable with `features = ["bitvec"]`.
#[cfg(feature = "bitvec")]
pub mod bitvec_impl;
#[cfg(feature = "bitvec")]
pub use bitvec_impl::{BitVec, Lsb0, Msb0, RBitVec};

// =============================================================================
// Formatting
// =============================================================================

/// Integration with the `tabled` crate for table formatting.
///
/// Provides helpers for formatting data as ASCII/Unicode tables.
///
/// Enable with `features = ["tabled"]`.
#[cfg(feature = "tabled")]
pub mod tabled_impl;
#[cfg(feature = "tabled")]
pub use tabled_impl::{
    Builder, Table, Tabled, builder_to_string, table_from_vecs, table_to_string,
    table_to_string_opts, table_to_string_styled,
};
