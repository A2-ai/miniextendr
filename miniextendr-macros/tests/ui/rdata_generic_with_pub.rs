//! Test: generic types with pub #[r_data] fields should fail.

use miniextendr_macros::ExternalPtr;

// Stub types for the test
struct RSidecar;
struct RData;

#[derive(ExternalPtr)]
struct Bad<T> {
    value: T,
    #[r_data]
    r: RSidecar,
    #[r_data]
    pub slot: RData, // Error: generic type with pub r_data
}

fn main() {}
