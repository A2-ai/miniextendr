//! Examples and tests for data frame conversion features.

use miniextendr_api::{miniextendr, DataFrameRow, IntoList, List};
use miniextendr_api::convert::{DataFrameRows, IntoDataFrame, ToDataFrame};

// =============================================================================
// Approach 1: Derive Macro with IntoList
// =============================================================================

/// Example row type with scalar fields.
#[derive(Clone, Debug, PartialEq, IntoList, DataFrameRow)]
pub struct Measurement {
    pub time: f64,
    pub value: f64,
    pub sensor_id: String,
}

/// Create a data frame from measurement rows using the derive macro.
///
/// @export
#[miniextendr]
pub fn create_measurements_df() -> ToDataFrame<MeasurementDataFrame> {
    let rows = vec![
        Measurement {
            time: 1.0,
            value: 10.5,
            sensor_id: "sensor_A".into(),
        },
        Measurement {
            time: 2.0,
            value: 20.3,
            sensor_id: "sensor_B".into(),
        },
    ];
    ToDataFrame(Measurement::to_dataframe(rows))
}

// =============================================================================
// Approach 2: DataFrameRows with IntoList
// =============================================================================

#[derive(Clone, IntoList)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Create a data frame using DataFrameRows wrapper.
///
/// @export
#[miniextendr]
pub fn create_points_df() -> DataFrameRows<Point> {
    DataFrameRows::from_rows(vec![
        Point { x: 1.0, y: 2.0 },
        Point { x: 3.0, y: 4.0 },
    ])
}

// =============================================================================
// Approach 3: Manual IntoDataFrame Implementation
// =============================================================================

/// Example with column-oriented data.
#[derive(Clone)]
pub struct TimeSeries {
    pub timestamps: Vec<f64>,
    pub values: Vec<f64>,
}

impl IntoDataFrame for TimeSeries {
    fn into_data_frame(self) -> List {
        let len = self.timestamps.len();
        List::from_pairs(vec![
            ("timestamp", self.timestamps),
            ("value", self.values),
        ])
        .set_class_str(&["data.frame"])
        .set_row_names_int(len)
    }
}

/// Return column-oriented data as data frame.
///
/// @export
#[miniextendr]
pub fn create_timeseries() -> ToDataFrame<TimeSeries> {
    ToDataFrame(TimeSeries {
        timestamps: vec![1.0, 2.0, 3.0],
        values: vec![10.0, 20.0, 30.0],
    })
}

// =============================================================================
// Module Registration
// =============================================================================

use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod dataframe_examples;
    fn create_measurements_df;
    fn create_points_df;
    fn create_timeseries;
}
