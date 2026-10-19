//! Test: `fallback_names` lists the names `fallback` answers, so it needs
//! `fallback` (#2150).

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
    fn column_names(&self) -> Vec<String> {
        Vec::new()
    }
}

#[miniextendr(s3(r_data_accessors, fallback_names = "Self::column_names"))]
impl Results {
    pub fn new(id: i32) -> Self {
        Results { _r: RSidecar, id }
    }
}

fn main() {}
