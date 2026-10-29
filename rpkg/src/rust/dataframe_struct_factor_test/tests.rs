use super::*;
use miniextendr_api::ColumnarFrame;

#[test]
fn struct_factor_companion_preserves_owned_rows() {
    let companion = FactorRowDataFrame::from_rows(factor_rows(false));
    let rows = companion.into_rows();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1].id, 2);
    assert!(matches!(rows[1].level, ReadLevel::High));
    assert_eq!(rows[1].label, "row 2");
}

#[cfg(feature = "rayon")]
#[test]
fn struct_factor_parallel_fill_needs_no_default_or_clone() {
    let companion = FactorRowDataFrame::from_rows_par(factor_rows(false));
    assert_eq!(companion.id, [1, 2, 3]);
    assert!(matches!(companion.stage[1], ReadLevel::High));
}
