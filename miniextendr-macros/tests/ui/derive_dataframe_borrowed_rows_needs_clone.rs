//! Test: borrowed rows (`&[Row]` / `&Vec<Row>`) clone each value they don't
//! write by reference, so a row with a field type that is not `Clone` has no
//! borrowed conversion, and the error names that field type (#2307). The row
//! derives, and its owned conversion compiles: only the borrowed one is
//! refused. A type that does not derive `DataFrameRow` gets the trait's own
//! message.
//!
//! (`(&rows).into_dataframe()` on a `&Vec<Row>` falls through to the owned
//! `Vec<Row>` impl instead, and borrowck reports the move, E0507; borrowck does
//! not run after these type errors, so that case is documented, not tested.)

use miniextendr_api::{DataFrameRow, IntoDataFrame};

#[derive(DataFrameRow)]
struct Inner {
    q: i32,
}

#[derive(DataFrameRow)]
struct Row {
    id: i32,
    inner: Inner,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Event {
    A { inner: Inner },
    B { id: i32 },
}

fn owned(rows: Vec<Row>, events: Vec<Event>) {
    let _ = rows.into_dataframe();
    let _ = events.into_dataframe();
}

fn borrowed(rows: &[Row], events: &Vec<Event>, numbers: &[i32]) {
    let _ = IntoDataFrame::into_dataframe(rows);
    let _ = rows.into_dataframe();
    let _ = IntoDataFrame::into_dataframe(events);
    let _ = IntoDataFrame::into_dataframe(numbers);
}

fn main() {}
