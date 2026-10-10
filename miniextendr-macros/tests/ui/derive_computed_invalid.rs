//! Compile-fail test: the `Computed` field rules of `#[derive(ExternalPtr)]`
//! (#1883). `get` applies to a `Computed` field only; a `Computed` field
//! needs `get`; `ref` / `mut` / `setter` don't apply to it; it must be `pub`;
//! and it is refused under `#[externalptr(r6)]` / `#[externalptr(s7)]`,
//! whose bindings / properties need a setter.

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{Computed, RSidecar};

#[derive(ExternalPtr)]
struct GetOnPlainField {
    #[r_data]
    _r: RSidecar,
    #[r_data(get = "Self::f")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct NoGet {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub count: Computed,
}

#[derive(ExternalPtr)]
struct RefOnComputed {
    #[r_data]
    _r: RSidecar,
    #[r_data(ref, get = "Self::f")]
    pub count: Computed,
}

#[derive(ExternalPtr)]
struct SetterOnComputed {
    #[r_data]
    _r: RSidecar,
    #[r_data(setter = "visible", get = "Self::f")]
    pub count: Computed,
}

#[derive(ExternalPtr)]
struct PrivateComputed {
    #[r_data]
    _r: RSidecar,
    #[r_data(get = "Self::f")]
    count: Computed,
}

#[derive(ExternalPtr)]
#[externalptr(r6)]
struct R6Computed {
    #[r_data]
    _r: RSidecar,
    #[r_data(get = "Self::f")]
    pub count: Computed,
}

#[derive(ExternalPtr)]
#[externalptr(s7)]
struct S7Computed {
    #[r_data]
    _r: RSidecar,
    #[r_data(get = "Self::f")]
    pub count: Computed,
}

fn main() {}
