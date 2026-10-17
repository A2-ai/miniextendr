//! Package-qualified S3 methods register when their generic's namespace loads.

use miniextendr_api::dataframe::{BuiltDataFrame, DataFrame};
use miniextendr_api::dots::Dots;
use miniextendr_api::list::NamedList;
use miniextendr_api::{ExternalPtr, SEXP, miniextendr};

/// Columns of an `mx_qualified_rows` list as a data frame.
/// @param x A list of equal-length columns.
/// @param ... Unused.
#[miniextendr(s3(generic = "tibble::as_tibble", class = "mx_qualified_rows"))]
pub fn qualified_rows(x: NamedList, _dots: &Dots) -> BuiltDataFrame {
    x.as_data_frame().unwrap_or_else(|e| panic!("{e}"))
}

/// An explicit export tag also uses the qualified registration.
/// @param x A list of equal-length columns.
/// @param ... Unused.
/// @export
#[miniextendr(s3(generic = "tibble::as_tibble", class = "mx_qualified_export"))]
pub fn qualified_export(x: NamedList, _dots: &Dots) -> BuiltDataFrame {
    x.as_data_frame().unwrap_or_else(|e| panic!("{e}"))
}

/// Pointer class with a lazy S3 method on a Suggests generic.
#[derive(ExternalPtr)]
pub struct QualifiedRows {
    value: i32,
}

/// A row whose conversion is registered lazily with tibble.
#[miniextendr(s3, class = "qualified_rows")]
impl QualifiedRows {
    /// @param value The stored value.
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// Convert the stored value to a one-row data frame.
    /// @export
    #[miniextendr(s3(generic = "tibble::as_tibble"))]
    pub fn as_rows(&self) -> BuiltDataFrame {
        let value = self.value;
        DataFrame::builder(1)
            .column::<i32>("value", move |column, _| column.fill(value))
            .build()
    }
}

/// Trait used to check lazy registration from a trait impl.
#[miniextendr]
pub trait QualifiedSummary {
    fn summary(&self) -> String;
    fn format_value(&self) -> String;
}

/// A trait method on the renamed S3 class.
#[miniextendr(s3)]
impl QualifiedSummary for QualifiedRows {
    #[miniextendr(s3(generic = "tibble::tbl_sum"))]
    fn summary(&self) -> String {
        format!("value={}", self.value)
    }

    #[miniextendr(s3(generic = "format"))]
    fn format_value(&self) -> String {
        format!("qualified={}", self.value)
    }
}

/// Double dispatch keeps both class names and skips the vctrs import.
/// @param x First prototype.
/// @param y Second prototype.
/// @param ... Unused.
#[miniextendr(s3(
    generic = "vctrs::vec_ptype2",
    class = "mx_qualified_vec.mx_qualified_vec"
))]
pub fn qualified_ptype(x: SEXP, y: SEXP, _dots: &Dots) -> SEXP {
    let _ = y;
    x
}
