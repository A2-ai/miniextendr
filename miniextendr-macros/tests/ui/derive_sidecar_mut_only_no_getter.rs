//! `#[r_data(mut)]` generates the setter only: the getter is absent.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data(mut)]
    pub keys: Sidecar<Vec<i32>>,
}

fn edit(engine: &mut ExternalPtr<Engine>) {
    Engine::set_keys(engine, vec![3]);
    let _keys: Vec<i32> = Engine::keys(engine);
}

fn main() {
    let _ = Engine {
        keys: Sidecar::new(vec![1, 2]),
    };
    let _ = edit;
}
