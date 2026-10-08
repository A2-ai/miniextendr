//! A sidecar accessor takes its field's visibility: a private field's getter
//! is private to the field's module.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::Sidecar;

mod inner {
    use super::*;

    #[derive(ExternalPtr)]
    pub struct Engine {
        #[r_data(ref)]
        keys: Sidecar<Vec<i32>>,
    }

    impl Engine {
        pub fn new() -> Self {
            Engine {
                keys: Sidecar::new(vec![1, 2]),
            }
        }

        pub fn key_count(&self) -> usize {
            self.keys().len()
        }
    }
}

fn main() {
    let engine = inner::Engine::new();
    let _count = engine.key_count();
    let _keys: Vec<i32> = engine.keys();
}
