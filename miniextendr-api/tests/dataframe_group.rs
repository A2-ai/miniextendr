//! Integration test for `GroupedDataFrame::extract` on Tuple-keyed groups
//! (`DataFrame::group_by_multi`). The `frames()`/`iter()` paths are exercised
//! by rpkg's testthat parity suite (`test-dataframe-groups.R`); this pins the
//! typed-extraction path. Runs on the R thread via `r_test_utils::with_r_thread`.

mod r_test_utils;

use miniextendr_api::dataframe::IntoDataFrame;
use miniextendr_api::{DataFrameRow, GroupKey, IntoList};

/// Row schema for the extract fixture: two character key columns + a value.
/// `pub` because the derive emits public helper items that reference the type.
#[derive(Clone, Debug, PartialEq, IntoList, DataFrameRow)]
pub struct AbvRow {
    pub a: String,
    pub b: String,
    pub v: f64,
}

fn row(a: &str, b: &str, v: f64) -> AbvRow {
    AbvRow {
        a: a.into(),
        b: b.into(),
        v,
    }
}

fn tuple(a: &str, b: &str) -> GroupKey {
    GroupKey::Tuple(vec![GroupKey::Str(a.into()), GroupKey::Str(b.into())])
}

#[test]
fn extract_partitions_tuple_keyed_groups() {
    r_test_utils::with_r_thread(|| {
        // interaction() order (first column varies fastest): x.p, y.p, x.q, y.q.
        let rows = vec![
            row("x", "p", 1.0),
            row("y", "p", 2.0),
            row("x", "q", 3.0),
            row("y", "q", 4.0),
            row("x", "p", 5.0),
        ];
        let df = rows.into_dataframe().expect("build frame");
        let grouped = df.group_by_multi(&["a", "b"]).expect("group_by_multi");
        let parts: Vec<(GroupKey, Vec<AbvRow>)> = grouped.extract().expect("extract");

        let keys: Vec<GroupKey> = parts.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(
            keys,
            vec![
                tuple("x", "p"),
                tuple("y", "p"),
                tuple("x", "q"),
                tuple("y", "q"),
            ]
        );
        assert_eq!(parts[0].1, vec![row("x", "p", 1.0), row("x", "p", 5.0)]);
        assert_eq!(parts[1].1, vec![row("y", "p", 2.0)]);
        assert_eq!(parts[2].1, vec![row("x", "q", 3.0)]);
        assert_eq!(parts[3].1, vec![row("y", "q", 4.0)]);
    });
}
