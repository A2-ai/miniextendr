//! Compile-fail test: the `read_only` option of `#[r_data(...)]` (#2268).
//! It takes no value and is given once. It makes a public field R can write
//! read-only, so it doesn't apply to a `Computed` field (read-only already),
//! the RSidecar selector, a private field, or a field with `setter = ...`
//! (no R setter whose return to choose). It goes with `ref` / `mut`, which
//! select the Rust accessors (see `pass/derive_read_only_fields.rs`).

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{Computed, RSidecar};

#[derive(ExternalPtr)]
struct WithValue {
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only = "yes")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct Twice {
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only, read_only)]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct OnComputed {
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only, get = "Self::f")]
    pub count: Computed,
}

#[derive(ExternalPtr)]
struct OnSelector {
    #[r_data(read_only)]
    _r: RSidecar,
}

#[derive(ExternalPtr)]
struct OnPrivate {
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only)]
    count: i32,
}

#[derive(ExternalPtr)]
struct WithSetter {
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only, setter = "visible")]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct Misspelt {
    #[r_data]
    _r: RSidecar,
    #[r_data(readonly)]
    pub count: i32,
}

fn main() {}
