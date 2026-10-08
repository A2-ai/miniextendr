//! `#[r_data(mut)]` generates the setter only: the getter is absent.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data(mut)]
    pub keys: Sidecar<Vec<i32>>,
}

fn main() {
    let mut engine = Engine {
        keys: Sidecar::new(vec![1, 2]),
    };
    engine.set_keys(vec![3]);
    let _keys: Vec<i32> = engine.keys();
}
