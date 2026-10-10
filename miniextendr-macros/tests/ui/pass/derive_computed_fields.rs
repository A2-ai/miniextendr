//! Compile-pass test: `#[derive(ExternalPtr)]` with `Computed` fields
//! (#1883), per-field R names and the package's restored classes (#1891).
//! The generated getters call the `get` function (`Self::f` resolved to the
//! struct, or a free function of the module) on the live value and convert
//! its result with `IntoR`; a renamed field keeps its Rust name for the Rust
//! accessors; `restored(class = [...], message = "...")` fills the
//! `TypedExternal` constants. Nothing here touches the R runtime.

#![allow(dead_code)]

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar, TypedExternal};

#[derive(ExternalPtr)]
#[externalptr(
    s3,
    restored(
        class = ["pkg_saved", "pkg_error"],
        message = "this engine was saved; build a new one"
    )
)]
pub struct Engine {
    n: i32,
    #[r_data]
    _r: RSidecar,
    #[r_data(get = "Self::group_by")]
    pub group_by: Computed,
    #[r_data(name = "keys")]
    pub r_keys: Sidecar<Vec<i32>>,
    #[r_data(get = "n_rows_of")]
    pub n_rows: Computed,
    #[r_data(get = "Self::nothing")]
    pub nothing: Computed,
    #[r_data(name = "n.base")]
    pub base: i32,
}

impl Engine {
    fn group_by(&self) -> Vec<String> {
        vec![format!("g{}", self.n)]
    }

    fn nothing(&self) {}

    pub fn new(n: i32) -> Self {
        Engine {
            n,
            _r: RSidecar,
            group_by: Computed,
            r_keys: Sidecar::new((1..=n).collect()),
            n_rows: Computed,
            nothing: Computed,
            base: n,
        }
    }
}

fn n_rows_of(engine: &Engine) -> i32 {
    engine.n
}

/// Only the type with the option carries the classes and the message.
#[derive(ExternalPtr)]
#[externalptr(restored(class = "pkg_saved"))]
pub struct Plain {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub count: i32,
}

const _: () = {
    assert!(Engine::RESTORED_ERROR_CLASS.len() == 2);
    assert!(Engine::RESTORED_ERROR_MESSAGE.is_some());
    assert!(Plain::RESTORED_ERROR_CLASS.len() == 1);
    assert!(Plain::RESTORED_ERROR_MESSAGE.is_none());
    assert!(<() as TypedExternal>::RESTORED_ERROR_CLASS.is_empty());
};

fn rust_side(engine: &mut ExternalPtr<Engine>) {
    // The Rust accessors keep the Rust name.
    let keys: Vec<i32> = Engine::r_keys(engine);
    Engine::set_r_keys(engine, keys);
}

fn main() {}
