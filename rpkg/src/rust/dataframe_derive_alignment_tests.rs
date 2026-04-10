//! Tests that standalone #[derive(DataFrameRow)] produces a companion type
//! with IntoR, so it can be returned from #[miniextendr] functions.
//!
//! Before the fix, only #[miniextendr(dataframe)] generated the companion IntoR.
//! Now #[derive(DataFrameRow)] also generates it.

use miniextendr_api::miniextendr;
use miniextendr_api::{DataFrameRow, IntoList};

/// A row type using standalone derives (not #[miniextendr(dataframe)]).
#[derive(Clone, IntoList, DataFrameRow)]
pub struct StandaloneRow {
    pub name: String,
    pub value: f64,
}

/// Returns a DataFrame built from standalone derives — this proves
/// the companion type has IntoR.
#[miniextendr]
pub fn standalone_dataframe_roundtrip() -> StandaloneRowDataFrame {
    StandaloneRow::to_dataframe(vec![
        StandaloneRow {
            name: "a".into(),
            value: 1.0,
        },
        StandaloneRow {
            name: "b".into(),
            value: 2.0,
        },
    ])
}
