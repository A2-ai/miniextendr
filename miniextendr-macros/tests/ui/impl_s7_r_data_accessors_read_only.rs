//! Test: the read-only form of `r_data_accessors` is for S3, S4 and env
//! field methods (#2268); an S7 class marks a property it must not write
//! `#[r_data(read_only)]`, field by field.

use miniextendr_api::externalptr::RSidecar;
use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(s7)]
struct Gauge {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub level: i32,
}

#[miniextendr(s7(r_data_accessors = "read_only"))]
impl Gauge {
    pub fn new(level: i32) -> Self {
        Gauge { _r: RSidecar, level }
    }
}

fn main() {}
