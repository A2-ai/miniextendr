//! Test: as_factor on an enum variant's Option<T> field → compile error.
//! A struct field may be `Option<T>` (an NA-able factor column); an enum
//! variant's may not, because absent variants already fill that column with NA.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
enum Level {
    Low,
    High,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Event {
    A {
        #[dataframe(as_factor)]
        level: Option<Level>,
    },
    B,
}

fn main() {}
