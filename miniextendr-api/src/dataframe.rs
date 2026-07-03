//! The unified owned R `data.frame` type and its conversion traits.
//!
//! [`DataFrame`] is **the** data-frame type: a single owned wrapper around a built
//! `data.frame` SEXP that serves every direction —
//!
//! - **build** (Rust → R): [`IntoDataFrame::into_dataframe`] / `into_dataframe_par` (`feature = "rayon"`),
//! - **read** (R → Rust): [`DataFrame::column`] / [`FromDataFrame::from_dataframe`],
//! - **edit** (post-assembly): [`DataFrame::rename`] / [`DataFrame::drop`] / [`DataFrame::select`] / …
//!
//! The trait family mirrors the crate's existing [`IntoR`] /
//! [`TryFromSexp`] pair, specialised to the data-frame SEXP:
//!
//! ```ignore
//! use miniextendr_api::dataframe::{DataFrame, IntoDataFrame, FromDataFrame};
//!
//! // Rust → R
//! let df: DataFrame = rows.into_dataframe()?;          // sequential
//! let df: DataFrame = rows.into_dataframe_par()?;      // parallel (feature = "rayon")
//!
//! // R → Rust
//! let rows: Vec<Row> = Vec::<Row>::from_dataframe(&df)?;
//! ```
//!
//! `DataFrame` implements both `IntoR` and `TryFromSexp`, so it slots into
//! `#[miniextendr]` function codegen with no special-casing — return it directly or accept
//! it as an argument.
//!
//! # One error contract
//!
//! Every conversion failure surfaces as [`DataFrameError`]. The serde column assembler's
//! internal `RSerdeError` is bridged via `From<RSerdeError>`; the parallel R→Rust reader
//! reports through `DataFrameError` rather than a bare `String`.

use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::list::{List, NamedList};
use crate::typed_list::{TypedList, TypedListError, TypedListSpec, validate_list};
use crate::{SEXP, SEXPTYPE, SexpExt};
use std::ffi::CStr;

pub mod group;
pub use group::{GroupKey, GroupedDataFrame, group_rows};

// region: Error type

/// Error returned by any [`DataFrame`] construction, read, or conversion path.
///
/// This is the single data-frame error contract: the row-buffer build path, the serde
/// columnar path, the parallel R→Rust reader, and validation all surface a `DataFrameError`.
#[derive(Debug, Clone)]
pub enum DataFrameError {
    /// The SEXP is not a VECSXP.
    NotList(String),
    /// The object does not inherit from `data.frame`.
    NotDataFrame,
    /// The list has no `names` attribute (columns must be named).
    NoNames,
    /// Could not extract `nrow` from `row.names` attribute.
    BadRowNames(String),
    /// Columns have unequal lengths (when promoting from NamedList).
    UnequalLengths {
        /// First column length encountered.
        expected: usize,
        /// The column name that differs.
        column: String,
        /// The actual length of that column.
        actual: usize,
    },
    /// A row could not be turned into named columns (e.g. unnamed list elements
    /// in a `IntoList`-derived row). Replaces the old `panic!` on this path.
    UnnamedColumns,
    /// [`DataFrame::group_by`] referenced a column name that does not exist.
    NoSuchColumn(String),
    /// [`DataFrame::group_by`] on a column type with no sane grouping
    /// semantics (doubles, list-columns, …).
    UnsupportedGroupColumn {
        /// The offending column name.
        column: String,
        /// Its SEXPTYPE, rendered for the message.
        type_of: String,
    },
    /// A serde-driven schema/serialize/deserialize failure (the bridged
    /// `RSerdeError` text) or another conversion failure carried as a message.
    Conversion(String),
}

impl std::fmt::Display for DataFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataFrameError::NotList(msg) => write!(f, "not a list: {}", msg),
            DataFrameError::NotDataFrame => write!(f, "object does not inherit from data.frame"),
            DataFrameError::NoNames => write!(f, "data.frame has no column names"),
            DataFrameError::BadRowNames(msg) => {
                write!(f, "could not extract nrow from row.names: {}", msg)
            }
            DataFrameError::UnequalLengths {
                expected,
                column,
                actual,
            } => write!(
                f,
                "column {:?} has length {} (expected {})",
                column, actual, expected
            ),
            DataFrameError::UnnamedColumns => {
                write!(f, "cannot create data frame from unnamed list elements")
            }
            DataFrameError::NoSuchColumn(name) => {
                write!(f, "no such column: {:?}", name)
            }
            DataFrameError::UnsupportedGroupColumn { column, type_of } => write!(
                f,
                "cannot group by column {:?} ({}): supported key types are factor, \
                 character, integer, and logical — cut() or factor() the column first",
                column, type_of
            ),
            DataFrameError::Conversion(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DataFrameError {}

#[cfg(feature = "serde")]
impl From<crate::serde::RSerdeError> for DataFrameError {
    fn from(e: crate::serde::RSerdeError) -> Self {
        DataFrameError::Conversion(e.to_string())
    }
}
// endregion

// region: DataFrame — the unified owned data.frame

/// An owned, validated R `data.frame`. **The** data-frame type.
///
/// Wraps a built VECSXP carrying the `data.frame` class + `row.names`. A single coherent
/// type for building (Rust → R), reading (R → Rust), and post-assembly editing — replacing
/// the historical row-buffer / built-SEXP / read-wrapper trio with one coherent type.
///
/// # Building
///
/// Prefer the [`IntoDataFrame`] trait on your data:
///
/// ```ignore
/// let df: DataFrame = rows.into_dataframe()?;
/// ```
///
/// or the closure-fill `DataFrame::builder` for heterogeneous parallel column fill
/// (`feature = "rayon"`).
///
/// # Reading
///
/// Wrap an incoming SEXP with [`DataFrame::from_sexp`] (or accept `DataFrame` directly as a
/// `#[miniextendr]` argument), then pull typed columns with [`DataFrame::column`], or
/// deserialize whole rows with [`FromDataFrame`].
#[derive(Clone, Copy)]
pub struct DataFrame {
    sexp: SEXP,
}

impl DataFrame {
    /// Wrap an already-built `data.frame` SEXP without re-validation.
    ///
    /// Used by the column assemblers, which produce a well-formed `data.frame` by
    /// construction.
    ///
    /// # Safety
    ///
    /// `sexp` must be a VECSXP with the `data.frame` class and consistent `row.names`.
    #[inline]
    pub unsafe fn from_built_sexp(sexp: SEXP) -> Self {
        Self { sexp }
    }

    /// Wrap an existing R `data.frame` SEXP, validating it.
    ///
    /// Validates that the object:
    /// 1. Is a VECSXP (list)
    /// 2. Inherits from `"data.frame"`
    /// 3. Has a `names` attribute
    /// 4. Has extractable `row.names` for nrow
    ///
    /// # Errors
    ///
    /// Returns [`DataFrameError`] if validation fails.
    pub fn from_sexp(sexp: SEXP) -> Result<Self, DataFrameError> {
        let stype = sexp.type_of();
        if stype != SEXPTYPE::VECSXP {
            return Err(DataFrameError::NotList(format!(
                "expected VECSXP, got {:?}",
                stype
            )));
        }
        if !sexp.is_data_frame() {
            return Err(DataFrameError::NotDataFrame);
        }
        // Require a names attribute (columns must be named).
        let list = unsafe { List::from_raw(sexp) };
        NamedList::new(list).ok_or(DataFrameError::NoNames)?;
        // Confirm nrow is extractable.
        extract_nrow(sexp)?;
        Ok(Self { sexp })
    }

    // region: Read API (R → Rust column / row access)

    /// Get a column by name, converting each element to type `T`.
    ///
    /// Returns `None` if the column name is not found or conversion fails.
    #[inline]
    pub fn column<T>(&self, name: &str) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        self.named_list().get(name)
    }

    /// Get a column by 0-based index, converting to type `T`.
    #[inline]
    pub fn column_index<T>(&self, idx: usize) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        let idx_isize: isize = idx.try_into().ok()?;
        self.named_list().get_index(idx_isize)
    }

    /// Get the raw SEXP for a column by name.
    #[inline]
    pub fn column_raw(&self, name: &str) -> Option<SEXP> {
        self.named_list().get_raw(name)
    }

    /// Number of rows.
    #[inline]
    pub fn nrow(&self) -> usize {
        extract_nrow(self.sexp).unwrap_or(0)
    }

    /// Number of columns.
    #[inline]
    pub fn ncol(&self) -> usize {
        self.sexp.len()
    }

    /// Collect column names in column order.
    pub fn names(&self) -> Vec<String> {
        let names_sexp = self.sexp.get_names();
        if names_sexp.is_nil() {
            return Vec::new();
        }
        let n = self.sexp.len() as isize;
        (0..n)
            .map(|i| names_sexp.string_elt_str(i).unwrap_or("").to_string())
            .collect()
    }

    /// Check whether a column name exists.
    #[inline]
    pub fn contains_column(&self, name: &str) -> bool {
        self.named_list().contains(name)
    }

    /// Validate the data frame's column types against a [`TypedListSpec`].
    pub fn validate(&self, spec: &TypedListSpec) -> Result<TypedList, TypedListError> {
        validate_list(unsafe { List::from_raw(self.sexp) }, spec)
    }
    // endregion

    // region: Conversions

    /// Get the underlying [`List`].
    #[inline]
    pub fn as_list(&self) -> List {
        unsafe { List::from_raw(self.sexp) }
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.sexp
    }

    /// Build the `NamedList` index for O(1) column-by-name access.
    #[inline]
    fn named_list(&self) -> NamedList {
        NamedList::new(unsafe { List::from_raw(self.sexp) })
            .expect("DataFrame always carries a names attribute")
    }
    // endregion

    // region: Post-assembly editing (absorbed from the old serde columnar assembler)

    /// Rename a column. No-op if `from` doesn't match any column name.
    pub fn rename(self, from: &str, to: &str) -> Self {
        unsafe {
            // Root `self.sexp` so its `names` attribute survives the
            // `SEXP::charsxp` (Rf_mkCharLenCE) allocation below, which can GC.
            let _guard = crate::OwnedProtect::new(self.sexp);
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
    pub fn strip_prefix(self, prefix: &str) -> Self {
        unsafe {
            // Root `self.sexp` so its `names` attribute survives the
            // `SEXP::charsxp` (Rf_mkCharLenCE) allocation below, which can GC.
            let _guard = crate::OwnedProtect::new(self.sexp);
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

            DataFrame { sexp: *new_list }
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

            DataFrame { sexp: *new_list }
        }
    }

    /// Keep only the rows at the given 0-based indices, in order.
    ///
    /// Subsets every column (each a vector or list-column) to the specified rows
    /// and rebuilds compact integer `row.names`. Used by the enum reader to
    /// densify a flattened sub-frame before recursing into the inner type's reader.
    ///
    /// # PROTECT discipline
    ///
    /// Allocates one new column vector per column — `OwnedProtect`s the output list
    /// across the loop so previously-built column SEXPs survive subsequent allocations.
    pub fn select_rows(&self, idx: &[usize]) -> Self {
        use crate::SexpExt as _;

        unsafe {
            let names_sexp = self.sexp.get_names();
            let ncol = self.sexp.xlength();
            let new_nrow = idx.len();

            let new_list = crate::OwnedProtect::new(SEXP::alloc_list(ncol));
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(ncol));

            for col_j in 0..ncol {
                let src_col = self.sexp.vector_elt(col_j);

                // Gather the requested rows into a new dense column via the shared
                // conversion helper (the row-selecting inverse of `scatter_column`).
                // It returns an unprotected SEXP; we root it into the protected
                // `new_list` immediately below, before any further allocation.
                let new_col: SEXP = crate::convert::gather_column(src_col, idx);

                // Root new_col in the protected output list BEFORE touching its
                // attributes. `gather_column` returns an unprotected SEXP, and
                // `set_class`/`set_levels` (Rf_setAttrib) allocate and can trigger
                // GC. set_vector_elt does not allocate, so this ordering keeps
                // new_col reachable (via new_list) across every allocating call.
                new_list.set_vector_elt(col_j, new_col);
                if names_sexp != SEXP::nil() {
                    new_names.set_string_elt(col_j, names_sexp.string_elt(col_j));
                }

                // Copy column attributes: class (for factor / Date / POSIXct) and
                // levels (for factor columns). Safe now — new_col is rooted in the
                // protected new_list, so GC during set_class/set_levels can't reap it.
                let class_attr = src_col.get_class();
                if class_attr != SEXP::nil() {
                    new_col.set_class(class_attr);
                }
                let levels_attr = src_col.get_levels();
                if levels_attr != SEXP::nil() {
                    new_col.set_levels(levels_attr);
                }
            }

            if names_sexp != SEXP::nil() {
                new_list.set_names(*new_names);
            }

            // Set compact integer row.names (c(NA_integer_, -new_nrow)).
            let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
            let _rn_guard = crate::OwnedProtect::new(row_names);
            rn[0] = i32::MIN;
            rn[1] = -(new_nrow as i32);
            new_list.set_row_names(row_names);
            // Copy the data.frame class attribute.
            new_list.set_class(self.sexp.get_class());

            DataFrame { sexp: *new_list }
        }
    }

    /// Insert a column at index 0 (leftmost), removing any same-named column first.
    pub fn prepend_column(self, name: &str, column: SEXP) -> Self {
        let cleaned = self.drop(name);
        unsafe {
            let names_sexp = cleaned.sexp.get_names();
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

            DataFrame { sexp: *new_list }
        }
    }

    /// Upsert a column: replace the column named `name` if it exists, else append.
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

            DataFrame { sexp: *new_list }
        }
    }
    // endregion

    // region: builder (ex-RDataFrameBuilder, #768)

    /// Start a closure-per-column builder yielding a [`DataFrame`].
    ///
    /// The heterogeneous-column analogue of `with_r_matrix`: each column buffer is R memory
    /// filled by a per-column closure. Available regardless of the `rayon` feature (#1055);
    /// the columns are filled **in parallel** when `rayon` is enabled and **serially**
    /// otherwise — the resulting `data.frame` is identical either way.
    ///
    /// ```ignore
    /// let df = DataFrame::builder(1000)
    ///     .column::<f64>("x", |chunk, off| for (i, v) in chunk.iter_mut().enumerate() { *v = (off + i) as f64 })
    ///     .column_str("label", |i| Some(format!("row{i}")))
    ///     .build();
    /// ```
    #[inline]
    pub fn builder(nrow: usize) -> crate::dataframe_builder::RDataFrameBuilder {
        crate::dataframe_builder::RDataFrameBuilder::new(nrow)
    }
    // endregion
}
// endregion

// region: column-order name helper + attr copy (absorbed from columnar)

/// Read the i-th column name from a STRSXP names vector.
///
/// # Safety
/// `names_sexp` must be a valid STRSXP with at least `i + 1` elements.
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
// endregion

// region: IntoR / TryFromSexp for DataFrame

impl TryFromSexp for DataFrame {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        DataFrame::from_sexp(sexp).map_err(|e| SexpError::InvalidValue(e.to_string()))
    }
}

impl IntoR for DataFrame {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.sexp)
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.sexp
    }
}
// endregion

// region: The conversion trait family (mirrors IntoR / TryFromSexp)

/// Rust data → R `data.frame`. The data-frame analogue of [`IntoR`].
///
/// Implemented by `#[derive(DataFrameRow)]` on a row struct/enum (for `Vec<Row>`), by the
/// blanket impl for any [`ColumnSource`] (`IntoList`-derived rows), and by the serde column
/// path. Call it on your data: `rows.into_dataframe()?`.
///
/// # Parallel fast path
///
/// `into_dataframe_par` (present only with
/// `feature = "rayon"`) produces the **same** [`DataFrame`] as
/// [`into_dataframe`](IntoDataFrame::into_dataframe). It defaults to the sequential path, so
/// every implementor gets a correct `_par` for free; `#[derive(DataFrameRow)]` row types
/// override it with a genuinely parallel column fill (the #777 flattened `(column,row-range)`
/// work-list). The verb is stable across feature sets — dropping `_par` degrades cleanly to
/// the sequential call.
pub trait IntoDataFrame: Sized {
    /// Convert this value into a [`DataFrame`].
    fn into_dataframe(self) -> Result<DataFrame, DataFrameError>;

    /// Parallel column fill (`feature = "rayon"`). Same result as `into_dataframe()`.
    ///
    /// Defaults to the sequential path; overridden by the derive for a real parallel fill.
    #[cfg(feature = "rayon")]
    fn into_dataframe_par(self) -> Result<DataFrame, DataFrameError> {
        self.into_dataframe()
    }
}

/// R `data.frame` → Rust data. The data-frame analogue of
/// [`TryFromSexp`].
///
/// Implemented by `#[derive(DataFrameRow)]` for `Vec<Row>` and by the serde row path.
///
/// # Parallel fast path
///
/// `from_dataframe_par` (`feature = "rayon"`) reads the
/// same rows as [`from_dataframe`](FromDataFrame::from_dataframe), defaulting to the
/// sequential reader; the derive overrides it with the #765 off-main-thread row assembly.
pub trait FromDataFrame: Sized {
    /// Read rows back out of a [`DataFrame`].
    fn from_dataframe(df: &DataFrame) -> Result<Self, DataFrameError>;

    /// Parallel row read (`feature = "rayon"`). Same result as `from_dataframe()`.
    #[cfg(feature = "rayon")]
    fn from_dataframe_par(df: &DataFrame) -> Result<Self, DataFrameError> {
        Self::from_dataframe(df)
    }
}
// endregion

// region: ColumnSource — internal column-assembly engine (ex-public convert::IntoDataFrame)

/// Internal engine that turns a value into a `data.frame`-shaped [`List`].
///
/// This was the historical public `convert::IntoDataFrame` (`-> List`). It is now an internal
/// engine: the public [`IntoDataFrame`] (`-> Result<DataFrame, _>`) and the enum-flatten
/// codegen both delegate to it. Not part of the public verb surface.
#[doc(hidden)]
pub trait ColumnSource {
    /// Convert into a `data.frame`-shaped [`List`] (named columns, `data.frame` class,
    /// `row.names`).
    fn into_column_list(self) -> List;

    /// Extract named column SEXPs from this value.
    ///
    /// Returns `(name, raw SEXP)` per column. The SEXPs are owned by the produced
    /// data-frame SEXP and must be protected by the caller before it is released.
    ///
    /// # Safety
    ///
    /// Calls R API functions; must run on the R main thread.
    fn into_named_columns(self) -> Vec<(String, crate::SEXP)>
    where
        Self: Sized,
    {
        use crate::SexpExt as _;
        let list = self.into_column_list();
        let sexp = list.as_sexp();
        let n = sexp.len();
        let mut out = Vec::with_capacity(n);
        let names_sexp = sexp.get_names();
        let has_names = !names_sexp.is_nil();
        for i in 0..(n as isize) {
            let col_sexp = sexp.vector_elt(i);
            let col_name = if has_names {
                names_sexp.string_elt_str(i).unwrap_or("").to_string()
            } else {
                i.to_string()
            };
            out.push((col_name, col_sexp));
        }
        out
    }

    /// Assemble this source into a validated [`DataFrame`].
    ///
    /// The column engine always sets the `data.frame` class (even for an empty frame); the one
    /// exception is the unnamed-row degradation, which returns a bare unclassed empty list — the
    /// old `panic!("unnamed list elements")` case, now a clean `Err(UnnamedColumns)`.
    ///
    /// This is the bridge from the internal column-assembly engine to the public [`DataFrame`].
    /// We deliberately do **not** offer a blanket `impl<T: ColumnSource> IntoDataFrame for T`:
    /// `#[derive(DataFrameRow)]` emits a *concrete* `impl IntoDataFrame for Vec<Row>` per row
    /// type (and serde uses the `SerdeRows<T>` newtype), so a generic `for T` blanket would
    /// coherence-conflict with every one of those (the compiler treats `Vec<Row>: ColumnSource`
    /// as possibly-true). The derive's `into_dataframe` glue calls this method instead.
    fn into_dataframe(self) -> Result<DataFrame, DataFrameError>
    where
        Self: Sized,
    {
        use crate::SexpExt as _;
        let sexp = self.into_column_list().as_sexp();
        if !sexp.is_data_frame() {
            return Err(DataFrameError::UnnamedColumns);
        }
        Ok(unsafe { DataFrame::from_built_sexp(sexp) })
    }
}
// endregion

// region: DataFrameRowConvert — orphan-rule bridge for `Vec<Row>` conversions

/// Row → DataFrame conversion glue emitted by `#[derive(DataFrameRow)]` on the **row type**.
///
/// The orphan rule forbids the derive from writing `impl IntoDataFrame for Vec<Row>` in the user
/// crate: both `IntoDataFrame` and `Vec` are foreign there, and `Row` only appears *covered*
/// inside `Vec<_>`, so there is no uncovered local type. Instead the derive implements this
/// `#[doc(hidden)]` trait on the local `Row` type (legal — `Row` is local), and `miniextendr_api`
/// carries the blanket [`IntoDataFrame`] / [`FromDataFrame`] impls for `Vec<T: DataFrameRowConvert>`
/// below (legal — `IntoDataFrame` is local *here*). Users still call the public
/// `rows.into_dataframe()?` / `Vec::<Row>::from_dataframe(&df)?` verbs.
#[doc(hidden)]
pub trait DataFrameRowConvert: Sized {
    /// Build a [`DataFrame`] from a row vector (sequential).
    fn rows_into_dataframe(rows: Vec<Self>) -> Result<DataFrame, DataFrameError>;

    /// Build a [`DataFrame`] from a row vector (parallel; defaults to sequential).
    #[cfg(feature = "rayon")]
    fn rows_into_dataframe_par(rows: Vec<Self>) -> Result<DataFrame, DataFrameError> {
        Self::rows_into_dataframe(rows)
    }

    /// Read a row vector out of a [`DataFrame`]. `None` means this row shape has no reader
    /// (scalar, column-expansion, and struct-flatten struct shapes do; tagged enum shapes with
    /// reader-capable fields do too; tagless/map-column/coerced/skip/`as_list` enum shapes and
    /// opaque-map shapes do not); the blanket surfaces that as a clear error.
    fn rows_from_dataframe(_df: &DataFrame) -> Option<Result<Vec<Self>, DataFrameError>> {
        None
    }

    /// Parallel reader (defaults to the sequential reader).
    #[cfg(feature = "rayon")]
    fn rows_from_dataframe_par(df: &DataFrame) -> Option<Result<Vec<Self>, DataFrameError>> {
        Self::rows_from_dataframe(df)
    }
}

/// Error returned by `Vec::<Row>::from_dataframe` when the row shape has no R→Rust reader.
fn no_reader_error() -> DataFrameError {
    DataFrameError::Conversion(
        "this DataFrameRow shape has no R→Rust reader (struct shapes with scalar/expansion/\
         struct-flatten fields do; tagged enum shapes with reader-capable fields do; \
         tagless/map-column/coerced/skip/as_list enum shapes and opaque-map shapes do not)"
            .to_string(),
    )
}

impl<T: DataFrameRowConvert> IntoDataFrame for Vec<T> {
    fn into_dataframe(self) -> Result<DataFrame, DataFrameError> {
        T::rows_into_dataframe(self)
    }

    #[cfg(feature = "rayon")]
    fn into_dataframe_par(self) -> Result<DataFrame, DataFrameError> {
        T::rows_into_dataframe_par(self)
    }
}

impl<T: DataFrameRowConvert> FromDataFrame for Vec<T> {
    fn from_dataframe(df: &DataFrame) -> Result<Self, DataFrameError> {
        // Root the input across the read. A `.Call` caller gets this from R's
        // argument frame, but a Rust caller may hand in a freshly-built,
        // unprotected frame (`into_dataframe` returns an unrooted SEXP wrapper);
        // reader-internal allocations would reclaim it mid-read under
        // `gctorture(TRUE)` (caught by gc_stress_reader_nested_flatten).
        // Mirrors the guard in serde's `dataframe_to_vec` — see
        // reviews/2026-05-29-serde-deserialize-fixture-gctorture-input-protect.md.
        // SAFETY: reader entry runs on the R main thread; `df` wraps a valid SEXP.
        let _input = unsafe { crate::OwnedProtect::new(df.as_sexp()) };
        T::rows_from_dataframe(df).unwrap_or_else(|| Err(no_reader_error()))
    }

    #[cfg(feature = "rayon")]
    fn from_dataframe_par(df: &DataFrame) -> Result<Self, DataFrameError> {
        // SAFETY: as in `from_dataframe` above.
        let _input = unsafe { crate::OwnedProtect::new(df.as_sexp()) };
        T::rows_from_dataframe_par(df).unwrap_or_else(|| Err(no_reader_error()))
    }
}
// endregion

// region: nrow extraction from row.names

/// Extract `nrow` from R's `row.names` attribute.
fn extract_nrow(sexp: SEXP) -> Result<usize, DataFrameError> {
    let row_names = sexp.get_row_names();

    if row_names.is_nil() {
        return nrow_from_first_column(sexp);
    }

    let rn_type = row_names.type_of();
    let rn_len = row_names.xlength();

    if rn_type == SEXPTYPE::INTSXP && rn_len == 2 {
        let rn: &[i32] = unsafe { row_names.as_slice() };
        if rn[0] == i32::MIN && rn[1] < 0 {
            return Ok((-rn[1]) as usize);
        }
    }

    if let Ok(n) = usize::try_from(rn_len) {
        Ok(n)
    } else {
        Err(DataFrameError::BadRowNames(format!(
            "row.names has negative length: {}",
            rn_len
        )))
    }
}

/// Fall back: extract nrow from the length of the first column.
fn nrow_from_first_column(sexp: SEXP) -> Result<usize, DataFrameError> {
    let ncol = sexp.xlength();
    if ncol == 0 {
        return Ok(0);
    }
    let first_col = sexp.vector_elt(0);
    if first_col == SEXP::nil() {
        return Ok(0);
    }
    let len = first_col.xlength();
    if let Ok(n) = usize::try_from(len) {
        Ok(n)
    } else {
        Err(DataFrameError::BadRowNames(
            "first column has negative length".to_string(),
        ))
    }
}
// endregion

// region: NamedList / List → DataFrame promotion

/// Validate that all columns in a NamedList have equal length, returning the common length.
fn validate_equal_lengths(named: &NamedList) -> Result<usize, DataFrameError> {
    let list = named.as_list();
    let n = list.len();

    if n == 0 {
        return Ok(0);
    }

    let first_col = list.as_sexp().vector_elt(0);
    let expected: usize = first_col.len();

    let names_sexp = list.names();
    for i in 1..n {
        let col = list.as_sexp().vector_elt(i);
        let col_len: usize = col.len();
        if col_len != expected {
            let column = if let Some(names) = names_sexp {
                let name_sexp = names.string_elt(i);
                if name_sexp != SEXP::na_string() {
                    let name_ptr = name_sexp.r_char();
                    let name_cstr = unsafe { CStr::from_ptr(name_ptr) };
                    name_cstr.to_str().unwrap_or("<invalid>").to_string()
                } else {
                    format!("column {}", i)
                }
            } else {
                format!("column {}", i)
            };

            return Err(DataFrameError::UnequalLengths {
                expected,
                column,
                actual: col_len,
            });
        }
    }

    Ok(expected)
}

impl NamedList {
    /// Promote this named list to a [`DataFrame`].
    ///
    /// Validates equal column lengths, sets the `data.frame` class, and adds compact integer
    /// `row.names`.
    ///
    /// # Errors
    ///
    /// Returns [`DataFrameError::UnequalLengths`] if columns differ in length.
    pub fn as_data_frame(&self) -> Result<DataFrame, DataFrameError> {
        let nrow = validate_equal_lengths(self)?;
        self.as_list().set_data_frame_class();
        self.as_list().set_row_names_int(nrow);
        Ok(DataFrame {
            sexp: self.as_list().as_sexp(),
        })
    }
}

impl List {
    /// Promote this named list to a [`DataFrame`].
    ///
    /// # Errors
    ///
    /// Returns [`DataFrameError`] if the list has no names or columns differ in length.
    pub fn as_data_frame(&self) -> Result<DataFrame, DataFrameError> {
        let named = NamedList::new(*self).ok_or(DataFrameError::NoNames)?;
        named.as_data_frame()
    }
}
// endregion

// region: NamedDataFrameListBuilder (moved from serde::columnar — no serde dependency)

/// Assemble a named list whose elements are [`DataFrame`]s,
/// without per-result `OwnedProtect` bookkeeping.
///
/// # Why this is distinct from [`DataFrame::builder`]
///
/// [`DataFrame::builder`](crate::dataframe::DataFrame::builder) and the serde
/// `SerdeRowBuilder` both produce a *single* [`DataFrame`]. This builder
/// produces a different shape — a named *list of* data.frames, e.g.
/// `list(results = df, error = df)` — so it deliberately keeps its own name
/// rather than folding into the `DataFrame::builder` vocabulary. Its inputs
/// are [`DataFrame`]s (from any producer: [`IntoDataFrame`], the serde
/// `vec_to_dataframe`, or [`GroupedDataFrame::frames`]); its output is a
/// [`List`](crate::list::List).
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
///     .push("results", oks.into_dataframe()?)
///     .push("error",   errs.into_dataframe()?)
///     .build();
/// ```
pub struct NamedDataFrameListBuilder {
    scope: crate::ProtectScope,
    pairs: Vec<(String, SEXP)>,
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
    pub fn push<S: Into<String>>(mut self, name: S, df: DataFrame) -> Self {
        use crate::IntoR as _;
        let sexp = df.into_sexp();
        // SAFETY: R main thread (constructor invariant); sexp is a valid
        // VECSXP just produced by DataFrame::into_sexp.
        unsafe {
            self.scope.protect_raw(sexp);
        }
        self.pairs.push((name.into(), sexp));
        self
    }

    /// Append an arbitrary SEXP under a name, protected like
    /// [`push`](Self::push). Used by the serde split-shape writer to carry
    /// the caller-supplied empty-`Ok` sentinel, which is deliberately not a
    /// `DataFrame`.
    ///
    /// # Safety
    ///
    /// `sexp` must be a valid R object; R main thread (constructor invariant).
    #[cfg(feature = "serde")]
    #[must_use]
    pub(crate) unsafe fn push_raw<S: Into<String>>(mut self, name: S, sexp: SEXP) -> Self {
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

// region: Debug impl

impl std::fmt::Debug for DataFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataFrame")
            .field("nrow", &self.nrow())
            .field("ncol", &self.ncol())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_frame_error_display() {
        let err = DataFrameError::NotDataFrame;
        assert_eq!(err.to_string(), "object does not inherit from data.frame");

        let err = DataFrameError::NoNames;
        assert_eq!(err.to_string(), "data.frame has no column names");

        let err = DataFrameError::UnequalLengths {
            expected: 3,
            column: "y".to_string(),
            actual: 5,
        };
        assert_eq!(err.to_string(), "column \"y\" has length 5 (expected 3)");

        let err = DataFrameError::UnnamedColumns;
        assert_eq!(
            err.to_string(),
            "cannot create data frame from unnamed list elements"
        );

        let err = DataFrameError::NoSuchColumn("g".to_string());
        assert_eq!(err.to_string(), "no such column: \"g\"");
    }

    // region: NamedDataFrameListBuilder structural invariants

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
    // endregion
}
// endregion
