//! A sidecar accessor takes the `ExternalPtr` handle, not the struct: the
//! struct holds nothing that points back at its pointer, so `engine.keys()`
//! on a `&Engine` does not compile (#1856).

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data(ref)]
    pub keys: Sidecar<Vec<i32>>,
}

fn read(engine: &Engine) -> usize {
    engine.keys().len()
}

fn main() {
    let _ = Engine {
        keys: Sidecar::new(vec![1, 2]),
    };
    let _ = read;
}
