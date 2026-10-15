use miniextendr_api::DataFrameRow;

#[derive(DataFrameRow)]
struct Row {
    #[dataframe(as_factor)]
    value: i32,
}

fn main() {}
