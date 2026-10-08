//! A `Sidecar<T>` field needs `#[r_data]`: the derive attaches only the fields
//! it knows to the external pointer.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

#[derive(ExternalPtr)]
struct Engine {
    pub keys: Sidecar<Vec<i32>>,
}

fn main() {}
