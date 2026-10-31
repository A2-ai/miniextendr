//! Compile-pass test: fields that read `NULL` for `None` (#2270) under S3,
//! R6 and S7. `#[r_data(null_if_none)]` on an `Option<T>` struct field,
//! with `read_only` and `name` too; a `Nullable<T>` struct field, whose
//! setter needs `TryFromSexp for Nullable<T>`; and a `Sidecar<Nullable<T>>`
//! field, the spelling the refusal of `null_if_none` on a `Sidecar` points
//! at. Nothing here touches the R runtime.

#![allow(dead_code)]

use miniextendr_api::externalptr::{RSidecar, Sidecar};
use miniextendr_api::{ExternalPtr, Nullable};

#[derive(ExternalPtr)]
#[externalptr(s3)]
pub struct Engine {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none)]
    pub label: Option<String>,
    #[r_data(null_if_none, read_only, name = "code")]
    pub r_code: Option<i32>,
    #[r_data]
    pub tag: Nullable<String>,
    #[r_data]
    pub note: Sidecar<Nullable<String>>,
}

#[derive(ExternalPtr)]
#[externalptr(r6)]
pub struct Gauge {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none, setter = "visible")]
    pub label: Option<Vec<f64>>,
    #[r_data(read_only)]
    pub tag: Nullable<i32>,
}

#[derive(ExternalPtr)]
#[externalptr(s7)]
pub struct Meter {
    #[r_data]
    _r: RSidecar,
    #[r_data(null_if_none, prop_doc = "The label, NULL when unset.")]
    pub label: Option<String>,
    #[r_data(prop_doc = "The tag, NULL when unset.")]
    pub tag: Nullable<bool>,
}

/// Builds an `Engine` (never called: `Sidecar::new` needs R's main thread).
fn engine() -> Engine {
    Engine {
        _r: RSidecar,
        label: None,
        r_code: Some(1),
        tag: Nullable(None),
        note: Sidecar::new(Nullable(None)),
    }
}

fn main() {}
