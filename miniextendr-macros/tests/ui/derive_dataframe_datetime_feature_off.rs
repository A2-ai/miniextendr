//! Test: a datetime crate path whose miniextendr-api feature is off names the
//! feature at the field. The UI tests build the api without `jiff`; a local
//! `jiff` module stands in for the crate, which is also the shadowing case the
//! second note covers.

use miniextendr_macros::DataFrameRow;

mod jiff {
    pub struct Timestamp;
}

#[derive(DataFrameRow)]
struct Reading {
    id: i32,
    at: jiff::Timestamp,
}

fn main() {}
