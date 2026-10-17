use miniextendr_api::DataFrameRow;

#[derive(DataFrameRow)]
enum Payload {
    Value { x: i32 },
}

#[derive(DataFrameRow)]
struct Row {
    #[dataframe(as_factor)]
    value: Payload,
}

fn main() {}
