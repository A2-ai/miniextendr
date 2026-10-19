//! Test: `fallback` answers the names no field has for the readers of
//! `s3(r_data_accessors)`, so it needs the option (#2150).

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

impl Results {
    fn column(&self, _name: &str) -> Option<Vec<f64>> {
        None
    }
}

#[miniextendr(s3(fallback = "Self::column"))]
impl Results {
    pub fn new(id: i32) -> Self {
        Results { _r: RSidecar, id }
    }
}

fn main() {}
