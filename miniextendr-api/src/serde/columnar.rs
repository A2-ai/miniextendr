//! Columnar serializer for converting `Vec<T>` directly to R data.frames.
//!
//! Instead of serializing each struct as a named list (row-oriented), this
//! module transposes the data into column-oriented R vectors — one atomic
//! vector per field. This is more memory-efficient and produces native R
//! data.frames directly.
//!
//! Nested structs are recursively flattened into prefixed columns
//! (e.g., `metadata_size`). `#[serde(flatten)]` and `#[serde(skip_serializing_if)]`
//! are handled correctly.

use std::collections::HashMap;

use super::error::RSerdeError;
use crate::altrep_traits::{NA_LOGICAL, NA_REAL};
use crate::ffi::{Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt};
use serde::ser::{self, Serialize};

/// Generate serde `Serializer` error stubs for methods that should reject non-struct input.
/// Accepts `struct`/`map` to allow, and an error message for the rest.
macro_rules! reject_non_struct {
    ($msg:expr, allow_some_none) => {
        fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
            value.serialize(self)
        }
        fn serialize_none(self) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message(concat!($msg, " (got None)").into()))
        }
        reject_non_struct!(@primitives $msg);
    };
    ($msg:expr) => {
        fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message($msg.into()))
        }
        fn serialize_none(self) -> Result<(), RSerdeError> {
            Err(RSerdeError::Message($msg.into()))
        }
        reject_non_struct!(@primitives $msg);
    };
    (@primitives $msg:expr) => {
        fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_char(self, _: char) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_str(self, _: &str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit(self) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _: &'static str, _: &T) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_newtype_variant<T: ?Sized + Serialize>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<(), RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
        fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, RSerdeError> { Err(RSerdeError::Message($msg.into())) }
    };
}

/// Read a column name from an R STRSXP names vector at index `i`.
///
/// # Safety
/// `names_sexp` must be a valid STRSXP and `i` must be in bounds.
unsafe fn col_name(names_sexp: SEXP, i: isize) -> &'static str {
    unsafe {
        let s = names_sexp.string_elt(i);
        let p = s.r_char();
        std::ffi::CStr::from_ptr(p).to_str().unwrap_or("")
    }
}

/// Copy class and row.names attributes from one data.frame SEXP to another.
///
/// # Safety
/// Both SEXPs must be valid VECSXPs.
unsafe fn copy_df_attrs(from: SEXP, to: SEXP) {
    to.set_class(from.get_class());
    to.set_row_names(from.get_row_names());
}

/// A data.frame produced by the columnar serializer.
///
/// Supports post-assembly customization via builder-style methods:
///
/// ```ignore
/// ColumnarDataFrame::from_rows(&rows)?
///     .rename("hashes_blake3", "hash")
///     .with_column("status", status_sexp)
///     .drop("internal_id")
/// ```
pub struct ColumnarDataFrame {
    sexp: SEXP,
}

impl ColumnarDataFrame {
    /// Convert a slice of serializable structs to an R data.frame in columnar layout.
    ///
    /// Each field of `T` becomes a column (R atomic vector). Nested structs are
    /// recursively flattened into prefixed columns (`parent_child` naming).
    ///
    /// The result supports post-assembly customization:
    ///
    /// ```ignore
    /// ColumnarDataFrame::from_rows(&rows)?
    ///     .rename("hashes_blake3", "hash")
    ///     .with_column("status", status_sexp)
    ///     .drop("internal_id")
    /// ```
    ///
    /// # Supported Field Types
    ///
    /// | Rust Type | R Column Type |
    /// |-----------|---------------|
    /// | `bool` | `logical` |
    /// | `i8/i16/i32` | `integer` |
    /// | `i64/u64/f32/f64` | `numeric` |
    /// | `String/&str` | `character` |
    /// | `Option<T>` | Same type with NA for `None` |
    /// | `Option<T>` (every row `None`) | `logical` NA column — R coerces to the surrounding type on first use (`c(NA, 1L)` → integer, `c(NA, "x")` → character) |
    /// | Nested struct | Recursively flattened with `parent_child` naming |
    /// | Other | Falls back to per-element list column |
    pub fn from_rows<T: Serialize>(rows: &[T]) -> Result<ColumnarDataFrame, RSerdeError> {
        if rows.is_empty() {
            return Ok(ColumnarDataFrame {
                sexp: empty_dataframe(),
            });
        }

        // Phase 1: Discover schema from ALL rows (union of fields across enum variants)
        let mut acc = SchemaAccumulator::new(SchemaMode::Union);
        for row in rows {
            // Skip rows that fail discovery (e.g., top-level None) — Union mode tolerates partial probes.
            let _ = acc.feed(row);
        }
        let schema = acc.finalize()?;
        let ncol = schema.fields.len();
        let nrow = rows.len();

        if ncol == 0 {
            return Err(RSerdeError::Message(
                "ColumnarDataFrame::from_rows: type has no fields".into(),
            ));
        }

        // Phase 2: Allocate column buffers
        let mut columns: Vec<ColumnBuffer> = schema
            .fields
            .iter()
            .map(|f| ColumnBuffer::new(f.col_type, nrow))
            .collect();

        // Roots every SEXP that lands in a `ColumnBuffer::Generic` (via
        // `RSerializer::serialize`) for the duration of this call. Without
        // this, the unprotected SEXPs in the generic-list buffer can be
        // GC'd before `assemble_dataframe` reads them — surfaced under
        // `gctorture(TRUE)` and on glibc-strict R runtimes.
        let scope = unsafe { crate::ProtectScope::new() };

        // Phase 3: Fill columns from all rows
        let mut filled = vec![false; ncol];
        for row in rows {
            let filler = ColumnFiller {
                columns: &mut columns,
                field_map: &schema.field_map,
                filled: &mut filled,
                col_start: 0,
                col_count: ncol,
                is_top_level: true,
                pending_key: None,
                scope: &scope,
                strict: false,
            };
            row.serialize(filler)?;
        }

        // Phase 4: Assemble data.frame. `assemble_with_scope` sequences
        // `assemble_dataframe(...)` followed by `drop(scope)` so the
        // protected SEXPs in `ColumnBuffer::Generic` cells are still rooted
        // while `set_vector_elt` copies them into the parent VECSXP.
        Ok(unsafe { assemble_with_scope(&schema, &columns, nrow, scope) })
    }

    /// Rename a column. No-op if `from` doesn't match any column name.
    pub fn rename(self, from: &str, to: &str) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = names_sexp.xlength();
            for i in 0..ncol {
                if col_name(names_sexp, i) == from {
                    names_sexp.set_string_elt(i, SEXP::charsxp(to));
                    break;
                }
            }
        }
        self
    }

    /// Strip a prefix from all column names that start with it.
    /// E.g., `.strip_prefix("metadata_")` turns `metadata_size` into `size`.
    pub fn strip_prefix(self, prefix: &str) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = names_sexp.xlength();
            for i in 0..ncol {
                let name = col_name(names_sexp, i);
                if let Some(stripped) = name.strip_prefix(prefix) {
                    names_sexp.set_string_elt(i, SEXP::charsxp(stripped));
                }
            }
        }
        self
    }

    /// Remove a column by name. No-op if the column doesn't exist.
    pub fn drop(self, col: &str) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = names_sexp.xlength();
            let drop_idx = (0..ncol).find(|&i| col_name(names_sexp, i) == col);
            let Some(drop_idx) = drop_idx else {
                return self;
            };

            let new_ncol = ncol - 1;
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            let mut j: isize = 0;
            for i in 0..ncol {
                if i == drop_idx {
                    continue;
                }
                new_list.set_vector_elt(j, self.sexp.vector_elt(i));
                new_names.set_string_elt(j, names_sexp.string_elt(i));
                j += 1;
            }

            new_list.set_names(*new_names);
            copy_df_attrs(self.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }

    /// Keep only the named columns, in the order given. Unknown names are skipped.
    pub fn select(self, cols: &[&str]) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = names_sexp.xlength();

            let indices: Vec<isize> = cols
                .iter()
                .filter_map(|&want| (0..ncol).find(|&i| col_name(names_sexp, i) == want))
                .collect();

            let new_ncol: isize = indices.len().try_into().expect("ncol overflow");
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            for (j, &src_idx) in indices.iter().enumerate() {
                let j_r: isize = j.try_into().expect("index overflow");
                new_list.set_vector_elt(j_r, self.sexp.vector_elt(src_idx));
                new_names.set_string_elt(j_r, names_sexp.string_elt(src_idx));
            }

            new_list.set_names(*new_names);
            copy_df_attrs(self.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }

    /// Insert a column at index 0 (leftmost). If a column with the same
    /// name already exists it is removed first so the prepended copy wins.
    /// Caller is responsible for matching row length and for ensuring
    /// `column` is a valid R vector; miniextendr does not validate.
    pub fn prepend_column(self, name: &str, column: SEXP) -> Self {
        // Drop any existing column with this name first to avoid duplicates.
        let cleaned = self.drop(name);
        unsafe {
            let names_sexp = cleaned.sexp.get_names();
            // For a freshly-built data.frame `get_names` always returns a STRSXP,
            // but a defensive guard keeps us symmetric with `with_column`.
            let ncol = if names_sexp == SEXP::nil() {
                0
            } else {
                names_sexp.xlength()
            };

            let new_ncol = ncol + 1;
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            new_list.set_vector_elt(0, column);
            new_names.set_string_elt(0, SEXP::charsxp(name));

            for i in 0..ncol {
                new_list.set_vector_elt(i + 1, cleaned.sexp.vector_elt(i));
                new_names.set_string_elt(i + 1, names_sexp.string_elt(i));
            }

            new_list.set_names(*new_names);
            copy_df_attrs(cleaned.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }

    /// Upsert a column: replace the column named `name` with `column` if it
    /// already exists, otherwise append `column` at the end. Caller is
    /// responsible for matching row length and for ensuring `column` is a
    /// valid R vector; miniextendr does not validate.
    pub fn with_column(self, name: &str, column: SEXP) -> Self {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return self;
            }
            let ncol = names_sexp.xlength();
            for i in 0..ncol {
                if col_name(names_sexp, i) == name {
                    self.sexp.set_vector_elt(i, column);
                    return self;
                }
            }

            // Not found — append at the end. Reallocate the list and names,
            // copy over existing entries, add the new column.
            let new_ncol = ncol + 1;
            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(new_ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(new_ncol));

            for i in 0..ncol {
                new_list.set_vector_elt(i, self.sexp.vector_elt(i));
                new_names.set_string_elt(i, names_sexp.string_elt(i));
            }
            new_list.set_vector_elt(ncol, column);
            new_names.set_string_elt(ncol, SEXP::charsxp(name));

            new_list.set_names(*new_names);
            copy_df_attrs(self.sexp, *new_list);

            ColumnarDataFrame { sexp: *new_list }
        }
    }
}

impl crate::IntoR for ColumnarDataFrame {
    type Error = std::convert::Infallible;

    fn into_sexp(self) -> SEXP {
        self.sexp
    }

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }
}

/// Convert a row-oriented `DataFrame<T>` into a `ColumnarDataFrame` for
/// post-assembly customization (rename, drop, select).
impl<T: crate::list::IntoList> From<crate::convert::DataFrame<T>> for ColumnarDataFrame {
    fn from(df: crate::convert::DataFrame<T>) -> Self {
        use crate::IntoR;
        use crate::convert::IntoDataFrame;
        ColumnarDataFrame {
            sexp: df.into_data_frame().into_sexp(),
        }
    }
}

impl crate::from_r::TryFromSexp for ColumnarDataFrame {
    type Error = crate::from_r::SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Validate it's a data.frame before wrapping
        crate::dataframe::DataFrameView::from_sexp(sexp)
            .map(ColumnarDataFrame::from)
            .map_err(|e| crate::from_r::SexpError::InvalidValue(e.to_string()))
    }
}

/// Convert a `DataFrameView` (received from R) into a `ColumnarDataFrame`
/// for post-hoc customization (rename, drop, select).
impl From<crate::dataframe::DataFrameView> for ColumnarDataFrame {
    fn from(view: crate::dataframe::DataFrameView) -> Self {
        use crate::IntoR;
        ColumnarDataFrame {
            sexp: view.into_sexp(),
        }
    }
}

/// Convenience alias for [`ColumnarDataFrame::from_rows`].
#[inline]
pub fn vec_to_dataframe<T: Serialize>(rows: &[T]) -> Result<ColumnarDataFrame, RSerdeError> {
    ColumnarDataFrame::from_rows(rows)
}

// region: NamedDataFrameListBuilder

/// Assemble a named list whose elements are [`ColumnarDataFrame`]s,
/// without per-result `OwnedProtect` bookkeeping.
///
/// Each [`push`](NamedDataFrameListBuilder::push) protects the input
/// data.frame's SEXP via an internal [`ProtectScope`](crate::ProtectScope);
/// [`build`](NamedDataFrameListBuilder::build) consumes the builder and emits
/// a named list via [`List::from_raw_pairs`](crate::list::List::from_raw_pairs).
/// The scope drops at the end of `build`, releasing the per-input protects —
/// by which point the children are reachable from the assembled list.
///
/// # Example
///
/// ```ignore
/// let result = NamedDataFrameListBuilder::new()
///     .push("results", vec_to_dataframe(&oks)?)
///     .push("error",   vec_to_dataframe(&errs)?)
///     .build();
/// ```
pub struct NamedDataFrameListBuilder {
    scope: crate::ProtectScope,
    pairs: Vec<(String, crate::ffi::SEXP)>,
}

impl NamedDataFrameListBuilder {
    /// Create an empty builder.
    ///
    /// # Safety (caller)
    ///
    /// Must be called from the R main thread. The internal
    /// [`ProtectScope`](crate::ProtectScope) carries `!Send + !Sync`
    /// so the builder cannot be moved to another thread.
    pub fn new() -> Self {
        Self {
            // SAFETY: ProtectScope requires the R main thread. The builder is
            // constructible only on the R main thread; ProtectScope carries
            // NoSendSync so it cannot be moved off-thread.
            scope: unsafe { crate::ProtectScope::new() },
            pairs: Vec::new(),
        }
    }

    /// Create a builder pre-allocated for `n` entries.
    ///
    /// Equivalent to [`new`](Self::new) but avoids repeated re-allocations
    /// when the number of partitions is known up front.
    pub fn with_capacity(n: usize) -> Self {
        Self {
            scope: unsafe { crate::ProtectScope::new() },
            pairs: Vec::with_capacity(n),
        }
    }

    /// Append a named data.frame. The input's SEXP is protected
    /// internally for the lifetime of the builder.
    #[must_use]
    pub fn push<S: Into<String>>(mut self, name: S, df: ColumnarDataFrame) -> Self {
        use crate::IntoR as _;
        let sexp = df.into_sexp();
        // SAFETY: R main thread (constructor invariant); sexp is a valid
        // VECSXP just produced by ColumnarDataFrame::into_sexp.
        unsafe {
            self.scope.protect_raw(sexp);
        }
        self.pairs.push((name.into(), sexp));
        self
    }

    /// Number of entries pushed so far.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Whether no entries have been pushed yet.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Consume the builder and return the assembled named [`List`](crate::list::List).
    ///
    /// The returned `List`'s SEXP is *not* separately protected on return — the
    /// caller takes responsibility for protection (typically by immediately
    /// handing it back to R via the `.Call` return path). This matches the
    /// contract of [`List::from_raw_pairs`](crate::list::List::from_raw_pairs).
    pub fn build(self) -> crate::list::List {
        // pairs[i].1 is protected by self.scope; from_raw_pairs protects the
        // assembled VECSXP and STRSXP during construction. When self drops at
        // this function's exit, the input SEXPs are unprotected — but they are
        // now children of the returned list, so they remain reachable.
        crate::list::List::from_raw_pairs(self.pairs)
    }
}

impl Default for NamedDataFrameListBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// endregion

// region: Streaming serialize (iter_to_dataframe + DataFrameBuilder)

/// Stream rows from an iterator into a columnar data.frame.
///
/// Schema is taken from the **first row**; subsequent rows must match that
/// schema. If a later row introduces a field not seen in the first row,
/// returns [`RSerdeError::Message`]. Fields present in the first row but
/// missing from a later row are NA-padded.
///
/// `nrow_hint` lets callers pre-size column buffers; `None` is fine — buffers
/// grow exponentially via `Vec::push`.
///
/// # When to use this vs [`vec_to_dataframe`]
///
/// Use [`vec_to_dataframe`] when you already have a `&[T]` in memory. Use
/// `iter_to_dataframe` when the rows arrive incrementally (a file iterator,
/// a DB cursor, a generator) — materialising into a `Vec` first would defeat
/// the purpose.
///
/// # Errors
///
/// - A row fails to serialize.
/// - A row introduces a field not present in the first row's schema.
/// - Column assembly fails.
///
/// # Example
///
/// ```rust,ignore
/// #[derive(serde::Serialize)]
/// struct Row { id: i32, name: String }
///
/// let rows = (0..10).map(|i| Row { id: i, name: format!("item_{i}") });
/// let df = iter_to_dataframe(rows, Some(10))?;
/// ```
pub fn iter_to_dataframe<T, I>(
    iter: I,
    nrow_hint: Option<usize>,
) -> Result<ColumnarDataFrame, RSerdeError>
where
    T: Serialize,
    I: IntoIterator<Item = T>,
{
    let mut builder = DataFrameBuilder::<T>::new(nrow_hint);
    for row in iter {
        builder.push(row)?;
    }
    builder.finish()
}

/// User-facing column type descriptor for [`DataFrameBuilder::with_schema`].
///
/// Maps onto the internal `ColumnType` and unlocks an NA-tolerance hint via
/// `Optional(_)`. The wrapper does **not** change the underlying column type —
/// `Optional(Integer)` produces an integer column where `None` lands as
/// `NA_INTEGER`. Without the hint, an all-`None` column discovered from the
/// first row would otherwise degrade to a logical-NA column (see
/// `ColumnarDataFrame::from_rows` doc).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    /// R `logical` column (`bool`).
    Logical,
    /// R `integer` column (`i8`/`i16`/`i32`).
    Integer,
    /// R `numeric` column (`f32`/`f64`/`i64`/`u64`).
    Real,
    /// R `character` column (`String`/`&str`).
    Character,
    /// R generic list column (per-element SEXP fallback).
    Generic,
    /// NA-tolerance hint wrapping a base type. `Optional(Integer)` is an
    /// integer column where `None` is `NA_INTEGER`.
    Optional(Box<TypeSpec>),
}

impl TypeSpec {
    /// Collapse the user-facing spec to the internal `ColumnType`. The
    /// `Optional` hint is discarded — column types are already NA-tolerant
    /// at the R level (`NA_INTEGER` / `NA_REAL` / `NA_character_` are part
    /// of the corresponding atomic vector types).
    fn into_column_type(self) -> ColumnType {
        match self {
            TypeSpec::Logical => ColumnType::Logical,
            TypeSpec::Integer => ColumnType::Integer,
            TypeSpec::Real => ColumnType::Real,
            TypeSpec::Character => ColumnType::Character,
            TypeSpec::Generic => ColumnType::Generic,
            TypeSpec::Optional(inner) => inner.into_column_type(),
        }
    }
}

/// Builder for incremental data.frame assembly.
///
/// Three schema modes:
///
/// 1. **Default** ([`DataFrameBuilder::new`]) — schema discovered from the
///    first [`push`](Self::push); subsequent rows that introduce new fields
///    are rejected.
/// 2. **Pre-declared** ([`DataFrameBuilder::with_schema`]) — schema fixed at
///    construction; first push skips discovery; later pushes must conform.
/// 3. **Growing** ([`DataFrameBuilder::grow_schema`]) — new fields seen in
///    later rows are added on-the-fly and back-filled with NA on prior rows.
///    Composes with [`with_schema`](Self::with_schema) to start from a
///    declared partial schema.
///
/// Call [`finish`](Self::finish) to produce the [`ColumnarDataFrame`].
///
/// Use [`iter_to_dataframe`] when an iterator suffices; reach for this when
/// you need explicit control over push points (conditional skipping,
/// streaming from multiple sources, custom NA strategies).
///
/// # Examples
///
/// ```rust,ignore
/// # use miniextendr_api::serde::{DataFrameBuilder, TypeSpec};
/// # use serde::Serialize;
/// #[derive(Serialize)]
/// struct Row { id: i32, label: Option<String> }
///
/// // Pre-declared schema. Optional(Character) keeps the column character-typed
/// // even if the first row's label is None.
/// let mut b = DataFrameBuilder::<Row>::with_schema(
///     [
///         ("id", TypeSpec::Integer),
///         ("label", TypeSpec::Optional(Box::new(TypeSpec::Character))),
///     ],
///     None,
/// );
/// b.push(Row { id: 1, label: None }).unwrap();
/// b.push(Row { id: 2, label: Some("two".into()) }).unwrap();
/// let df = b.finish().unwrap();
/// ```
pub struct DataFrameBuilder<T: Serialize> {
    /// Schema set on first push (default mode), at construction
    /// ([`with_schema`](Self::with_schema)), or grown lazily ([`grow_schema`](Self::grow_schema)).
    schema: Option<Schema>,
    /// One buffer per column; lengths grow with each row.
    columns: Vec<ColumnBuffer>,
    /// Per-row "did this column get a value?" tracking; resized to ncol on first push.
    filled: Vec<bool>,
    /// Number of rows pushed so far.
    nrow: usize,
    /// Initial capacity hint; if `Some`, columns are allocated with this capacity.
    nrow_hint: Option<usize>,
    /// Holds protected SEXPs from `ColumnBuffer::Generic` cells until `finish` assembles them.
    scope: crate::ProtectScope,
    /// When true, each push runs a Union-mode discovery pass and adds any
    /// previously-unseen fields (back-filling NA on existing rows).
    grow: bool,
    _marker: core::marker::PhantomData<fn(T)>,
}

impl<T: Serialize> DataFrameBuilder<T> {
    /// Create a new builder with schema discovered on first [`push`](Self::push).
    ///
    /// `nrow_hint` pre-sizes column buffers; `None` is acceptable.
    pub fn new(nrow_hint: Option<usize>) -> Self {
        Self {
            schema: None,
            columns: Vec::new(),
            filled: Vec::new(),
            nrow: 0,
            nrow_hint,
            // SAFETY: builder must be constructed on the R main thread.
            // ProtectScope carries NoSendSync; the builder cannot escape.
            scope: unsafe { crate::ProtectScope::new() },
            grow: false,
            _marker: core::marker::PhantomData,
        }
    }

    /// Create a builder with a pre-declared flat schema.
    ///
    /// Skips the first-row discovery pass. All later pushes are validated
    /// against this schema by the strict [`ColumnFiller`]; fields not in
    /// the schema produce an error (unless [`grow_schema`](Self::grow_schema)
    /// is chained, in which case new fields are added on the fly).
    ///
    /// `schema` is an iterable of `(name, TypeSpec)` pairs. Order is
    /// preserved in the resulting data.frame's column layout.
    ///
    /// **Limitation**: this constructor takes a flat schema only — nested
    /// struct flattening (`parent_child` columns) is not supported here.
    /// Callers who need flattened nested structs either let default
    /// discovery handle it, or pre-flatten the names themselves
    /// (`"parent_child"` strings).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use miniextendr_api::serde::{DataFrameBuilder, TypeSpec};
    /// # use serde::Serialize;
    /// #[derive(Serialize)]
    /// struct R { id: i32, name: String }
    ///
    /// let mut b = DataFrameBuilder::<R>::with_schema(
    ///     [("id", TypeSpec::Integer), ("name", TypeSpec::Character)],
    ///     Some(100),
    /// );
    /// for i in 0..100 {
    ///     b.push(R { id: i, name: format!("row_{i}") }).unwrap();
    /// }
    /// let df = b.finish().unwrap();
    /// ```
    pub fn with_schema<S, I>(schema: I, nrow_hint: Option<usize>) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = (S, TypeSpec)>,
    {
        let mut fields: Vec<FieldInfo> = Vec::new();
        let mut map: HashMap<String, FieldMapping> = HashMap::new();
        for (name, spec) in schema {
            let name = name.into();
            let col_idx = fields.len();
            let col_type = spec.into_column_type();
            fields.push(FieldInfo {
                name: name.clone(),
                col_type,
            });
            map.insert(name, FieldMapping::Scalar { col_idx });
        }
        let total = fields.len();
        let cap = nrow_hint.unwrap_or(0);
        let columns: Vec<ColumnBuffer> = fields
            .iter()
            .map(|f| ColumnBuffer::new(f.col_type, cap))
            .collect();
        let filled = vec![false; total];
        let schema = Schema {
            fields,
            field_map: FieldMap {
                map,
                col_start: 0,
                total_cols: total,
            },
        };
        Self {
            schema: Some(schema),
            columns,
            filled,
            nrow: 0,
            nrow_hint,
            // SAFETY: builder must be constructed on the R main thread.
            scope: unsafe { crate::ProtectScope::new() },
            grow: false,
            _marker: core::marker::PhantomData,
        }
    }

    /// Enable growing-schema mode: new fields discovered in later rows are
    /// added on the fly and back-filled with NA on prior rows.
    ///
    /// Composes with [`with_schema`](Self::with_schema) — call
    /// `DataFrameBuilder::with_schema(...).grow_schema()` to start with a
    /// declared partial schema and let new fields appear as rows arrive.
    ///
    /// Cost: O(new_fields × existing_nrow) on each push that introduces a
    /// new field. For row-by-row growing types this is amortised
    /// O(nrow × ncols) — the same shape as `vec_to_dataframe` today.
    ///
    /// **Type clashes**: a later row writing a `String` to a column whose
    /// first-seen value was an `Integer` follows today's union-path
    /// behaviour — the value is coerced or NA-filled by
    /// [`ColumnBuffer::push_value`]. No new error is raised. If your data
    /// is genuinely heterogeneous, declare the column as
    /// `TypeSpec::Generic` to get a list-column.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use miniextendr_api::serde::DataFrameBuilder;
    /// # use std::collections::BTreeMap;
    /// // Heterogeneous rows: each row is a map; later rows introduce new keys.
    /// let mut b = DataFrameBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
    ///
    /// let r1: BTreeMap<String, i32> = [("a".into(), 1)].into_iter().collect();
    /// let r2: BTreeMap<String, i32> = [("a".into(), 2), ("b".into(), 3)].into_iter().collect();
    /// b.push(r1).unwrap();
    /// b.push(r2).unwrap();  // adds column "b", back-fills NA on row 0
    /// let df = b.finish().unwrap();
    /// ```
    pub fn grow_schema(mut self) -> Self {
        self.grow = true;
        self
    }

    /// Append a row.
    ///
    /// In default mode the first call discovers the schema. In
    /// [`with_schema`](Self::with_schema) mode the schema is fixed at
    /// construction. In [`grow_schema`](Self::grow_schema) mode each push
    /// also runs a discovery pass and absorbs any new fields, back-filling
    /// NA on prior rows.
    pub fn push(&mut self, row: T) -> Result<(), RSerdeError> {
        if self.schema.is_none() {
            let mut acc = SchemaAccumulator::new(SchemaMode::SingleRow);
            acc.feed(&row)?;
            let schema = acc.finalize()?;
            let ncol = schema.fields.len();
            let cap = self.nrow_hint.unwrap_or(0);
            self.columns = schema
                .fields
                .iter()
                .map(|f| ColumnBuffer::new(f.col_type, cap))
                .collect();
            self.filled = vec![false; ncol];
            self.schema = Some(schema);
        } else if self.grow {
            self.absorb_new_fields(&row)?;
        }

        let schema = self.schema.as_ref().expect("schema set above");
        let ncol = schema.fields.len();

        let filler = ColumnFiller {
            columns: &mut self.columns,
            field_map: &schema.field_map,
            filled: &mut self.filled,
            col_start: 0,
            col_count: ncol,
            is_top_level: true,
            pending_key: None,
            scope: &self.scope,
            strict: true,
        };
        // ColumnFiller::SerializeStruct::end calls pad_unfilled, which
        // NA-pads any column the row didn't touch and resets `filled` for
        // the next push — same flow as ColumnarDataFrame::from_rows.
        row.serialize(filler)?;

        self.nrow += 1;
        Ok(())
    }

    /// Run a Union-mode discovery pass over `row` and absorb any fields not
    /// present in the current schema. New columns get a fresh
    /// [`ColumnBuffer`] back-filled with `self.nrow` NA values so the
    /// per-column length invariant is preserved going into the fill step.
    ///
    /// Compound (nested-struct) discoveries are flattened by
    /// [`SchemaAccumulator::finalize`]; each leaf field appears as its own
    /// top-level entry in the per-row schema and is added here as a flat
    /// scalar column under that name.
    fn absorb_new_fields(&mut self, row: &T) -> Result<(), RSerdeError> {
        let mut acc = SchemaAccumulator::new(SchemaMode::Union);
        // Union mode tolerates rows that probe to no fields (e.g. top-level
        // None); we propagate hard serialization errors.
        let _ = acc.feed(row);
        // If the row produced no fields at all, `finalize` would error; treat
        // that as "no new fields" rather than failing the push.
        let discovered = match acc.finalize() {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };

        let cap = self.nrow_hint.unwrap_or(0);
        let schema = self.schema.as_mut().expect("schema set by push gating");

        for field in discovered.fields {
            if schema.field_map.map.contains_key(&field.name) {
                continue;
            }
            // Allocate a fresh column and back-fill prior rows with NA so
            // the column length matches the existing nrow.
            let mut buf = ColumnBuffer::new(field.col_type, cap);
            for _ in 0..self.nrow {
                buf.push_na();
            }
            let col_idx = schema.fields.len();
            self.columns.push(buf);
            self.filled.push(false);
            schema
                .field_map
                .map
                .insert(field.name.clone(), FieldMapping::Scalar { col_idx });
            schema.field_map.total_cols = col_idx + 1;
            schema.fields.push(field);
        }
        Ok(())
    }

    /// Number of rows pushed so far.
    pub fn len(&self) -> usize {
        self.nrow
    }

    /// Whether no rows have been pushed yet.
    pub fn is_empty(&self) -> bool {
        self.nrow == 0
    }

    /// Consume the builder and produce the data.frame.
    ///
    /// An empty builder produces an empty 0-row 0-column data.frame
    /// (matching `vec_to_dataframe(&[])`).
    pub fn finish(self) -> Result<ColumnarDataFrame, RSerdeError> {
        let Some(schema) = self.schema else {
            return Ok(ColumnarDataFrame {
                sexp: empty_dataframe(),
            });
        };
        // SAFETY: nrow columns × matching lengths invariant maintained by push.
        // `assemble_with_scope` runs `assemble_dataframe` then drops the scope —
        // see its docstring for the drop-order rationale.
        Ok(unsafe { assemble_with_scope(&schema, &self.columns, self.nrow, self.scope) })
    }
}

// endregion

// region: Field mapping (recursive name → column routing)

/// Maps a field name to its column location in the flat column array.
enum FieldMapping {
    /// Scalar field: writes directly to one column.
    Scalar { col_idx: usize },
    /// Compound field (flattened nested struct): spans multiple columns.
    Compound {
        col_start: usize,
        col_count: usize,
        sub_fields: FieldMap,
    },
}

/// Name-to-column mapping for one level of struct fields.
struct FieldMap {
    map: HashMap<String, FieldMapping>,
    col_start: usize,
    total_cols: usize,
}

// endregion

// region: Schema discovery

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Logical,
    Integer,
    Real,
    Character,
    Generic,
}

struct FieldInfo {
    name: String,
    col_type: ColumnType,
}

struct Schema {
    fields: Vec<FieldInfo>,
    field_map: FieldMap,
}

/// A candidate mapping for a single key, extracted from one probe row.
enum Candidate {
    Scalar(ColumnType),
    Compound {
        fields: Vec<FieldInfo>,
        sub_map: FieldMap,
    },
}

/// Resolve a slice of candidates for one key into the best single candidate.
///
/// Lattice (highest wins):
/// - `Compound` beats everything (has concrete shape).
/// - `Scalar(non-Generic)` beats `Scalar(Generic)`.
/// - `Scalar(Generic)` is the bottom (None-only probes land here).
///
/// Two `Scalar(non-Generic)` of different types: keep the first seen (no widening).
/// Two `Compound` of different shapes: keep the first seen (recursive union is out of scope).
fn resolve_candidates(candidates: &mut Vec<Candidate>) -> Candidate {
    // Walk candidates and pick the best.
    // We need to own the winner, so find its index then swap-remove.
    let mut best_idx = 0;
    for (i, c) in candidates.iter().enumerate() {
        match (&candidates[best_idx], c) {
            // Compound is always at least as good as what we have.
            (_, Candidate::Compound { .. }) => {
                best_idx = i;
                break; // Compound is the top of the lattice — no need to look further.
            }
            // Scalar(non-Generic) beats Scalar(Generic).
            (Candidate::Scalar(ColumnType::Generic), Candidate::Scalar(t))
                if *t != ColumnType::Generic =>
            {
                best_idx = i;
            }
            _ => {}
        }
    }
    candidates.swap_remove(best_idx)
}

/// Mode controls accumulation strictness and the final empty-schema error wording.
///
/// `SingleRow` produces the `DataFrameBuilder: first row has no fields` error; `Union`
/// produces the `ColumnarDataFrame::from_rows: no fields discovered from any row` error.
/// Field collection itself is identical — both feed candidates into the same lattice
/// resolver. SingleRow callers feed exactly once; Union callers feed per row.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SchemaMode {
    SingleRow,
    Union,
}

/// Accumulates per-row probes into per-key candidate lists, then resolves them
/// into a unified [`Schema`].
///
/// Two limitations vs. a fully-typed schema (apply to Union mode; SingleRow sees ≤1
/// candidate per key so the lattice degenerates):
///
/// - **Truly-all-None nested Option<Struct>**: when every row has `None` for an
///   `Option<UserStruct>`, the probe never sees the inner struct's fields. The key lands
///   as `Scalar(Generic)` (no Compound ever seen), which the assembly-time all-None
///   downgrade converts to a single logical-NA column. Structurally unfixable without a
///   type-level hint on stable Rust.
///
/// - **Compound-vs-Compound recursive union**: when two rows produce different Compound
///   shapes for the same key (e.g., enum variants with different nested structs), the
///   first Compound wins and the second is silently discarded. Recursive union is tracked
///   as a separate follow-up.
struct SchemaAccumulator {
    candidates: HashMap<String, Vec<Candidate>>,
    key_order: Vec<String>,
    mode: SchemaMode,
}

impl SchemaAccumulator {
    fn new(mode: SchemaMode) -> Self {
        Self {
            candidates: HashMap::new(),
            key_order: Vec::new(),
            mode,
        }
    }

    /// Probe one row and append its per-key candidates. Caller controls whether
    /// to propagate or swallow the error (Union mode treats per-row failures
    /// like top-level `None` as expected and skips them).
    fn feed<T: ?Sized + Serialize>(&mut self, row: &T) -> Result<(), RSerdeError> {
        let mut discoverer = SchemaDiscoverer::new(0);
        row.serialize(&mut discoverer)?;

        for key in &discoverer.key_order {
            if !self.candidates.contains_key(key) {
                self.key_order.push(key.clone());
                self.candidates.insert(key.clone(), Vec::new());
            }

            let Some(mapping) = discoverer.mappings.remove(key) else {
                continue;
            };

            let candidate = match mapping {
                FieldMapping::Scalar { col_idx } => {
                    Candidate::Scalar(discoverer.fields[col_idx].col_type)
                }
                FieldMapping::Compound {
                    col_start,
                    col_count,
                    sub_fields,
                } => {
                    let fields: Vec<FieldInfo> = (col_start..col_start + col_count)
                        .map(|i| FieldInfo {
                            name: discoverer.fields[i].name.clone(),
                            col_type: discoverer.fields[i].col_type,
                        })
                        .collect();
                    Candidate::Compound {
                        fields,
                        sub_map: sub_fields,
                    }
                }
            };
            self.candidates.get_mut(key).unwrap().push(candidate);
        }
        Ok(())
    }

    /// Resolve each key's candidate list into the best single candidate and
    /// build the unified [`Schema`]. Errors with a mode-specific message when
    /// no fields were collected.
    fn finalize(mut self) -> Result<Schema, RSerdeError> {
        let mut unified_fields: Vec<FieldInfo> = Vec::new();
        let mut unified_mappings: HashMap<String, FieldMapping> = HashMap::new();

        for key in &self.key_order {
            let candidates = self.candidates.get_mut(key).unwrap();
            if candidates.is_empty() {
                continue;
            }
            let new_start = unified_fields.len();
            match resolve_candidates(candidates) {
                Candidate::Scalar(col_type) => {
                    unified_fields.push(FieldInfo {
                        name: key.clone(),
                        col_type,
                    });
                    unified_mappings
                        .insert(key.clone(), FieldMapping::Scalar { col_idx: new_start });
                }
                Candidate::Compound { fields, sub_map } => {
                    let col_count = fields.len();
                    // sub_map indices were relative to col_offset=0 in the per-row probe;
                    // remap them to the actual position in the unified layout.
                    let old_base = sub_map.col_start;
                    for field in fields {
                        unified_fields.push(field);
                    }
                    let remapped = remap_field_map(sub_map, old_base, new_start);
                    unified_mappings.insert(
                        key.clone(),
                        FieldMapping::Compound {
                            col_start: new_start,
                            col_count,
                            sub_fields: remapped,
                        },
                    );
                }
            }
        }

        if unified_fields.is_empty() {
            return Err(RSerdeError::Message(match self.mode {
                SchemaMode::SingleRow => "DataFrameBuilder: first row has no fields".into(),
                SchemaMode::Union => {
                    "ColumnarDataFrame::from_rows: no fields discovered from any row".into()
                }
            }));
        }

        let total = unified_fields.len();
        Ok(Schema {
            fields: unified_fields,
            field_map: FieldMap {
                map: unified_mappings,
                col_start: 0,
                total_cols: total,
            },
        })
    }
}

/// Remap all column indices in a FieldMap from old base to new base.
fn remap_field_map(old: FieldMap, old_base: usize, new_base: usize) -> FieldMap {
    FieldMap {
        map: old
            .map
            .into_iter()
            .map(|(k, v)| (k, remap_field_mapping(v, old_base, new_base)))
            .collect(),
        col_start: new_base,
        total_cols: old.total_cols,
    }
}

fn remap_field_mapping(m: FieldMapping, old_base: usize, new_base: usize) -> FieldMapping {
    match m {
        FieldMapping::Scalar { col_idx } => FieldMapping::Scalar {
            col_idx: col_idx - old_base + new_base,
        },
        FieldMapping::Compound {
            col_start,
            col_count,
            sub_fields,
        } => {
            let new_col_start = col_start - old_base + new_base;
            FieldMapping::Compound {
                col_start: new_col_start,
                col_count,
                sub_fields: remap_field_map(sub_fields, col_start, new_col_start),
            }
        }
    }
}

/// Try to recursively discover and flatten a nested struct value.
/// Returns (flat_fields, field_map) if the value serializes as a struct.
fn try_discover_nested<T: ?Sized + Serialize>(
    value: &T,
    col_offset: usize,
) -> Option<(Vec<FieldInfo>, FieldMap)> {
    // Use single-row discovery for nested values (they're not enums at this level)
    let mut discoverer = SchemaDiscoverer::new(col_offset);
    if value.serialize(&mut discoverer).is_ok() && !discoverer.fields.is_empty() {
        let total = discoverer.fields.len();
        Some((
            discoverer.fields,
            FieldMap {
                map: discoverer.mappings,
                col_start: col_offset,
                total_cols: total,
            },
        ))
    } else {
        None
    }
}

struct SchemaDiscoverer {
    fields: Vec<FieldInfo>,
    mappings: HashMap<String, FieldMapping>,
    key_order: Vec<String>,
    col_offset: usize,
}

impl SchemaDiscoverer {
    fn new(col_offset: usize) -> Self {
        Self {
            fields: Vec::new(),
            mappings: HashMap::new(),
            key_order: Vec::new(),
            col_offset,
        }
    }

    /// Process a field: try to flatten as nested struct, else probe as scalar.
    fn process_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.key_order.push(key.to_string());
        let abs_col = self.col_offset + self.fields.len();
        if let Some((sub_fields, sub_map)) = try_discover_nested(value, abs_col) {
            let count = sub_fields.len();
            for mut field in sub_fields {
                field.name = format!("{key}_{}", field.name);
                self.fields.push(field);
            }
            self.mappings.insert(
                key.to_string(),
                FieldMapping::Compound {
                    col_start: abs_col,
                    col_count: count,
                    sub_fields: sub_map,
                },
            );
        } else {
            let mut type_probe = TypeProbe {
                col_type: ColumnType::Generic,
            };
            let _ = value.serialize(&mut type_probe);
            self.fields.push(FieldInfo {
                name: key.to_string(),
                col_type: type_probe.col_type,
            });
            self.mappings
                .insert(key.to_string(), FieldMapping::Scalar { col_idx: abs_col });
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut SchemaDiscoverer {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = SchemaMapDiscoverer<'a>;
    type SerializeStruct = SchemaStructDiscoverer<'a>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SchemaStructDiscoverer { parent: self })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(SchemaMapDiscoverer {
            parent: self,
            pending_key: None,
        })
    }

    reject_non_struct!(
        "ColumnarDataFrame::from_rows: expected struct",
        allow_some_none
    );
}

struct SchemaStructDiscoverer<'a> {
    parent: &'a mut SchemaDiscoverer,
}

impl ser::SerializeStruct for SchemaStructDiscoverer<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.parent.process_field(key, value)
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

/// Map-based schema discoverer for structs using `#[serde(flatten)]`.
struct SchemaMapDiscoverer<'a> {
    parent: &'a mut SchemaDiscoverer,
    pending_key: Option<String>,
}

impl ser::SerializeMap for SchemaMapDiscoverer<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.parent.process_field(&key, value)
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}
// endregion

// region: Type probe (discovers column type from a single value)

struct TypeProbe {
    col_type: ColumnType,
}

impl ser::Serializer for &mut TypeProbe {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = ser::Impossible<(), RSerdeError>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Logical;
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Integer;
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Real;
        Ok(())
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        // Keep existing type (handles Option<T> where first element is None)
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Character;
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        self.col_type = ColumnType::Generic;
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        self.col_type = ColumnType::Generic;
        Err(RSerdeError::Message("probe complete".into()))
    }
}
// endregion

// region: Column buffers

enum ColumnBuffer {
    Logical(Vec<i32>),
    Integer(Vec<i32>),
    Real(Vec<f64>),
    Character(Vec<Option<String>>),
    Generic(Vec<Option<SEXP>>),
}

impl ColumnBuffer {
    fn new(col_type: ColumnType, capacity: usize) -> Self {
        match col_type {
            ColumnType::Logical => ColumnBuffer::Logical(Vec::with_capacity(capacity)),
            ColumnType::Integer => ColumnBuffer::Integer(Vec::with_capacity(capacity)),
            ColumnType::Real => ColumnBuffer::Real(Vec::with_capacity(capacity)),
            ColumnType::Character => ColumnBuffer::Character(Vec::with_capacity(capacity)),
            ColumnType::Generic => ColumnBuffer::Generic(Vec::with_capacity(capacity)),
        }
    }

    fn push_na(&mut self) {
        match self {
            ColumnBuffer::Logical(v) => v.push(i32::MIN),
            ColumnBuffer::Integer(v) => v.push(i32::MIN),
            ColumnBuffer::Real(v) => v.push(NA_REAL),
            ColumnBuffer::Character(v) => v.push(None),
            ColumnBuffer::Generic(v) => v.push(None),
        }
    }

    fn push_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
        scope: &crate::ProtectScope,
    ) -> Result<(), RSerdeError> {
        match self {
            ColumnBuffer::Logical(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Bool(b) => i32::from(b),
                    ExtractedValue::None => i32::MIN, // NA_LOGICAL
                    _ => i32::MIN,
                });
            }
            ColumnBuffer::Integer(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Int(i) => i,
                    ExtractedValue::None => i32::MIN, // NA_INTEGER
                    _ => i32::MIN,
                });
            }
            ColumnBuffer::Real(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Real(f) => f,
                    ExtractedValue::Int(i) => f64::from(i),
                    ExtractedValue::None => NA_REAL,
                    _ => NA_REAL,
                });
            }
            ColumnBuffer::Character(v) => {
                let mut probe = ValueExtractor::default();
                value.serialize(&mut probe)?;
                v.push(match probe.value {
                    ExtractedValue::Str(s) => Some(s),
                    ExtractedValue::None => None, // NA_character_
                    _ => None,
                });
            }
            ColumnBuffer::Generic(v) => {
                // Fall back to full serde serialization for this element.
                // The returned SEXP is freshly allocated and unrooted; root it
                // in the caller's ProtectScope so it survives the GC pressure
                // from subsequent rows' fills and from `assemble_dataframe`'s
                // own allocations.
                let sexp = value.serialize(super::ser::RSerializer)?;
                let rooted = unsafe { scope.protect_raw(sexp) };
                v.push(Some(rooted));
            }
        }
        Ok(())
    }
}
// endregion

// region: Value extraction

#[derive(Default)]
struct ValueExtractor {
    value: ExtractedValue,
}

#[derive(Default)]
enum ExtractedValue {
    #[default]
    None,
    Bool(bool),
    Int(i32),
    Real(f64),
    Str(String),
}

impl ser::Serializer for &mut ValueExtractor {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = ser::Impossible<(), RSerdeError>;
    type SerializeStruct = ser::Impossible<(), RSerdeError>;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_bool(self, v: bool) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Bool(v);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Int(i32::from(v));
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<(), RSerdeError> {
        if let Ok(i) = i32::try_from(v) {
            self.value = ExtractedValue::Int(i);
        } else {
            self.value = ExtractedValue::Real(f64::from(v));
        }
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v as f64);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(f64::from(v));
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Real(v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::None;
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        v: &'static str,
    ) -> Result<(), RSerdeError> {
        self.value = ExtractedValue::Str(v.to_string());
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        Err(RSerdeError::Message(
            "compound type in value extractor".into(),
        ))
    }
}
// endregion

// region: Column filling (name-based, handles skip_serializing_if)

/// Column filler. Dispatches each field by name to the correct column(s).
///
/// When `is_top_level` is true, this is the top-level row filler: `pad_unfilled`
/// resets filled flags for the next row. When false, this is a sub-filler for
/// nested struct fields: `pad_unfilled` marks columns as filled (the top-level
/// filler handles the reset), and `serialize_some`/`serialize_none` support
/// Option-wrapped nested structs with NA-fill logic.
struct ColumnFiller<'a> {
    columns: &'a mut [ColumnBuffer],
    field_map: &'a FieldMap,
    filled: &'a mut Vec<bool>,
    col_start: usize,
    col_count: usize,
    is_top_level: bool,
    pending_key: Option<String>,
    scope: &'a crate::ProtectScope,
    /// When true, fields not in the schema produce an error instead of being
    /// silently ignored. `DataFrameBuilder` sets this to enforce first-row
    /// schema; `ColumnarDataFrame::from_rows` leaves it false because its
    /// schema is the union of all rows (so misses can't happen).
    strict: bool,
}

impl ColumnFiller<'_> {
    fn fill_field<T: ?Sized + Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        match self.field_map.map.get(key) {
            Some(FieldMapping::Scalar { col_idx }) => {
                self.columns[*col_idx].push_value(value, self.scope)?;
                self.filled[*col_idx] = true;
            }
            Some(FieldMapping::Compound {
                sub_fields,
                col_start,
                col_count,
                ..
            }) => {
                let sub = ColumnFiller {
                    columns: self.columns,
                    field_map: sub_fields,
                    filled: self.filled,
                    col_start: *col_start,
                    col_count: *col_count,
                    is_top_level: false,
                    pending_key: None,
                    scope: self.scope,
                    strict: self.strict,
                };
                value.serialize(sub)?;
            }
            None => {
                // Field not in schema. The union-schema path (from_rows) never
                // hits this because the schema absorbed every row's keys; the
                // streaming path (DataFrameBuilder, strict=true) does, when a
                // later row introduces a new field.
                if self.strict {
                    return Err(RSerdeError::Message(format!(
                        "DataFrameBuilder: row introduced field {key:?} not in initial schema"
                    )));
                }
            }
        }
        Ok(())
    }

    fn pad_unfilled(&mut self) {
        let start = self.field_map.col_start;
        let end = start + self.field_map.total_cols;
        if self.is_top_level {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                self.filled[i] = false; // reset for next row
            }
        } else {
            for i in start..end {
                if !self.filled[i] {
                    self.columns[i].push_na();
                }
                // Don't reset — the top-level filler handles reset
                self.filled[i] = true;
            }
        }
    }
}

impl<'a> ser::Serializer for ColumnFiller<'a> {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = ser::Impossible<(), RSerdeError>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = ser::Impossible<(), RSerdeError>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(self)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        if self.is_top_level {
            return Err(RSerdeError::Message("expected struct".into()));
        }
        // Fill all columns owned by this sub-filler with NA
        for i in self.col_start..self.col_start + self.col_count {
            self.columns[i].push_na();
            self.filled[i] = true;
        }
        Ok(())
    }

    reject_non_struct!(@primitives "expected struct");
}

impl ser::SerializeStruct for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        self.fill_field(key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}

impl ser::SerializeMap for ColumnFiller<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        let mut extractor = ValueExtractor::default();
        key.serialize(&mut extractor)?;
        self.pending_key = match extractor.value {
            ExtractedValue::Str(s) => Some(s),
            _ => Some(String::new()),
        };
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        let key = self.pending_key.take().unwrap_or_default();
        self.fill_field(&key, value)
    }

    fn end(mut self) -> Result<(), RSerdeError> {
        self.pad_unfilled();
        Ok(())
    }
}
// endregion

// region: Data.frame assembly

/// Build an empty R data.frame (0 rows, 0 columns).
fn empty_dataframe() -> SEXP {
    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, 0);
        Rf_protect(list);

        // Set class = "data.frame"
        list.set_class(crate::cached_class::data_frame_class_sexp());

        // Set compact row.names: c(NA_integer_, 0)
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        Rf_protect(row_names);
        rn[0] = i32::MIN; // NA_integer_
        rn[1] = 0;
        list.set_row_names(row_names);

        Rf_unprotect(2);
        list
    }
}

/// Assemble column buffers into a data.frame, then drop `scope` only after the
/// assembled VECSXP has copied every protected element. The returned
/// [`ColumnarDataFrame`] is rooted by the caller's protection of its SEXP.
///
/// Centralises the protect-scope drop discipline shared by
/// [`ColumnarDataFrame::from_rows`] and [`DataFrameBuilder::finish`].
///
/// # Safety
///
/// Must be called from the R main thread. All column buffers must have
/// exactly `nrow` elements.
unsafe fn assemble_with_scope(
    schema: &Schema,
    columns: &[ColumnBuffer],
    nrow: usize,
    scope: crate::ProtectScope,
) -> ColumnarDataFrame {
    let df = ColumnarDataFrame {
        sexp: unsafe { assemble_dataframe(&schema.fields, columns, nrow) },
    };
    drop(scope);
    df
}

/// Assemble column buffers into an R data.frame SEXP.
///
/// # Safety
///
/// Must be called from the R main thread. All column buffers must have
/// exactly `nrow` elements.
unsafe fn assemble_dataframe(fields: &[FieldInfo], columns: &[ColumnBuffer], nrow: usize) -> SEXP {
    let ncol: isize = fields.len().try_into().expect("ncol exceeds isize::MAX");

    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, ncol);
        Rf_protect(list);

        // Build each column and set into list
        for (i, col) in columns.iter().enumerate() {
            let idx: isize = i.try_into().expect("column index exceeds isize::MAX");
            let col_sexp = column_to_sexp(col, nrow);
            Rf_protect(col_sexp);
            list.set_vector_elt(idx, col_sexp);
            Rf_unprotect(1); // col_sexp is now held by list
        }

        // Set names
        let names_sexp = Rf_allocVector(SEXPTYPE::STRSXP, ncol);
        Rf_protect(names_sexp);
        for (i, field) in fields.iter().enumerate() {
            let idx: isize = i.try_into().expect("field index exceeds isize::MAX");
            names_sexp.set_string_elt(idx, SEXP::charsxp(&field.name));
        }
        list.set_names(names_sexp);

        // Set class = "data.frame"
        list.set_class(crate::cached_class::data_frame_class_sexp());

        // Set compact row.names: c(NA_integer_, -nrow)
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        Rf_protect(row_names);
        rn[0] = i32::MIN; // NA_integer_
        rn[1] = -i32::try_from(nrow).expect("data.frame row count exceeds i32::MAX");
        list.set_row_names(row_names);

        Rf_unprotect(3); // list, names, row_names
        list
    }
}

/// Convert a single column buffer into an R SEXP vector.
unsafe fn column_to_sexp(col: &ColumnBuffer, nrow: usize) -> SEXP {
    use crate::into_r::alloc_r_vector;

    unsafe {
        match col {
            ColumnBuffer::Logical(v) => {
                let (sexp, dst) = alloc_r_vector::<crate::ffi::RLogical>(nrow);
                let dst_i32: &mut [i32] =
                    std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), nrow);
                dst_i32.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Integer(v) => {
                let (sexp, dst) = alloc_r_vector::<i32>(nrow);
                dst.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Real(v) => {
                let (sexp, dst) = alloc_r_vector::<f64>(nrow);
                dst.copy_from_slice(v);
                sexp
            }
            ColumnBuffer::Character(v) => {
                let nrow_r: isize = nrow.try_into().expect("nrow exceeds isize::MAX");
                let sexp = Rf_allocVector(SEXPTYPE::STRSXP, nrow_r);
                // PROTECT discipline: SEXP::charsxp (Rf_mkCharLenCE) allocates
                // and can trigger GC under gctorture, which would reclaim our
                // unprotected STRSXP. Protect here; balance with Rf_unprotect
                // before returning.
                Rf_protect(sexp);
                for (i, val) in v.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    match val {
                        Some(s) => {
                            sexp.set_string_elt(idx, SEXP::charsxp(s));
                        }
                        None => {
                            sexp.set_string_elt(idx, SEXP::na_string());
                        }
                    }
                }
                Rf_unprotect(1);
                sexp
            }
            ColumnBuffer::Generic(v) => {
                let nrow_r: isize = nrow.try_into().expect("nrow exceeds isize::MAX");
                // If every entry is None or Some(NULL) — meaning all rows had
                // `Option<T> = None` (which serializes as NULL) or the column
                // was always NA-padded — emit a logical NA vector instead of
                // list(NULL, …).  R coerces logical NA to the surrounding type
                // on first use, so this is invisible downstream:
                //   c(NA, 1L) → integer,  c(NA, "x") → character, etc.
                //
                // `push_na` (pad for missing rows) stores `None`.
                // `push_value(&None::<T>)` stores `Some(SEXP::nil())` via
                // RSerializer::serialize_none.
                // Both are "NA-like" in the generic-list context.
                let all_null = v.iter().all(|e| match e {
                    None => true,
                    Some(s) => s.is_nil(),
                });
                if all_null {
                    let (sexp, dst) = alloc_r_vector::<crate::ffi::RLogical>(nrow);
                    let dst_i32: &mut [i32] =
                        std::slice::from_raw_parts_mut(dst.as_mut_ptr().cast::<i32>(), nrow);
                    dst_i32.fill(NA_LOGICAL);
                    return sexp;
                }
                let sexp = Rf_allocVector(SEXPTYPE::VECSXP, nrow_r);
                for (i, val) in v.iter().enumerate() {
                    let idx: isize = i.try_into().expect("index exceeds isize::MAX");
                    if let Some(elem) = val {
                        sexp.set_vector_elt(idx, *elem);
                    } else {
                        sexp.set_vector_elt(idx, SEXP::nil());
                    }
                }
                sexp
            }
        }
    }
}
// endregion

// region: Enum split (vec_to_dataframe_split)

struct VariantInfo {
    name: String,
    is_unit: bool,
    tag_field: Option<String>,
}

fn extract_variant_info<T: Serialize>(row: &T) -> Option<VariantInfo> {
    let mut ext = VariantNameExtractor::default();
    let _ = row.serialize(&mut ext);
    ext.name.map(|name| VariantInfo {
        name,
        is_unit: ext.is_unit,
        tag_field: ext.tag_field,
    })
}

// ── VariantNameExtractor ──────────────────────────────────────────────────────

#[derive(Default)]
struct VariantNameExtractor {
    name: Option<String>,
    is_unit: bool,
    tag_field: Option<String>,
}

struct NoopStructVariant;
impl ser::SerializeStructVariant for NoopStructVariant {
    type Ok = ();
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

struct NoopTupleVariant;
impl ser::SerializeTupleVariant for NoopTupleVariant {
    type Ok = ();
    type Error = RSerdeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, _: &T) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

struct TagStructCapture<'a> {
    parent: &'a mut VariantNameExtractor,
    first_done: bool,
}

impl ser::SerializeStruct for TagStructCapture<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), RSerdeError> {
        if !self.first_done {
            self.first_done = true;
            let mut ve = ValueExtractor::default();
            let _ = value.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.parent.name = Some(s);
                self.parent.tag_field = Some(key.to_string());
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

// Defends against custom `Serialize` impls that emit internally-tagged enums via
// `serialize_map` rather than `serialize_struct`. `#[derive(Serialize)]` always
// uses `serialize_struct` for internally-tagged enums, so this path doesn't fire
// for derive-generated impls — but hand-written serializers may use a map.
struct TagMapCapture<'a> {
    parent: &'a mut VariantNameExtractor,
    pending_key: Option<String>,
    first_done: bool,
}

impl ser::SerializeMap for TagMapCapture<'_> {
    type Ok = ();
    type Error = RSerdeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), RSerdeError> {
        if !self.first_done {
            let mut ve = ValueExtractor::default();
            let _ = key.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.pending_key = Some(s);
            }
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), RSerdeError> {
        if !self.first_done {
            self.first_done = true;
            let key = self.pending_key.take().unwrap_or_default();
            let mut ve = ValueExtractor::default();
            let _ = value.serialize(&mut ve);
            if let ExtractedValue::Str(s) = ve.value {
                self.parent.name = Some(s);
                self.parent.tag_field = Some(key);
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), RSerdeError> {
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut VariantNameExtractor {
    type Ok = ();
    type Error = RSerdeError;
    type SerializeSeq = ser::Impossible<(), RSerdeError>;
    type SerializeTuple = ser::Impossible<(), RSerdeError>;
    type SerializeTupleStruct = ser::Impossible<(), RSerdeError>;
    type SerializeTupleVariant = NoopTupleVariant;
    type SerializeMap = TagMapCapture<'a>;
    type SerializeStruct = TagStructCapture<'a>;
    type SerializeStructVariant = NoopStructVariant;

    fn serialize_bool(self, _: bool) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i8(self, _: i8) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i16(self, _: i16) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i32(self, _: i32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_i64(self, _: i64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u8(self, _: u8) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u16(self, _: u16) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u32(self, _: u32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_u64(self, _: u64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_f32(self, _: f32) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_f64(self, _: f64) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_char(self, _: char) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_str(self, _: &str) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_none(self) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), RSerdeError> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), RSerdeError> {
        self.name = Some(variant.to_string());
        self.is_unit = true;
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), RSerdeError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: &T,
    ) -> Result<(), RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, RSerdeError> {
        Err(RSerdeError::Message("seq in variant extractor".into()))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, RSerdeError> {
        Err(RSerdeError::Message("tuple in variant extractor".into()))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, RSerdeError> {
        Err(RSerdeError::Message(
            "tuple_struct in variant extractor".into(),
        ))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(NoopTupleVariant)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, RSerdeError> {
        Ok(TagMapCapture {
            parent: self,
            pending_key: None,
            first_done: false,
        })
    }
    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, RSerdeError> {
        Ok(TagStructCapture {
            parent: self,
            first_done: false,
        })
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, RSerdeError> {
        self.name = Some(variant.to_string());
        Ok(NoopStructVariant)
    }
}

// ── VariantStrippingSerializer ────────────────────────────────────────────────

struct VariantPayload<T>(T);

impl<T: Serialize> Serialize for VariantPayload<T> {
    fn serialize<S: ser::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(VariantStrippingSerializer { inner: s })
    }
}

struct VariantStrippingSerializer<S: ser::Serializer> {
    inner: S,
}

struct VariantAsStruct<S: ser::SerializeStruct>(S);

impl<S: ser::SerializeStruct> ser::SerializeStructVariant for VariantAsStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), S::Error> {
        self.0.serialize_field(key, value)
    }
    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

struct VariantAsTupleStruct<S: ser::SerializeTupleStruct>(S);

impl<S: ser::SerializeTupleStruct> ser::SerializeTupleVariant for VariantAsTupleStruct<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), S::Error> {
        self.0.serialize_field(value)
    }
    fn end(self) -> Result<S::Ok, S::Error> {
        self.0.end()
    }
}

impl<S: ser::Serializer> ser::Serializer for VariantStrippingSerializer<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    type SerializeSeq = S::SerializeSeq;
    type SerializeTuple = S::SerializeTuple;
    type SerializeTupleStruct = S::SerializeTupleStruct;
    type SerializeTupleVariant = VariantAsTupleStruct<S::SerializeTupleStruct>;
    type SerializeMap = S::SerializeMap;
    type SerializeStruct = S::SerializeStruct;
    type SerializeStructVariant = VariantAsStruct<S::SerializeStruct>;

    fn serialize_bool(self, v: bool) -> Result<S::Ok, S::Error> {
        self.inner.serialize_bool(v)
    }
    fn serialize_i8(self, v: i8) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i8(v)
    }
    fn serialize_i16(self, v: i16) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i16(v)
    }
    fn serialize_i32(self, v: i32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i32(v)
    }
    fn serialize_i64(self, v: i64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_i64(v)
    }
    fn serialize_u8(self, v: u8) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u8(v)
    }
    fn serialize_u16(self, v: u16) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u16(v)
    }
    fn serialize_u32(self, v: u32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u32(v)
    }
    fn serialize_u64(self, v: u64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_u64(v)
    }
    fn serialize_f32(self, v: f32) -> Result<S::Ok, S::Error> {
        self.inner.serialize_f32(v)
    }
    fn serialize_f64(self, v: f64) -> Result<S::Ok, S::Error> {
        self.inner.serialize_f64(v)
    }
    fn serialize_char(self, v: char) -> Result<S::Ok, S::Error> {
        self.inner.serialize_char(v)
    }
    fn serialize_str(self, v: &str) -> Result<S::Ok, S::Error> {
        self.inner.serialize_str(v)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<S::Ok, S::Error> {
        self.inner.serialize_bytes(v)
    }
    fn serialize_none(self) -> Result<S::Ok, S::Error> {
        self.inner.serialize_none()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<S::Ok, S::Error> {
        self.inner.serialize_some(v)
    }
    fn serialize_unit(self) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit_struct(name)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<S::Ok, S::Error> {
        self.inner.serialize_unit_struct(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        self.inner.serialize_newtype_struct(name, v)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<S::Ok, S::Error> {
        v.serialize(self.inner)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<S::SerializeSeq, S::Error> {
        self.inner.serialize_seq(len)
    }
    fn serialize_tuple(self, len: usize) -> Result<S::SerializeTuple, S::Error> {
        self.inner.serialize_tuple(len)
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<S::SerializeTupleStruct, S::Error> {
        self.inner.serialize_tuple_struct(name, len)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, S::Error> {
        let ts = self.inner.serialize_tuple_struct(variant, len)?;
        Ok(VariantAsTupleStruct(ts))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<S::SerializeMap, S::Error> {
        self.inner.serialize_map(len)
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<S::SerializeStruct, S::Error> {
        self.inner.serialize_struct(name, len)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, S::Error> {
        let s = self.inner.serialize_struct(variant, len)?;
        Ok(VariantAsStruct(s))
    }
}

// ── 0-column data.frame for unit variants ────────────────────────────────────

fn unit_variant_dataframe(nrow: usize) -> SEXP {
    unsafe {
        let list = Rf_allocVector(SEXPTYPE::VECSXP, 0);
        Rf_protect(list);
        list.set_class(crate::cached_class::data_frame_class_sexp());
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        Rf_protect(row_names);
        rn[0] = i32::MIN;
        rn[1] = -i32::try_from(nrow).expect("nrow overflow");
        list.set_row_names(row_names);
        Rf_unprotect(2);
        list
    }
}

// ── vec_to_dataframe_split ────────────────────────────────────────────────────

/// Output-shape selector for [`vec_to_dataframe_split`].
///
/// Configures whether per-variant data.frames carry an explicit variant-tag
/// column, and whether the result is one list per variant or a single
/// collated data.frame with the variant name on every row.
///
/// The variant name on the R side is whatever serde emits (PascalCase by
/// default). Override with `#[serde(rename_all = "snake_case")]` (or
/// similar) on the enum definition.
pub enum SplitShape {
    /// `list(VariantA = df, VariantB = df, …)` — historical behaviour.
    ///
    /// Single-variant input short-circuits to a bare data.frame instead of a
    /// one-element list. Per-variant data.frames do not carry the variant
    /// name — it lives on the list-element name.
    PerVariantList,

    /// Same shape as [`PerVariantList`](Self::PerVariantList) but each
    /// per-variant data.frame gets a leading column whose name is
    /// `column` and whose values are the variant name repeated nrow times.
    ///
    /// Use when the per-variant data.frame is going to be passed through
    /// `bind_rows` / `rbind` downstream and the variant tag needs to
    /// survive that pass.
    PerVariantListWithTag { column: String },

    /// Single collated data.frame containing the union of every variant's
    /// fields plus a leading variant-tag column named `column`. Fields
    /// belonging to other variants land as NA per row.
    ///
    /// Returns an error on empty input — the variant set is unknowable
    /// from zero rows so the union schema cannot be inferred.
    Collated { column: String },
}

/// Categorical return shape for the dataframe-helpers family
/// ([`vec_to_dataframe_split`] / [`result_to_dataframe`]).
///
/// Carries enough type information that downstream Rust code can `match`
/// on the variant without dispatching on SEXP type. Convert to a SEXP at
/// the `#[miniextendr]` function boundary via the [`crate::IntoR`] impl,
/// which collapses every variant to the equivalent R value (bare
/// data.frame / named list of data.frames / `list(results=, error=)`).
pub enum DataFrameShape {
    /// Single data.frame.
    ///
    /// Used by:
    /// - [`vec_to_dataframe_split`] when only one variant is present (and
    ///   [`PerVariantList`](SplitShape::PerVariantList) or
    ///   [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) is
    ///   selected — the single-variant short-circuit) or always under
    ///   [`Collated`](SplitShape::Collated).
    /// - [`result_to_dataframe`] under [`Auto`](ResultShape::Auto) when
    ///   every row is `Ok`, and always under
    ///   [`Collated`](ResultShape::Collated).
    Bare(ColumnarDataFrame),

    /// `list(results = <df | sentinel>, error = df)`.
    ///
    /// Produced by [`result_to_dataframe`] under
    /// [`Auto`](ResultShape::Auto) when at least one `Err` is present, and
    /// always under [`Split`](ResultShape::Split).
    Split {
        /// The Ok partition.
        results: SplitResults,
        /// The error partition (always present, possibly zero-row).
        error: ColumnarDataFrame,
    },

    /// `list(VariantA = df, VariantB = df, …)`.
    ///
    /// Produced by [`vec_to_dataframe_split`] under
    /// [`PerVariantList`](SplitShape::PerVariantList) /
    /// [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) when
    /// the input contains more than one variant. Order matches first-seen
    /// order in the input slice.
    PerVariantList(Vec<(String, ColumnarDataFrame)>),
}

/// Result partition for [`DataFrameShape::Split`].
///
/// Used to distinguish "no Ok rows at all" (which lets the caller supply
/// a sentinel value such as `NULL`, `NA`, `FALSE`, …) from a real
/// zero-row data.frame.
pub enum SplitResults {
    /// At least one `Ok` row — partition has a concrete data.frame.
    Some(ColumnarDataFrame),
    /// No `Ok` rows — sentinel SEXP supplied by the caller via
    /// `empty_ok_sentinel`.
    ///
    /// The SEXP is consumed by the [`crate::IntoR`] impl on
    /// [`DataFrameShape`]; until then it must be kept reachable
    /// (typically by being a child of `DataFrameShape`, which is itself
    /// rooted by the `#[miniextendr]` framework's return-value handling).
    None(SEXP),
}

impl crate::IntoR for DataFrameShape {
    type Error = std::convert::Infallible;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        match self {
            DataFrameShape::Bare(df) => Ok(df.into_sexp()),
            DataFrameShape::Split { results, error } => {
                // Protect both children via a NamedDataFrameListBuilder so
                // neither is reaped between the two set_vector_elt /
                // CHARSXP allocations in from_raw_pairs.
                let mut builder = NamedDataFrameListBuilder::with_capacity(2);
                builder = match results {
                    SplitResults::Some(df) => builder.push("results", df),
                    SplitResults::None(sentinel) => {
                        // SAFETY: sentinel originates from the user's `IntoR`
                        // implementation which has already produced a valid
                        // SEXP; protect via the builder's scope.
                        unsafe {
                            builder.scope.protect_raw(sentinel);
                        }
                        builder.pairs.push(("results".to_string(), sentinel));
                        builder
                    }
                };
                builder = builder.push("error", error);
                Ok(builder.build().into_sexp())
            }
            DataFrameShape::PerVariantList(pairs) => {
                let mut builder = NamedDataFrameListBuilder::with_capacity(pairs.len());
                for (name, df) in pairs {
                    builder = builder.push(name, df);
                }
                Ok(builder.build().into_sexp())
            }
        }
    }

    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
}

/// Partition a slice of serializable enum rows into per-variant data.frames.
///
/// The output shape is selected via [`SplitShape`]:
///
/// - [`PerVariantList`](SplitShape::PerVariantList) returns the historical
///   `list(VariantA = df, …)` shape (single-variant short-circuit to a
///   bare data.frame).
/// - [`PerVariantListWithTag`](SplitShape::PerVariantListWithTag) is the
///   same shape but each per-variant data.frame carries a leading
///   variant-tag column. Use when downstream `rbind`/`bind_rows` needs
///   the tag to survive.
/// - [`Collated`](SplitShape::Collated) returns one data.frame with all
///   variants stacked, plus a leading variant-tag column. Other-variant
///   fields are NA-filled per row.
///
/// Each variant's per-variant data.frame contains only that variant's
/// fields. For internally-tagged enums (`#[serde(tag = "...")]`), the
/// implicit tag column is dropped from each partition before any explicit
/// tag column is added back.
///
/// Variant-name casing: whatever serde emits. PascalCase by default;
/// override with `#[serde(rename_all = "snake_case")]` on the enum.
///
/// # Errors
///
/// - Any row serializes without a variant name (i.e. it's not an enum) —
///   use [`vec_to_dataframe`] for plain structs instead.
/// - [`Collated`](SplitShape::Collated) on empty input — the variant set
///   is unknowable.
/// - Underlying column-buffer assembly fails.
pub fn vec_to_dataframe_split<T: Serialize>(
    rows: &[T],
    shape: SplitShape,
) -> Result<DataFrameShape, RSerdeError> {
    if rows.is_empty() {
        return match shape {
            SplitShape::PerVariantList | SplitShape::PerVariantListWithTag { .. } => {
                Ok(DataFrameShape::PerVariantList(Vec::new()))
            }
            SplitShape::Collated { .. } => Err(RSerdeError::Message(
                "vec_to_dataframe_split(Collated): empty input — variant set is unknowable".into(),
            )),
        };
    }

    // Phase 1: extract variant info for each row.
    let infos: Vec<VariantInfo> = {
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            match extract_variant_info(row) {
                Some(info) => out.push(info),
                None => {
                    return Err(RSerdeError::Message(
                        "vec_to_dataframe_split: row has no variant — use vec_to_dataframe for plain structs".into(),
                    ));
                }
            }
        }
        out
    };

    // Detect internally-tagged enum style from the first row carrying one.
    let tag_field: Option<&str> = infos.iter().find_map(|i| i.tag_field.as_deref());

    match shape {
        SplitShape::Collated { column } => {
            // One data.frame, union schema across variants, leading tag column.
            // We synthesize TaggedVariantRow wrappers that emit the tag column
            // first and then route the row's payload through the existing
            // variant-stripping serializer so externally-tagged enums work too.
            let wrapped: Vec<TaggedVariantRow<'_, T>> = rows
                .iter()
                .zip(infos.iter())
                .map(|(row, info)| TaggedVariantRow {
                    tag_column: column.as_str(),
                    tag_value: info.name.as_str(),
                    inner: row,
                    tag_field: tag_field.map(str::to_string),
                })
                .collect();
            let df = ColumnarDataFrame::from_rows(&wrapped)?;
            Ok(DataFrameShape::Bare(df))
        }
        SplitShape::PerVariantList | SplitShape::PerVariantListWithTag { .. } => {
            // Group indices by variant name (preserve first-seen order).
            let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
            for (i, info) in infos.iter().enumerate() {
                if let Some(grp) = groups.iter_mut().find(|(n, _)| n == &info.name) {
                    grp.1.push(i);
                } else {
                    groups.push((info.name.clone(), vec![i]));
                }
            }

            let tag_column: Option<String> = match &shape {
                SplitShape::PerVariantListWithTag { column } => Some(column.clone()),
                _ => None,
            };

            // Build per-partition data.frames. We don't use NamedDataFrameListBuilder
            // here because we want to return owned ColumnarDataFrames, not a List.
            // Single-variant short-circuit needs to inspect the count below.
            let mut partitions: Vec<(String, ColumnarDataFrame)> = Vec::with_capacity(groups.len());

            for (name, indices) in &groups {
                let is_unit = infos[indices[0]].is_unit;

                let df = if is_unit {
                    let sexp = unit_variant_dataframe(indices.len());
                    ColumnarDataFrame { sexp }
                } else if tag_field.is_some() {
                    let refs: Vec<&T> = indices.iter().map(|&i| &rows[i]).collect();
                    let df = ColumnarDataFrame::from_rows(&refs)?;
                    if let Some(tf) = tag_field {
                        df.drop(tf)
                    } else {
                        df
                    }
                } else {
                    let wrapped: Vec<VariantPayload<&T>> =
                        indices.iter().map(|&i| VariantPayload(&rows[i])).collect();
                    ColumnarDataFrame::from_rows(&wrapped)?
                };

                let df = if let Some(col_name) = tag_column.as_deref() {
                    // SAFETY: R main thread. `make_strsxp_repeat` returns an
                    // unprotected STRSXP; protect across `prepend_column`'s
                    // internal allocations (drop → alloc_list → alloc_strsxp).
                    let tag_protect = unsafe {
                        crate::OwnedProtect::new(make_strsxp_repeat(name, indices.len()))
                    };
                    let out = df.prepend_column(col_name, *tag_protect);
                    drop(tag_protect);
                    out
                } else {
                    df
                };

                partitions.push((name.clone(), df));
            }

            // Single-variant short-circuit: collapse to a bare data.frame. The
            // variant name lives in the tag column already if requested; for
            // PerVariantList it lives on neither side (historical behaviour).
            if partitions.len() == 1 {
                let (_name, df) = partitions.into_iter().next().expect("len == 1");
                Ok(DataFrameShape::Bare(df))
            } else {
                Ok(DataFrameShape::PerVariantList(partitions))
            }
        }
    }
}

/// Allocate a STRSXP of length `n` filled with `value`.
///
/// # Safety
///
/// Must be called from the R main thread.
unsafe fn make_strsxp_repeat(value: &str, n: usize) -> SEXP {
    unsafe {
        let n_r: isize = n.try_into().expect("nrow exceeds isize::MAX");
        let sexp = Rf_allocVector(SEXPTYPE::STRSXP, n_r);
        // Protect the freshly-allocated STRSXP across `SEXP::charsxp`
        // (Rf_mkCharLenCE) — which can trigger GC under gctorture.
        Rf_protect(sexp);
        // Allocate the CHARSXP exactly once, then reuse it for every slot.
        let charsxp = SEXP::charsxp(value);
        for i in 0..n_r {
            sexp.set_string_elt(i, charsxp);
        }
        Rf_unprotect(1);
        sexp
    }
}

// endregion

// region: TaggedVariantRow (collated emit for vec_to_dataframe_split + result_to_dataframe)

/// Wrapper that prepends a `(tag_column, tag_value)` field to a struct row
/// when serialized through the columnar pipeline.
///
/// For externally-tagged enums (the default) the inner row goes through
/// [`VariantStrippingSerializer`] to flatten `Variant { fields… }` into a
/// plain struct so schema discovery sees the fields directly. For
/// internally-tagged enums the implicit tag field is captured but the
/// caller is expected to suppress it by routing through
/// [`SuppressFieldSerializer`] (we use the existing tag-drop on the
/// resulting data.frame instead).
struct TaggedVariantRow<'a, T> {
    tag_column: &'a str,
    tag_value: &'a str,
    inner: &'a T,
    /// `Some(field_name)` for internally-tagged enums — that field will be
    /// suppressed in the emitted struct so it doesn't collide with the
    /// explicit `tag_column`.
    tag_field: Option<String>,
}

impl<T: Serialize> Serialize for TaggedVariantRow<'_, T> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // We don't know the inner struct's field count up front, but
        // serialize_struct accepts a hint — use a reasonable upper bound
        // (the value is advisory).
        use ser::SerializeMap as _;
        // Use a map under the hood: the columnar discoverer treats both
        // serialize_struct and serialize_map as struct-like input. Maps
        // give us the freedom to forward arbitrary inner field counts.
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(self.tag_column, self.tag_value)?;
        // Re-emit the inner row's fields. We funnel them through a
        // serializer that strips the variant wrapper (externally-tagged
        // enums) and skips the implicit tag field (internally-tagged).
        let routed = RoutedInner {
            inner: self.inner,
            suppress_field: self.tag_field.as_deref(),
        };
        // RoutedInner emits its serialize_struct via map.serialize_entry,
        // so we don't need to call serialize_entry per field here.
        routed.serialize_into_map(&mut map)?;
        map.end()
    }
}

/// Internal carrier that emits the inner row's fields one-by-one into a
/// parent serde map.
struct RoutedInner<'a, T> {
    inner: &'a T,
    suppress_field: Option<&'a str>,
}

impl<T: Serialize> RoutedInner<'_, T> {
    fn serialize_into_map<M: ser::SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
        // Drive the inner Serialize through a forwarder that captures each
        // (key, value) and forwards it to the parent map.
        let forwarder = MapForwarder {
            map,
            suppress: self.suppress_field,
        };
        self.inner
            .serialize(VariantStrippingMapForwarder { forwarder })
    }
}

/// Forwarder serializer that re-emits a struct's fields as map entries on
/// a *parent* map serializer, skipping `suppress` if set.
struct MapForwarder<'m, M: ser::SerializeMap> {
    map: &'m mut M,
    suppress: Option<&'m str>,
}

impl<M: ser::SerializeMap> MapForwarder<'_, M> {
    fn emit<V: ?Sized + Serialize>(&mut self, key: &str, value: &V) -> Result<(), M::Error> {
        if Some(key) == self.suppress {
            return Ok(());
        }
        self.map.serialize_entry(key, value)
    }
}

/// Serializer that flattens the inner row (handling enum-variant wrapping)
/// and forwards each (key, value) to a [`MapForwarder`].
struct VariantStrippingMapForwarder<'m, M: ser::SerializeMap> {
    forwarder: MapForwarder<'m, M>,
}

impl<'m, M: ser::SerializeMap> ser::Serializer for VariantStrippingMapForwarder<'m, M> {
    type Ok = ();
    type Error = M::Error;
    type SerializeSeq = ser::Impossible<(), M::Error>;
    type SerializeTuple = ser::Impossible<(), M::Error>;
    type SerializeTupleStruct = ser::Impossible<(), M::Error>;
    type SerializeTupleVariant = ForwardingMapEmitter<'m, M>;
    type SerializeMap = ForwardingMapEmitter<'m, M>;
    type SerializeStruct = ForwardingMapEmitter<'m, M>;
    type SerializeStructVariant = ForwardingMapEmitter<'m, M>;

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(ForwardingMapEmitter {
            forwarder: self.forwarder,
            tuple_idx: 0,
        })
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        v: &T,
    ) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<(), Self::Error> {
        // Unit variants emit no fields — handled by the caller's unit-variant
        // branch when building per-variant DFs. Collated-shape unit variants
        // arrive here and contribute zero columns aside from the tag column.
        Ok(())
    }

    // Primitives + simple values cannot appear at the top of a row (the row
    // is required to be a struct/map). Fail informatively.
    fn serialize_bool(self, _: bool) -> Result<(), Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
    fn serialize_i8(self, _: i8) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_i16(self, _: i16) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_i32(self, _: i32) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_i64(self, _: i64) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u8(self, _: u8) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u16(self, _: u16) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u32(self, _: u32) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_u64(self, _: u64) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_f32(self, _: f32) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_f64(self, _: f64) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_char(self, _: char) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_str(self, _: &str) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<(), Self::Error> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.serialize_bool(false)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Self::Error> {
        Ok(())
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: expected struct or map at top level",
        ))
    }
}

/// Sink that re-emits struct fields / map entries as map entries on the
/// parent serializer. Tuple-variant indices are mapped to `_0`, `_1`, …
struct ForwardingMapEmitter<'m, M: ser::SerializeMap> {
    forwarder: MapForwarder<'m, M>,
    tuple_idx: u32,
}

impl<M: ser::SerializeMap> ser::SerializeStruct for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.forwarder.emit(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeStructVariant for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.forwarder.emit(key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeMap for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _key: &T) -> Result<(), Self::Error> {
        // We don't have a way to extract the key string here without a side
        // channel; the TaggedVariantRow's collated path only feeds derive-
        // generated impls (serialize_struct or serialize_struct_variant),
        // so this map-path is dead for them. Hand-written Serialize impls
        // emitting maps with non-string keys aren't supported.
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: hand-written serialize_map not supported in collated shape",
        ))
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Err(serde::ser::Error::custom(
            "TaggedVariantRow: hand-written serialize_map not supported in collated shape",
        ))
    }

    fn serialize_entry<K: ?Sized + Serialize, V: ?Sized + Serialize>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        // Extract the key string via a sibling extractor.
        let mut ve = ValueExtractor::default();
        let _ = key.serialize(&mut ve);
        let key_str = match ve.value {
            ExtractedValue::Str(s) => s,
            _ => return Ok(()),
        };
        self.forwarder.emit(&key_str, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: ser::SerializeMap> ser::SerializeTupleVariant for ForwardingMapEmitter<'_, M> {
    type Ok = ();
    type Error = M::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = format!("_{}", self.tuple_idx);
        self.tuple_idx += 1;
        self.forwarder.emit(&key, value)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// endregion

// region: map_to_dataframe (closes #700)

/// Serialize a [`BTreeMap`](std::collections::BTreeMap) to an R data.frame
/// with the keys as one column and the value struct's fields as the rest.
///
/// Output column order: `<key_column>` first, then `V`'s flattened serde
/// fields in declaration order. Nested struct flattening, `#[serde(flatten)]`,
/// and `#[serde(skip_serializing_if)]` all work the same way as in
/// [`ColumnarDataFrame::from_rows`].
///
/// `BTreeMap`'s ordered iteration gives a deterministic row order. For
/// [`HashMap`](std::collections::HashMap), see [`hashmap_to_dataframe`].
///
/// # Errors
///
/// - `V` does not serialize as a struct or map.
/// - Underlying column-buffer assembly fails.
///
/// # Example
///
/// ```ignore
/// use std::collections::BTreeMap;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Summary {
///     cmax: f64,
///     tmax: f64,
/// }
///
/// let summary: BTreeMap<i32, Summary> = /* … */;
/// let df = map_to_dataframe(&summary, "subject")?;
/// // Columns: subject, cmax, tmax
/// ```
pub fn map_to_dataframe<K, V>(
    map: &std::collections::BTreeMap<K, V>,
    key_column: &str,
) -> Result<ColumnarDataFrame, RSerdeError>
where
    K: Serialize,
    V: Serialize,
{
    let rows: Vec<MapEntry<'_, K, V>> = map
        .iter()
        .map(|(k, v)| MapEntry {
            key_column,
            key: k,
            value: v,
        })
        .collect();
    ColumnarDataFrame::from_rows(&rows)
}

/// Serialize a [`HashMap`](std::collections::HashMap) to an R data.frame.
///
/// Keys are sorted by their `Ord` impl to produce a deterministic row order.
/// For maps with non-`Ord` keys (or callers happy with insertion-order
/// non-determinism), wrap into a `BTreeMap` first or convert manually.
///
/// Output column order matches [`map_to_dataframe`]: `<key_column>` first,
/// then `V`'s flattened serde fields in declaration order.
///
/// # Errors
///
/// Same as [`map_to_dataframe`].
pub fn hashmap_to_dataframe<K, V>(
    map: &std::collections::HashMap<K, V>,
    key_column: &str,
) -> Result<ColumnarDataFrame, RSerdeError>
where
    K: Serialize + Ord,
    V: Serialize,
{
    // Collect references and sort by key for deterministic output order.
    let mut entries: Vec<(&K, &V)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let rows: Vec<MapEntry<'_, K, V>> = entries
        .into_iter()
        .map(|(k, v)| MapEntry {
            key_column,
            key: k,
            value: v,
        })
        .collect();
    ColumnarDataFrame::from_rows(&rows)
}

/// Wrapper that serializes a single `(K, V)` map entry as a struct/map
/// whose first field is the user's key column and whose remaining fields
/// are flattened from `V`.
struct MapEntry<'a, K, V> {
    key_column: &'a str,
    key: &'a K,
    value: &'a V,
}

impl<K: Serialize, V: Serialize> Serialize for MapEntry<'_, K, V> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use ser::SerializeMap as _;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry(self.key_column, self.key)?;
        // Forward V's fields directly into the map. V must serialize as a
        // struct or map; primitives at the top level fail explicitly.
        let routed = RoutedInner {
            inner: self.value,
            suppress_field: None,
        };
        routed.serialize_into_map(&mut map)?;
        map.end()
    }
}

// endregion

// region: result_to_dataframe (closes #697)

/// Shape selector for [`result_to_dataframe`].
///
/// Configures whether the helper returns a bare data.frame, a split
/// `list(results=, error=)`, or a collated single-data.frame with an
/// `is_error` column and the union of Ok and Err fields.
pub enum ResultShape<S> {
    /// All-Ok input → bare data.frame; otherwise → `list(results=, error=)`.
    ///
    /// `empty_ok_sentinel` is unused when at least one `Ok` is present.
    Auto {
        /// Returned in the `results` slot when *every* row was `Err`.
        empty_ok_sentinel: S,
    },
    /// Single collated data.frame: every row, with an `is_error` LGLSXP
    /// column plus the union of Ok and Err fields. Other-variant fields
    /// land as NA per row.
    Collated,
    /// Always `list(results=, error=)`, even when all rows are `Ok` (in
    /// which case `error` is a zero-row data.frame) or all are `Err` (in
    /// which case `results` is `empty_ok_sentinel`).
    Split {
        /// Returned in the `results` slot when *every* row was `Err`.
        empty_ok_sentinel: S,
    },
}

/// Partition a slice of `Result<T, E>` into Ok and Err data.frames.
///
/// The shape of the return is controlled by [`ResultShape`]. The output is
/// always a [`DataFrameShape`]; convert via [`crate::IntoR`] to get the
/// equivalent R-side SEXP at the `#[miniextendr]` function boundary.
///
/// # Sentinel
///
/// For [`Auto`](ResultShape::Auto) and [`Split`](ResultShape::Split), the
/// `empty_ok_sentinel` field is the value placed in the `results` slot
/// when every row is `Err`. Any [`crate::IntoR`] type works — `NULL`,
/// `FALSE`, `NA`, an empty zero-row data.frame, etc. The sentinel is only
/// allocated when needed.
///
/// # GC discipline
///
/// All intermediate data.frames are protected via
/// [`NamedDataFrameListBuilder`]'s `ProtectScope` while the helper is on
/// the stack; the returned [`DataFrameShape`] owns the inner SEXPs until
/// the caller consumes it.
///
/// # Errors
///
/// - `T` or `E` does not serialize as a struct or map.
/// - Underlying column-buffer assembly fails.
///
/// # Example
///
/// ```ignore
/// # use miniextendr_api::serde::{result_to_dataframe, ResultShape};
/// # use serde::Serialize;
/// # #[derive(Serialize)] struct Obs { id: i32, value: f64 }
/// # #[derive(Serialize)] struct Err { id: i32, reason: String }
/// let rows: Vec<Result<Obs, Err>> = /* … */ vec![];
/// // Default dispatch: bare on all-Ok, list otherwise.
/// let shape = result_to_dataframe(&rows, ResultShape::Auto { empty_ok_sentinel: () })?;
/// ```
pub fn result_to_dataframe<T, E, S>(
    rows: &[Result<T, E>],
    shape: ResultShape<S>,
) -> Result<DataFrameShape, RSerdeError>
where
    T: Serialize,
    E: Serialize,
    S: crate::IntoR,
{
    match shape {
        ResultShape::Collated => {
            // Union-schema across T and E, with is_error column.
            let wrapped: Vec<CollatedResultRow<'_, T, E>> =
                rows.iter().map(CollatedResultRow::from).collect();
            let df = ColumnarDataFrame::from_rows(&wrapped)?;
            Ok(DataFrameShape::Bare(df))
        }
        ResultShape::Auto { empty_ok_sentinel } => {
            let (oks, errs) = partition_results(rows);
            if errs.is_empty() {
                let df = ColumnarDataFrame::from_rows(&oks)?;
                Ok(DataFrameShape::Bare(df))
            } else {
                build_split(oks, errs, empty_ok_sentinel)
            }
        }
        ResultShape::Split { empty_ok_sentinel } => {
            let (oks, errs) = partition_results(rows);
            build_split(oks, errs, empty_ok_sentinel)
        }
    }
}

fn partition_results<T, E>(rows: &[Result<T, E>]) -> (Vec<&T>, Vec<&E>) {
    let mut oks: Vec<&T> = Vec::new();
    let mut errs: Vec<&E> = Vec::new();
    for row in rows {
        match row {
            Ok(t) => oks.push(t),
            Err(e) => errs.push(e),
        }
    }
    (oks, errs)
}

fn build_split<T, E, S>(
    oks: Vec<&T>,
    errs: Vec<&E>,
    empty_ok_sentinel: S,
) -> Result<DataFrameShape, RSerdeError>
where
    T: Serialize,
    E: Serialize,
    S: crate::IntoR,
{
    let error_df = ColumnarDataFrame::from_rows(&errs)?;
    let results = if oks.is_empty() {
        let sentinel = empty_ok_sentinel.into_sexp();
        SplitResults::None(sentinel)
    } else {
        let df = ColumnarDataFrame::from_rows(&oks)?;
        SplitResults::Some(df)
    };
    Ok(DataFrameShape::Split {
        results,
        error: error_df,
    })
}

/// Wrapper that serializes one `Result<T, E>` as a flat struct with the
/// fields of the variant plus a leading `is_error` boolean.
struct CollatedResultRow<'a, T, E> {
    is_error: bool,
    payload: CollatedPayload<'a, T, E>,
}

enum CollatedPayload<'a, T, E> {
    Ok(&'a T),
    Err(&'a E),
}

impl<'a, T, E> From<&'a Result<T, E>> for CollatedResultRow<'a, T, E> {
    fn from(r: &'a Result<T, E>) -> Self {
        match r {
            Ok(t) => CollatedResultRow {
                is_error: false,
                payload: CollatedPayload::Ok(t),
            },
            Err(e) => CollatedResultRow {
                is_error: true,
                payload: CollatedPayload::Err(e),
            },
        }
    }
}

impl<T: Serialize, E: Serialize> Serialize for CollatedResultRow<'_, T, E> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use ser::SerializeMap as _;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("is_error", &self.is_error)?;
        match &self.payload {
            CollatedPayload::Ok(t) => {
                let routed = RoutedInner {
                    inner: *t,
                    suppress_field: None,
                };
                routed.serialize_into_map(&mut map)?;
            }
            CollatedPayload::Err(e) => {
                let routed = RoutedInner {
                    inner: *e,
                    suppress_field: None,
                };
                routed.serialize_into_map(&mut map)?;
            }
        }
        map.end()
    }
}

// endregion

// region: Tests (NamedDataFrameListBuilder structural invariants)

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    /// A new builder has zero length and reports is_empty().
    #[test]
    fn builder_new_is_empty() {
        let b = NamedDataFrameListBuilder::default();
        assert_eq!(b.len(), 0);
        assert!(b.is_empty());
    }

    /// with_capacity reserves space but the builder is still empty.
    #[test]
    fn builder_with_capacity_starts_empty() {
        let b = NamedDataFrameListBuilder::with_capacity(8);
        assert_eq!(b.len(), 0);
        assert!(b.is_empty());
    }

    /// The builder's scope count starts at zero (no protections yet).
    #[test]
    fn builder_scope_count_zero_before_push() {
        let b = NamedDataFrameListBuilder::new();
        assert_eq!(b.scope.count(), 0);
    }

    // region: TypeSpec → ColumnType collapse

    /// Each `TypeSpec` variant maps to the expected `ColumnType`. `Optional`
    /// unwraps to its inner column type — the NA-tolerance hint never
    /// changes the underlying R atomic type.
    #[test]
    fn typespec_collapses_to_column_type() {
        assert_eq!(TypeSpec::Logical.into_column_type(), ColumnType::Logical);
        assert_eq!(TypeSpec::Integer.into_column_type(), ColumnType::Integer);
        assert_eq!(TypeSpec::Real.into_column_type(), ColumnType::Real);
        assert_eq!(
            TypeSpec::Character.into_column_type(),
            ColumnType::Character
        );
        assert_eq!(TypeSpec::Generic.into_column_type(), ColumnType::Generic);
        assert_eq!(
            TypeSpec::Optional(Box::new(TypeSpec::Integer)).into_column_type(),
            ColumnType::Integer
        );
        // Nested Optional collapses transitively.
        assert_eq!(
            TypeSpec::Optional(Box::new(TypeSpec::Optional(Box::new(TypeSpec::Real))))
                .into_column_type(),
            ColumnType::Real
        );
    }

    // endregion

    // region: DataFrameBuilder::with_schema (#693)

    #[derive(Serialize)]
    struct WideRow {
        a: i32,
        b: Option<String>,
    }

    /// `with_schema` populates the schema before any push, allocates the
    /// expected number of column buffers, and seeds `filled` to the matching
    /// length. First push therefore skips discovery — the schema is fixed.
    #[test]
    fn with_schema_allocates_columns_up_front() {
        let b = DataFrameBuilder::<WideRow>::with_schema(
            [
                ("a", TypeSpec::Integer),
                ("b", TypeSpec::Optional(Box::new(TypeSpec::Character))),
            ],
            Some(4),
        );
        let schema = b.schema.as_ref().expect("schema set by with_schema");
        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[0].name, "a");
        assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
        assert_eq!(schema.fields[1].name, "b");
        // Optional(Character) collapses to Character — NA-tolerance is implicit.
        assert_eq!(schema.fields[1].col_type, ColumnType::Character);
        assert_eq!(b.columns.len(), 2);
        assert_eq!(b.filled.len(), 2);
        assert!(!b.grow);
    }

    /// `with_schema` + `Optional(Integer)` + first row's value `None` lands
    /// in the integer buffer as `NA_INTEGER`, not as a logical-NA column
    /// (which is what default discovery would produce when the first row's
    /// `Option<i32>` is `None`).
    ///
    /// Inspects the buffer state directly; doesn't call `finish` (avoids R
    /// allocation in unit tests).
    #[test]
    fn with_schema_optional_first_none_keeps_declared_type() {
        let mut b = DataFrameBuilder::<WideRow>::with_schema(
            [
                ("a", TypeSpec::Integer),
                ("b", TypeSpec::Optional(Box::new(TypeSpec::Character))),
            ],
            None,
        );
        b.push(WideRow { a: 1, b: None })
            .expect("first push should succeed");
        assert_eq!(b.nrow, 1);
        // Column "a" is integer with value 1.
        match &b.columns[0] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![1]),
            _ => panic!("expected Integer buffer"),
        }
        // Column "b" is character with NA (None) — *not* a logical column.
        match &b.columns[1] {
            ColumnBuffer::Character(v) => assert_eq!(v, &vec![None]),
            _ => panic!("expected Character buffer for Optional(Character)"),
        }
    }

    /// Default mode: first push discovers schema; a later push with a new
    /// field is rejected by the strict filler.
    #[test]
    fn default_mode_rejects_new_field_on_later_push() {
        #[derive(Serialize)]
        struct R1 {
            a: i32,
        }
        #[derive(Serialize)]
        struct R2 {
            a: i32,
            b: i32,
        }
        // Push R1 then R2 — but builder is parameterised over a single T.
        // Use a wrapper enum to allow heterogeneous serialization.
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Either {
            One(R1),
            Two(R2),
        }
        let mut b = DataFrameBuilder::<Either>::new(None);
        b.push(Either::One(R1 { a: 1 })).expect("first push");
        let err = b
            .push(Either::Two(R2 { a: 2, b: 99 }))
            .expect_err("strict mode should reject field 'b' not in schema");
        match err {
            RSerdeError::Message(m) => {
                assert!(
                    m.contains("row introduced field"),
                    "unexpected error: {m}"
                );
            }
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    /// `with_schema` mode: an unknown field on push is rejected.
    #[test]
    fn with_schema_rejects_unknown_field() {
        #[derive(Serialize)]
        struct R {
            x: i32,
            extra: i32,
        }
        let mut b =
            DataFrameBuilder::<R>::with_schema([("x", TypeSpec::Integer)], None);
        let err = b
            .push(R { x: 1, extra: 9 })
            .expect_err("'extra' not declared");
        match err {
            RSerdeError::Message(m) => assert!(m.contains("extra"), "unexpected error: {m}"),
            other => panic!("expected Message variant, got {other:?}"),
        }
    }

    // endregion

    // region: DataFrameBuilder::grow_schema (#692)

    /// `grow_schema()` lets a later row introduce a new field. The new
    /// column is back-filled with `self.nrow` NA values so its length
    /// matches the existing rows.
    #[test]
    fn grow_schema_back_fills_na_on_new_field() {
        use std::collections::BTreeMap;
        let mut b = DataFrameBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
        let r1: BTreeMap<String, i32> = [("a".to_string(), 1)].into_iter().collect();
        let r2: BTreeMap<String, i32> = [("a".to_string(), 2), ("b".to_string(), 3)]
            .into_iter()
            .collect();
        b.push(r1).expect("first push");
        assert_eq!(b.nrow, 1);
        assert_eq!(b.columns.len(), 1, "only 'a' so far");
        b.push(r2).expect("growth push");
        assert_eq!(b.nrow, 2);
        assert_eq!(b.columns.len(), 2, "'b' added");
        match &b.columns[0] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![1, 2]),
            _ => panic!("expected Integer for 'a'"),
        }
        // Column 'b' was allocated on row 1; row 0 back-filled to NA_INTEGER.
        match &b.columns[1] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![i32::MIN, 3]),
            _ => panic!("expected Integer for 'b'"),
        }
    }

    /// `grow_schema()` composed with `with_schema(...)`: pre-declared
    /// columns stay typed; new columns from later rows are appended.
    #[test]
    fn grow_schema_combined_with_with_schema() {
        use std::collections::BTreeMap;
        let mut b = DataFrameBuilder::<BTreeMap<String, i32>>::with_schema(
            [("a", TypeSpec::Integer)],
            None,
        )
        .grow_schema();
        let r1: BTreeMap<String, i32> = [("a".to_string(), 10)].into_iter().collect();
        let r2: BTreeMap<String, i32> = [("a".to_string(), 20), ("c".to_string(), 99)]
            .into_iter()
            .collect();
        b.push(r1).expect("push 1");
        b.push(r2).expect("push 2");
        assert_eq!(b.nrow, 2);
        // Original declared column 'a' preserved.
        let schema = b.schema.as_ref().unwrap();
        assert_eq!(schema.fields[0].name, "a");
        assert_eq!(schema.fields[0].col_type, ColumnType::Integer);
        // New column 'c' added after first push that introduced it.
        assert_eq!(schema.fields.len(), 2);
        assert_eq!(schema.fields[1].name, "c");
        assert_eq!(schema.fields[1].col_type, ColumnType::Integer);
        match &b.columns[1] {
            ColumnBuffer::Integer(v) => assert_eq!(v, &vec![i32::MIN, 99]),
            _ => panic!("expected Integer for 'c'"),
        }
    }

    /// `grow_schema()` plus a row that introduces no new fields is a no-op
    /// on schema state — the discovery pass runs but nothing is added.
    #[test]
    fn grow_schema_noop_when_no_new_fields() {
        use std::collections::BTreeMap;
        let mut b = DataFrameBuilder::<BTreeMap<String, i32>>::new(None).grow_schema();
        let r: BTreeMap<String, i32> = [("k".to_string(), 5)].into_iter().collect();
        b.push(r.clone()).unwrap();
        let pre_len = b.schema.as_ref().unwrap().fields.len();
        b.push(r).unwrap();
        let post_len = b.schema.as_ref().unwrap().fields.len();
        assert_eq!(pre_len, post_len);
        assert_eq!(b.nrow, 2);
    }

    // endregion
}

// endregion
