//! Deserializer for converting an R `data.frame` SEXP into `Vec<T>`.
//!
//! Provides two public entry points:
//!
//! - [`dataframe_to_vec`] — owned; `T: DeserializeOwned`; always materialises
//!   `String` for character cells.
//! - [`with_dataframe_rows`] — scoped callback; `T: for<'a> Deserialize<'a>`;
//!   supports zero-copy borrows (`name: &'a str`).
//!
//! # Design (unified row/cell deserializer)
//!
//! A single `RowDeserializer<'sexp>` + `RowMapAccess<'sexp>` + `CellDeserializer<'sexp, 'n>`
//! covers all three entry points. Each entry point hands rows back to the
//! caller in its own way (owned `Vec`, scoped callback, RAII handle), but the
//! per-row deserialisation pipeline is shared via [`deserialize_rows`].
//!
//! Character cells go through `visit_borrowed_str` — visitors that want
//! `String` (i.e. `DeserializeOwned`) fall back to `visit_str` via serde's
//! default `Visitor::visit_borrowed_str` impl, which copies into a `String`.
//! Visitors that want `&'sexp str` (a future zero-copy path) get the borrow
//! directly. The unified shape pre-loads the seam for that future variant.
//!
//! # Factor columns
//!
//! R factor columns are `INTSXP` with a `levels` STRSXP attribute. Serde's
//! type-driven visitor dispatch lets the cell deserialiser pick the right
//! representation per field:
//!
//! - `String` / `Option<String>` / `char` — receives the level **label**
//!   (e.g. `"active"`).
//! - `i32` / `Option<i32>` / any other integer type — receives the 1-based
//!   level **code**.
//! - `serde_json::Value` / other `deserialize_any` consumers — receives the
//!   label.
//!
//! Out-of-range factor codes (corruption via `attr<-`) raise an error;
//! `NA_INTEGER` is translated to `visit_none` for `Option<…>` fields and
//! [`RSerdeError::UnexpectedNa`] for non-optional fields. See [issue #689].
//!
//! # Limitations
//!
//! 1. **Nested struct un-flattening uses single-underscore prefix matching.**
//!    A field named `address` whose value is a struct `Address { city, zip }`
//!    serializes to columns `address_city` / `address_zip`, and the inverse
//!    is now supported. The deserialiser splits each column name at the
//!    *first* `_` after the current prefix to derive nested keys, then lets
//!    the visitor's type drive whether to read a scalar cell or recurse into
//!    a nested struct. This means **flat field names containing `_`**
//!    (e.g., `last_name`) are interpreted as nested-struct paths: a struct
//!    with a `last_name` field whose column is literally named `last_name`
//!    will fail unless the column is renamed on the R side. `#[serde(flatten)]`
//!    is not supported. See [issue #688] for the original ambiguity
//!    discussion and [issue #689] for follow-up tracking.
//!
//! [issue #688]: https://github.com/A2-ai/miniextendr/issues/688
//! [issue #689]: https://github.com/A2-ai/miniextendr/issues/689

use super::error::RSerdeError;
use crate::altrep_traits::{NA_INTEGER, NA_LOGICAL, NA_REAL};
use crate::dataframe::DataFrame;
use crate::from_r::charsxp_to_str;
use crate::{OwnedProtect, SEXP, SEXPTYPE, SexpExt};
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};

// region: public API

/// Convert an R `data.frame` SEXP into a `Vec<T>`.
///
/// Each row of the data.frame is deserialized as one `T`. `T` must be a flat
/// struct (no nested `#[derive(Deserialize)]` structs as fields — see
/// "Limitations" below).
///
/// # Type mapping
///
/// | R column type | Rust field type |
/// |---|---|
/// | `logical` | `bool` / `Option<bool>` |
/// | `integer` | `i32` / `Option<i32>` (also `i8`, `i16`, `i64`, `u*` via overflow check) |
/// | `numeric` | `f64` / `Option<f64>` |
/// | `character` | `String` / `Option<String>` |
/// | `factor` | `String` / `Option<String>` (label) or `i32` / `Option<i32>` (1-based code) |
/// | NA cell | `Option<T>` → `None`; non-optional → error |
///
/// # Errors
///
/// - `sexp` does not inherit from `"data.frame"`.
/// - A column required by `T`'s schema is missing.
/// - A cell's R type does not match the Rust field type.
/// - A non-`Option<…>` field encounters an NA cell.
///
/// # Limitations
///
/// 1. **Nested struct un-flattening uses single-underscore prefix matching**
///    ([#688](https://github.com/A2-ai/miniextendr/issues/688)). Flat fields
///    whose name contains `_` are interpreted as nested-struct paths; rename
///    the R column if you need a flat string-typed field with an underscore.
///    `#[serde(flatten)]` is not supported.
pub fn dataframe_to_vec<T>(sexp: SEXP) -> Result<Vec<T>, RSerdeError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if is_empty_dataframe(sexp) {
        return Ok(Vec::new());
    }
    // Root the input across deserialisation. A real `.Call` caller gets this from
    // R's argument frame, but a Rust caller may hand in a freshly-built,
    // unprotected data.frame; `deserialize_rows` allocates (→ GC under
    // `gctorture(TRUE)`), which would otherwise reclaim the input and its `names`
    // STRSXP and recycle the slot. See reviews/2026-05-29-serde-deserialize-fixture-gctorture-input-protect.md.
    let _input = unsafe { OwnedProtect::new(sexp) };
    let view = DataFrame::from_sexp(sexp).map_err(|e| RSerdeError::Message(e.to_string()))?;
    deserialize_rows(&view)
}

/// Pass a slice of deserialized rows to a scoped callback.
///
/// The `T: for<'a> Deserialize<'a>` bound is equivalent to `DeserializeOwned`,
/// so character fields materialise as `String` (same as [`dataframe_to_vec`]).
/// The advantage of this surface over `dataframe_to_vec` is ergonomic: the
/// callback receives `&[T]`, the borrow is scoped to `f`, and no intermediate
/// row-list SEXPs are allocated on the R heap.
///
/// For a zero-copy variant where `T` can hold `&'a str` borrowing from the SEXP's
/// CHARSXP cache, use the `BorrowedRows<'a, T>` RAII type from #671b (ships
/// separately on top of [`Protected`](crate::Protected)).
///
/// The input SEXP is rooted with an `OwnedProtect` for the duration of the call,
/// so a Rust caller passing a freshly-built data.frame is safe under GC (a
/// `.Call` caller's argument frame would also protect it).
///
/// # Errors
///
/// Same as [`dataframe_to_vec`].
///
/// # Limitations
///
/// Same as [`dataframe_to_vec`] — single-underscore nested-struct path
/// matching.
pub fn with_dataframe_rows<T, F, R>(sexp: SEXP, f: F) -> Result<R, RSerdeError>
where
    T: for<'a> serde::Deserialize<'a>,
    F: FnOnce(&[T]) -> R,
{
    if is_empty_dataframe(sexp) {
        let rows: Vec<T> = Vec::new();
        return Ok(f(&rows));
    }
    // Root the input across deserialisation — see [`dataframe_to_vec`].
    let _input = unsafe { OwnedProtect::new(sexp) };
    let view = DataFrame::from_sexp(sexp).map_err(|e| RSerdeError::Message(e.to_string()))?;
    let rows: Vec<T> = deserialize_rows(&view)?;
    Ok(f(&rows))
}

// endregion

// region: BorrowedRows<'a, T> RAII (#671 Follow-up B on top of #681)

/// RAII handle around a `Vec<T>` deserialised from a data.frame, with the
/// source SEXP kept GC-rooted for the bundle's lifetime.
///
/// Type alias for [`Protected<'a, Vec<T>>`](crate::Protected) — see #681 for
/// the underlying primitive. Use [`dataframe_to_vec_borrowed`] to construct.
///
/// Access rows via `Deref<Target = Vec<T>>`:
///
/// ```ignore
/// let rows: BorrowedRows<'_, Row> = dataframe_to_vec_borrowed(sexp)?;
/// for r in &*rows {
///     // …
/// }
/// ```
pub type BorrowedRows<'a, T> = crate::Protected<'a, Vec<T>>;

/// Deserialise a data.frame into a `BorrowedRows<'a, T>` that holds the
/// source SEXP rooted for `'a`.
///
/// Sister function to [`dataframe_to_vec`]: same per-row deserialisation,
/// same flat-struct limitations ([#688]), but the returned handle
/// keeps the input SEXP protected on the R protect stack while the caller
/// holds it. Use this when [`with_dataframe_rows`]'s closure shape is too
/// restrictive — e.g., a parser that threads rows through multiple helper
/// functions before producing a summary.
///
/// # Cost vs alternatives
///
/// One extra `Rf_protect` entry vs. [`with_dataframe_rows`], plus the
/// [`Protected`](crate::Protected) wrapper itself. Prefer the closure form
/// when it fits.
///
/// # Note on the lifetime parameter
///
/// `T: for<'b> serde::Deserialize<'b>` is equivalent to `DeserializeOwned`
/// today, so character fields materialise as `String` (no zero-copy yet).
/// The `'a` lifetime ties the protect entry to the returned handle; future
/// work threading `'a` through [`DataFrame`](crate::dataframe::DataFrame)
/// would enable true zero-copy `T = Borrowed<'a> { name: &'a str }` — see
/// the issue thread on #671 for the design discussion.
///
/// [#688]: https://github.com/A2-ai/miniextendr/issues/688
pub fn dataframe_to_vec_borrowed<'a, T>(sexp: SEXP) -> Result<BorrowedRows<'a, T>, RSerdeError>
where
    T: for<'b> serde::Deserialize<'b>,
{
    let rows: Vec<T> = if is_empty_dataframe(sexp) {
        Vec::new()
    } else {
        // Root the input across deserialisation (see [`dataframe_to_vec`]); this
        // guard drops at the end of the block, before `Protected::new` re-protects
        // `sexp` for the returned handle (keeps the `UNPROTECT(1)` order correct).
        let _input = unsafe { OwnedProtect::new(sexp) };
        let view = DataFrame::from_sexp(sexp).map_err(|e| RSerdeError::Message(e.to_string()))?;
        deserialize_rows(&view)?
    };
    // SAFETY:
    // - Caller is on the R main thread (entry via #[miniextendr] wrapper).
    // - `sexp` is a valid SEXP — DataFrame::from_sexp validated it for
    //   the non-empty path; the empty short-circuit checked is_data_frame +
    //   xlength == 0.
    // - `rows` carries no SEXP-internal borrows under the `for<'b>` bound
    //   (DeserializeOwned-equivalent), so `'a` is satisfied trivially.
    Ok(unsafe { crate::Protected::new(sexp, rows) })
}

// endregion

// region: Public IntoDataFrame / FromDataFrame on serde rows

/// Wrapper that converts `Vec<T: Serialize>` into a [`DataFrame`](crate::dataframe::DataFrame)
/// through the two-phase columnar serializer (schema discovery + column fill), the richer
/// serde build path than the per-row `IntoList` transposition.
///
/// ```ignore
/// use miniextendr_api::dataframe::IntoDataFrame;
/// use miniextendr_api::serde::SerdeRows;
///
/// let df = SerdeRows(people).into_dataframe()?;
/// ```
pub struct SerdeRows<T>(pub Vec<T>);

impl<T: serde::Serialize> crate::dataframe::IntoDataFrame for SerdeRows<T> {
    fn into_dataframe(
        self,
    ) -> Result<crate::dataframe::DataFrame, crate::dataframe::DataFrameError> {
        crate::serde::vec_to_dataframe(&self.0).map_err(Into::into)
    }
}

/// Read a `data.frame` into `SerdeRows<T>` via serde (the [`dataframe_to_vec`] path),
/// surfacing a [`DataFrameError`](crate::dataframe::DataFrameError).
///
/// Reading targets the [`SerdeRows`] wrapper (rather than a blanket `Vec<T>`) so the serde
/// read path never collides with the concrete `FromDataFrame for Vec<Row>` impls that
/// `#[derive(DataFrameRow)]` emits when a row also derives `Deserialize`. Unwrap via
/// `SerdeRows.0` or [`SerdeRows::into_inner`].
impl<T> crate::dataframe::FromDataFrame for SerdeRows<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    fn from_dataframe(
        df: &crate::dataframe::DataFrame,
    ) -> Result<Self, crate::dataframe::DataFrameError> {
        Ok(SerdeRows(dataframe_to_vec(df.as_sexp())?))
    }
}

impl<T> SerdeRows<T> {
    /// Unwrap the inner `Vec<T>`.
    #[inline]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }
}

// endregion

// region: empty-df short-circuit

/// Detect a 0-row 0-column data.frame for which `DataFrame::from_sexp`
/// would error on the missing names attribute.
///
/// `vec_to_dataframe(&[])` returns such a value (a VECSXP of length 0 with
/// `class = "data.frame"` and `row.names = c(NA, 0)` but no names attribute).
/// Round-tripping that through `dataframe_to_vec` should yield `Ok(vec![])`,
/// not an error.
fn is_empty_dataframe(sexp: SEXP) -> bool {
    sexp.type_of() == SEXPTYPE::VECSXP && sexp.xlength() == 0 && sexp.is_data_frame()
}

// endregion

// region: row + cell deserializer

/// Row deserializer.
///
/// The lifetime `'sexp` ties cell deserializers to `DataFrame`'s borrow of
/// the source SEXP. The `impl<'de> Deserializer<'de> for RowDeserializer<'de>`
/// constrains `'de = 'sexp` so visitors that want `&'de str` see a borrow
/// rooted in the SEXP's CHARSXP cache; `String`-wanting visitors fall back
/// through serde's default `visit_borrowed_str → visit_str`.
struct RowDeserializer<'sexp> {
    view: &'sexp DataFrame,
    row: usize,
    /// Ordered list of column names from the SEXP's `names` attribute.
    /// Shared across the top-level row and every nested `RowMapAccess` so
    /// derived prefix lookups stay deterministic.
    column_names: std::rc::Rc<[String]>,
}

impl<'sexp> RowDeserializer<'sexp> {
    fn new(view: &'sexp DataFrame, row: usize) -> Self {
        let column_names = ordered_column_names(view);
        Self {
            view,
            row,
            column_names,
        }
    }
}

impl<'de> Deserializer<'de> for RowDeserializer<'de> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(RSerdeError::Message(
            "dataframe row deserialiser only supports struct/map deserialisation".into(),
        ))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(RowMapAccess::new(
            self.view,
            self.row,
            self.column_names,
            "",
        ))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(RowMapAccess::new(
            self.view,
            self.row,
            self.column_names,
            "",
        ))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

/// Collect the data.frame's column names in *file* (i.e., R-side) order.
///
/// `DataFrame::names()` is HashMap-backed and unordered, but nested
/// prefix matching needs deterministic enumeration that matches what
/// `vec_to_dataframe`'s columnar serializer emits.
fn ordered_column_names(view: &DataFrame) -> std::rc::Rc<[String]> {
    let list = view.as_list();
    let Some(names_sexp) = list.names() else {
        return std::rc::Rc::from(Vec::<String>::new().into_boxed_slice());
    };
    let n: isize = names_sexp.xlength();
    let mut out: Vec<String> = Vec::with_capacity(n.max(0) as usize);
    for i in 0..n {
        let charsxp = names_sexp.string_elt(i);
        if charsxp == SEXP::na_string() {
            out.push(String::new());
        } else {
            // SAFETY: charsxp is a CHARSXP element of `names`, rooted by the
            // enclosing call. `to_owned` copies before the borrow ends.
            let s: &str = unsafe { charsxp_to_str(charsxp) };
            out.push(s.to_owned());
        }
    }
    std::rc::Rc::from(out.into_boxed_slice())
}

/// `MapAccess` that yields struct fields, optionally under a `prefix`.
///
/// At the top level `prefix == ""` and emitted keys are the heads of every
/// column name. For a nested struct accessed via field `address`, the
/// recursive `RowMapAccess` carries `prefix = "address_"` and emits heads
/// derived from columns starting with that prefix.
///
/// Each emitted key is the **first underscore-delimited segment** of a
/// column name after stripping the prefix. The visitor's choice of
/// `deserialize_*` decides whether to read a scalar cell or recurse via
/// [`MaybeNestedDeserializer`].
struct RowMapAccess<'sexp> {
    view: &'sexp DataFrame,
    row: usize,
    column_names: std::rc::Rc<[String]>,
    prefix: String,
    /// Distinct heads at this nesting level, in first-occurrence order.
    keys: Vec<String>,
    /// Cursor over `keys`.
    idx: usize,
}

impl<'sexp> RowMapAccess<'sexp> {
    fn new(
        view: &'sexp DataFrame,
        row: usize,
        column_names: std::rc::Rc<[String]>,
        prefix: &str,
    ) -> Self {
        let keys = derive_keys_at_prefix(&column_names, prefix);
        Self {
            view,
            row,
            column_names,
            prefix: prefix.to_owned(),
            keys,
            idx: 0,
        }
    }
}

/// Derive the distinct keys visible at a given prefix.
///
/// Algorithm: for every column name starting with `prefix`, take the part
/// up to the first `_` after the prefix (or the whole remainder if no `_`).
/// Deduplicate while preserving the first-occurrence order in
/// `column_names`.
fn derive_keys_at_prefix(column_names: &[String], prefix: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for name in column_names {
        let Some(rest) = name.strip_prefix(prefix) else {
            continue;
        };
        let head = match rest.find('_') {
            Some(i) => &rest[..i],
            None => rest,
        };
        if seen.insert(head) {
            out.push(head.to_owned());
        }
    }
    out
}

impl<'de> MapAccess<'de> for RowMapAccess<'de> {
    type Error = RSerdeError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if self.idx >= self.keys.len() {
            return Ok(None);
        }
        let name = self.keys[self.idx].as_str();
        seed.deserialize(de::value::StrDeserializer::new(name))
            .map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let head = self.keys[self.idx].clone();
        self.idx += 1;

        // The bare column at `prefix + head` (if any).
        let bare = format!("{}{}", self.prefix, head);
        let bare_col = self.view.column_raw(&bare);
        // The prefix to use for nested recursion (e.g., "address_" if head is "address").
        let nested_prefix = format!("{}{}_", self.prefix, head);

        let de = MaybeNestedDeserializer {
            view: self.view,
            row: self.row,
            column_names: self.column_names.clone(),
            bare_name: bare,
            bare_col,
            nested_prefix,
        };
        seed.deserialize(de)
    }
}

/// Deserialiser handed to the visitor for each map value.
///
/// Carries both possible interpretations of a key: a bare column SEXP (for
/// scalar fields) and a nested prefix (for nested struct fields). The
/// visitor's `deserialize_*` call picks which one.
struct MaybeNestedDeserializer<'sexp> {
    view: &'sexp DataFrame,
    row: usize,
    column_names: std::rc::Rc<[String]>,
    /// Full column name (`prefix + head`) for the scalar interpretation.
    bare_name: String,
    /// SEXP for the bare column, or `None` if no such column exists.
    bare_col: Option<SEXP>,
    /// Prefix for the nested-struct interpretation (e.g., `"address_"`).
    nested_prefix: String,
}

impl<'sexp> MaybeNestedDeserializer<'sexp> {
    /// Build a `CellDeserializer` for the bare-column path, erroring if the
    /// bare column does not exist.
    fn cell_de<'a>(&'a self) -> Result<CellDeserializer<'sexp, 'a>, RSerdeError> {
        let col = self
            .bare_col
            .ok_or_else(|| RSerdeError::MissingField(self.bare_name.clone()))?;
        Ok(CellDeserializer::new(col, self.row, &self.bare_name))
    }
}

impl<'de> Deserializer<'de> for MaybeNestedDeserializer<'de> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // If a bare column exists, dispatch like a scalar; otherwise recurse
        // into the nested-prefix map. This is a best-effort fallback for
        // visitors that don't pick a concrete `deserialize_*` path; user
        // code via `#[derive(Deserialize)]` always picks one.
        if self.bare_col.is_some() {
            self.cell_de()?.deserialize_any(visitor)
        } else {
            visitor.visit_map(RowMapAccess::new(
                self.view,
                self.row,
                self.column_names.clone(),
                &self.nested_prefix,
            ))
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(RowMapAccess::new(
            self.view,
            self.row,
            self.column_names.clone(),
            &self.nested_prefix,
        ))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(RowMapAccess::new(
            self.view,
            self.row,
            self.column_names.clone(),
            &self.nested_prefix,
        ))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // For `Option<NestedStruct>`: if no bare column exists, treat as
        // `Some` and recurse into the nested map. (We have no row-wise NA
        // signal for nested struct presence — `vec_to_dataframe` doesn't
        // emit one either; round-tripping `Option<Struct>` always produces
        // `Some(...)`. Documented limitation.) Otherwise delegate to the
        // cell deserialiser's NA-aware path.
        if self.bare_col.is_some() {
            self.cell_de()?.deserialize_option(visitor)
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // serde calls this when a field is unknown to the visitor; just
        // consume and discard. Doesn't matter whether bare or nested.
        let _ = self;
        visitor.visit_unit()
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_bool(visitor)
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_i8(visitor)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_i16(visitor)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_i32(visitor)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_i64(visitor)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_u8(visitor)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_u16(visitor)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_u32(visitor)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_u64(visitor)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_f32(visitor)
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_f64(visitor)
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_char(visitor)
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_str(visitor)
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_string(visitor)
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.cell_de()?.deserialize_identifier(visitor)
    }

    serde::forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct enum
    }
}

/// Deserializer for a single cell.
///
/// `'sexp` is the lifetime of CHARSXP data pointers (rooted by R's protect
/// stack for the duration of the enclosing call). `'n` is the borrow of the
/// column name from the parent [`RowMapAccess`]. Character cells go through
/// `visit_borrowed_str` so visitors that want `&'sexp str` see the borrow;
/// `DeserializeOwned` visitors fall back via serde's default
/// `visit_borrowed_str → visit_str`, which copies into a `String`.
struct CellDeserializer<'sexp, 'n> {
    col: SEXP,
    row: usize,
    col_name: &'n str,
    _marker: std::marker::PhantomData<&'sexp ()>,
}

impl<'sexp, 'n> CellDeserializer<'sexp, 'n> {
    fn new(col: SEXP, row: usize, col_name: &'n str) -> Self {
        Self {
            col,
            row,
            col_name,
            _marker: std::marker::PhantomData,
        }
    }

    fn sexp_type(&self) -> SEXPTYPE {
        self.col.type_of()
    }

    fn col_type_name(&self) -> String {
        self.sexp_type().type_name().to_string()
    }

    fn type_mismatch(&self, expected: &'static str) -> RSerdeError {
        RSerdeError::Message(format!(
            "column {:?}: type mismatch: expected {}, got {}",
            self.col_name,
            expected,
            self.col_type_name()
        ))
    }

    fn row_isize(&self) -> isize {
        self.row as isize
    }

    /// Is the column an R factor (INTSXP + `class = "factor"`)?
    fn is_factor_column(&self) -> bool {
        self.col.is_factor()
    }
}

/// Look up the factor label for cell `(col, row)`.
///
/// Caller must have already established `col.is_factor() == true`.
/// Returns:
///
/// - `Ok(Some(&'sexp str))` — valid 1-based code; the borrow's lifetime
///   matches the source SEXP (rooted by R's argument frame for the
///   enclosing call).
/// - `Ok(None)` — `NA_INTEGER` code.
/// - `Err(_)` — code out of range vs. `length(levels)`.
unsafe fn factor_label<'sexp>(
    col: SEXP,
    row: isize,
    col_name: &str,
) -> Result<Option<&'sexp str>, RSerdeError> {
    let code = col.integer_elt(row);
    if code == NA_INTEGER {
        return Ok(None);
    }
    let levels = col.get_levels();
    let n_levels = levels.xlength();
    // R factor codes are 1-based; 0 and negatives are corruption.
    // `code` is i32 (R's stored level index); `n_levels` is R_xlen_t (isize).
    if code < 1 || (code as isize) > n_levels {
        return Err(RSerdeError::Message(format!(
            "column {:?}: factor code {} out of range (n_levels = {})",
            col_name, code, n_levels,
        )));
    }
    let charsxp = levels.string_elt((code - 1) as isize);
    // SAFETY: `levels` is a STRSXP attribute of `col`, which is part of the
    // data.frame SEXP rooted by R's argument frame for the enclosing call.
    // The CHARSXP cache pointer is valid for the caller-chosen `'sexp`
    // lifetime; turbofish from a `Deserializer<'de>` impl uses `'de = 'sexp`.
    let s: &'sexp str = unsafe { charsxp_to_str(charsxp) };
    Ok(Some(s))
}

impl<'de> Deserializer<'de> for CellDeserializer<'de, '_> {
    type Error = RSerdeError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let i = self.row_isize();
        match self.sexp_type() {
            SEXPTYPE::LGLSXP => {
                let v = self.col.logical_elt(i);
                if v == NA_LOGICAL {
                    visitor.visit_none()
                } else {
                    visitor.visit_bool(v != 0)
                }
            }
            SEXPTYPE::INTSXP => {
                // Factors (INTSXP + class = "factor") surface as their level
                // labels in `deserialize_any` — that's the most natural
                // representation for self-describing consumers like
                // `serde_json::Value`. Code-typed fields go through
                // `deserialize_i32` instead and stay on the integer path.
                if self.is_factor_column() {
                    // SAFETY: `self.col` is a column of the data.frame SEXP
                    // rooted by R's argument frame; CHARSXP borrows are valid
                    // for `'de` (= `'sexp`).
                    match unsafe { factor_label(self.col, i, self.col_name) }? {
                        Some(s) => visitor.visit_borrowed_str(s),
                        None => visitor.visit_none(),
                    }
                } else {
                    let v = self.col.integer_elt(i);
                    if v == NA_INTEGER {
                        visitor.visit_none()
                    } else {
                        visitor.visit_i32(v)
                    }
                }
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(i);
                if v.to_bits() == NA_REAL.to_bits() {
                    visitor.visit_none()
                } else {
                    visitor.visit_f64(v)
                }
            }
            SEXPTYPE::STRSXP => {
                let charsxp = self.col.string_elt(i);
                if charsxp == SEXP::na_string() {
                    visitor.visit_none()
                } else {
                    // SAFETY: The SEXP is rooted by R's argument frame for the
                    // duration of the enclosing call, so the CHARSXP data
                    // pointer is valid for `'de` (= `'sexp`).
                    let s: &'de str = unsafe { charsxp_to_str(charsxp) };
                    visitor.visit_borrowed_str(s)
                }
            }
            _ => Err(RSerdeError::UnsupportedType {
                sexptype: self.sexp_type() as i32,
            }),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        if self.sexp_type() != SEXPTYPE::LGLSXP {
            return Err(self.type_mismatch("logical"));
        }
        let v = self.col.logical_elt(self.row_isize());
        if v == NA_LOGICAL {
            return Err(RSerdeError::UnexpectedNa);
        }
        visitor.visit_bool(v != 0)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < i8::MIN as i32 || v > i8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i8",
            });
        }
        visitor.visit_i8(v as i8)
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < i16::MIN as i32 || v > i16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "i16",
            });
        }
        visitor.visit_i16(v as i16)
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        visitor.visit_i32(v)
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                visitor.visit_i64(v as i64)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < i64::MIN as f64 || v > i64::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "i64",
                    });
                }
                visitor.visit_i64(v as i64)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < 0 || v > u8::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u8",
            });
        }
        visitor.visit_u8(v as u8)
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_integer_cell(&self)?;
        if v < 0 || v > u16::MAX as i32 {
            return Err(RSerdeError::Overflow {
                from: "i32",
                to: "u16",
            });
        }
        visitor.visit_u16(v as u16)
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if v < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u32",
                    });
                }
                visitor.visit_u32(v as u32)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < 0.0 || v > u32::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u32",
                    });
                }
                visitor.visit_u32(v as u32)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self.sexp_type() {
            SEXPTYPE::INTSXP => {
                let v = self.col.integer_elt(self.row_isize());
                if v == NA_INTEGER {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if v < 0 {
                    return Err(RSerdeError::Overflow {
                        from: "i32",
                        to: "u64",
                    });
                }
                visitor.visit_u64(v as u64)
            }
            SEXPTYPE::REALSXP => {
                let v = self.col.real_elt(self.row_isize());
                if v.to_bits() == NA_REAL.to_bits() {
                    return Err(RSerdeError::UnexpectedNa);
                }
                if !v.is_finite() || v != v.trunc() || v < 0.0 || v > u64::MAX as f64 {
                    return Err(RSerdeError::Overflow {
                        from: "f64",
                        to: "u64",
                    });
                }
                visitor.visit_u64(v as u64)
            }
            _ => Err(self.type_mismatch("integer or numeric")),
        }
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_real_cell(&self)?;
        visitor.visit_f32(v as f32)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let v = deserialize_real_cell(&self)?;
        visitor.visit_f64(v)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let s: &'de str = if self.is_factor_column() {
            // SAFETY: `self.col` is a column of the data.frame SEXP rooted by
            // R's argument frame; CHARSXP borrows are valid for `'de`.
            match unsafe { factor_label(self.col, self.row_isize(), self.col_name) }? {
                Some(s) => s,
                None => return Err(RSerdeError::UnexpectedNa),
            }
        } else if self.sexp_type() == SEXPTYPE::STRSXP {
            let charsxp = self.col.string_elt(self.row_isize());
            if charsxp == SEXP::na_string() {
                return Err(RSerdeError::UnexpectedNa);
            }
            // SAFETY: SEXP rooted by the enclosing call; `'de` = `'sexp`.
            unsafe { charsxp_to_str(charsxp) }
        } else {
            return Err(self.type_mismatch("character"));
        };
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(RSerdeError::TypeMismatch {
                expected: "single character",
                actual: format!("string of length {}", s.len()),
            }),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        // Factor (INTSXP + class = "factor") → emit the label string.
        // STRSXP → emit the CHARSXP borrow. Everything else → type mismatch.
        // `DeserializeOwned` visitors handle the borrowed path via serde's
        // default `visit_borrowed_str → visit_str` fallback, which copies into
        // a `String`.
        if self.is_factor_column() {
            // SAFETY: `self.col` is a column of the data.frame SEXP rooted by
            // R's argument frame; CHARSXP borrows are valid for `'de`.
            return match unsafe { factor_label(self.col, self.row_isize(), self.col_name) }? {
                Some(s) => visitor.visit_borrowed_str(s),
                None => Err(RSerdeError::UnexpectedNa),
            };
        }
        if self.sexp_type() != SEXPTYPE::STRSXP {
            return Err(self.type_mismatch("character"));
        }
        let charsxp = self.col.string_elt(self.row_isize());
        if charsxp == SEXP::na_string() {
            return Err(RSerdeError::UnexpectedNa);
        }
        // SAFETY: The SEXP is rooted by R's argument frame for the duration
        // of the enclosing call; `'de` equals `'sexp` here.
        let s: &'de str = unsafe { charsxp_to_str(charsxp) };
        visitor.visit_borrowed_str(s)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let i = self.row_isize();
        let is_na = match self.sexp_type() {
            SEXPTYPE::LGLSXP => self.col.logical_elt(i) == NA_LOGICAL,
            SEXPTYPE::INTSXP => self.col.integer_elt(i) == NA_INTEGER,
            SEXPTYPE::REALSXP => self.col.real_elt(i).to_bits() == NA_REAL.to_bits(),
            SEXPTYPE::STRSXP => self.col.string_elt(i) == SEXP::na_string(),
            _ => false,
        };
        if is_na {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    serde::forward_to_deserialize_any! {
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum
    }
}

// endregion

// region: per-row dispatch

/// Per-row deserialisation loop shared by all three public entry points.
///
/// Borrows `view` for the duration of deserialisation; the HRTB constraint
/// (`T: for<'de> Deserialize<'de>`) lets the call site pick `'de = 'sexp`
/// internally — visitors that want `&'sexp str` see a borrow rooted in the
/// SEXP, `DeserializeOwned` visitors copy via serde's default fallback.
fn deserialize_rows<'sexp, T>(view: &'sexp DataFrame) -> Result<Vec<T>, RSerdeError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let nrow = view.nrow();
    let mut rows: Vec<T> = Vec::with_capacity(nrow);
    for i in 0..nrow {
        let de = RowDeserializer::new(view, i);
        rows.push(T::deserialize(de)?);
    }
    Ok(rows)
}

// endregion

// region: shared cell helpers

/// Read an integer cell, failing on NA and type mismatch.
fn deserialize_integer_cell(de: &CellDeserializer<'_, '_>) -> Result<i32, RSerdeError> {
    if de.sexp_type() != SEXPTYPE::INTSXP {
        return Err(de.type_mismatch("integer"));
    }
    let v = de.col.integer_elt(de.row_isize());
    if v == NA_INTEGER {
        return Err(RSerdeError::UnexpectedNa);
    }
    Ok(v)
}

/// Read a real cell, accepting integer columns via widening.
fn deserialize_real_cell(de: &CellDeserializer<'_, '_>) -> Result<f64, RSerdeError> {
    match de.sexp_type() {
        SEXPTYPE::REALSXP => {
            let v = de.col.real_elt(de.row_isize());
            if v.to_bits() == NA_REAL.to_bits() {
                return Err(RSerdeError::UnexpectedNa);
            }
            Ok(v)
        }
        SEXPTYPE::INTSXP => {
            let v = de.col.integer_elt(de.row_isize());
            if v == NA_INTEGER {
                return Err(RSerdeError::UnexpectedNa);
            }
            Ok(v as f64)
        }
        _ => Err(de.type_mismatch("numeric or integer")),
    }
}

// endregion
