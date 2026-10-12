//! Test: the elements of a borrowed `&[T]` field expanded into columns are
//! cloned out of the borrow, so `T` must be `Clone`; the error points at the
//! element type. (Owned `Vec<T>` / `Box<[T]>` / `[T; N]` fields move their
//! elements and need no `Clone`.)

use miniextendr_api::{DataFrameRow, IntoR};

#[derive(IntoR)]
struct Col(f64);

#[derive(DataFrameRow)]
struct Row<'a> {
    id: i32,
    #[dataframe(expand)]
    values: &'a [Col],
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Event<'a> {
    A {
        id: i32,
        #[dataframe(expand)]
        values: &'a [Col],
    },
    B {
        id: i32,
    },
}

fn main() {}
