//! Test: `fallback` is for the `$` / `[[` readers of S3, S4 and env field
//! syntax; an R6 object's `$` reads its environment (#2150).

use miniextendr_api::externalptr::RSidecar;
use miniextendr_macros::miniextendr;

#[derive(miniextendr_api::ExternalPtr)]
#[externalptr(r6)]
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

#[miniextendr(r6(r_data_accessors, fallback = "Self::column"))]
impl Results {
    pub fn new(id: i32) -> Self {
        Results { _r: RSidecar, id }
    }
}

fn main() {}
