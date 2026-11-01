//! Compile-fail test: the `cache` option of `#[r_data(...)]` (#2271). It
//! takes no value and is given once. It keeps the R value a public struct
//! field's getter converts until the next exclusive borrow, so it doesn't
//! apply to the RSidecar selector, a `Computed` field (its function may read
//! more than the struct), a `Sidecar` field (its getter converts nothing), a
//! private field or a scalar field (a length-1 vector is as fast to make as
//! to return). It goes with `read_only`, `null_if_none`, `name`, `setter`
//! and `prop_doc` (see `pass/derive_cache_fields.rs`).

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};

#[derive(ExternalPtr)]
struct WithValue {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache = "yes")]
    pub values: Vec<f64>,
}

#[derive(ExternalPtr)]
struct Twice {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache, cache)]
    pub values: Vec<f64>,
}

#[derive(ExternalPtr)]
struct OnSelector {
    #[r_data(cache)]
    _r: RSidecar,
}

#[derive(ExternalPtr)]
struct OnComputed {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache, get = "Self::f")]
    pub total: Computed,
}

#[derive(ExternalPtr)]
struct OnSidecar {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache)]
    pub values: Sidecar<Vec<f64>>,
}

#[derive(ExternalPtr)]
struct OnPrivate {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache)]
    values: Vec<f64>,
}

#[derive(ExternalPtr)]
struct OnScalar {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache)]
    pub count: i32,
}

#[derive(ExternalPtr)]
struct Misspelt {
    #[r_data]
    _r: RSidecar,
    #[r_data(cached)]
    pub values: Vec<f64>,
}

fn main() {}
