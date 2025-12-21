// Minimal test to understand the defaults parsing error

use miniextendr_api::{miniextendr, miniextendr_module};

#[derive(miniextendr_api::ExternalPtr)]
pub struct TestStruct {
    value: i32,
}

#[miniextendr(r6)]
impl TestStruct {
    // Try with defaults
    #[miniextendr(defaults(x = "42"))]
    pub fn new(x: i32) -> Self {
        TestStruct { value: x }
    }
}

miniextendr_module! {
    mod test_defaults_parsing;
    impl TestStruct;
}
