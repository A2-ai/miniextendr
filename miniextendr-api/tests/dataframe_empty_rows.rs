//! Column-free rows retain their count and ignore user conversion lookalikes.
mod r_test_utils;

use miniextendr_api::dataframe::{DataFrame, DataFrameError, IntoDataFrame};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{DataFrameRow, IntoR};
use std::rc::Rc;

#[derive(DataFrameRow)]
pub struct UnitRow;

#[derive(DataFrameRow)]
pub struct NamedRow {}

#[derive(DataFrameRow)]
pub struct TupleRow();

#[derive(DataFrameRow)]
pub struct SkippedRow {
    #[dataframe(skip)]
    pub owner: Rc<()>,
}

#[derive(DataFrameRow)]
pub enum UnitEvent {
    First,
    Second,
}

#[derive(DataFrameRow)]
pub enum SkippedEvent {
    Named {
        #[dataframe(skip)]
        owner: Rc<()>,
    },
    Unit,
}

// These user functions must not capture the parallel-to-sequential fallback.
impl UnitRow {
    fn rows_into_dataframe(_rows: Vec<Self>) -> Result<DataFrame, DataFrameError> {
        Err(DataFrameError::Conversion("user struct function".into()))
    }
}
impl UnitEvent {
    fn rows_into_dataframe(_rows: Vec<Self>) -> Result<DataFrame, DataFrameError> {
        Err(DataFrameError::Conversion("user enum function".into()))
    }
}

fn assert_empty_columns<T: IntoDataFrame>(rows: T, nrow: usize) {
    let frame = rows.into_dataframe().unwrap();
    assert_eq!(frame.ncol(), 0);
    assert_eq!(frame.nrow(), nrow);
}

fn assert_companion<T: IntoR>(companion: T, nrow: usize) {
    // SAFETY: every caller runs on the embedded R thread.
    let scope = unsafe { ProtectScope::new() };
    let sexp = unsafe { scope.protect(companion.into_sexp()) }.get();
    let frame = DataFrame::from_sexp(sexp).unwrap();
    assert_eq!(frame.ncol(), 0);
    assert_eq!(frame.nrow(), nrow);
}

#[cfg(feature = "rayon")]
fn assert_empty_columns_par<T: IntoDataFrame>(rows: T, nrow: usize) {
    let frame = rows.into_dataframe_par().unwrap();
    assert_eq!(frame.ncol(), 0);
    assert_eq!(frame.nrow(), nrow);
}

#[test]
fn empty_struct_shapes_keep_their_row_count() {
    assert!(UnitRow::rows_into_dataframe(vec![]).is_err());
    r_test_utils::with_r_thread(|| {
        assert_empty_columns(Vec::<UnitRow>::new(), 0);
        assert_empty_columns(vec![UnitRow, UnitRow], 2);
        assert_empty_columns(vec![NamedRow {}, NamedRow {}, NamedRow {}], 3);
        assert_empty_columns(vec![TupleRow()], 1);
        let companion = UnitRow::to_dataframe(vec![UnitRow, UnitRow]);
        assert_companion(companion.clone(), 2);
        assert_companion(companion, 2);
        #[cfg(feature = "rayon")]
        {
            assert_empty_columns_par(Vec::<UnitRow>::new(), 0);
            assert_empty_columns_par(vec![UnitRow, UnitRow], 2);
            assert_empty_columns_par(vec![NamedRow {}], 1);
            assert_empty_columns_par(vec![TupleRow(), TupleRow()], 2);
        }
    });
}

#[test]
fn tagless_unit_enums_keep_their_row_count() {
    assert!(UnitEvent::rows_into_dataframe(vec![]).is_err());
    r_test_utils::with_r_thread(|| {
        assert_empty_columns(Vec::<UnitEvent>::new(), 0);
        assert_empty_columns(vec![UnitEvent::First, UnitEvent::Second], 2);
        let companion = UnitEvent::to_dataframe(vec![UnitEvent::First]);
        assert_companion(companion.clone(), 1);
        assert_companion(companion, 1);
        #[cfg(feature = "rayon")]
        {
            assert_empty_columns_par(Vec::<UnitEvent>::new(), 0);
            assert_empty_columns_par(vec![UnitEvent::First, UnitEvent::Second], 2);
        }
    });
}

#[test]
fn skipped_fields_need_no_clone_default_or_send_and_drop_once() {
    r_test_utils::with_r_thread(|| {
        let owner = Rc::new(());
        assert_empty_columns(
            vec![SkippedRow {
                owner: owner.clone(),
            }],
            1,
        );
        assert_eq!(Rc::strong_count(&owner), 1);
        assert_empty_columns(
            vec![
                SkippedEvent::Named {
                    owner: owner.clone(),
                },
                SkippedEvent::Unit,
            ],
            2,
        );
        assert_eq!(Rc::strong_count(&owner), 1);
        #[cfg(feature = "rayon")]
        {
            assert_empty_columns_par(
                vec![SkippedRow {
                    owner: owner.clone(),
                }],
                1,
            );
            assert_empty_columns_par(
                vec![SkippedEvent::Named {
                    owner: owner.clone(),
                }],
                1,
            );
            assert_eq!(Rc::strong_count(&owner), 1);
        }
    });
}
