//! Borrowed rows → R `data.frame`: `IntoDataFrame` for `&[Row]` and `&Vec<Row>`.
//!
//! `#[derive(DataFrameRow)]` implements [`DataFrameRowBorrow`] on the row type
//! next to [`DataFrameRowConvert`](super::DataFrameRowConvert). The generated
//! fill walks the borrowed rows once into locals that have the companion's own
//! column types, then runs the same column assembly code as the owned
//! conversion, so both produce the same frame. `String` and `Option<String>`
//! columns are written from `&str` (no `String` is cloned); every other value
//! is cloned once into its column.

use super::{BuiltDataFrame, DataFrame, DataFrameError, IntoDataFrame};
use crate::SexpExt as _;
use crate::list::List;

// region: DataFrameRowBorrow — the derive's borrowed fill

/// Borrowed rows → `data.frame` column list, emitted by `#[derive(DataFrameRow)]`
/// on the **row type**.
///
/// The same orphan-rule bridge as [`DataFrameRowConvert`](super::DataFrameRowConvert):
/// the derive implements this `#[doc(hidden)]` trait on the local row type, and
/// `miniextendr_api` carries the blanket [`IntoDataFrame`] impls for `&[T]` and
/// `&Vec<T>` below. Users call `rows.as_slice().into_dataframe()?` or
/// `(&rows).into_dataframe()?`.
///
/// The derive implements it only where the fill can run: every value the fill
/// clones has a `Clone` type, each bound behind a higher-ranked binder so the
/// impl simply does not apply when one fails. A row with a non-`Clone` field
/// therefore still derives, and converts as an owned `Vec<Row>`; asking for
/// its borrowed conversion names the field type that is not `Clone`. The
/// message below is for a row type that does not derive `DataFrameRow`.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a `#[derive(DataFrameRow)]` row, so borrowed rows of it \
               have no data.frame conversion",
    label = "`&[{Self}]` / `&Vec<{Self}>` convert only for `DataFrameRow` rows",
    note = "derive `DataFrameRow` on `{Self}`"
)]
pub trait DataFrameRowBorrow: Sized {
    /// Build the `data.frame`-shaped column list of `rows` without consuming them.
    fn rows_ref_into_column_list(rows: &[Self]) -> List;
}

/// Root a freshly assembled column list as a [`BuiltDataFrame`], refusing the
/// unclassed list the column engine returns for unnamed columns.
fn adopt_column_list(list: List) -> Result<BuiltDataFrame, DataFrameError> {
    let sexp = list.as_sexp();
    if !sexp.is_data_frame() {
        return Err(DataFrameError::UnnamedColumns);
    }
    // SAFETY: the column engine built `sexp` on the R main thread just now and
    // nothing allocated since; `adopt` roots it before anything else runs.
    Ok(unsafe { BuiltDataFrame::adopt(DataFrame::from_built_sexp(sexp)) })
}

/// Borrowed rows → R `data.frame`, for a `#[derive(DataFrameRow)]` row type.
///
/// Builds the same frame as the owned `rows.to_vec().into_dataframe()` without
/// the intermediate `Vec<Row>`: `String` and `Option<String>` columns are
/// written straight from `&str`, and every other value is cloned once into its
/// column (so those field types need `Clone`; the row type does not).
///
/// From a `&self` method, borrow the rows explicitly:
/// `self.rows.as_slice().into_dataframe()` or `(&self.rows).into_dataframe()`.
/// A bare `self.rows.into_dataframe()` resolves to the owned `Vec<Row>` impl
/// and cannot move out of `&self`.
///
/// `into_dataframe_par` (`feature = "rayon"`) on borrowed rows is the same
/// sequential fill.
impl<T: DataFrameRowBorrow> IntoDataFrame for &[T] {
    fn into_dataframe(self) -> Result<BuiltDataFrame, DataFrameError> {
        adopt_column_list(T::rows_ref_into_column_list(self))
    }
}

/// `&Vec<Row>` converts exactly as the borrowed slice `&[Row]` does.
///
/// This impl is what makes `(&self.rows).into_dataframe()` work: without it,
/// method lookup would dereference to the owned `Vec<Row>` impl. Where this
/// impl does not apply (a field type that is not `Clone`), lookup still falls
/// through to that owned impl, so the error is "cannot move out of a shared
/// reference" (E0507). `self.rows.as_slice().into_dataframe()` or
/// `IntoDataFrame::into_dataframe(&self.rows)` names the field type instead.
impl<T: DataFrameRowBorrow> IntoDataFrame for &Vec<T> {
    fn into_dataframe(self) -> Result<BuiltDataFrame, DataFrameError> {
        self.as_slice().into_dataframe()
    }
}
// endregion

// region: StringColumn — the by-reference string columns

/// A `String` column the borrowed fill writes from `&str`.
///
/// Implemented only for `std::string::String`. The derive recognises a
/// `String` column by name and bounds the borrowed impl on this trait, so a
/// user type that happens to be called `String` never takes the `&str` path
/// (its borrowed impl does not apply instead).
#[doc(hidden)]
pub trait StringColumn {
    /// The string a column cell is written from.
    fn __mx_column_str(&self) -> &str;
}

impl StringColumn for String {
    #[inline(always)]
    fn __mx_column_str(&self) -> &str {
        self.as_str()
    }
}
// endregion
