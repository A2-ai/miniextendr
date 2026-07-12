//! Fixtures for DataFrame group-level iteration (`DataFrame::group_by`,
//! `GroupedDataFrame`, `group_rows`), driven by
//! `tests/testthat/test-dataframe-groups.R`.
//!
//! Parity target: R `split()` + `lapply()` — with the documented deviation
//! that NA keys form one trailing group instead of being dropped.

use miniextendr_api::dataframe::{DataFrame, IntoDataFrame, NamedDataFrameListBuilder};
use miniextendr_api::{DataFrameRow, IntoList, IntoR, SEXP, group_rows, miniextendr};

// region: row types

/// Fixed extraction schema for the typed-partition fixture: a character key
/// plus a double value. `DataFrameRow` has no `Option<…>` fields, so the
/// NA-key path is covered by the index/frame fixtures instead.
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct GvRow {
    pub g: String,
    pub v: f64,
}

/// Per-group summary row returned by the aggregation fixtures.
#[derive(Clone, Debug, IntoList, DataFrameRow)]
pub struct GroupSumRow {
    pub key: String,
    pub sum: f64,
    pub n: i32,
}
// endregion

// region: index-level fixtures (group order, sizes)

/// Group key labels in group order for any supported key column.
/// @param df A data.frame.
/// @param col The key column name.
#[miniextendr]
pub fn group_by_keys(df: DataFrame, col: &str) -> Vec<String> {
    let grouped = df.group_by(col).unwrap_or_else(|e| panic!("{}", e));
    grouped.iter().map(|(k, _)| k.label()).collect()
}

/// Group sizes in group order.
/// @param df A data.frame.
/// @param col The key column name.
#[miniextendr]
pub fn group_by_sizes(df: DataFrame, col: &str) -> Vec<i32> {
    let grouped = df.group_by(col).unwrap_or_else(|e| panic!("{}", e));
    grouped
        .iter()
        .map(|(_, idx)| i32::try_from(idx.len()).expect("group size exceeds i32"))
        .collect()
}
// endregion

// region: multi-column index-level fixtures (composite-key group order, sizes)

/// Composite group-key labels (`.`-joined) in group order, for multiple key
/// columns. Mirrors [`group_by_keys`] for `DataFrame::group_by_multi`.
/// @param df A data.frame.
/// @param cols The key column names.
#[miniextendr]
pub fn group_by_multi_keys(df: DataFrame, cols: Vec<String>) -> Vec<String> {
    let refs: Vec<&str> = cols.iter().map(String::as_str).collect();
    let grouped = df.group_by_multi(&refs).unwrap_or_else(|e| panic!("{}", e));
    grouped.iter().map(|(k, _)| k.label()).collect()
}

/// Composite-key group sizes in group order.
/// @param df A data.frame.
/// @param cols The key column names.
#[miniextendr]
pub fn group_by_multi_sizes(df: DataFrame, cols: Vec<String>) -> Vec<i32> {
    let refs: Vec<&str> = cols.iter().map(String::as_str).collect();
    let grouped = df.group_by_multi(&refs).unwrap_or_else(|e| panic!("{}", e));
    grouped
        .iter()
        .map(|(_, idx)| i32::try_from(idx.len()).expect("group size exceeds i32"))
        .collect()
}
// endregion

// region: multi-column frame-level fixture (the split(interaction()) analogue)

/// Named list of per-group sub-frames keyed by a composite `interaction()`-style
/// key — the `split(df, interaction(...))` analogue (except NA-containing tuples
/// form trailing groups instead of being dropped). Mirrors [`group_by_frames`].
/// @param df A data.frame.
/// @param cols The key column names.
#[miniextendr]
pub fn group_by_multi_frames(df: DataFrame, cols: Vec<String>) -> SEXP {
    let refs: Vec<&str> = cols.iter().map(String::as_str).collect();
    let grouped = df.group_by_multi(&refs).unwrap_or_else(|e| panic!("{}", e));
    let mut out = NamedDataFrameListBuilder::with_capacity(grouped.len());
    for (key, sub) in grouped.frames() {
        // `sub` is a rooted `BuiltDataFrame` (#1247); deref to the view for
        // push, which protects it in the builder's scope before `sub` drops.
        out = out.push(key.label(), *sub);
    }
    out.build().into_sexp()
}
// endregion

// region: frame-level fixture (the split() analogue)

/// Named list of per-group sub-frames — the `split()` analogue (except NA
/// keys form a trailing "NA" entry instead of being dropped).
/// @param df A data.frame.
/// @param col The key column name.
#[miniextendr]
pub fn group_by_frames(df: DataFrame, col: &str) -> SEXP {
    let grouped = df.group_by(col).unwrap_or_else(|e| panic!("{}", e));
    let mut out = NamedDataFrameListBuilder::with_capacity(grouped.len());
    for (key, sub) in grouped.frames() {
        // `sub` is a rooted `BuiltDataFrame` (#1247); deref to the view for
        // push, which protects it in the builder's scope before `sub` drops.
        out = out.push(key.label(), *sub);
    }
    out.build().into_sexp()
}
// endregion

// region: typed-extraction fixture (extract::<Row>)

/// One typed extraction + move-partition: per-group sum and count of `v`,
/// grouped by `g`. Input schema: `g` (character, no NAs), `v` (double).
/// Returns a data.frame with columns key/sum/n in group order.
/// @param df A data.frame with columns `g` (character) and `v` (double).
#[miniextendr]
pub fn group_by_extract_sums(df: DataFrame) -> SEXP {
    let grouped = df.group_by("g").unwrap_or_else(|e| panic!("{}", e));
    let parts = grouped
        .extract::<GvRow>()
        .unwrap_or_else(|e| panic!("{}", e));
    let rows: Vec<GroupSumRow> = parts
        .into_iter()
        .map(|(key, members)| GroupSumRow {
            key: key.label(),
            sum: members.iter().map(|r| r.v).sum(),
            n: i32::try_from(members.len()).expect("group size exceeds i32"),
        })
        .collect();
    rows.into_dataframe()
        .unwrap_or_else(|e| panic!("{}", e))
        .into_sexp()
}
// endregion

// region: rung-1 fixture (group_rows on typed rows)

/// Rung 1: rows synthesized internally, grouped Rust-side with `group_rows`
/// (no SEXP contact in the grouping itself; `Option<String>` keys give NA a
/// home — `None` sorts first). Returns a key/sum/n summary frame.
#[miniextendr]
pub fn group_rows_summary() -> SEXP {
    // Plain Rust rows — rung 1 never touches R, so no DataFrameRow derive
    // (and Option<String> keys are fine here).
    struct TypedObs {
        g: Option<String>,
        v: f64,
    }

    let rows: Vec<TypedObs> = (0..12)
        .map(|i| TypedObs {
            g: match i % 4 {
                0 => Some("a".to_string()),
                1 => Some("b".to_string()),
                2 => Some("c".to_string()),
                _ => None,
            },
            v: f64::from(i),
        })
        .collect();

    let by_key = group_rows(rows, |r| r.g.clone());
    let out: Vec<GroupSumRow> = by_key
        .into_iter()
        .map(|(key, members)| GroupSumRow {
            key: key.unwrap_or_else(|| "NA".to_string()),
            sum: members.iter().map(|r| r.v).sum(),
            n: i32::try_from(members.len()).expect("group size exceeds i32"),
        })
        .collect();
    out.into_dataframe()
        .unwrap_or_else(|e| panic!("{}", e))
        .into_sexp()
}
// endregion
