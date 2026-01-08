//! Direct R serialization via serde (no JSON intermediate).
//!
//! This module provides efficient type-preserving conversions between Rust types
//! and native R objects using serde's `Serialize` and `Deserialize` traits.
//!
//! # Features
//!
//! Enable this module with the `serde_r` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["serde_r"] }
//! ```
//!
//! # Type Mappings
//!
//! ## Serialization (Rust → R)
//!
//! | Rust Type | R Type |
//! |-----------|--------|
//! | `bool` | `logical(1)` |
//! | `i8/i16/i32` | `integer(1)` |
//! | `i64/u64/f32/f64` | `numeric(1)` |
//! | `String/&str` | `character(1)` |
//! | `Option<T>::None` | NA or NULL |
//! | `Vec<primitive>` | atomic vector (smart dispatch) |
//! | `Vec<struct>` | list of lists |
//! | `HashMap<String, T>` | named list |
//! | `struct { fields }` | named list |
//! | `()` / unit struct | NULL |
//! | unit enum variant | character scalar |
//! | data enum variant | tagged list `list(tag = value)` |
//!
//! ## Deserialization (R → Rust)
//!
//! | R Type | Rust Type |
//! |--------|-----------|
//! | `logical(1)` | `bool` |
//! | `integer(1)` | `i32` |
//! | `numeric(1)` | `f64` |
//! | `character(1)` | `String` |
//! | NA values | `Option<T>::None` |
//! | atomic vectors | `Vec<primitive>` |
//! | lists | `Vec<T>` or struct |
//! | named lists | struct or `HashMap<String, T>` |
//! | NULL | `()` or `Option<T>::None` |
//!
//! # Smart Vec Dispatch
//!
//! When serializing `Vec<T>`, the serializer uses smart dispatch:
//!
//! - `Vec<i32>` → `integer` vector
//! - `Vec<f64>` → `numeric` vector
//! - `Vec<bool>` → `logical` vector
//! - `Vec<String>` → `character` vector
//! - `Vec<struct>` → list of lists
//!
//! # NA Roundtrip
//!
//! NA values are preserved through `Option<T>`:
//!
//! ```rust,ignore
//! // Serialization
//! let v: Option<i32> = None;
//! // → NA_integer_
//!
//! // Deserialization
//! let v: Option<i32> = from_r(na_integer_sexp)?;
//! assert!(v.is_none());
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use serde::{Serialize, Deserialize};
//! use miniextendr_api::serde_r::{RSerializeNative, RDeserializeNative};
//!
//! #[derive(Serialize, Deserialize, Clone, ExternalPtr)]
//! struct Point {
//!     x: f64,
//!     y: f64,
//! }
//!
//! #[miniextendr]
//! impl RSerializeNative for Point {}
//!
//! #[miniextendr]
//! impl RDeserializeNative for Point {}
//!
//! miniextendr_module! {
//!     mod mymodule;
//!     impl RSerializeNative for Point;
//!     impl RDeserializeNative for Point;
//! }
//! ```
//!
//! In R:
//!
//! ```r
//! # Create a Point
//! p <- Point$new(1.0, 2.0)
//!
//! # Serialize to R list
//! data <- p$to_r()
//! # list(x = 1.0, y = 2.0)
//!
//! # Deserialize from R list
//! p2 <- Point$from_r(list(x = 3.0, y = 4.0))
//! p2$x  # 3.0
//! p2$y  # 4.0
//! ```
//!
//! # Comparison with `serde` (JSON) Feature
//!
//! | Feature | `serde` (JSON) | `serde_r` (Native) |
//! |---------|----------------|-------------------|
//! | Intermediate format | JSON string | None |
//! | Type preservation | No (all numbers → f64) | Yes (i32 stays i32) |
//! | NA handling | Limited | Full support |
//! | Performance | Extra parse/stringify | Direct conversion |
//! | External tools | JSON interop | R-native only |
//!
//! Use `serde_r` when:
//! - You need efficient Rust ↔ R conversion
//! - Type preservation matters (integers vs doubles)
//! - You want NA/NULL handling
//!
//! Use `serde` (JSON) when:
//! - You need JSON string output for files/APIs
//! - You want to share data with non-R systems
//! - You need JSON inspection/manipulation

mod de;
mod error;
mod ser;
mod traits;

pub use de::RDeserializer;
pub use error::RSerdeError;
pub use ser::RSerializer;
pub use traits::{RDeserializeNative, RSerializeNative, from_r, to_r};
