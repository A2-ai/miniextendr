//! Test: `s3(r_data_accessors)` generates `names()`, `as.list()` and
//! `.DollarNames()` for the `#[r_data]` fields, in the getters-only form too
//! (#1891), so an impl method on `names` would define the class's method
//! twice.

use miniextendr_api::externalptr::RSidecar;
use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(s3)]
struct Gauge {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub level: i32,
}

#[miniextendr(s3(r_data_accessors = "get"))]
impl Gauge {
    pub fn new(level: i32) -> Self {
        Gauge { _r: RSidecar, level }
    }

    #[miniextendr(s3(generic = "names"))]
    pub fn field_names(&self) -> Vec<String> {
        vec!["level".to_string()]
    }
}

fn main() {}
