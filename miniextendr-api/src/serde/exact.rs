//! The snapshot encoding: an exact serde-to-R mode (#2256).
//!
//! An [`ExternalPtr`](struct@crate::ExternalPtr) to a serde type keeps its
//! value in a save through this encoding (`externalptr/snapshot.rs`). It is the
//! crate's serde-to-R mapping made exact: the value comes back from the R
//! list it writes bit for bit, including through serde's buffered `Content`
//! (untagged and internally tagged enums, `#[serde(flatten)]`), which only
//! sees what `deserialize_any` visits. [`to_r`](super::to_r) and
//! [`from_r`](super::from_r) keep their mapping, which reads better in R and
//! is not exact (#1417, #1419).
//!
//! The encoding is a readable R list. Where the general mapping would be
//! ambiguous, a value carries the attribute `serde`, one string naming its
//! kind. Only this encoding writes attributes, so every attribute in it is
//! one of these:
//!
//! | Rust value | R value |
//! |---|---|
//! | `bool` | `logical(1)` |
//! | an integer that fits `i32` (any type, `i32::MIN` included) | `integer(1)` |
//! | an integer outside `i32` (`i64`, `u64`, `i128`, `u128`, `u32`) | `character(1)` in decimal, `serde = "int"` |
//! | `f32`, `f64` | `double(1)`, the bits as they are (`NA_real_`'s payload, NaN, `-0.0`, `Inf`) |
//! | `char`, `str` | `character(1)` |
//! | a string holding a NUL byte | `raw` of its UTF-8 bytes, `serde = "str"` |
//! | bytes (`serialize_bytes`) | `raw` |
//! | `None`, `()`, unit struct | `NULL` |
//! | `Some(x)` | `x`'s value; `list(x)` with `serde = "some"` when `x`'s value is `NULL` or such a list |
//! | newtype struct | its field's value |
//! | sequence of `bool`s, of integers that fit `i32`, of floats, or of strings (at least one element) | that atomic vector, `serde = "seq"` |
//! | sequence of integers, at least one outside `i32` | `character` in decimal, `serde = "ints"` |
//! | any other sequence (empty, a `None`, nested, mixed), tuple, tuple struct | unnamed list |
//! | struct, map whose keys are all strings | named list (an empty one is `named list()`) |
//! | any other map | `list(keys = <keys>, values = <values>)`, each a sequence, `serde = "map"` |
//! | unit variant | `character(1)`, the variant's name |
//! | newtype, tuple or struct variant | `list(<variant> = <payload>)` |
//! | [`RObject`](super::RObject) | `list(<the object>)`, `serde = "robject"` |
//!
//! So an atomic vector without the attribute is always a scalar, and a list
//! without it is a sequence (no names) or a struct or map (names). An R
//! object is wrapped in a list, so its own attributes are never read as the
//! encoding's. Every
//! sequence of scalars of one kind stays one vector: a million `f64`s are one
//! `double` vector (8 MB), not a list of a million scalars (56 MB).
//!
//! # `deserialize_any`
//!
//! Each value visits as its own kind, decided from the R value alone:
//!
//! | R value | visit |
//! |---|---|
//! | `NULL` | `visit_unit` (`None` and `()` read it) |
//! | `logical(1)`, `integer(1)`, `double(1)`, `character(1)` | `visit_bool`, `visit_i32`, `visit_f64`, `visit_str` |
//! | `raw` | `visit_bytes` |
//! | `serde = "int"` | `visit_i64`, else `visit_u64`, `visit_i128`, `visit_u128`: the first that holds the value |
//! | `serde = "str"` | `visit_string` |
//! | `serde = "seq"` or `"ints"`, any length (one too) | `visit_seq`, each element by the rows above |
//! | unnamed list | `visit_seq` |
//! | named list | `visit_map`, the names as string keys |
//! | `serde = "map"` | `visit_map`, the keys by these rules |
//! | `serde = "some"` | `visit_some` |
//! | `serde = "robject"` | `visit_newtype_struct` with a token for the object, which an [`RObject`](super::RObject) in the buffered value resolves while the decoder runs |
//!
//! No visit maps an `NA` to `None`: only `NULL` (and the `"some"` list) is
//! the `None` encoding, so `Some(NA_real_)` stays `Some`. serde's `Content`
//! reads no 128-bit integer, so an `i128` or `u128` reads back in a plain
//! field, a sequence or a map, but not inside an untagged or internally
//! tagged enum or a flattened struct (#2257, serde-rs/serde#1682).

mod de;
mod ser;

use super::error::RSerdeError;
use crate::{SEXP, SEXPTYPE, SexpExt};
use std::ffi::CStr;

/// Encode `value` in the snapshot encoding, which the module docs describe.
///
/// The value is first built on the Rust side, then made into R objects in
/// one pass, so encoding allocates no R object per sequence element and
/// keeps R's protect stack shallow.
pub fn to_r_exact<T: ?Sized + serde::Serialize>(value: &T) -> Result<SEXP, RSerdeError> {
    Ok(value.serialize(ser::NodeSerializer)?.build())
}

/// Decode a value [`to_r_exact`] encoded.
pub fn from_r_exact<T: serde::de::DeserializeOwned>(sexp: SEXP) -> Result<T, RSerdeError> {
    super::robject::with_buffer(|| T::deserialize(de::ExactDeserializer::new(sexp)))
}

// region: The `serde` attribute

/// The kinds the `serde` attribute names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// An atomic vector that is a sequence.
    Seq,
    /// `character(1)`: one integer outside `i32`, in decimal.
    Int,
    /// `character`: a sequence of integers, in decimal.
    Ints,
    /// `raw`: a string holding a NUL byte.
    Str,
    /// `list(keys, values)`: a map whose keys are not all strings.
    Map,
    /// `list(x)`: `Some(x)` where `x`'s value is the `None` encoding.
    Some,
    /// `list(object)`: an [`RObject`](super::RObject)'s R object.
    Object,
}

const KINDS: [Kind; 7] = [
    Kind::Seq,
    Kind::Int,
    Kind::Ints,
    Kind::Str,
    Kind::Map,
    Kind::Some,
    Kind::Object,
];

impl Kind {
    fn name(self) -> &'static CStr {
        match self {
            Kind::Seq => c"seq",
            Kind::Int => c"int",
            Kind::Ints => c"ints",
            Kind::Str => c"str",
            Kind::Map => c"map",
            Kind::Some => c"some",
            Kind::Object => c"robject",
        }
    }

    /// The kind `sexp`'s `serde` attribute names, if it has one.
    fn of(sexp: SEXP) -> Result<Option<Kind>, RSerdeError> {
        let attr = sexp.get_attr(attr_symbol());
        if attr.is_nil() {
            return Ok(None);
        }
        let name = (attr.type_of() == SEXPTYPE::STRSXP && attr.len() == 1)
            .then(|| attr.string_elt(0))
            .filter(|&charsxp| charsxp != SEXP::na_string())
            // SAFETY: a CHARSXP other than `NA_character_`.
            .map(|charsxp| unsafe { crate::from_r::try_charsxp_to_str(charsxp) })
            .transpose()
            .map_err(RSerdeError::not_text)?;
        KINDS
            .into_iter()
            .find(|kind| name.is_some_and(|name| kind.name().to_bytes() == name.as_bytes()))
            .map(Some)
            .ok_or_else(|| {
                RSerdeError::Message(format!(
                    "not a value of the snapshot encoding: its `serde` attribute is {}",
                    name.map_or_else(|| "not a string".to_owned(), |name| format!("{name:?}"))
                ))
            })
    }
}

/// The symbol `serde`.
fn attr_symbol() -> SEXP {
    // SAFETY: R's main thread (every caller works on R objects); a symbol is
    // never collected.
    unsafe { crate::sys::Rf_install(c"serde".as_ptr()) }
}

/// Gives `sexp`, a fresh object, the attribute `serde = <kind>`.
fn mark(sexp: SEXP, kind: Kind) -> SEXP {
    // SAFETY: R's main thread; `sexp` and the attribute value are rooted
    // while the other allocates.
    unsafe {
        let scope = crate::gc_protect::ProtectScope::new();
        scope.protect_raw(sexp);
        let value = scope.protect_raw(crate::sys::Rf_mkString(kind.name().as_ptr()));
        crate::sys::Rf_setAttrib(sexp, attr_symbol(), value);
    }
    sexp
}

// endregion

#[cfg(test)]
mod tests;
