//! Test: as_factor + as_list conflict → compile error.

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug)]
enum Dir {
    N,
    S,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Ev {
        #[dataframe(as_factor, as_list)]
        dir: Dir,
    },
    Other,
}

fn main() {}
