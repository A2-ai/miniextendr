//! Test: multiple RSidecar fields should fail.

use miniextendr_macros::ExternalPtr;

// Stub types for the test
struct RSidecar;
struct RData;

#[derive(ExternalPtr)]
struct Bad {
    #[r_data]
    r1: RSidecar,
    #[r_data]
    r2: RSidecar, // Error: only one RSidecar allowed
}

fn main() {}
