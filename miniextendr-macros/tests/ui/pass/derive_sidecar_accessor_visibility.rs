//! Compile-pass test: `#[derive(ExternalPtr)]` gives a `Sidecar<T>` field
//! typed accessors with the field's own visibility (#1855). They are
//! associated functions taking the `ExternalPtr` handle (#1856). A
//! `pub(crate)` field's accessors are callable from another module of the
//! crate, a `pub(super)` field's from the parent module, and `#[r_data(ref)]`
//! / `#[r_data(mut)]` select the getter or the setter. Nothing here touches
//! the R runtime: the accessors are only named, never called.

#![allow(dead_code)]

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{RSidecar, Sidecar};

mod engine {
    use super::*;

    #[derive(ExternalPtr, Clone)]
    pub struct Engine {
        #[r_data]
        _r: RSidecar,
        /// Both accessors, crate-visible.
        #[r_data(ref, mut)]
        pub(crate) keys: Sidecar<Vec<i32>>,
        /// Getter only, public.
        #[r_data(ref)]
        pub labels: Sidecar<Vec<String>>,
        /// Setter only, crate-visible.
        #[r_data(mut)]
        pub(crate) note: Sidecar<String>,
        /// Both accessors (a bare `#[r_data]`), private: used by `forget`.
        #[r_data]
        cache: Sidecar<Option<miniextendr_api::List>>,
    }

    pub mod nested {
        use super::*;

        #[derive(ExternalPtr)]
        pub struct Inner {
            /// Both accessors, visible to the parent module.
            #[r_data]
            pub(super) weights: Sidecar<Vec<f64>>,
        }
    }

    impl Engine {
        pub fn new(n: i32) -> Self {
            Engine {
                _r: RSidecar,
                keys: Sidecar::new((1..=n).collect()),
                labels: Sidecar::new(vec![]),
                note: Sidecar::new(String::new()),
                cache: Sidecar::default(),
            }
        }

        pub fn forget(ptr: &mut ExternalPtr<Self>) {
            Self::set_cache(ptr, None);
            let _cached = Self::cache(ptr);
        }

        pub fn inner_total(inner: &ExternalPtr<nested::Inner>) -> f64 {
            nested::Inner::weights(inner).iter().sum()
        }
    }
}

fn use_from_another_module(engine: &mut ExternalPtr<engine::Engine>) {
    let keys: Vec<i32> = engine::Engine::keys(engine);
    engine::Engine::set_keys(engine, keys);
    let _labels: Vec<String> = engine::Engine::labels(engine);
    engine::Engine::set_note(engine, String::from("seen"));
}

fn wrapped(engine: engine::Engine) -> ExternalPtr<engine::Engine> {
    ExternalPtr::new(engine)
}

fn main() {}
