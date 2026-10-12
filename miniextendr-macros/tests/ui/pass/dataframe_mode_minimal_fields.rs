//! Dataframe shorthand must not require IntoList or row-field IntoR (#2026).
#![allow(dead_code)]

use miniextendr_api::{ColumnarFrame, DataFrameRow, IntoList, IntoR, miniextendr};

#[derive(DataFrameRow)]
struct Inner {
    value: i32,
}

struct Cache;

#[miniextendr(dataframe)]
struct Outer {
    inner: Inner,
    #[dataframe(skip)]
    cache: Cache,
}

#[miniextendr(prefer = "dataframe")]
struct Preferred {
    inner: Inner,
}

// Row-level list conversion remains an explicit opt-in.
#[miniextendr(dataframe)]
#[derive(IntoList)]
struct Both {
    value: i32,
}

fn output<T: IntoR>() {}
fn list_row<T: IntoList>() {}

fn check_columns() {
    output::<OuterDataFrame>();
    output::<PreferredDataFrame>();
    output::<BothDataFrame>();
    list_row::<Both>();
    let _: OuterDataFrame = OuterDataFrame::from_rows(vec![Outer {
        inner: Inner { value: 1 },
        cache: Cache,
    }]);
    let _: PreferredDataFrame = PreferredDataFrame::from_rows(vec![Preferred {
        inner: Inner { value: 2 },
    }]);
}

fn main() {}
