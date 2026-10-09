//! `#[r_data(ref)]` generates the getter only: the setter is absent.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data(ref)]
    pub keys: Sidecar<Vec<i32>>,
}

fn edit(engine: &mut ExternalPtr<Engine>) {
    let _keys: Vec<i32> = Engine::keys(engine);
    Engine::set_keys(engine, vec![3]);
}

fn main() {
    let _ = Engine {
        keys: Sidecar::new(vec![1, 2]),
    };
    let _ = edit;
}
