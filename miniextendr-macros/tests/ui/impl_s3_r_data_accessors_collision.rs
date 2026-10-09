//! Test: `s3(r_data_accessors)` generates `$<-` for the `#[r_data]` fields, so
//! an impl method on `$<-` would define the class's method twice (#1848). The
//! getters-only form `s3(r_data_accessors = "get")` leaves `$<-` to the class.

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

#[miniextendr(s3(r_data_accessors))]
impl Gauge {
    pub fn new(level: i32) -> Self {
        Gauge { _r: RSidecar, level }
    }

    #[miniextendr(s3(generic = "$<-"))]
    pub fn with_level(
        self: &miniextendr_api::externalptr::ExternalPtr<Self>,
        name: &str,
        value: i32,
    ) -> Self {
        let _ = name;
        Gauge { _r: RSidecar, level: value + self.level }
    }
}

fn main() {}
