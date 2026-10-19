//! Test: the fallback returns an `Option`, whose `None` the readers handle;
//! a function that returns a plain value is reported at the attribute's
//! string (#2150).

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
    fn column(&self, _name: &str) -> Vec<f64> {
        Vec::new()
    }
}

#[miniextendr(s3(r_data_accessors, fallback = "Self::column"))]
impl Results {
    pub fn new(id: i32) -> Self {
        Results { _r: RSidecar, id }
    }
}

fn main() {}
