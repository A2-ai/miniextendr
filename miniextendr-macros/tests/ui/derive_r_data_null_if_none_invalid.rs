//! Compile-fail test: the `null_if_none` option of `#[r_data(...)]` (#2270).
//! It takes no value and is given once. It makes a public `Option<T>` struct
//! field read `NULL` for `None`, so it doesn't apply to the RSidecar
//! selector, a `Computed` field (its `get` function returns `Nullable<T>`),
//! a `Sidecar` field (declared `Sidecar<Nullable<T>>`), a private field, a
//! `Nullable<T>` field (`NULL` already) or a field of another type. It goes
//! with `read_only` (see `pass/derive_null_if_none_fields.rs`).

use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};
use miniextendr_api::{ExternalPtr, Nullable};

#[derive(ExternalPtr)]
struct WithValue {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none = "yes")]
    pub label: Option<String>,
}

#[derive(ExternalPtr)]
struct Twice {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none, null_if_none)]
    pub label: Option<String>,
}

#[derive(ExternalPtr)]
struct OnSelector {
    #[r_data(null_if_none)]
    _r: RSidecar,
}

#[derive(ExternalPtr)]
struct OnComputed {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none, get = "Self::f")]
    pub label: Computed,
}

#[derive(ExternalPtr)]
struct OnSidecar {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none)]
    pub label: Sidecar<Option<String>>,
}

#[derive(ExternalPtr)]
struct OnPrivate {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none)]
    label: Option<String>,
}

#[derive(ExternalPtr)]
struct OnNullable {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none)]
    pub label: Nullable<String>,
}

#[derive(ExternalPtr)]
struct OnString {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none)]
    pub label: String,
}

#[derive(ExternalPtr)]
struct Misspelt {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_null)]
    pub label: Option<String>,
}

fn main() {}
