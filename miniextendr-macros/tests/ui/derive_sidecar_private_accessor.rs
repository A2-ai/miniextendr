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

        pub fn key_count(ptr: &ExternalPtr<Self>) -> usize {
            Self::keys(ptr).len()
        }
    }
}

fn read(engine: &ExternalPtr<inner::Engine>) {
    let _count = inner::Engine::key_count(engine);
    let _keys: Vec<i32> = inner::Engine::keys(engine);
}

fn main() {
    let _ = inner::Engine::new();
    let _ = read;
}
