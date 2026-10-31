//! Compile-pass test: `#[r_data(read_only)]` fields (#2268) under S3, R6 and
//! S7, and `Computed` fields under R6 and S7 (#1888). A read-only field gets
//! no R setter, so its type needs only `IntoR` + `Clone`: `Label` has no
//! `TryFromSexp`. A read-only `Sidecar<T>` field keeps its Rust accessors
//! (`mut` here), since read-only applies to R only. Nothing here touches the
//! R runtime.

#![allow(dead_code)]

use miniextendr_api::ExternalPtr;
use miniextendr_api::externalptr::{Computed, RSidecar, Sidecar};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::prelude::SEXP;

/// A value R reads and never writes: `IntoR` only.
#[derive(Debug, Clone)]
pub struct Label(String);

impl IntoR for Label {
    type Error = <String as IntoR>::Error;

    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        self.0.try_into_sexp()
    }
}

#[derive(ExternalPtr)]
#[externalptr(
    s3,
    field_error(class = "pkg_error", read_only_hint = "Use `with_%s()` instead.")
)]
pub struct Engine {
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only)]
    pub label: Label,
    #[r_data(mut, read_only, name = "keys")]
    pub r_keys: Sidecar<Vec<i32>>,
    #[r_data]
    pub level: i32,
}

#[derive(ExternalPtr)]
#[externalptr(r6, field_error(read_only_hint = "Read-only."))]
pub struct Gauge {
    n: i32,
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only)]
    pub label: Label,
    #[r_data(get = "Self::doubled")]
    pub doubled: Computed,
    #[r_data]
    pub level: i32,
}

impl Gauge {
    fn doubled(&self) -> i32 {
        self.n * 2
    }
}

#[derive(ExternalPtr)]
#[externalptr(s7)]
pub struct Meter {
    n: i32,
    #[r_data]
    _r: RSidecar,
    #[r_data(read_only, prop_doc = "A label R can't write.")]
    pub label: Label,
    #[r_data(get = "Self::doubled", prop_doc = "Twice `n`.")]
    pub doubled: Computed,
    #[r_data(prop_doc = "A level R writes.")]
    pub level: i32,
}

impl Meter {
    fn doubled(&self) -> i32 {
        self.n * 2
    }
}

fn rust_side(engine: &mut ExternalPtr<Engine>) {
    // The Rust setter of the read-only `Sidecar` field stays.
    Engine::set_r_keys(engine, vec![1, 2]);
}

fn main() {}
