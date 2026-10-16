use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Scalar {
    #[dataframe(as_factor)]
    level: Option<i32>,
}

#[derive(DataFrameRow)]
struct Nested {
    #[dataframe(as_factor)]
    level: Option<Option<Level>>,
}

#[derive(DataFrameRow)]
enum Level {
    Low,
    High,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Named {
    Value {
        #[dataframe(as_factor)]
        level: Option<Level>,
    },
    Empty,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
enum Tuple {
    Value(#[dataframe(as_factor)] ::std::option::Option<Level>),
    Empty,
}

fn main() {}
