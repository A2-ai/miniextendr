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

// region: dplyr grouping metadata (group_declaration, group_by_metadata)

const GROUPED: &str = r#"c("grouped_df", "tbl_df", "tbl", "data.frame")"#;
const ROWWISE: &str = r#"c("rowwise_df", "tbl_df", "tbl", "data.frame")"#;

/// R code for a `groups` frame with the given columns and `n` rows, built
/// without dplyr.
fn groups_frame(columns: &str, n: usize) -> String {
    format!("structure(list({columns}), class = 'data.frame', row.names = c(NA, -{n}L))")
}

/// Run `f` on a 3-row frame (`ID = c(1L, 1L, 2L)`) whose class is the R code
/// `class` and whose `groups` attribute is the R code `groups`.
fn with_grouped<T>(
    class: &str,
    groups: &str,
    f: impl FnOnce(miniextendr_api::dataframe::DataFrame) -> T,
) -> T {
    use miniextendr_api::dataframe::DataFrame;
    use miniextendr_api::{OwnedProtect, r_str};

    let code = format!(
        "structure(data.frame(ID = c(1L, 1L, 2L), x = 1:3), class = {class}, groups = {groups})"
    );
    let frame = r_str!(code).expect("build the frame");
    // SAFETY: R thread; the frame stays protected while `f` reads it.
    let _guard = unsafe { OwnedProtect::new(frame) };
    f(DataFrame::from_sexp(frame).expect("a data.frame"))
}

fn declaration(
    class: &str,
    groups: &str,
) -> Result<Option<miniextendr_api::dataframe::GroupDeclaration>, miniextendr_api::DataFrameError> {
    with_grouped(class, groups, |df| df.group_declaration())
}

/// `group_declaration()` follows dplyr's `validate_grouped_df()`, which
/// `group_vars()` runs: a corrupt `groups` attribute is an error, stale row
/// indices are not checked.
#[test]
fn group_declaration_validates_groups_like_dplyr() {
    use miniextendr_api::DataFrameError;

    r_test_utils::with_r_thread(|| {
        let valid = groups_frame("ID = 1:2, .rows = list(1:2, 3L)", 2);

        // Not grouped: no grouped class, even with a stray `groups` attribute.
        assert!(matches!(declaration("'data.frame'", &valid), Ok(None)));
        assert!(matches!(declaration("'data.frame'", "NULL"), Ok(None)));

        // "The `groups` attribute must be a data frame."
        for groups in [
            "NULL",
            "list(ID = 1:2, .rows = list(1:2, 3L))",
            "data.frame()",
        ] {
            let err = declaration(GROUPED, groups).expect_err(groups);
            assert!(
                matches!(err, DataFrameError::GroupsNotDataFrame),
                "{groups}: {err:?}"
            );
            assert!(
                err.to_string()
                    .contains("the `groups` attribute must be a data frame"),
                "{err}"
            );
        }

        // "The last column of the `groups` attribute must be called `.rows`."
        for columns in ["ID = 1:2", ".rows = list(1:2, 3L), ID = 1:2"] {
            let err = declaration(GROUPED, &groups_frame(columns, 2)).expect_err(columns);
            assert!(
                matches!(err, DataFrameError::MissingGroupRows),
                "{columns}: {err:?}"
            );
            assert!(
                err.to_string()
                    .contains("the last column of the `groups` attribute must be called `.rows`"),
                "{err}"
            );
        }

        // "The `.rows` column must be list of one-based integer vectors."
        let bad_rows =
            |columns: &str| declaration(GROUPED, &groups_frame(columns, 2)).expect_err(columns);
        let err = bad_rows("ID = 1:2, .rows = c(1L, 3L)");
        assert!(
            matches!(&err, DataFrameError::BadGroupRows { group: None, type_of } if type_of == "integer"),
            "{err:?}"
        );
        assert!(
            err.to_string()
                .contains("the `.rows` column must be a list of one-based integer vectors"),
            "{err}"
        );
        let err = bad_rows("ID = 1:2, .rows = list(c(1, 2), 3)");
        assert!(
            matches!(&err, DataFrameError::BadGroupRows { group: Some(0), type_of } if type_of == "double"),
            "{err:?}"
        );
        assert!(
            err.to_string().contains("element for group 1 is double"),
            "{err}"
        );
        let err = bad_rows("ID = 1:2, .rows = list(1:2, NULL)");
        assert!(
            matches!(&err, DataFrameError::BadGroupRows { group: Some(1), type_of } if type_of == "NULL"),
            "{err:?}"
        );

        // Row bounds are not checked (`check_bounds = FALSE`).
        let decl = declaration(GROUPED, &groups_frame("ID = 1:2, .rows = list(1:2, 9L)", 2))
            .expect("bounds are not checked")
            .expect("grouped");
        assert_eq!(decl.vars, ["ID"]);
        assert!(decl.drop);

        // `.drop = FALSE` keeps empty groups.
        let decl = declaration(GROUPED, &format!("structure({valid}, .drop = FALSE)"))
            .expect("valid")
            .expect("grouped");
        assert!(!decl.drop);

        // `group_vars()` is `setdiff(names(groups), ".rows")`: each name once.
        let decl = declaration(
            GROUPED,
            &groups_frame("ID = 1:2, ID = 1:2, x = 1:2, .rows = list(1:2, 3L)", 2),
        )
        .expect("valid")
        .expect("grouped");
        assert_eq!(decl.vars, ["ID", "x"]);
    });
}

/// A `rowwise_df` gets the same structural checks; its `drop` is the default
/// method's `TRUE` unless it is also a `grouped_df`.
#[test]
fn group_declaration_reads_rowwise_frames() {
    use miniextendr_api::DataFrameError;

    r_test_utils::with_r_thread(|| {
        let no_keys = groups_frame(".rows = list(1L, 2L, 3L)", 3);
        let keyed = groups_frame("ID = c(1L, 1L, 2L), .rows = list(1L, 2L, 3L)", 3);

        let decl = declaration(ROWWISE, &no_keys)
            .expect("valid")
            .expect("rowwise");
        assert!(decl.vars.is_empty());
        assert!(decl.drop);
        let decl = declaration(ROWWISE, &keyed)
            .expect("valid")
            .expect("rowwise");
        assert_eq!(decl.vars, ["ID"]);

        // group_by_drop_default() has no rowwise_df method: always TRUE ...
        let drop_false = format!("structure({keyed}, .drop = FALSE)");
        let decl = declaration(ROWWISE, &drop_false)
            .expect("valid")
            .expect("rowwise");
        assert!(decl.drop);
        // ... unless the frame is a grouped_df too.
        let both = r#"c("rowwise_df", "grouped_df", "tbl_df", "tbl", "data.frame")"#;
        let decl = declaration(both, &drop_false)
            .expect("valid")
            .expect("grouped");
        assert!(!decl.drop);

        assert!(matches!(
            declaration(ROWWISE, "NULL"),
            Err(DataFrameError::GroupsNotDataFrame)
        ));
        assert!(matches!(
            declaration(ROWWISE, &groups_frame("ID = c(1L, 1L, 2L)", 3)),
            Err(DataFrameError::MissingGroupRows)
        ));
    });
}

/// `group_by_metadata()` shares the structural checks: a grouped class with a
/// corrupt `groups` attribute is reported as corrupt, not as ungrouped.
#[test]
fn group_by_metadata_validates_groups_like_dplyr() {
    use miniextendr_api::DataFrameError;

    r_test_utils::with_r_thread(|| {
        let metadata = |class: &str, groups: &str| {
            with_grouped(class, groups, |df| {
                df.group_by_metadata().map(|grouped| {
                    grouped
                        .iter()
                        .map(|(key, rows)| (key.label(), rows.to_vec()))
                        .collect::<Vec<_>>()
                })
            })
        };
        let valid = groups_frame("ID = 1:2, .rows = list(1:2, 3L)", 2);
        assert!(matches!(
            metadata("'data.frame'", &valid),
            Err(DataFrameError::NotGroupedDataFrame)
        ));
        assert!(matches!(
            metadata(GROUPED, "NULL"),
            Err(DataFrameError::GroupsNotDataFrame)
        ));
        assert!(matches!(
            metadata(GROUPED, &groups_frame("ID = 1:2", 2)),
            Err(DataFrameError::MissingGroupRows)
        ));
        // Doubles are rejected, as in dplyr.
        assert!(matches!(
            metadata(
                GROUPED,
                &groups_frame("ID = 1:2, .rows = list(c(1, 2), 3)", 2)
            ),
            Err(DataFrameError::BadGroupRows { group: Some(0), .. })
        ));
        // A key column longer than `.rows`.
        assert!(matches!(
            metadata(GROUPED, &groups_frame("ID = 1:3, .rows = list(1:2, 3L)", 2)),
            Err(DataFrameError::UnequalLengths { expected: 2, ref column, actual: 3 }) if column == "ID"
        ));

        // `.rows` is the last column: an earlier column of that name is not
        // read as the rows, nor as a key.
        let groups = metadata(
            GROUPED,
            &groups_frame(".rows = list(9L, 9L), ID = 1:2, .rows = list(1:2, 3L)", 2),
        )
        .expect("valid");
        assert_eq!(
            groups,
            vec![("1".to_string(), vec![0, 1]), ("2".to_string(), vec![2])]
        );
    });
}
// endregion
