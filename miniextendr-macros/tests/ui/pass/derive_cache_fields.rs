//! Compile-pass test: fields whose getter keeps its converted value (#2271)
//! under S3, R6 and S7. `#[r_data(cache)]` on public conversion fields,
//! alone and with `read_only`, `null_if_none`, `name`, `setter` and
//! `prop_doc`, next to uncached, scalar and `Sidecar` fields. Nothing here
//! touches the R runtime.

#![allow(dead_code)]

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{RSidecar, Sidecar, TypedExternal};

#[derive(ExternalPtr)]
#[externalptr(s3)]
pub struct Engine {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache)]
    pub values: Vec<f64>,
    #[r_data(cache, read_only, name = "names")]
    pub r_names: Vec<String>,
    #[r_data(cache, null_if_none)]
    pub label: Option<String>,
    #[r_data]
    pub plain: Vec<f64>,
    #[r_data]
    pub n: i32,
    #[r_data]
    pub note: Sidecar<Vec<i32>>,
}

#[derive(ExternalPtr)]
#[externalptr(r6)]
pub struct Gauge {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache, setter = "visible")]
    pub values: Vec<i32>,
}

#[derive(ExternalPtr)]
#[externalptr(s7)]
pub struct Meter {
    #[r_data]
    _r: RSidecar,
    #[r_data(cache, prop_doc = "The readings.")]
    pub values: Vec<f64>,
    #[r_data(cache, null_if_none, read_only)]
    pub label: Option<Vec<String>>,
}

/// A type without a cached field keeps the trait's default count.
#[derive(ExternalPtr)]
pub struct Plain {
    #[r_data]
    _r: RSidecar,
    #[r_data]
    pub values: Vec<f64>,
}

const _: () = assert!(<Engine as TypedExternal>::CACHED_FIELD_COUNT == 3);
const _: () = assert!(<Gauge as TypedExternal>::CACHED_FIELD_COUNT == 1);
const _: () = assert!(<Meter as TypedExternal>::CACHED_FIELD_COUNT == 2);
const _: () = assert!(<Plain as TypedExternal>::CACHED_FIELD_COUNT == 0);

fn main() {}
