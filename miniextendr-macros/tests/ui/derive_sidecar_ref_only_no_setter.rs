//! `#[r_data(ref)]` generates the getter only: the setter is absent.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data(ref)]
    pub keys: Sidecar<Vec<i32>>,
}

fn main() {
    let mut engine = Engine {
        keys: Sidecar::new(vec![1, 2]),
    };
    let _keys: Vec<i32> = engine.keys();
    engine.set_keys(vec![3]);
}
