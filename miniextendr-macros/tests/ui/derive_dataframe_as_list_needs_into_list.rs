//! Test: an `as_list` field whose type is a plain struct or enum needs `IntoList`
//! (each cell is that value's list), and the error points at the field's type.

use miniextendr_macros::DataFrameRow;

// Neither type implements IntoList.
struct Inner {
    a: i32,
}

enum Dir {
    Up(i32),
    Down,
}

#[derive(DataFrameRow)]
struct Row {
    id: i32,
    #[dataframe(as_list)]
    inner: Inner,
}

#[derive(DataFrameRow)]
enum Event {
    Move {
        id: i32,
        #[dataframe(as_list)]
        dir: Dir,
    },
    Stop {
        id: i32,
    },
}

fn main() {}
