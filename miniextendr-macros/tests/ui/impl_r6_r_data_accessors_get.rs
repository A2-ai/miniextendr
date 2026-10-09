//! Test: the getters-only form of `r_data_accessors` is for S3, S4 and env
//! field methods; R6 active bindings take the bare option (#1848).

use miniextendr_api::externalptr::RSidecar;
use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(r6)]
struct Gauge {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub level: i32,
}

#[miniextendr(r6(r_data_accessors = "get"))]
impl Gauge {
    pub fn new(level: i32) -> Self {
        Gauge { _r: RSidecar, level }
    }
}

fn main() {}
