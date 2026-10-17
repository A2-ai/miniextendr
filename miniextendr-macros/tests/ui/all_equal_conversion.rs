use miniextendr_api::{AllEqual, ExternalPtr};

#[derive(Clone)]
struct NotConvertible;

#[derive(ExternalPtr, AllEqual)]
struct NeedsGetter {
    field: NotConvertible,
}

fn main() {}
