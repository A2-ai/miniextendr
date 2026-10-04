//! R `data.frame` views, rooted Rust-built handles, and conversion traits.
//!
//! [`DataFrame`] is a cheap `Copy` view over a validated `data.frame` SEXP;
//! [`BuiltDataFrame`] is the owned, GC-rooted handle returned by every
//! Rust-side constructor. Together they serve every direction:
//!
//! - **build** (Rust → R): [`IntoDataFrame::into_dataframe`] / `into_dataframe_par` (`feature = "rayon"`),
//! - **read** (R → Rust): [`DataFrame::column`] / [`FromDataFrame::from_dataframe`],
//! - **edit** (post-assembly): [`DataFrame::rename`] / [`DataFrame::drop`] / [`DataFrame::select`] / …
//!   — the new-frame producers return an owned, GC-rooted [`BuiltDataFrame`] (#1247),
//!   so constructor → edit chains stay rooted at every link.
//!
//! The trait family mirrors the crate's existing [`IntoR`] /
//! [`TryFromSexp`] pair, specialised to the data-frame SEXP:
//!
//! ```ignore
//! use miniextendr_api::dataframe::{DataFrame, IntoDataFrame, FromDataFrame};
//!
//! // Rust → R (returns an owned, GC-rooted `BuiltDataFrame` that `Deref`s to `DataFrame`)
//! let df = rows.into_dataframe()?;          // sequential
//! let df = rows.into_dataframe_par()?;      // parallel (feature = "rayon")
//!
//! // R → Rust
//! let rows: Vec<Row> = Vec::<Row>::from_dataframe(&df)?;
//! ```
//!
//! `DataFrame` implements `TryFromSexp` and `IntoR`, so caller-rooted views slot
//! into `#[miniextendr]` argument and return conversion. `BuiltDataFrame` also
//! implements `IntoR`; return rooted Rust-side constructor results directly.
//!
//! # One error contract
//!
//! Every conversion failure surfaces as [`DataFrameError`]. The serde column assembler's
//! internal `RSerdeError` is bridged via `From<RSerdeError>`; the parallel R→Rust reader
//! reports through `DataFrameError` rather than a bare `String`.

use crate::from_r::{SexpClassError, SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::list::{List, NamedList};
use crate::typed_list::{TypedList, TypedListError, TypedListSpec, validate_list};
use crate::{SEXP, SEXPTYPE, SexpExt};
use std::ffi::CStr;

pub mod group;
pub use group::{GroupDeclaration, GroupKey, GroupedDataFrame, RealKey, group_rows};

// region: Error type

/// Error returned by any [`DataFrame`] construction, read, or conversion path.
///
/// This is the single data-frame error contract: the row-buffer build path, the serde
/// columnar path, the parallel R→Rust reader, and validation all surface a `DataFrameError`.
#[derive(Debug, Clone)]
pub enum DataFrameError {
    /// The SEXP is not a VECSXP; carries the SEXPTYPE it was.
    NotList(SEXPTYPE),
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
    /// [`DataFrame::group_by_multi`] was called with an empty column slice.
    EmptyGroupColumns,
    /// [`DataFrame::group_by`] on a column type with no grouping semantics
    /// (list-columns, complex, raw, bit64 `integer64`, …).
    UnsupportedGroupColumn {
        /// The offending column name.
        column: String,
        /// Its SEXPTYPE (or `"integer64"`), rendered for the message.
        type_of: String,
    },
    /// [`DataFrame::group_by_metadata`] was called on a frame that does not
    /// inherit from `grouped_df` or `rowwise_df`.
    NotGroupedDataFrame,
    /// A frame inherits from `grouped_df` or `rowwise_df`, but its `groups`
    /// attribute is missing or is not a data frame with at least one column.
    /// dplyr treats such a frame as corrupt, not as ungrouped ("The `groups`
    /// attribute must be a data frame.").
    GroupsNotDataFrame,
    /// The last column of a grouped frame's `groups` attribute is not called
    /// `.rows` (dplyr: "The last column of the `groups` attribute must be
    /// called `.rows`.").
    MissingGroupRows,
    /// The `.rows` column of a grouped frame's `groups` attribute is not a
    /// list of integer vectors (dplyr: "The `.rows` column must be list of
    /// one-based integer vectors."). Doubles are rejected, as in dplyr.
    BadGroupRows {
        /// The 0-based group (row of the `groups` frame) whose element is not
        /// an integer vector, or `None` when the `.rows` column itself is not
        /// a list. The message numbers groups from 1, as R does.
        group: Option<usize>,
        /// R's `typeof()` of the offending element (or of the `.rows`
        /// column), rendered for the message.
        type_of: String,
    },
    /// A `.rows` index was `< 1` or `> nrow` of the source frame: the
    /// grouping metadata is stale. The message names the usual cause (the
    /// frame was subset without dplyr loaded) and the fix.
    GroupIndexOutOfRange {
        /// The 0-based group whose `.rows` carried the bad index; the message
        /// numbers it from 1, as R does.
        group: usize,
        /// The offending 1-based index value.
        value: i64,
        /// The source frame's row count (valid indices are `1..=nrow`).
        nrow: usize,
    },
    /// No group's `.rows` holds this row of the source frame: the grouping
    /// metadata is stale (the message names the cause and the fix).
    GroupRowUncovered {
        /// The first uncovered 0-based row; the message numbers it from 1.
        row: usize,
        /// The source frame's row count.
        nrow: usize,
    },
    /// Two `.rows` entries hold the same row of the source frame (in two
    /// groups, or twice in one): the grouping metadata is stale (the message
    /// names the cause and the fix).
    GroupRowDuplicated {
        /// The 0-based row; the message numbers it from 1.
        row: usize,
        /// The 0-based group that held the row first.
        first_group: usize,
        /// The 0-based group that held it again (equal to `first_group` for a
        /// row listed twice in one group).
        second_group: usize,
    },
    /// A serde-driven schema/serialize/deserialize failure (the bridged
    /// `RSerdeError` text) or another conversion failure carried as a message.
    Conversion(String),
}

impl std::fmt::Display for DataFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataFrameError::NotList(actual) => write!(
                f,
                "expected a data frame, got {}",
                crate::typed_list::sexptype_name(*actual)
            ),
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
            DataFrameError::EmptyGroupColumns => {
                write!(f, "group_by_multi requires at least one column")
            }
            DataFrameError::UnsupportedGroupColumn { column, type_of } => write!(
                f,
                "cannot group by column {:?} ({}): supported key types are factor, \
                 character, integer, logical, and double (including Date and POSIXct)",
                column, type_of
            ),
            DataFrameError::NotGroupedDataFrame => write!(
                f,
                "not a grouped_df: the frame does not inherit from grouped_df or rowwise_df; \
                 use group_by/group_by_multi to compute grouping instead"
            ),
            DataFrameError::GroupsNotDataFrame => write!(
                f,
                "not a valid grouped_df or rowwise_df: the `groups` attribute must be a data \
                 frame. {}",
                CORRUPT_GROUPS_HINT
            ),
            DataFrameError::MissingGroupRows => write!(
                f,
                "not a valid grouped_df or rowwise_df: the last column of the `groups` \
                 attribute must be called `.rows`. {}",
                CORRUPT_GROUPS_HINT
            ),
            DataFrameError::BadGroupRows {
                group: None,
                type_of,
            } => write!(
                f,
                "not a valid grouped_df or rowwise_df: the `.rows` column must be a list of \
                 one-based integer vectors, not {}. {}",
                type_of, CORRUPT_GROUPS_HINT
            ),
            DataFrameError::BadGroupRows {
                group: Some(group),
                type_of,
            } => write!(
                f,
                "not a valid grouped_df or rowwise_df: the `.rows` column must be a list of \
                 one-based integer vectors, but the element for group {} is {}. {}",
                group + 1,
                type_of,
                CORRUPT_GROUPS_HINT
            ),
            DataFrameError::GroupIndexOutOfRange { group, value, nrow } => write!(
                f,
                "grouped_df `.rows` index {} for group {} is out of range (the frame has \
                 {} rows; valid indices are 1..={}). {}",
                value,
                group + 1,
                nrow,
                nrow,
                STALE_GROUPS_HINT
            ),
            DataFrameError::GroupRowUncovered { row, nrow } => write!(
                f,
                "grouped_df `.rows` do not cover row {} (the frame has {} rows; each row \
                 belongs to exactly one group). {}",
                row + 1,
                nrow,
                STALE_GROUPS_HINT
            ),
            DataFrameError::GroupRowDuplicated {
                row,
                first_group,
                second_group,
            } if first_group == second_group => write!(
                f,
                "grouped_df `.rows` list row {} twice in group {} (each row belongs to \
                 exactly one group). {}",
                row + 1,
                first_group + 1,
                STALE_GROUPS_HINT
            ),
            DataFrameError::GroupRowDuplicated {
                row,
                first_group,
                second_group,
            } => write!(
                f,
                "grouped_df `.rows` put row {} in both group {} and group {} (each row \
                 belongs to exactly one group). {}",
                row + 1,
                first_group + 1,
                second_group + 1,
                STALE_GROUPS_HINT
            ),
            DataFrameError::Conversion(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DataFrameError {}

/// The cause and the fix that the stale-`.rows` errors
/// ([`DataFrameError::GroupIndexOutOfRange`],
/// [`DataFrameError::GroupRowUncovered`],
/// [`DataFrameError::GroupRowDuplicated`]) append to their message.
const STALE_GROUPS_HINT: &str = "The `groups` metadata is stale: the grouped frame was \
     likely subset or reordered without dplyr loaded, which keeps the old `groups` attribute. \
     Regroup it with dplyr::group_by() (with dplyr loaded, `[` regroups by itself), or read just \
     the grouping declaration (DataFrame::group_declaration) and group the current rows.";

/// The fix that the corrupt-`groups` errors ([`DataFrameError::GroupsNotDataFrame`],
/// [`DataFrameError::MissingGroupRows`], [`DataFrameError::BadGroupRows`])
/// append to their message.
const CORRUPT_GROUPS_HINT: &str = "dplyr's validators reject such a frame too. Strip the \
     grouping with dplyr::ungroup(), then regroup it with dplyr::group_by().";

#[cfg(feature = "serde")]
impl From<crate::serde::RSerdeError> for DataFrameError {
    fn from(e: crate::serde::RSerdeError) -> Self {
        DataFrameError::Conversion(e.to_string())
    }
}
// endregion

// region: DataFrame — cheap, unrooted data.frame view

/// A cheap `Copy` view over a validated R `data.frame`.
///
/// The view carries no GC root. It is sound while an R `.Call` argument frame,
/// a [`ProtectScope`](crate::ProtectScope), or an owning [`BuiltDataFrame`]
/// keeps the SEXP reachable. Rust-side constructors never return this bare
/// view; they return `BuiltDataFrame`.
///
/// # Building
///
/// Prefer the [`IntoDataFrame`] trait on your data (it returns an owned,
/// GC-rooted [`BuiltDataFrame`] that `Deref`s to `DataFrame`):
///
/// ```ignore
/// let df = rows.into_dataframe()?;
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
            return Err(DataFrameError::NotList(stype));
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

    /// Get a column by name, converting to type `T`.
    ///
    /// Returns `None` if the column name is not found or conversion fails.
    ///
    /// `T` may be a vector/collection target (`Vec<f64>`, `Vec<i32>`,
    /// `Vec<String>`, …) — the natural shape for a column — or a scalar for a
    /// length-1 column. The conversion error is discarded (that is what makes
    /// this return `Option`), so `T::Error` is unconstrained; use
    /// [`column_raw`](Self::column_raw) when you need the error.
    #[inline]
    pub fn column<T>(&self, name: &str) -> Option<T>
    where
        T: TryFromSexp,
    {
        self.named_list().get(name)
    }

    /// Get a column by 0-based index, converting to type `T`.
    ///
    /// As with [`column`](Self::column), `T` may be a vector/collection or a
    /// scalar target type; the conversion error is discarded.
    #[inline]
    pub fn column_index<T>(&self, idx: usize) -> Option<T>
    where
        T: TryFromSexp,
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
    ///
    /// In-place edit of the `names` attribute — returns the **same** frame (and
    /// therefore inherits whatever root the input already had), unlike the
    /// new-frame producers ([`drop`](Self::drop) and friends), which return a
    /// rooted [`BuiltDataFrame`]. On a `BuiltDataFrame` receiver the inherent
    /// forward ([`BuiltDataFrame::rename`]) keeps the handle instead.
    ///
    /// Other attributes are untouched, so renaming a grouping column of a
    /// dplyr `grouped_df` leaves its `groups` naming the old column (#1702).
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
    ///
    /// In-place edit of the `names` attribute — returns the **same** frame; see
    /// [`rename`](Self::rename)'s rooting note.
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
    ///
    /// # Frame attributes
    ///
    /// The new frame keeps every attribute of the input except `names`: its
    /// `class`, `row.names`, and anything else on the frame (a `units` map, a
    /// package's metadata), as `dplyr::select()` and tibble's `[` do. This
    /// holds for every column producer ([`select`](Self::select),
    /// [`prepend_column`](Self::prepend_column),
    /// [`with_column`](Self::with_column)).
    ///
    /// A dplyr `grouped_df` / `rowwise_df` keeps its grouping while every
    /// grouping column is still there, unchanged. Removing or replacing one
    /// returns the ungrouped frame instead (no `groups` attribute, no
    /// `grouped_df` / `rowwise_df` class), where dplyr's `[` would regroup by
    /// the grouping columns that remain.
    ///
    /// # Rooting
    ///
    /// Returns an owned, GC-rooted [`BuiltDataFrame`] (#1247): the result frame
    /// is rooted before this method returns, so holding it across R allocations
    /// is safe by construction. The no-op path re-roots the input frame (each
    /// `BuiltDataFrame` releases exactly its own root — `R_PreserveObject`
    /// entries stack, so re-rooting an already-rooted SEXP is sound).
    ///
    /// The input view must be reachable when calling (the usual [`DataFrame`]
    /// view contract: an R `.Call` argument frame, a
    /// [`ProtectScope`](crate::ProtectScope), or a live [`BuiltDataFrame`]).
    #[must_use]
    pub fn drop(self, col: &str) -> BuiltDataFrame {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return BuiltDataFrame::adopt(self);
            }
            let ncol = names_sexp.xlength();
            let drop_idx = (0..ncol).find(|&i| col_name(names_sexp, i) == col);
            let Some(drop_idx) = drop_idx else {
                return BuiltDataFrame::adopt(self);
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
            copy_frame_attrs(self.sexp, *new_list, true);

            // Root the new frame before the OwnedProtect guards drop:
            // `adopt_sexp` preserves first (protecting `new_list` across its own
            // cons-cell allocation), the guards UNPROTECT after — no gap.
            BuiltDataFrame::adopt_sexp(*new_list)
        }
    }

    /// Keep only the named columns, in the order given. Unknown names are skipped.
    ///
    /// Keeps the frame's attributes and, while every grouping column is
    /// selected, its dplyr grouping; see [`drop`](Self::drop)'s frame-attribute
    /// note.
    ///
    /// # Rooting
    ///
    /// Returns an owned, GC-rooted [`BuiltDataFrame`] — see
    /// [`drop`](Self::drop)'s rooting note (#1247).
    #[must_use]
    pub fn select(self, cols: &[&str]) -> BuiltDataFrame {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return BuiltDataFrame::adopt(self);
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
            copy_frame_attrs(self.sexp, *new_list, true);

            // Root before the guards drop (see `drop` for the ordering argument).
            BuiltDataFrame::adopt_sexp(*new_list)
        }
    }

    /// Keep only the rows at the given 0-based indices, in order.
    ///
    /// Subsets every column to the specified rows and rebuilds compact integer
    /// `row.names`. Used by the enum reader to densify a flattened sub-frame
    /// before recursing into the inner type's reader, and by group iteration.
    ///
    /// A row is a row of every column, as in `df[idx, , drop = FALSE]`:
    ///
    /// - A vector or list column is gathered by element, and its element
    ///   `names`, if any, are subset with the values.
    /// - A column with a `dim` (an `I(matrix)` column, or an array) is gathered
    ///   along its first dimension: `dim` becomes `c(length(idx), dim[-1])` and
    ///   the first `dimnames` component (the row names) is subset, the rest kept.
    ///   Arrays of three or more dimensions follow `vctrs::vec_slice()` here;
    ///   base `[.data.frame` subsets those by element.
    /// - A data.frame column (a packed column) is subset the same way,
    ///   recursively.
    ///
    /// Each column keeps all of its other attributes (`Rf_copyMostAttrib`, the
    /// `vctrs::vec_slice()` rule), so a `POSIXct` column keeps its `tzone`, a
    /// `difftime` its `units` and an `I()` column its `AsIs` class.
    ///
    /// # Frame attributes
    ///
    /// The new frame keeps every attribute of the input frame except `names`
    /// (the columns' names, as before) and `row.names` (fresh, `1:length(idx)`):
    /// its `class`, and anything else on the frame, such as a `units` map
    /// describing some columns or a package's metadata. This is what
    /// `dplyr::slice()` (`dplyr_row_slice()` / `dplyr_reconstruct()`),
    /// `vctrs::vec_slice()`, tibble's `[` and base `[.data.frame` keep on a row
    /// subset. A packed data.frame column keeps its own the same way.
    ///
    /// dplyr's grouping does not survive. A `grouped_df` or `rowwise_df` caches
    /// each group's row indices in its `groups` attribute, which no longer
    /// matches the selected rows, so the result has no `groups` attribute and
    /// no `grouped_df` / `rowwise_df` class: it is the ungrouped frame (a
    /// tibble, for a dplyr input). `dplyr::slice()` recomputes the groups
    /// instead. To keep the grouping, read it from the input with
    /// [`group_declaration`](Self::group_declaration) and regroup the result,
    /// in Rust with [`group_by_multi`](Self::group_by_multi) over its
    /// [`vars`](crate::dataframe::GroupDeclaration::vars), or in R with
    /// `dplyr::grouped_df(out, dplyr::group_vars(df))`.
    ///
    /// Any other attribute indexed by row is copied as it is. A package that
    /// keeps one must rebuild it after a row subset, as dplyr's extension guide
    /// (`?dplyr_row_slice`) asks of `dplyr_row_slice()` methods.
    ///
    /// # PROTECT discipline
    ///
    /// Allocates new column vectors (and `dim` / `dimnames` for matrix columns)
    /// while the output list is `OwnedProtect`ed, rooting each new column in it
    /// before the next allocation.
    ///
    /// # Rooting
    ///
    /// Returns an owned, GC-rooted [`BuiltDataFrame`] — see
    /// [`drop`](Self::drop)'s rooting note (#1247).
    #[must_use]
    pub fn select_rows(&self, idx: &[usize]) -> BuiltDataFrame {
        unsafe {
            let out = select_frame_rows(self.sexp, idx);
            // Root before the guard drops (see `drop` for the ordering argument).
            BuiltDataFrame::adopt_sexp(*out)
        }
    }

    /// Insert a column at index 0 (leftmost), removing any same-named column first.
    ///
    /// Keeps the frame's attributes and its dplyr grouping, unless `name`
    /// replaces a grouping column; see [`drop`](Self::drop)'s frame-attribute
    /// note.
    ///
    /// # Rooting
    ///
    /// Returns an owned, GC-rooted [`BuiltDataFrame`] — see
    /// [`drop`](Self::drop)'s rooting note (#1247). `column` must be kept
    /// reachable by the caller (e.g. under an
    /// [`OwnedProtect`](crate::OwnedProtect)) across this call — the new frame
    /// is allocated before `column` is stored into it.
    #[must_use]
    pub fn prepend_column(self, name: &str, column: SEXP) -> BuiltDataFrame {
        // `drop` returns a rooted handle, so the intermediate frame survives the
        // allocations below (pre-#1247 this was an unrooted view — a latent UAF
        // window inside this very method).
        let cleaned = self.drop(name);
        unsafe {
            let names_sexp = cleaned.as_sexp().get_names();
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
                new_list.set_vector_elt(i + 1, cleaned.as_sexp().vector_elt(i));
                new_names.set_string_elt(i + 1, names_sexp.string_elt(i));
            }

            new_list.set_names(*new_names);
            copy_frame_attrs(cleaned.as_sexp(), *new_list, true);

            // Root before the guards (and `cleaned`) drop — see `drop` for the
            // ordering argument.
            BuiltDataFrame::adopt_sexp(*new_list)
        }
    }

    /// Upsert a column: replace the column named `name` if it exists, else append.
    ///
    /// The append path builds a new frame that keeps the input's attributes
    /// and dplyr grouping (see [`drop`](Self::drop)'s frame-attribute note).
    /// The replace path stores `column` into the input frame itself and
    /// returns that frame, attributes untouched, so replacing a grouping
    /// column leaves dplyr's `groups` stale (#1702).
    ///
    /// # Rooting
    ///
    /// Both paths return an owned, GC-rooted [`BuiltDataFrame`] (#1247) — the
    /// in-place replace path re-roots the same frame it received (sound:
    /// `R_PreserveObject` entries stack; this handle releases exactly one).
    /// `column` must be kept reachable by the caller across this call — the
    /// append path allocates the new frame before `column` is stored into it.
    #[must_use]
    pub fn with_column(self, name: &str, column: SEXP) -> BuiltDataFrame {
        unsafe {
            let names_sexp = self.sexp.get_names();
            if names_sexp == SEXP::nil() {
                return BuiltDataFrame::adopt(self);
            }
            let ncol = names_sexp.xlength();
            for i in 0..ncol {
                if col_name(names_sexp, i) == name {
                    // set_vector_elt does not allocate; `column` is reachable
                    // from the (caller-rooted) frame before `adopt` allocates
                    // its cons cell.
                    self.sexp.set_vector_elt(i, column);
                    return BuiltDataFrame::adopt(self);
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
            copy_frame_attrs(self.sexp, *new_list, true);

            // Root before the guards drop (see `drop` for the ordering argument).
            BuiltDataFrame::adopt_sexp(*new_list)
        }
    }
    // endregion

    // region: builder (ex-RDataFrameBuilder, #768)

    /// Start a closure-per-column builder yielding a rooted [`BuiltDataFrame`].
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

// region: column-name helper + frame-attribute copy

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

/// The body of [`DataFrame::select_rows`]: row-subset the data.frame VECSXP
/// `frame` at the 0-based rows `idx`. Shared with packed data.frame columns.
///
/// Returns the new frame still protected; the caller roots it before the guard
/// drops.
///
/// # Safety
///
/// R main thread; `frame` is a data.frame list whose columns all have at least
/// `max(idx) + 1` rows.
unsafe fn select_frame_rows(frame: SEXP, idx: &[usize]) -> crate::OwnedProtect {
    use crate::SexpExt as _;

    unsafe {
        let names_sexp = frame.get_names();
        let ncol = frame.xlength();
        let new_nrow = i32::try_from(idx.len()).expect("row count exceeds i32::MAX");

        let new_list = crate::OwnedProtect::new(SEXP::alloc_list(ncol));
        {
            let new_names = crate::OwnedProtect::new(SEXP::alloc_strsxp(ncol));

            for col_j in 0..ncol {
                let src_col = frame.vector_elt(col_j);
                // Protected on return; rooted in the protected `new_list` before
                // its guard (the top of the protect stack) drops.
                let new_col = select_column_rows(src_col, idx);
                new_list.set_vector_elt(col_j, *new_col);
                drop(new_col);
                if names_sexp != SEXP::nil() {
                    new_names.set_string_elt(col_j, names_sexp.string_elt(col_j));
                }
            }

            if names_sexp != SEXP::nil() {
                new_list.set_names(*new_names);
            }
        }

        // Every frame attribute of the source (`class` included), with dplyr's
        // row-indexed `groups` removed. This copies the source's `row.names`
        // too, so it runs before the fresh ones are set below.
        copy_frame_attrs(frame, *new_list, false);

        // Set compact integer row.names (c(NA_integer_, -new_nrow)).
        let (row_names, rn) = crate::into_r::alloc_r_vector::<i32>(2);
        {
            let _rn_guard = crate::OwnedProtect::new(row_names);
            rn[0] = i32::MIN;
            rn[1] = -new_nrow;
            new_list.set_row_names(row_names);
        }

        new_list
    }
}

/// Row-subset one data.frame column at the 0-based rows `idx` (see
/// [`DataFrame::select_rows`] for the per-kind rules). Returns it protected.
///
/// # Safety
///
/// R main thread; `col` is a valid column with at least `max(idx) + 1` rows.
unsafe fn select_column_rows(col: SEXP, idx: &[usize]) -> crate::OwnedProtect {
    use crate::SexpExt as _;

    unsafe {
        if col.type_of() == SEXPTYPE::VECSXP && col.is_data_frame() {
            return select_frame_rows(col, idx);
        }

        let dim = col.get_dim();
        if dim.type_of() != SEXPTYPE::INTSXP || dim.xlength() == 0 {
            // A plain vector or list column: a row is an element.
            let out = crate::OwnedProtect::new(crate::convert::gather_column(col, idx));
            // Every attribute except names/dim/dimnames, as `vctrs::vec_slice()`
            // does: `class` and `levels` (factor, Date), `tzone` (POSIXct), `units`
            // (difftime), labels, and anything else a package keeps on a column.
            // Base `[` would drop the last kind on an unclassed column; a row
            // subset should not change a column's meaning.
            crate::sys::Rf_copyMostAttrib(col, *out);
            // Element names are per row, so they are subset with the values.
            let col_names = col.get_names();
            if col_names != SEXP::nil() {
                let new_names =
                    crate::OwnedProtect::new(crate::convert::gather_column(col_names, idx));
                out.set_names(*new_names);
            }
            return out;
        }

        // A matrix or array column: a row is one index of the first dimension,
        // taken at every position of the trailing dimensions (column-major).
        let dims: Vec<usize> = dim
            .as_slice::<i32>()
            .iter()
            .map(|&d| usize::try_from(d).expect("dim entries are non-negative"))
            .collect();
        let nrow = dims[0];
        let trailing: usize = dims[1..].iter().product();
        let flat: Vec<usize> = (0..trailing)
            .flat_map(|k| idx.iter().map(move |&row| row + k * nrow))
            .collect();

        let out = crate::OwnedProtect::new(crate::convert::gather_column(col, &flat));
        crate::sys::Rf_copyMostAttrib(col, *out);

        {
            let (new_dim, dim_vals) = crate::into_r::alloc_r_vector::<i32>(dims.len());
            let _dim_guard = crate::OwnedProtect::new(new_dim);
            dim_vals[0] = i32::try_from(idx.len()).expect("row count exceeds i32::MAX");
            dim_vals[1..].copy_from_slice(&dim.as_slice::<i32>()[1..]);
            out.set_dim(new_dim);
        }

        let dimnames = col.get_dimnames();
        if dimnames != SEXP::nil() {
            let n = dimnames.xlength();
            let new_dimnames = crate::OwnedProtect::new(SEXP::alloc_list(n));
            let row_names = dimnames.vector_elt(0);
            if row_names != SEXP::nil() {
                // Unprotected until the `set_vector_elt` right after, which does
                // not allocate.
                new_dimnames.set_vector_elt(0, crate::convert::gather_column(row_names, idx));
            }
            for i in 1..n {
                new_dimnames.set_vector_elt(i, dimnames.vector_elt(i));
            }
            let dimnames_names = dimnames.get_names();
            if dimnames_names != SEXP::nil() {
                new_dimnames.set_names(dimnames_names);
            }
            out.set_dimnames(*new_dimnames);
        }

        out
    }
}

/// Copy the frame attributes of `from` onto the new frame `to`.
///
/// Every attribute except `names` is copied (`Rf_copyMostAttrib`), the rule
/// `vctrs::vec_slice()`, tibble's `[` and `dplyr_reconstruct()` follow:
/// `class`, `row.names` (verbatim; a producer that changes the rows sets its
/// own afterwards) and anything else a package keeps on the frame, such as a
/// `units` map or its metadata.
///
/// dplyr's grouping is the exception. A `grouped_df` / `rowwise_df` caches
/// its groups in a `groups` attribute that names the key columns and holds
/// each group's row indices, so it is copied only while it still describes
/// `to`: `same_rows`, and every grouping variable is the very column it was
/// in `from` (moved, not rebuilt or replaced). Otherwise `to` loses `groups`
/// and the `grouped_df` / `rowwise_df` class and is the ungrouped frame (a
/// tibble, for a dplyr input), never a grouped frame whose groups are stale.
///
/// # Safety
///
/// R main thread; `from` is a data.frame list and `to` a list whose `names`
/// are set, protected by the caller.
unsafe fn copy_frame_attrs(from: SEXP, to: SEXP, same_rows: bool) {
    unsafe {
        crate::sys::Rf_copyMostAttrib(from, to);
        let grouped = from.inherits_class(c"grouped_df") || from.inherits_class(c"rowwise_df");
        if grouped && !(same_rows && grouping_columns_kept(from, to)) {
            ungroup_frame(to);
        }
    }
}

/// Whether every grouping variable of the grouped frame `from` (the columns
/// its `groups` attribute names, `.rows` aside) is a column of `to` that is
/// the same SEXP as in `from`. `false` when `groups` is not a data frame.
///
/// # Safety
///
/// R main thread; `from` and `to` are data.frame lists.
unsafe fn grouping_columns_kept(from: SEXP, to: SEXP) -> bool {
    unsafe {
        let groups = from.get_attr(crate::sys::Rf_install(c"groups".as_ptr()));
        if groups.type_of() != SEXPTYPE::VECSXP || !groups.is_data_frame() {
            return false;
        }
        let vars = groups.get_names();
        if vars == SEXP::nil() {
            return false;
        }
        (0..vars.xlength())
            .map(|i| col_name(vars, i))
            .filter(|&var| var != ".rows")
            .all(
                |var| match (column_named(from, var), column_named(to, var)) {
                    (Some(before), Some(after)) => before == after,
                    _ => false,
                },
            )
    }
}

/// The first column of the data.frame list `frame` named `name`.
///
/// # Safety
///
/// R main thread; `frame` is a list.
unsafe fn column_named(frame: SEXP, name: &str) -> Option<SEXP> {
    unsafe {
        let names = frame.get_names();
        if names == SEXP::nil() {
            return None;
        }
        (0..names.xlength())
            .find(|&i| col_name(names, i) == name)
            .map(|i| frame.vector_elt(i))
    }
}

/// Remove dplyr's grouping from the frame `to`: its `groups` attribute and
/// the `grouped_df` / `rowwise_df` entries of its class.
///
/// # Safety
///
/// R main thread; `to` is a data.frame list protected by the caller.
unsafe fn ungroup_frame(to: SEXP) {
    unsafe {
        to.set_attr(crate::sys::Rf_install(c"groups".as_ptr()), SEXP::nil());
        // Still attached to (and so rooted by) `to` until `set_class` below.
        let class = to.get_class();
        let kept: Vec<isize> = (0..class.xlength())
            .filter(|&i| !matches!(col_name(class, i), "grouped_df" | "rowwise_df"))
            .collect();
        let new_class = crate::OwnedProtect::new(SEXP::alloc_strsxp(
            isize::try_from(kept.len()).expect("class length fits isize"),
        ));
        for (j, &i) in kept.iter().enumerate() {
            new_class.set_string_elt(
                isize::try_from(j).expect("class length fits isize"),
                class.string_elt(i),
            );
        }
        to.set_class(*new_class);
    }
}
// endregion

// region: IntoR / TryFromSexp for DataFrame

impl TryFromSexp for DataFrame {
    type Error = SexpError;

    // As an `Either` arm, a data frame refuses nothing under `no_na`: its
    // cells may be missing, and reading them is the function's own business.
    // (A `DataFrame` parameter keeps the R guard, which refuses any NA cell.)
    #[inline]
    fn __mx_input_has_na(&self, _input: SEXP) -> bool {
        false
    }

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Not a data frame at all (a non-list, or a list without the class) is
        // a class error, worded `got integer` / `got list` by the argument
        // error; a data frame that fails later keeps its own message.
        let not_a_data_frame = |actual| {
            SexpClassError {
                expected: "a data frame",
                actual,
            }
            .into()
        };
        DataFrame::from_sexp(sexp).map_err(|e| match e {
            DataFrameError::NotList(actual) => not_a_data_frame(actual),
            DataFrameError::NotDataFrame => not_a_data_frame(SEXPTYPE::VECSXP),
            other => SexpError::InvalidValue(other.to_string()),
        })
    }
}

/// `NULL` → `None`; any other value converts as a [`DataFrame`] (same view,
/// attributes and `NA` cells, same error) and becomes `Some`.
///
/// Concrete rather than left to the newtype blanket `Option<T>` impl
/// (`newtype.rs`), which `DataFrame` does not match: without this impl,
/// `Option<DataFrame>: TryFromSexp` fails with E0275 (overflow) instead of
/// converting.
impl TryFromSexp for Option<DataFrame> {
    type Error = <DataFrame as TryFromSexp>::Error;
    const NATIVE_BORROW: Option<crate::from_r::NativeBorrow> =
        <DataFrame as TryFromSexp>::NATIVE_BORROW;
    const CHARACTER_ONLY: bool = <DataFrame as TryFromSexp>::CHARACTER_ONLY;
    // `__MX_EXPECTATION` keeps its `None`: the inner text without `NULL or`
    // would be wrong, and `#[miniextendr]` words an `Option<_>` parameter or
    // arm itself (`NULL or <T>`).

    // `NULL` (`None`) is "not given" and passes `no_na`.
    #[inline]
    fn __mx_has_na(&self) -> bool {
        self.as_ref()
            .is_some_and(<DataFrame as TryFromSexp>::__mx_has_na)
    }

    // As an `Either` arm, a given frame is read by `DataFrame` (it keeps its
    // `NA` cells); `NULL` holds no `NA`.
    #[inline]
    fn __mx_input_has_na(&self, input: SEXP) -> bool {
        match self {
            Some(df) => <DataFrame as TryFromSexp>::__mx_input_has_na(df, input),
            None => false,
        }
    }

    #[inline]
    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            <DataFrame as TryFromSexp>::try_from_sexp(sexp).map(Some)
        }
    }

    #[inline]
    unsafe fn try_from_sexp_unchecked(sexp: SEXP) -> Result<Self, Self::Error> {
        if sexp.type_of() == SEXPTYPE::NILSXP {
            Ok(None)
        } else {
            unsafe { <DataFrame as TryFromSexp>::try_from_sexp_unchecked(sexp) }.map(Some)
        }
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

// region: BuiltDataFrame — owned, GC-rooted, Rust-constructed data.frame

/// An owned, GC-rooted `data.frame` **built on the Rust side**.
///
/// [`DataFrame`] is a cheap `Copy` *view* over a bare SEXP with no root of its
/// own — sound only while something else keeps the SEXP reachable (an R `.Call`
/// argument frame roots R-supplied frames; a surrounding
/// [`ProtectScope`](crate::ProtectScope) roots transients). A frame that Rust
/// *constructs* has no such external root, so holding the bare view across any R
/// allocation is a latent use-after-free (issue #1128): the GC reclaims the
/// frame, and a freed-but-intact read passes tests silently until the slot is
/// reused.
///
/// `BuiltDataFrame` is the return type of every Rust-side constructor
/// ([`IntoDataFrame::into_dataframe`], `SerdeRowBuilder::finish`,
/// [`DataFrame::builder`]`().build()`, the serde `*_to_dataframe` verbs,
/// [`NamedList::as_data_frame`]). It roots the frame with `R_PreserveObject` on
/// construction and releases it with `R_ReleaseObject` on drop, so holding it
/// across allocations is safe by construction. It [`Deref`](std::ops::Deref)s to
/// [`DataFrame`], so every read/edit method keeps working unchanged; hand the
/// frame back to R with [`IntoR::into_sexp`] — or just return it from a
/// `#[miniextendr]` fn, which converts it through [`IntoR`] transparently.
///
/// Not `Copy`/`Clone` (each root is released exactly once) and not `Send`
/// (R's precious list is R-main-thread state).
///
/// # Editing (chains stay rooted, #1247)
///
/// The new-frame editing methods ([`DataFrame::drop`], [`DataFrame::select`],
/// [`DataFrame::select_rows`], [`DataFrame::prepend_column`],
/// [`DataFrame::with_column`]) also return `BuiltDataFrame`, and this type
/// carries inherent forwards for the whole editing set, so
/// `rows.into_dataframe()?.drop("x").select(&["y"])` is rooted at every link.
/// (The `drop` forward is load-bearing: without it, method resolution on a
/// `BuiltDataFrame` receiver finds `Drop::drop` — E0040.)
///
/// # Residual
///
/// The cheap view can still be *smuggled* across allocations by hand:
/// dereferencing (`*built`) copies out an unrooted `DataFrame` view that
/// dangles once the handle drops. That is an opt-in footgun, not the
/// constructor/editing path this type makes safe.
pub struct BuiltDataFrame {
    view: DataFrame,
    /// `!Send + !Sync`: rooting/unrooting mutates R's global precious list,
    /// which is R-main-thread state. Construction and drop only ever happen
    /// inside framework code already on the R thread.
    _not_send: std::marker::PhantomData<*mut ()>,
}

impl BuiltDataFrame {
    /// Root an already-built `data.frame` SEXP and take ownership of the root.
    ///
    /// # Safety
    ///
    /// Must run on the R main thread. `sexp` must be a well-formed `data.frame`
    /// VECSXP. Rooting is immediate — no allocation happens between entry and
    /// `R_PreserveObject`, and `R_PreserveObject` protects `sexp` while it
    /// allocates its cons cell — so a freshly-assembled unprotected SEXP is safe
    /// to pass directly.
    #[inline]
    pub unsafe fn adopt_sexp(sexp: SEXP) -> Self {
        // SAFETY: R main thread (caller contract); R_PreserveObject keeps `sexp`
        // reachable across its own allocation.
        unsafe { crate::sys::R_PreserveObject(sexp) };
        Self {
            // SAFETY: caller guarantees `sexp` is a well-formed data.frame VECSXP.
            view: unsafe { DataFrame::from_built_sexp(sexp) },
            _not_send: std::marker::PhantomData,
        }
    }

    /// Root a [`DataFrame`] view, taking ownership of a fresh root.
    ///
    /// Rooting an already-rooted frame is sound: `R_PreserveObject` entries
    /// stack (the precious list is a cons list, duplicates allowed), and each
    /// `BuiltDataFrame` releases exactly the one entry it added.
    ///
    /// # Safety
    ///
    /// Must run on the R main thread. `df` must wrap a well-formed, still-live
    /// `data.frame` VECSXP (reachable at the moment of the call).
    #[inline]
    pub unsafe fn adopt(df: DataFrame) -> Self {
        // SAFETY: as documented on this fn.
        unsafe { Self::adopt_sexp(df.as_sexp()) }
    }

    /// Detach the rooted SEXP, releasing the Rust-side root without running
    /// `Drop`. Shared by the [`IntoR`] hand-off methods.
    ///
    /// Mirrors [`DataFrame::as_sexp`] / [`IntoR::into_sexp`]: the returned SEXP is
    /// unprotected and the caller (typically the `.Call` return path) takes over
    /// protection. Releasing the precious-list root then returning matches that
    /// existing hand-off contract — `R_ReleaseObject` does not allocate, so no GC
    /// can run between the release and the return. `into_sexp` is exposed only
    /// through [`IntoR`] (like [`DataFrame`]), so there is no inherent method to
    /// shadow the trait's.
    #[inline]
    fn release_into_sexp(self) -> SEXP {
        let sexp = self.view.as_sexp();
        // SAFETY: `!Send` guarantees we are on the constructing (R main) thread;
        // release the exact root added in `adopt_sexp`. `mem::forget` prevents
        // `Drop` from releasing it a second time.
        unsafe { crate::sys::R_ReleaseObject(sexp) };
        std::mem::forget(self);
        sexp
    }

    // region: editing forwards (#1247) — keep chains rooted at every link
    //
    // These consume the handle (`self` stays alive until the tail expression
    // has produced — and rooted — the result, so the input frame is reachable
    // throughout the edit; its root is then released with no allocation in
    // between). Plain `Deref` would instead copy out the view and, for `drop`,
    // resolve to `Drop::drop` (E0040) — an inherent method shadows the trait.

    /// Remove a column by name — see [`DataFrame::drop`].
    ///
    /// Inherent forward: consumes this handle and returns the rooted result.
    /// (Also shadows `Drop::drop`, which method resolution would otherwise
    /// select — E0040.)
    #[must_use]
    pub fn drop(self, col: &str) -> BuiltDataFrame {
        (*self).drop(col)
    }

    /// Keep only the named columns — see [`DataFrame::select`].
    #[must_use]
    pub fn select(self, cols: &[&str]) -> BuiltDataFrame {
        (*self).select(cols)
    }

    /// Keep only the given rows — see [`DataFrame::select_rows`].
    #[must_use]
    pub fn select_rows(&self, idx: &[usize]) -> BuiltDataFrame {
        (**self).select_rows(idx)
    }

    /// Insert a column at index 0 — see [`DataFrame::prepend_column`].
    ///
    /// `column` must be kept reachable by the caller across this call.
    #[must_use]
    pub fn prepend_column(self, name: &str, column: SEXP) -> BuiltDataFrame {
        (*self).prepend_column(name, column)
    }

    /// Upsert a column — see [`DataFrame::with_column`].
    ///
    /// `column` must be kept reachable by the caller across this call.
    #[must_use]
    pub fn with_column(self, name: &str, column: SEXP) -> BuiltDataFrame {
        (*self).with_column(name, column)
    }

    /// Rename a column — see [`DataFrame::rename`].
    ///
    /// In-place edit of the same frame: this handle (and its root) carries
    /// straight through.
    #[must_use]
    pub fn rename(self, from: &str, to: &str) -> BuiltDataFrame {
        let _ = (*self).rename(from, to);
        self
    }

    /// Strip a prefix from column names — see [`DataFrame::strip_prefix`].
    ///
    /// In-place edit of the same frame: this handle (and its root) carries
    /// straight through.
    #[must_use]
    pub fn strip_prefix(self, prefix: &str) -> BuiltDataFrame {
        let _ = (*self).strip_prefix(prefix);
        self
    }
    // endregion
}

impl std::ops::Deref for BuiltDataFrame {
    type Target = DataFrame;
    #[inline]
    fn deref(&self) -> &DataFrame {
        &self.view
    }
}

impl Drop for BuiltDataFrame {
    fn drop(&mut self) {
        // SAFETY: `!Send` guarantees drop runs on the R main thread; release the
        // exact root added in `adopt_sexp`. `R_ReleaseObject` does not allocate.
        unsafe { crate::sys::R_ReleaseObject(self.view.as_sexp()) };
    }
}

impl IntoR for BuiltDataFrame {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.release_into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        Ok(self.release_into_sexp())
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.release_into_sexp()
    }
}

impl std::fmt::Debug for BuiltDataFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltDataFrame")
            .field("nrow", &self.nrow())
            .field("ncol", &self.ncol())
            .finish()
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
/// `feature = "rayon"`) produces the **same** [`BuiltDataFrame`] as
/// [`into_dataframe`](IntoDataFrame::into_dataframe). It defaults to the sequential path, so
/// every implementor gets a correct `_par` for free; `#[derive(DataFrameRow)]` row types
/// override it with a genuinely parallel column fill (the #777 flattened `(column,row-range)`
/// work-list). The verb is stable across feature sets — dropping `_par` degrades cleanly to
/// the sequential call.
pub trait IntoDataFrame: Sized {
    /// Convert this value into an owned, GC-rooted [`BuiltDataFrame`].
    fn into_dataframe(self) -> Result<BuiltDataFrame, DataFrameError>;

    /// Parallel column fill (`feature = "rayon"`). Same result as `into_dataframe()`.
    ///
    /// Defaults to the sequential path; overridden by the derive for a real parallel fill.
    #[cfg(feature = "rayon")]
    fn into_dataframe_par(self) -> Result<BuiltDataFrame, DataFrameError> {
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

/// Enum rows → variant-partitioned R `data.frame`s. The split-representation
/// sibling of [`IntoDataFrame`].
///
/// Where [`IntoDataFrame`] produces one **aligned** `data.frame` (field-name
/// union, `NA` fill for columns a variant doesn't carry), this verb partitions
/// the rows by variant: each partition is a `data.frame` with only that
/// variant's own columns (non-optional types — no `NA` fill from sibling
/// variants), keyed by the snake_case variant name.
///
/// Implemented by `#[derive(DataFrameRow)]` on an **enum** row type (for
/// `Vec<Row>`, via the hidden [`DataFrameRowSplit`] bridge below). Struct rows
/// have no variants to partition, so the struct derive does not provide it.
/// Call it on your data: `rows.into_dataframe_split()`.
///
/// # Return shape
///
/// The returned [`List`] is a bare `data.frame` for a single-variant enum and a
/// named R list of `data.frame`s (one per variant) otherwise. Variants absent
/// from the input still appear as 0-row `data.frame`s with that variant's
/// column shape; unit variants produce a 0-column `data.frame` with the
/// correct row count. `List` is [`IntoR`], so a `#[miniextendr]` function
/// returns it directly.
pub trait IntoDataFrameSplit: Sized {
    /// Partition the rows by variant into one `data.frame` per variant.
    fn into_dataframe_split(self) -> List;
}

/// Rust rows ↔ the pure-Rust columnar companion generated by `#[derive(DataFrameRow)]`.
///
/// `#[derive(DataFrameRow)]` builds a companion struct (`<Row>DataFrame`) whose fields are
/// `Vec`-columns — the in-memory, column-oriented form of `Vec<Row>`. This trait is the
/// documented verb surface for that companion: build it from rows (sequentially, or in parallel
/// with `feature = "rayon"`) and, for row-iterable companions, read the rows back.
///
/// It is the pure-Rust complement to [`IntoDataFrame`] / [`FromDataFrame`], which cross the R
/// boundary (`Vec<Row>` ↔ R `data.frame`). In particular `from_rows_par`
/// (`feature = "rayon"`) builds the **pure-Rust companion** in parallel —
/// `IntoDataFrame::into_dataframe_par` cannot, since it yields an R-backed
/// [`BuiltDataFrame`] rather than the companion.
///
/// The derive implements this on the companion type; call `CompanionType::from_rows(rows)`.
/// `Row` is a type parameter (not an associated type) so the derive can implement it for a
/// `pub` companion even when the row type is private — mirroring `From<Vec<Row>>`, and
/// avoiding the private-type-in-public-interface error an associated type would raise.
///
/// # Reading rows back
///
/// [`into_rows`](ColumnarFrame::into_rows) is available for **row-iterable** companions
/// (named-field structs without column expansion), where it defers to the companion's
/// [`IntoIterator`]. Enum and column-expansion companions are write-only in pure Rust; read
/// those back across the R boundary via [`FromDataFrame`] or the derive's one-call
/// `try_from_dataframe`.
pub trait ColumnarFrame<Row>: Sized {
    /// Transpose a row vector into the columnar companion (sequential).
    fn from_rows(rows: Vec<Row>) -> Self;

    /// Parallel transposition (`feature = "rayon"`). Same result as
    /// [`from_rows`](ColumnarFrame::from_rows).
    ///
    /// Defaults to the sequential path; `#[derive(DataFrameRow)]` overrides it with a genuine
    /// parallel column fill for shapes that support one.
    #[cfg(feature = "rayon")]
    fn from_rows_par(rows: Vec<Row>) -> Self {
        Self::from_rows(rows)
    }

    /// Read the rows back out of the companion.
    ///
    /// Available for row-iterable companions (named-field structs without column expansion);
    /// defers to the companion's [`IntoIterator`]. Enum and column-expansion companions are
    /// write-only in pure Rust — read those back via [`FromDataFrame`] / `try_from_dataframe`.
    fn into_rows(self) -> Vec<Row>
    where
        Self: IntoIterator<Item = Row>,
    {
        self.into_iter().collect()
    }
}
// endregion

// region: ColumnSource — internal column-assembly engine (ex-public convert::IntoDataFrame)

/// Internal engine that turns a value into a `data.frame`-shaped [`List`].
///
/// This was the historical public `convert::IntoDataFrame` (`-> List`). It is now an internal
/// engine: the public [`IntoDataFrame`] (`-> Result<BuiltDataFrame, _>`) and the enum-flatten
/// codegen both delegate to it. Not part of the public verb surface.
#[doc(hidden)]
pub trait ColumnSource {
    /// Convert into a `data.frame`-shaped [`List`] (named columns, `data.frame` class,
    /// `row.names`).
    fn into_column_list(self) -> List;

    /// Extract named column SEXPs from this value.
    ///
    /// Returns `(name, raw SEXP)` per column. The assembled column list is
    /// protected in `scope` before anything is read from it, so every returned
    /// SEXP stays reachable (as an element of that list) for as long as `scope`
    /// lives, across any number of later allocations (#1748).
    ///
    /// # Safety
    ///
    /// Must run on the R main thread, and `scope` must be the innermost live
    /// [`ProtectScope`](crate::gc_protect::ProtectScope): its `Drop` unprotects
    /// from the top of R's protect stack.
    unsafe fn into_named_columns(
        self,
        scope: &crate::gc_protect::ProtectScope,
    ) -> Vec<(String, crate::SEXP)>
    where
        Self: Sized,
    {
        use crate::SexpExt as _;
        // SAFETY: R main thread and innermost scope (caller contract). Nothing
        // allocates between building the list and protecting it.
        let sexp = unsafe { scope.protect_raw(self.into_column_list().as_sexp()) };
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

// region: rayon-gating shim for #[derive(DataFrameRow)] parallel methods

/// Emit the wrapped items only when `miniextendr-api` itself was built with the
/// `rayon` feature.
///
/// `#[derive(DataFrameRow)]` uses this to gate its parallel `*_par` methods on
/// **this** crate's `rayon` feature instead of stamping a raw
/// `#[cfg(feature = "rayon")]` into the *consumer* crate. A `cfg` inside a
/// derive is evaluated against the destination crate, where `rayon` is usually
/// not a declared feature — so every downstream package that derives
/// `DataFrameRow` without a `rayon` feature of its own trips the
/// `unexpected_cfgs` lint (#1117). Routing through this macro moves the `#[cfg]`
/// decision into `miniextendr-api`, whose feature set always declares `rayon`,
/// so the consumer crate never sees the attribute. The parallel path now
/// activates purely on the API crate's `rayon` feature, independent of what the
/// consumer names its own.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "rayon")]
macro_rules! __dataframe_row_when_rayon {
    ($($item:tt)*) => { $($item)* };
}

/// No-rayon build: the wrapped parallel methods vanish entirely. The trait's
/// `*_par` methods are themselves `#[cfg(feature = "rayon")]`, so there is
/// nothing to override.
#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "rayon"))]
macro_rules! __dataframe_row_when_rayon {
    ($($item:tt)*) => {};
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
    fn into_dataframe(self) -> Result<BuiltDataFrame, DataFrameError> {
        // `rows_into_dataframe` returns the freshly-built frame as a bare view;
        // root it immediately (no allocation between the `?` and `adopt`) so the
        // returned handle owns its GC root.
        // SAFETY: builds SEXPs → runs on the R main thread.
        Ok(unsafe { BuiltDataFrame::adopt(T::rows_into_dataframe(self)?) })
    }

    #[cfg(feature = "rayon")]
    fn into_dataframe_par(self) -> Result<BuiltDataFrame, DataFrameError> {
        // SAFETY: as in `into_dataframe` above.
        Ok(unsafe { BuiltDataFrame::adopt(T::rows_into_dataframe_par(self)?) })
    }
}

impl<T: DataFrameRowConvert> FromDataFrame for Vec<T> {
    fn from_dataframe(df: &DataFrame) -> Result<Self, DataFrameError> {
        // No input root here: an R-supplied frame is rooted by R's `.Call`
        // argument frame, and a Rust-built frame now reaches the reader as a
        // borrowed `BuiltDataFrame` (rooted for the borrow) — the handle
        // supersedes the old `OwnedProtect` guard (#1128). The reader's own
        // sub-frame allocations stay protected by their internal guards.
        T::rows_from_dataframe(df).unwrap_or_else(|| Err(no_reader_error()))
    }

    #[cfg(feature = "rayon")]
    fn from_dataframe_par(df: &DataFrame) -> Result<Self, DataFrameError> {
        T::rows_from_dataframe_par(df).unwrap_or_else(|| Err(no_reader_error()))
    }
}

/// Row → split-representation glue emitted by `#[derive(DataFrameRow)]` on **enum** row types.
///
/// Same orphan-rule bridge as [`DataFrameRowConvert`]: the derive cannot write
/// `impl IntoDataFrameSplit for Vec<Row>` in the user crate (`Row` is covered by `Vec<_>`), so
/// it implements this `#[doc(hidden)]` trait on the local enum type and `miniextendr_api`
/// carries the blanket [`IntoDataFrameSplit`] impl for `Vec<T: DataFrameRowSplit>` below.
///
/// Deliberately a **separate** bridge from [`DataFrameRowConvert`]: the split representation
/// only exists for enums, and a method here (even defaulted) would grow the verb onto every
/// struct row type. Users call the public `rows.into_dataframe_split()` verb.
#[doc(hidden)]
pub trait DataFrameRowSplit: Sized {
    /// Partition a row vector by variant into one `data.frame` per variant.
    fn rows_into_dataframe_split(rows: Vec<Self>) -> List;
}

impl<T: DataFrameRowSplit> IntoDataFrameSplit for Vec<T> {
    fn into_dataframe_split(self) -> List {
        T::rows_into_dataframe_split(self)
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
    pub fn as_data_frame(&self) -> Result<BuiltDataFrame, DataFrameError> {
        let nrow = validate_equal_lengths(self)?;
        self.as_list().set_data_frame_class();
        self.as_list().set_row_names_int(nrow);
        // SAFETY: R main thread; the list is now a well-formed data.frame VECSXP.
        Ok(unsafe { BuiltDataFrame::adopt_sexp(self.as_list().as_sexp()) })
    }
}

impl List {
    /// Promote this named list to a [`DataFrame`].
    ///
    /// # Errors
    ///
    /// Returns [`DataFrameError`] if the list has no names or columns differ in length.
    pub fn as_data_frame(&self) -> Result<BuiltDataFrame, DataFrameError> {
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
/// `SerdeRowBuilder` both produce a single rooted [`BuiltDataFrame`]. This builder
/// produces a different shape — a named *list of* data.frames, e.g.
/// `list(results = df, error = df)` — so it deliberately keeps its own name
/// rather than folding into the `DataFrame::builder` vocabulary. Its inputs
/// are [`DataFrame`]s (from any producer: [`IntoDataFrame`], the serde
/// `vec_to_dataframe`, or [`GroupedDataFrame::frames`]); its output is a
/// [`List`].
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
///     .push("results", *oks.into_dataframe()?)   // deref the BuiltDataFrame to its view
///     .push("error",   *errs.into_dataframe()?)
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

    /// Consume the builder and return the assembled named [`List`].
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
