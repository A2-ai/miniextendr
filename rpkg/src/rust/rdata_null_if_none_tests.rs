//! `#[r_data]` fields that read `NULL` for `None` (#2270), under S3, R6 and
//! S7: `#[r_data(null_if_none)]` on an `Option<T>` field (`label`, and the
//! read-only `code`), and a `Nullable<T>` field (`tag`). `plain` is an
//! `Option<String>` field without the option, which reads `NA_character_`
//! for `None`. Writing `NULL` or `NA` stores `None` in every one of them.
//! `SavedNull` holds both forms in a serde type, whose value a save keeps
//! (#2186).

use miniextendr_api::externalptr::RSidecar;
use miniextendr_api::{Nullable, miniextendr};

// region: S3

/// An S3 class whose `label`, `code` and `tag` read `NULL` for `None` and
/// whose `plain` reads `NA`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s3)]
pub struct NullS3 {
    #[r_data]
    _r: RSidecar,

    /// A label, `NULL` when unset.
    #[r_data(null_if_none)]
    pub label: Option<String>,

    /// A code R reads and can't write, `NULL` when unset.
    #[r_data(null_if_none, read_only)]
    pub code: Option<i32>,

    /// A tag, `NULL` when unset.
    #[r_data]
    pub tag: Nullable<String>,

    /// A note, `NA` when unset.
    #[r_data]
    pub plain: Option<String>,
}

/// S3 class registration for NullS3: `$` / `[[` read the fields, `$<-` /
/// `[[<-` write them (and refuse `code`).
#[miniextendr(s3(r_data_accessors))]
impl NullS3 {
    /// Create a NullS3; every field takes `NULL` for unset.
    /// @param label The label.
    /// @param code The code.
    /// @param tag The tag.
    /// @param plain The note.
    pub fn new(
        label: Option<String>,
        code: Option<i32>,
        tag: Option<String>,
        plain: Option<String>,
    ) -> Self {
        NullS3 {
            _r: RSidecar,
            label,
            code,
            tag: Nullable(tag),
            plain,
        }
    }
}
// endregion

// region: R6

/// An R6 class whose `label`, `code` and `tag` bindings read `NULL` for
/// `None` and whose `plain` binding reads `NA`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(r6)]
pub struct NullR6 {
    #[r_data]
    _r: RSidecar,

    #[r_data(null_if_none)]
    pub label: Option<String>,

    #[r_data(null_if_none, read_only)]
    pub code: Option<i32>,

    #[r_data]
    pub tag: Nullable<String>,

    #[r_data]
    pub plain: Option<String>,
}

/// R6 class registration for NullR6.
/// @field label A label, `NULL` when unset (active binding).
/// @field code A code, `NULL` when unset (read-only active binding).
/// @field tag A tag, `NULL` when unset (active binding).
/// @field plain A note, `NA` when unset (active binding).
#[miniextendr(r6(r_data_accessors))]
impl NullR6 {
    /// Create a new NullR6; every field takes `NULL` for unset.
    /// @param label The label.
    /// @param code The code.
    /// @param tag The tag.
    /// @param plain The note.
    pub fn new(
        label: Option<String>,
        code: Option<i32>,
        tag: Option<String>,
        plain: Option<String>,
    ) -> Self {
        NullR6 {
            _r: RSidecar,
            label,
            code,
            tag: Nullable(tag),
            plain,
        }
    }
}
// endregion

// region: S7

/// An S7 class whose `label`, `code` and `tag` properties read `NULL` for
/// `None` and whose `plain` property reads `NA`.
#[derive(miniextendr_api::ExternalPtr, Debug)]
#[externalptr(s7)]
pub struct NullS7 {
    #[r_data]
    _r: RSidecar,

    #[r_data(null_if_none, prop_doc = "A label, `NULL` when unset.")]
    pub label: Option<String>,

    #[r_data(
        null_if_none,
        read_only,
        prop_doc = "A code, `NULL` when unset (read-only)."
    )]
    pub code: Option<i32>,

    #[r_data(prop_doc = "A tag, `NULL` when unset.")]
    pub tag: Nullable<String>,

    #[r_data(prop_doc = "A note, `NA` when unset.")]
    pub plain: Option<String>,
}

/// S7 class registration for NullS7 with property-based sidecar accessors.
#[miniextendr(s7(r_data_accessors))]
impl NullS7 {
    /// Create a new NullS7; every field takes `NULL` for unset.
    /// @param label The label.
    /// @param code The code.
    /// @param tag The tag.
    /// @param plain The note.
    pub fn new(
        label: Option<String>,
        code: Option<i32>,
        tag: Option<String>,
        plain: Option<String>,
    ) -> Self {
        NullS7 {
            _r: RSidecar,
            label,
            code,
            tag: Nullable(tag),
            plain,
        }
    }
}
// endregion

// region: serde, a saved value keeps both forms

/// A serde R6 class with a `Nullable<String>` field and a `null_if_none`
/// one: a save keeps `None` and `Some` in both.
#[cfg(feature = "serde")]
#[derive(
    miniextendr_api::ExternalPtr, Debug, crate::serde::Serialize, crate::serde::Deserialize,
)]
#[serde(crate = "crate::serde")]
#[externalptr(r6)]
pub struct SavedNull {
    #[serde(skip)]
    #[r_data]
    _r: RSidecar,

    #[r_data]
    pub tag: Nullable<String>,

    #[r_data(null_if_none)]
    pub label: Option<String>,
}

/// @title A serde R6 class with fields that read `NULL` for `None`
/// @description `SavedNull` comes back after `readRDS()` with its fields.
/// @field tag A tag, `NULL` when unset (active binding).
/// @field label A label, `NULL` when unset (active binding).
#[cfg(feature = "serde")]
#[miniextendr(r6(r_data_accessors))]
impl SavedNull {
    /// @param tag The tag.
    /// @param label The label.
    pub fn new(tag: Option<String>, label: Option<String>) -> Self {
        SavedNull {
            _r: RSidecar,
            tag: Nullable(tag),
            label,
        }
    }
}
// endregion
