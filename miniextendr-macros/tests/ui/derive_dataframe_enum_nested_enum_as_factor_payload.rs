//! Test: as_factor on payload-bearing enum (no IntoR impl) → compile error.

use miniextendr_macros::DataFrameRow;

// Inner has a payload variant and no UnitEnumFactor impl.
#[derive(Clone, Debug)]
enum Status {
    Active,
    Suspended { reason: String },
}

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Tracked {
        id: i32,
        #[dataframe(as_factor)]
        status: Status,
    },
    Other {
        id: i32,
    },
}

fn main() {}
