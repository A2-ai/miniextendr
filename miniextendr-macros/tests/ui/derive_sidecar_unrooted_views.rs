use miniextendr_api::{DataFrame, ExternalPtr, List};

#[derive(ExternalPtr)]
struct Bare {
    #[r_data]
    pub value: List,
}

#[derive(ExternalPtr)]
struct Optional {
    #[r_data]
    pub value: Option<List>,
}

#[derive(ExternalPtr)]
struct Frame {
    #[r_data]
    pub value: DataFrame,
}

#[derive(ExternalPtr)]
struct Nested {
    #[r_data]
    pub value: Vec<(i32, Option<List>)>,
}

fn main() {}
