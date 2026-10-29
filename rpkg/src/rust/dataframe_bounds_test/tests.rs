use super::*;

#[test]
fn skip_without_default_builds_columns() {
    let df = SkipNoDefaultDataFrame::from_rows(skip_no_default_rows());
    assert_eq!(df.id, vec![1, 2]);
    assert_eq!(df.label, vec!["a".to_string(), "b".to_string()]);
}

#[cfg(feature = "rayon")]
#[test]
fn skip_without_send_falls_back_to_sequential() {
    let df = SkipNoDefaultDataFrame::from_rows_par(skip_no_default_rows());
    assert_eq!(df.id, vec![1, 2]);
    assert_eq!(df.label, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn expanded_elements_are_moved() {
    let df = MovedElemsDataFrame::from_rows(moved_elems_rows());
    assert_eq!(
        df.pair_1.iter().map(|m| m.0).collect::<Vec<_>>(),
        [1.0, 3.0]
    );
    assert_eq!(
        df.pair_2.iter().map(|m| m.0).collect::<Vec<_>>(),
        [2.0, 4.0]
    );
    let firsts_2: Vec<Option<f64>> = df
        .firsts_2
        .iter()
        .map(|m| m.as_ref().map(|m| m.0))
        .collect();
    assert_eq!(firsts_2, [None, Some(21.0)]);
    assert_eq!(df.rest.len(), 2);
}

#[cfg(feature = "rayon")]
#[test]
fn expanded_elements_par_matches_sequential() {
    let seq = MovedElemsDataFrame::from_rows(moved_elems_rows());
    let par = MovedElemsDataFrame::from_rows_par(moved_elems_rows());
    let values = |col: &[Option<Moved>]| {
        col.iter()
            .map(|m| m.as_ref().map(|m| m.0))
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&seq.firsts_1), values(&par.firsts_1));
    assert_eq!(values(&seq.firsts_3), values(&par.firsts_3));
    assert_eq!(
        seq.pair_1.iter().map(|m| m.0).collect::<Vec<_>>(),
        par.pair_1.iter().map(|m| m.0).collect::<Vec<_>>()
    );
}

#[test]
fn companions_are_debug_and_clone() {
    assert_eq!(
        bounds_companion_debug_clone(),
        r#"DebugPointDataFrame { x: [1.5, 2.5], label: ["a", "b"] }"#
    );
    assert_eq!(
        bounds_enum_companion_debug_clone(),
        r#"DebugEventDataFrame { _tag: ["A", "B"], x: [Some(1), None], y: [None, Some("b")] }"#
    );
}
