//! Examples and tests for data frame conversion features.

use miniextendr_api::convert::ToDataFrame;
use miniextendr_api::{DataFrameRow, IntoList, miniextendr};

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
    let rows = vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
    ToDataFrame(Point::to_dataframe(rows))
}

// Test with just two different types first
#[derive(Clone, Debug, DataFrameRow)]
#[allow(dead_code)]
pub struct SimplePerson {
    pub name: String,
    pub age: i32,
}

impl ::miniextendr_api::list::IntoList for SimplePerson {
    fn into_list(self) -> ::miniextendr_api::List {
        use ::miniextendr_api::IntoR;
        ::miniextendr_api::List::from_raw_pairs(vec![
            ("name", self.name.into_sexp()),
            ("age", self.age.into_sexp()),
        ])
    }
}

// Test with heterogeneous types (different types in different fields)
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct Person {
    pub name: String,
    pub age: i32,
    pub height: f64,
    pub is_student: bool,
}

/// Create a data frame with heterogeneous types.
///
/// @export
#[miniextendr]
pub fn create_people_df() -> ToDataFrame<PersonDataFrame> {
    let rows = vec![
        Person {
            name: "Alice".to_string(),
            age: 25,
            height: 165.5,
            is_student: true,
        },
        Person {
            name: "Bob".to_string(),
            age: 30,
            height: 180.0,
            is_student: false,
        },
        Person {
            name: "Charlie".to_string(),
            age: 28,
            height: 175.2,
            is_student: true,
        },
    ];
    ToDataFrame(Person::to_dataframe(rows))
}

// ── Enum align examples ──────────────────────────────────────────────────────

/// Enum with align: different event types become rows with NA fill.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum EventRow {
    Click { id: i64, x: f64, y: f64 },
    Impression { id: i64, slot: String },
    Error { id: i64, code: i32, message: String },
}

/// Create a data frame from an aligned enum.
///
/// @export
#[miniextendr]
pub fn create_events_df() -> ToDataFrame<EventRowDataFrame> {
    let rows = vec![
        EventRow::Click {
            id: 1,
            x: 1.5,
            y: 2.5,
        },
        EventRow::Impression {
            id: 2,
            slot: "top_banner".to_string(),
        },
        EventRow::Error {
            id: 3,
            code: 404,
            message: "not found".to_string(),
        },
    ];
    ToDataFrame(EventRow::to_dataframe(rows))
}

/// Enum align without tag column.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align)]
pub enum ShapeRow {
    Circle { radius: f64, area: f64 },
    Rect { width: f64, height: f64, area: f64 },
}

/// Create a data frame from shapes (no tag column).
///
/// @export
#[miniextendr]
pub fn create_shapes_df() -> ToDataFrame<ShapeRowDataFrame> {
    let rows = vec![
        ShapeRow::Circle {
            radius: 5.0,
            area: 78.54,
        },
        ShapeRow::Rect {
            width: 3.0,
            height: 4.0,
            area: 12.0,
        },
        ShapeRow::Circle {
            radius: 1.0,
            area: std::f64::consts::PI,
        },
    ];
    ToDataFrame(ShapeRow::to_dataframe(rows))
}

// ── Collection expansion examples ────────────────────────────────────────────

/// Struct with array expansion: [f64; 3] → coords_1, coords_2, coords_3.
#[derive(Clone, Debug, DataFrameRow)]
pub struct PointExpanded {
    pub label: String,
    pub coords: [f64; 3],
}

/// Create a data frame with expanded array columns.
///
/// @export
#[miniextendr]
pub fn create_expanded_points_df() -> ToDataFrame<PointExpandedDataFrame> {
    let rows = vec![
        PointExpanded {
            label: "A".to_string(),
            coords: [1.0, 2.0, 3.0],
        },
        PointExpanded {
            label: "B".to_string(),
            coords: [4.0, 5.0, 6.0],
        },
    ];
    ToDataFrame(PointExpanded::to_dataframe(rows))
}

/// Struct with skip, rename, and pinned Vec expansion.
#[derive(Clone, Debug, DataFrameRow)]
pub struct ScoredItem {
    #[dataframe(rename = "item")]
    pub name: String,
    #[dataframe(skip)]
    pub _internal_id: u64,
    #[dataframe(width = 3)]
    pub scores: Vec<f64>,
}

/// Create a data frame with skip, rename, and Vec expansion.
///
/// @export
#[miniextendr]
pub fn create_scored_items_df() -> ToDataFrame<ScoredItemDataFrame> {
    let rows = vec![
        ScoredItem {
            name: "alpha".to_string(),
            _internal_id: 1,
            scores: vec![10.0, 20.0, 30.0],
        },
        ScoredItem {
            name: "beta".to_string(),
            _internal_id: 2,
            scores: vec![40.0],
        },
        ScoredItem {
            name: "gamma".to_string(),
            _internal_id: 3,
            scores: vec![],
        },
    ];
    ToDataFrame(ScoredItem::to_dataframe(rows))
}

/// Enum with array expansion in variants.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_type")]
pub enum SensorReading {
    Xyz {
        sensor_id: i32,
        values: [f64; 3],
    },
    Single {
        sensor_id: i32,
        reading: f64,
    },
}

/// Create a data frame from enum with array expansion.
///
/// @export
#[miniextendr]
pub fn create_sensor_readings_df() -> ToDataFrame<SensorReadingDataFrame> {
    let rows = vec![
        SensorReading::Xyz {
            sensor_id: 1,
            values: [1.0, 2.0, 3.0],
        },
        SensorReading::Single {
            sensor_id: 2,
            reading: 42.0,
        },
        SensorReading::Xyz {
            sensor_id: 3,
            values: [7.0, 8.0, 9.0],
        },
    ];
    ToDataFrame(SensorReading::to_dataframe(rows))
}

use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod dataframe_examples;
    fn create_points_df;
    fn create_people_df;
    fn create_events_df;
    fn create_shapes_df;
    fn create_expanded_points_df;
    fn create_scored_items_df;
    fn create_sensor_readings_df;
}
