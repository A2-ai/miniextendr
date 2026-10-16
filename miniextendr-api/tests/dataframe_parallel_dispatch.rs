//! Parallel dispatch must ignore consumer methods and preserve fallback bounds.
#![cfg(feature = "rayon")]

use miniextendr_api::DataFrameRow;
use miniextendr_api::dataframe::{ColumnarFrame, FromRowsParProbe, ParallelColumns};
use std::rc::Rc;

#[allow(dead_code)]
trait ConsumerSpecialization: Sized {
    fn fill_columns<T>(self, _rows: T) {}
    fn index<T>(self, _index: T) {}
}
impl<T> ConsumerSpecialization for T {}

#[derive(DataFrameRow)]
pub struct ScalarRow {
    pub id: i32,
    pub label: String,
}

#[derive(DataFrameRow)]
pub struct NonSendRow {
    pub id: i32,
    #[dataframe(skip)]
    pub owner: Rc<()>,
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum EventRow {
    Id { id: i32 },
    Label { label: String },
}

#[derive(DataFrameRow)]
pub struct BorrowedRow<'a> {
    pub values: &'a [i32; 2],
}

#[test]
fn derived_structs_and_enums_ignore_consumer_methods() {
    let frame = ScalarRowDataFrame::from_rows_par(vec![
        ScalarRow {
            id: 1,
            label: "one".into(),
        },
        ScalarRow {
            id: 2,
            label: "two".into(),
        },
    ]);
    assert_eq!(frame.id, [1, 2]);
    assert_eq!(frame.label, ["one", "two"]);

    let events = EventRowDataFrame::from_rows_par(vec![
        EventRow::Id { id: 3 },
        EventRow::Label {
            label: "four".into(),
        },
    ]);
    assert_eq!(events.id, [Some(3), None]);
    assert_eq!(events.label, [None, Some("four".into())]);
}

#[test]
fn non_send_rows_keep_the_sequential_fallback() {
    let frame = NonSendRowDataFrame::from_rows_par(vec![NonSendRow {
        id: 5,
        owner: Rc::new(()),
    }]);
    assert_eq!(frame.id, [5]);
}

#[test]
fn borrowed_rows_need_no_static_lifetime() {
    fn fill(values: &[i32; 2]) -> BorrowedRowDataFrame<'_> {
        BorrowedRowDataFrame::from_rows_par(vec![BorrowedRow { values }])
    }
    let values = [6, 7];
    let frame = fill(&values);
    assert_eq!(frame.values_1, [6]);
    assert_eq!(frame.values_2, [7]);
}

// Give the two routes different results so always taking the fallback cannot
// satisfy the regression merely by returning the right column values.
struct RoutedFrame<Row> {
    rows: Vec<Row>,
    parallel: bool,
}

impl<Row> ColumnarFrame<Row> for RoutedFrame<Row> {
    fn from_rows(rows: Vec<Row>) -> Self {
        Self {
            rows,
            parallel: false,
        }
    }
}

impl<Row: Send> ParallelColumns<Row> for RoutedFrame<Row> {
    fn from_rows_parallel(rows: Vec<Row>) -> Self {
        Self {
            rows,
            parallel: true,
        }
    }
}

#[test]
#[allow(clippy::needless_borrow)] // The borrow is the parallel specialization tier.
fn probe_prefers_the_parallel_fill_for_eligible_rows() {
    struct MoveOnly(i32);
    let fill = (&FromRowsParProbe::<RoutedFrame<MoveOnly>, MoveOnly>::new())[()];
    let frame = fill(vec![MoveOnly(8)]);
    assert!(frame.parallel);
    assert_eq!(frame.rows[0].0, 8);
}

#[test]
#[allow(clippy::needless_borrow)] // Same call shape as generated code.
fn probe_moves_ineligible_rows_through_the_sequential_fill() {
    let fill = (&FromRowsParProbe::<RoutedFrame<NonSendRow>, NonSendRow>::new())[()];
    let frame = fill(vec![NonSendRow {
        id: 9,
        owner: Rc::new(()),
    }]);
    assert!(!frame.parallel);
    assert_eq!(frame.rows[0].id, 9);
}
