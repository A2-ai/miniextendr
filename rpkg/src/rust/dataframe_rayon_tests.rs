use miniextendr_api::convert::ToDataFrame;
use miniextendr_api::{DataFrameRow, IntoList, miniextendr};

#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct ParPoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

/// Create a large parallel points data frame.
///
/// @param n Number of rows to create.
/// @export
#[miniextendr]
pub fn create_large_par_points(n: i32) -> ToDataFrame<ParPointDataFrame> {
    let rows: Vec<ParPoint> = (0..n)
        .map(|i| ParPoint {
            x: i as f64,
            y: (i * 2) as f64,
            label: format!("pt_{i}"),
        })
        .collect();
    ToDataFrame(ParPoint::to_dataframe(rows))
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "_kind")]
pub enum ParEvent {
    A { id: i32, value: f64 },
    B { id: i32, name: String },
}

/// Create a large parallel events data frame.
///
/// @param n Number of rows to create.
/// @export
#[miniextendr]
pub fn create_large_par_events(n: i32) -> ToDataFrame<ParEventDataFrame> {
    let rows: Vec<ParEvent> = (0..n)
        .map(|i| {
            if i % 2 == 0 {
                ParEvent::A {
                    id: i,
                    value: i as f64 * 0.5,
                }
            } else {
                ParEvent::B {
                    id: i,
                    name: format!("evt_{i}"),
                }
            }
        })
        .collect();
    ToDataFrame(ParEvent::to_dataframe(rows))
}
