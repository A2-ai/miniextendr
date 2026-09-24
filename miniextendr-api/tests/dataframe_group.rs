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

/// `select_rows` treats a row as a row of every column: a matrix column is
/// gathered along its first dimension (with its row names), and a packed
/// data.frame column is subset recursively.
#[test]
fn select_rows_keeps_matrix_and_packed_columns_row_aligned() {
    use miniextendr_api::dataframe::DataFrame;
    use miniextendr_api::{OwnedProtect, SexpExt, r_str};

    r_test_utils::with_r_thread(|| unsafe {
        let df = r_str!(
            "local({
                d <- data.frame(id = 1:3)
                d$m <- I(matrix(1:6, nrow = 3, dimnames = list(c('a', 'b', 'c'), c('x', 'y'))))
                d$p <- data.frame(q = c(10, 20, 30))
                d
            })"
        )
        .expect("build the frame");
        let _df_guard = OwnedProtect::new(df);

        let out = DataFrame::from_sexp(df)
            .expect("a data.frame")
            .select_rows(&[2, 0]);
        let out = out.as_sexp();

        let m = out.vector_elt(1);
        assert_eq!(m.get_dim().as_slice::<i32>(), &[2, 2]);
        assert_eq!(m.as_slice::<i32>(), &[3, 1, 6, 4]);
        let row_names = m.get_dimnames().vector_elt(0);
        let name = |i| std::ffi::CStr::from_ptr(row_names.string_elt(i).r_char());
        assert_eq!((name(0), name(1)), (c"c", c"a"));
        assert!(m.inherits_class(c"AsIs"), "the I() class is kept");

        let packed = out.vector_elt(2);
        assert!(packed.is_data_frame());
        assert_eq!(packed.vector_elt(0).as_slice::<f64>(), &[30.0, 10.0]);
    });
}
