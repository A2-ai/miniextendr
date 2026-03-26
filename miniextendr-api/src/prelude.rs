//! Convenience re-exports for common miniextendr items.
//!
//! A single `use miniextendr_api::prelude::*;` brings into scope the most
//! commonly used macros, traits, types, and helpers so user crates can avoid
//! a long list of individual imports.
//!
//! ## Optional feature types
//!
//! When a Cargo feature is enabled (e.g., `uuid`, `regex`, `ndarray`), the
//! prelude includes both the miniextendr adapter types **and** a re-export of
//! the upstream dependency crate itself. This means you do **not** need to add
//! optional crates as direct dependencies in your `Cargo.toml` — enabling the
//! feature on `miniextendr-api` is enough.
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["uuid", "ndarray"] }
//! # No need for: uuid = "1", ndarray = "0.16"
//! ```
//!
//! Access the upstream crate via the prelude:
//!
//! ```ignore
//! use miniextendr_api::prelude::*;
//!
//! // Uuid is re-exported from the uuid crate
//! let id = Uuid::new_v4();
//!
//! // The full crate is also available for advanced usage
//! let parsed = uuid::Uuid::parse_str("...").unwrap();
//! ```
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
//! ```

// region: Proc-macro re-exports
pub use crate::{
    // Derive macros
    Altrep,
    ExternalPtr,
    MatchArg,
    RFactor,
    list,
    miniextendr,
    typed_list,
};
// endregion

// region: Core traits
pub use crate::{Coerce, IntoR, IntoRAltrep, TryCoerce, TryFromSexp};
// endregion

// region: Types
pub use crate::{
    IntoList, Lazy, List, ListBuilder, ListMut, Missing, NamedVector, OwnedProtect, ProtectScope,
    StrVec, StrVecBuilder,
};
// endregion

// region: Worker thread
pub use crate::{Sendable, with_r_thread};
// endregion

// region: Error handling and console output
pub use crate::{r_print, r_println, r_warning};
// endregion

// region: FFI (SEXP is needed in almost every crate)
pub use crate::ffi::SEXP;
// endregion

// region: Optional feature re-exports

// --- either ---
#[cfg(feature = "either")]
pub use crate::{Either, Left, Right};

#[cfg(feature = "either")]
pub use either;

// --- uuid ---
#[cfg(feature = "uuid")]
pub use crate::{RUuidOps, Uuid};
#[cfg(feature = "uuid")]
pub use uuid;

// --- regex ---
#[cfg(feature = "regex")]
pub use crate::{CaptureGroups, RRegexOps, Regex};
#[cfg(feature = "regex")]
pub use regex;

// --- url ---
#[cfg(feature = "url")]
pub use crate::{RUrlOps, Url};
#[cfg(feature = "url")]
pub use url;

// --- time ---
#[cfg(feature = "time")]
pub use crate::{Date, Duration, OffsetDateTime, RDuration};
#[cfg(feature = "time")]
pub use time;

// --- ordered-float ---
#[cfg(feature = "ordered-float")]
pub use crate::{OrderedFloat, ROrderedFloatOps};
#[cfg(feature = "ordered-float")]
pub use ordered_float;

// --- num-bigint ---
#[cfg(feature = "num-bigint")]
pub use crate::{BigInt, BigUint, RBigIntOps, RBigUintOps};
#[cfg(feature = "num-bigint")]
pub use num_bigint;

// --- rust_decimal ---
#[cfg(feature = "rust_decimal")]
pub use crate::{Decimal, RDecimalOps};
#[cfg(feature = "rust_decimal")]
pub use rust_decimal;

// --- num-complex ---
#[cfg(feature = "num-complex")]
pub use crate::{Complex, RComplexOps};
#[cfg(feature = "num-complex")]
pub use num_complex;

// --- num-traits ---
#[cfg(feature = "num-traits")]
pub use crate::{RFloat, RNum, RSigned};
#[cfg(feature = "num-traits")]
pub use num_traits;

// --- ndarray ---
#[cfg(feature = "ndarray")]
pub use crate::{
    Array1, Array2, Array3, Array4, Array5, Array6, ArrayD, ArrayView1, ArrayView2, ArrayViewD,
    ArrayViewMut1, ArrayViewMut2, ArrayViewMutD, RNdArrayOps,
};
#[cfg(feature = "ndarray")]
pub use ndarray;

// --- nalgebra ---
#[cfg(feature = "nalgebra")]
pub use crate::{DMatrix, DVector, RMatrixOps, RVectorOps, SMatrix, SVector};
#[cfg(feature = "nalgebra")]
pub use nalgebra;

// --- indexmap ---
#[cfg(feature = "indexmap")]
pub use crate::{IndexMap, RIndexMapOps};
#[cfg(feature = "indexmap")]
pub use indexmap;

// --- rayon ---
#[cfg(feature = "rayon")]
pub use crate::{RParallelExtend, RParallelIterator};
#[cfg(feature = "rayon")]
pub use rayon;

// --- rand ---
#[cfg(feature = "rand")]
pub use crate::{RDistributionOps, RDistributions, RRng, RRngOps};
#[cfg(feature = "rand")]
pub use rand;
#[cfg(feature = "rand_distr")]
pub use rand_distr;

// --- serde (direct R serialization) ---
#[cfg(feature = "serde")]
pub use crate::serde::{AsSerialize, RDeserializeNative, RSerializeNative};
#[cfg(feature = "serde")]
pub use serde;

// --- serde_json ---
#[cfg(feature = "serde_json")]
pub use crate::{JsonOptions, JsonValue, RDeserialize, RSerialize};
#[cfg(feature = "serde_json")]
pub use serde_json;

// --- toml ---
#[cfg(feature = "toml")]
pub use crate::{TomlValue, toml_from_str, toml_to_string};
#[cfg(feature = "toml")]
pub use toml;

// --- bytes ---
#[cfg(feature = "bytes")]
pub use crate::{Bytes, BytesMut, RBuf, RBufMut};
#[cfg(feature = "bytes")]
pub use bytes;

// --- aho-corasick ---
#[cfg(feature = "aho-corasick")]
pub use crate::{AhoCorasick, aho_compile};
#[cfg(feature = "aho-corasick")]
pub use aho_corasick;

// --- bitflags ---
#[cfg(feature = "bitflags")]
pub use crate::RFlags;
#[cfg(feature = "bitflags")]
pub use bitflags;

// --- bitvec ---
#[cfg(feature = "bitvec")]
pub use crate::RBitVec;
#[cfg(feature = "bitvec")]
pub use bitvec;

// --- borsh ---
#[cfg(feature = "borsh")]
pub use crate::{Borsh, RBorshOps};
#[cfg(feature = "borsh")]
pub use borsh;

// --- raw_conversions ---
#[cfg(feature = "raw_conversions")]
pub use crate::{Pod, Raw, RawSlice, Zeroable};
#[cfg(feature = "raw_conversions")]
pub use bytemuck;

// --- sha2 ---
#[cfg(feature = "sha2")]
pub use crate::{sha256_bytes, sha256_str, sha512_bytes, sha512_str};
#[cfg(feature = "sha2")]
pub use sha2;

// --- tabled ---
#[cfg(feature = "tabled")]
pub use crate::{Table, Tabled, table_to_string};
#[cfg(feature = "tabled")]
pub use tabled;

// --- tinyvec ---
#[cfg(feature = "tinyvec")]
pub use crate::{ArrayVec, TinyVec};
#[cfg(feature = "tinyvec")]
pub use tinyvec;

// --- indicatif ---
#[cfg(feature = "indicatif")]
pub use crate::progress;
#[cfg(feature = "indicatif")]
pub use indicatif;

// --- vctrs ---
#[cfg(feature = "vctrs")]
pub use crate::{IntoVctrs, VctrsClass};
// endregion
