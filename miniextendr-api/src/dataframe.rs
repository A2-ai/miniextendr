//! Runtime wrapper for R `data.frame` objects.
//!
//! Provides [`DataFrameView`], a typed wrapper around an R `data.frame` SEXP backed
//! by [`NamedList`] for O(1) column access by name. This complements the existing
//! [`DataFrame<T>`](crate::convert::DataFrame) which handles row-to-column transposition
//! for *creating* data frames; `DataFrameView` is for *receiving* and inspecting them.
//!
//! # Example
//!
//! ```ignore
//! use miniextendr_api::dataframe::DataFrameView;
//! use miniextendr_api::typed_list::{TypedListSpec, TypedEntry, TypeSpec};
//!
//! #[miniextendr]
//! fn summarize(df: DataFrameView) -> f64 {
//!     let x: Vec<f64> = df.column("x").unwrap();
//!     x.iter().sum()
//! }
//! ```

use crate::ffi::{self, Rboolean, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, TryFromSexp};
use crate::into_r::IntoR;
use crate::list::{List, NamedList};
use crate::typed_list::{TypedList, TypedListError, TypedListSpec, validate_list};
use std::ffi::CStr;

// region: Error type

/// Error returned when constructing or validating an [`DataFrameView`].
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
        }
    }
}

impl std::error::Error for DataFrameError {}
// endregion

// region: DataFrameView

/// A validated R `data.frame` backed by [`NamedList`] for O(1) column access.
///
/// This type wraps an existing R data.frame SEXP and provides:
/// - O(1) column access by name via the [`NamedList`] index
/// - O(1) column access by position
/// - Schema validation via [`TypedListSpec`]
/// - Conversion back to [`NamedList`] or raw SEXP
///
/// # Construction
///
/// Use [`DataFrameView::from_sexp`] to wrap an existing R data.frame, or
/// [`NamedList::as_data_frame`] / [`List::as_data_frame`] to promote a list.
///
/// # Relationship to `DataFrame<T>`
///
/// [`DataFrame<T>`](crate::convert::DataFrame) is for *creating* data frames from
/// row-oriented Rust data (implements `IntoDataFrame`). `DataFrameView` is for
/// *receiving* data frames from R and inspecting their contents.
pub struct DataFrameView {
    inner: NamedList,
    nrow: usize,
}

impl DataFrameView {
    /// Wrap an existing R `data.frame` SEXP.
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
        // 1. Check it's a list (VECSXP)
        let stype = sexp.type_of();
        if stype != SEXPTYPE::VECSXP {
            return Err(DataFrameError::NotList(format!(
                "expected VECSXP, got {:?}",
                stype
            )));
        }

        // 2. Check it inherits from data.frame
        let inherits = unsafe { ffi::Rf_inherits(sexp, c"data.frame".as_ptr()) } != Rboolean::FALSE;
        if !inherits {
            return Err(DataFrameError::NotDataFrame);
        }

        // 3. Build NamedList (requires names attribute)
        let list = unsafe { List::from_raw(sexp) };
        let inner = NamedList::new(list).ok_or(DataFrameError::NoNames)?;

        // 4. Extract nrow from row.names
        let nrow = extract_nrow(sexp)?;

        Ok(Self { inner, nrow })
    }

    /// Construct an `DataFrameView` from a [`NamedList`] and a pre-validated nrow.
    ///
    /// This is used internally by [`NamedList::as_data_frame`] after validation.
    fn from_named_list(inner: NamedList, nrow: usize) -> Self {
        Self { inner, nrow }
    }

    // region: Column access

    /// Get a column by name, converting each element to type `T`.
    ///
    /// Uses O(1) name lookup via the [`NamedList`] index.
    ///
    /// Returns `None` if the column name is not found or conversion fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let x: Option<Vec<f64>> = df.column("x");
    /// ```
    #[inline]
    pub fn column<T>(&self, name: &str) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        self.inner.get(name)
    }

    /// Get a column by 0-based index, converting to type `T`.
    ///
    /// Returns `None` if the index is out of bounds or conversion fails.
    #[inline]
    pub fn column_index<T>(&self, idx: usize) -> Option<T>
    where
        T: TryFromSexp<Error = SexpError>,
    {
        let idx_isize: isize = idx.try_into().ok()?;
        self.inner.get_index(idx_isize)
    }

    /// Get the raw SEXP for a column by name.
    ///
    /// Returns `None` if the column name is not found.
    #[inline]
    pub fn column_raw(&self, name: &str) -> Option<SEXP> {
        self.inner.get_raw(name)
    }
    // endregion

    // region: Accessors

    /// Number of rows.
    #[inline]
    pub fn nrow(&self) -> usize {
        self.nrow
    }

    /// Number of columns (number of named elements in the list).
    #[inline]
    pub fn ncol(&self) -> usize {
        self.inner.named_len()
    }

    /// Iterate over column names (unordered, from the HashMap index).
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.inner.names()
    }

    /// Check if a column name exists.
    #[inline]
    pub fn contains_column(&self, name: &str) -> bool {
        self.inner.contains(name)
    }
    // endregion

    // region: Validation

    /// Validate the data frame's column types against a [`TypedListSpec`].
    ///
    /// This bridges the [`typed_list`](crate::typed_list!) validation infrastructure
    /// to data frames, allowing schema checks like:
    ///
    /// ```ignore
    /// let spec = TypedListSpec::new(vec![
    ///     TypedEntry::required("x", TypeSpec::Numeric(None)),
    ///     TypedEntry::required("y", TypeSpec::Integer(None)),
    /// ]);
    /// df.validate(&spec)?;
    /// ```
    pub fn validate(&self, spec: &TypedListSpec) -> Result<TypedList, TypedListError> {
        validate_list(self.inner.as_list(), spec)
    }
    // endregion

    // region: Conversions

    /// Convert to the underlying [`NamedList`], consuming the data frame wrapper.
    ///
    /// The SEXP retains its `data.frame` class attribute.
    #[inline]
    pub fn into_named_list(self) -> NamedList {
        self.inner
    }

    /// Get the underlying [`List`].
    #[inline]
    pub fn as_list(&self) -> List {
        self.inner.as_list()
    }

    /// Get the underlying SEXP.
    #[inline]
    pub fn as_sexp(&self) -> SEXP {
        self.inner.as_list().as_sexp()
    }
    // endregion
}
// endregion

// region: TryFromSexp for DataFrameView

impl TryFromSexp for DataFrameView {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        DataFrameView::from_sexp(sexp).map_err(|e| SexpError::InvalidValue(e.to_string()))
    }
}
// endregion

// region: IntoR for DataFrameView — returns the backing SEXP unchanged

impl IntoR for DataFrameView {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    #[inline]
    fn into_sexp(self) -> SEXP {
        self.inner.as_list().as_sexp()
    }
}
// endregion

// region: nrow extraction from row.names

/// Extract `nrow` from R's `row.names` attribute.
///
/// R data frames store row.names in two forms:
/// 1. **Compact integer form**: `c(NA_integer_, -n)` — meaning `1:n` row names
/// 2. **Explicit form**: A character or integer vector of length n
///
/// If no row.names attribute exists, we fall back to the length of the first column.
fn extract_nrow(sexp: SEXP) -> Result<usize, DataFrameError> {
    let row_names = unsafe { ffi::Rf_getAttrib(sexp, ffi::R_RowNamesSymbol) };

    if row_names == unsafe { ffi::R_NilValue } {
        // No row.names — fall back to first column length
        return nrow_from_first_column(sexp);
    }

    let rn_type = row_names.type_of();
    let rn_len = unsafe { ffi::Rf_xlength(row_names) };

    // Compact integer form: c(NA_integer_, -n) where n is the row count
    if rn_type == SEXPTYPE::INTSXP && rn_len == 2 {
        let rn: &[i32] = unsafe { row_names.as_slice() };

        // NA_INTEGER = i32::MIN, second is -nrow
        if rn[0] == i32::MIN && rn[1] < 0 {
            return Ok((-rn[1]) as usize);
        }
        // Not compact form, but an actual 2-element integer vector — nrow = 2
        // (This is unusual but valid)
    }

    // Explicit form: nrow = length of row.names vector
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
    let ncol = unsafe { ffi::Rf_xlength(sexp) };
    if ncol == 0 {
        // 0 columns → 0 rows
        return Ok(0);
    }
    let first_col = unsafe { ffi::VECTOR_ELT(sexp, 0) };
    if first_col == unsafe { ffi::R_NilValue } {
        return Ok(0);
    }
    let len = unsafe { ffi::Rf_xlength(first_col) };
    if let Ok(n) = usize::try_from(len) {
        Ok(n)
    } else {
        Err(DataFrameError::BadRowNames(
            "first column has negative length".to_string(),
        ))
    }
}
// endregion

// region: NamedList → DataFrameView promotion

/// Validate that all columns in a NamedList have equal length, returning the common length.
fn validate_equal_lengths(named: &NamedList) -> Result<usize, DataFrameError> {
    let list = named.as_list();
    let n = list.len();

    if n == 0 {
        return Ok(0);
    }

    // Get the length of the first column
    let first_col = unsafe { ffi::VECTOR_ELT(list.as_sexp(), 0) };
    let expected: usize = unsafe { ffi::Rf_xlength(first_col) }
        .try_into()
        .expect("column length must be non-negative");

    // Check all columns match
    let names_sexp = list.names();
    for i in 1..n {
        let col = unsafe { ffi::VECTOR_ELT(list.as_sexp(), i) };
        let col_len: usize = unsafe { ffi::Rf_xlength(col) }
            .try_into()
            .expect("column length must be non-negative");
        if col_len != expected {
            // Try to get the column name for the error message
            let col_name = if let Some(names) = names_sexp {
                let name_sexp = unsafe { ffi::STRING_ELT(names, i) };
                if name_sexp != unsafe { ffi::R_NaString } {
                    let name_ptr = unsafe { ffi::R_CHAR(name_sexp) };
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
                column: col_name,
                actual: col_len,
            });
        }
    }

    Ok(expected)
}

impl NamedList {
    /// Promote this named list to a [`DataFrameView`].
    ///
    /// Validates that all columns have equal length, then sets the `class`
    /// attribute to `"data.frame"` and adds compact integer `row.names`.
    ///
    /// # Errors
    ///
    /// Returns [`DataFrameError::UnequalLengths`] if columns differ in length.
    pub fn as_data_frame(&self) -> Result<DataFrameView, DataFrameError> {
        let nrow = validate_equal_lengths(self)?;

        // Set class attribute to "data.frame"
        self.as_list().set_class_str(&["data.frame"]);

        // Set compact row.names: c(NA_integer_, -nrow)
        self.as_list().set_row_names_int(nrow);

        // Clone the NamedList (List is Copy, we rebuild the HashMap index)
        let inner = NamedList::new(self.as_list())
            .expect("NamedList already has names; promotion should not lose them");

        Ok(DataFrameView::from_named_list(inner, nrow))
    }
}

impl List {
    /// Promote this named list to a [`DataFrameView`].
    ///
    /// The list must have a `names` attribute and all columns must have equal length.
    ///
    /// # Errors
    ///
    /// Returns [`DataFrameError`] if the list has no names or columns differ in length.
    pub fn as_data_frame(&self) -> Result<DataFrameView, DataFrameError> {
        let named = NamedList::new(*self).ok_or(DataFrameError::NoNames)?;
        named.as_data_frame()
    }
}
// endregion

// region: Debug impl

impl std::fmt::Debug for DataFrameView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataFrameView")
            .field("nrow", &self.nrow)
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
    }
}
// endregion
