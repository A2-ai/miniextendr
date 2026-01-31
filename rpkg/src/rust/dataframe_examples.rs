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

use miniextendr_api::miniextendr_module;

miniextendr_module! {
    mod dataframe_examples;
    fn create_points_df;
    fn create_people_df;
}
