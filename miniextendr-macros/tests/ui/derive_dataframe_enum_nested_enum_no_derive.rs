//! Test: nested enum without DataFrameRow derive produces clear error.

use miniextendr_macros::DataFrameRow;

// StatusNoDerived deliberately does NOT derive DataFrameRow.
#[derive(Clone, Debug)]
enum Status {
    Active,
    Suspended { reason: String },
}

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Tracked { id: i32, status: Status },
    Other { id: i32 },
}

fn main() {}
