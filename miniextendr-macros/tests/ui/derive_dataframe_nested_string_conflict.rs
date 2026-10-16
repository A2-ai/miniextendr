//! Conflicting nested payloads cannot be replaced by scalar strings.
use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(tag = "kind", conflicts = "string")]
enum Row {
    First { payload: First },
    Second { payload: Second },
}

struct First;
struct Second;
fn main() {}
