#[cfg(feature = "rayon")]
use super::*;
#[cfg(feature = "rayon")]
use miniextendr_api::dataframe::ColumnarFrame;

#[cfg(feature = "rayon")]
#[test]
fn test_from_rows_par_struct_flatten_matches_sequential() {
    let make_rows = || -> Vec<FlatLocated> {
        (0..100)
            .map(|i| FlatLocated {
                id: i,
                origin: FlatPoint {
                    x: i as f64,
                    y: (i as f64) * 2.0,
                },
            })
            .collect()
    };

    let df_seq = FlatLocatedDataFrame::from_rows(make_rows());
    let df_par = FlatLocatedDataFrame::from_rows_par(make_rows());

    assert_eq!(df_seq.id, df_par.id);
    assert_eq!(df_seq.origin.len(), df_par.origin.len());
    for (s, p) in df_seq.origin.iter().zip(df_par.origin.iter()) {
        assert_eq!(s.x, p.x);
        assert_eq!(s.y, p.y);
    }
}

// The `as_list`-on-struct companion holds the struct values (the list-column
// is built only at frame assembly, #1748), so `from_rows_par` is pure Rust
// and can be compared with the sequential fill directly.
#[cfg(feature = "rayon")]
#[test]
fn test_from_rows_par_as_list_matches_sequential() {
    let make_rows = || -> Vec<FlatAsList> {
        (0..100)
            .map(|i| FlatAsList {
                id: i,
                origin: FlatPoint {
                    x: f64::from(i),
                    y: f64::from(i) * 2.0,
                },
            })
            .collect()
    };

    let df_seq = FlatAsListDataFrame::from_rows(make_rows());
    let df_par = FlatAsListDataFrame::from_rows_par(make_rows());

    assert_eq!(df_seq.id, df_par.id);
    assert_eq!(df_seq.origin.len(), df_par.origin.len());
    for (s, p) in df_seq.origin.iter().zip(df_par.origin.iter()) {
        assert_eq!(s.x, p.x);
        assert_eq!(s.y, p.y);
    }
}

// Regression for #513 review — `par_len_field` guard for as_list-only structs.
//
// `FlatOnlyAsList` has ONLY a `#[dataframe(as_list)]` struct field and no
// scalars. The companion `FlatOnlyAsListDataFrame` must NOT have a `_len`
// field (the `data` column provides the row-count directly via
// `into_data_frame`), and `from_rows_par` must fill that column.
#[cfg(feature = "rayon")]
#[test]
fn test_from_rows_par_only_as_list() {
    // The companion struct must NOT have a `_len` field — verify the field
    // layout is `{ data: Vec<FlatPoint> }` by constructing it directly.
    let df = FlatOnlyAsListDataFrame { data: vec![] };
    assert_eq!(df.data.len(), 0);

    let rows: Vec<FlatOnlyAsList> = (0..10)
        .map(|i| FlatOnlyAsList {
            data: FlatPoint {
                x: f64::from(i),
                y: 0.0,
            },
        })
        .collect();
    let df_par = FlatOnlyAsListDataFrame::from_rows_par(rows);
    let xs: Vec<f64> = df_par.data.iter().map(|p| p.x).collect();
    assert_eq!(xs, (0..10).map(f64::from).collect::<Vec<_>>());
}
