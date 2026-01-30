//! Examples and tests for data frame conversion features.

use miniextendr_api::{miniextendr, DataFrameRow, IntoList};
use miniextendr_api::convert::{DataFrame, ToDataFrame};

// Test with homogeneous types first
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Create a data frame using derive macro.
///
/// @export
#[miniextendr]
pub fn create_points_df() -> ToDataFrame<PointDataFrame> {
    let rows = vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 3.0, y: 4.0 },
    ];
    ToDataFrame(Point::to_dataframe(rows))
}

use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod dataframe_examples;
    fn create_points_df;
}
