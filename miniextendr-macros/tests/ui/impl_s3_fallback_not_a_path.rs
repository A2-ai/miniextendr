//! Test: `fallback` takes a path to a function in a string, in the style of
//! `#[r_data(get = "Self::f")]` (#2150).

use miniextendr_api::externalptr::RSidecar;
use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(s3)]
struct Results {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub id: i32,
}

#[miniextendr(s3(r_data_accessors, fallback = "the column"))]
impl Results {
    pub fn new(id: i32) -> Self {
        Results { _r: RSidecar, id }
    }
}

fn main() {}
