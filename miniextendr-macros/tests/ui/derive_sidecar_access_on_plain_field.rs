//! `ref` / `mut` in `#[r_data(...)]` apply to `Sidecar<T>` fields only.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::RSidecar;

#[derive(ExternalPtr)]
struct Engine {
    #[r_data]
    _r: RSidecar,
    #[r_data(ref)]
    pub count: i32,
}

fn main() {}
