//! Compile-fail test: `#[r_data(name = "...")]`, a field's R name (#1891),
//! must be non-empty, not `.ptr` (the element that carries the pointer in a
//! list or environment receiver), a syntactic R name, unique among the
//! type's R names, and on a slot rather than the `RSidecar` selector.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{RSidecar, Sidecar};

#[derive(ExternalPtr)]
struct Empty {
    #[r_data]
    _r: RSidecar,
    #[r_data(name = "")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct Ptr {
    #[r_data]
    _r: RSidecar,
    #[r_data(name = ".ptr")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct NonSyntactic {
    #[r_data]
    _r: RSidecar,
    #[r_data(name = "n rows")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct Reserved {
    #[r_data]
    _r: RSidecar,
    #[r_data(name = "for")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct Duplicate {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub keys: i32,
    #[r_data(name = "keys")]
    pub r_keys: Sidecar<Vec<i32>>,
}

#[derive(ExternalPtr)]
struct OnSelector {
    #[r_data(name = "r")]
    _r: RSidecar,
    #[r_data]
    pub count: i32,
}

fn main() {}
