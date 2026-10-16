//! Skipped tuple fields retain their positions in every dataframe writer.
mod r_test_utils;

use miniextendr_api::DataFrameRow;
use miniextendr_api::dataframe::{
    DataFrame, DataFrameShape, FromDataFrame, IntoDataFrame, IntoDataFrameSplit,
};
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

// Deliberately no Clone, Default, IntoR or TryFromSexp.
pub struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum Event {
    Leading(
        #[dataframe(skip)] Dropped,
        #[dataframe(rename = "value")] i32,
    ),
    Middle(
        #[dataframe(rename = "value")] i32,
        #[dataframe(skip)] Dropped,
        #[dataframe(rename = "extra")] String,
    ),
    Trailing(
        #[dataframe(rename = "value")] i32,
        #[dataframe(skip)] Dropped,
    ),
    All(#[dataframe(skip)] Dropped, #[dataframe(skip)] Dropped),
    Named {
        value: i32,
    },
}

fn rows(dropped: &Arc<AtomicUsize>) -> Vec<Event> {
    let token = || Dropped(Arc::clone(dropped));
    vec![
        Event::Leading(token(), 1),
        Event::Middle(2, token(), "two".into()),
        Event::Trailing(3, token()),
        Event::All(token(), token()),
        Event::Named { value: 4 },
    ]
}

fn check_frame(frame: &DataFrame) {
    assert_eq!(frame.nrow(), 5);
    assert_eq!(frame.ncol(), 3);
    assert_eq!(
        frame.column::<Vec<String>>("kind").unwrap(),
        ["Leading", "Middle", "Trailing", "All", "Named"],
    );
    assert_eq!(
        frame.column::<Vec<Option<i32>>>("value").unwrap(),
        [Some(1), Some(2), Some(3), None, Some(4)],
    );
    assert_eq!(
        frame.column::<Vec<Option<String>>>("extra").unwrap(),
        [None, Some("two".into()), None, None, None],
    );
    // R has no values from which to reconstruct skipped fields.
    assert!(Vec::<Event>::from_dataframe(frame).is_err());
    #[cfg(feature = "rayon")]
    assert!(Vec::<Event>::from_dataframe_par(frame).is_err());
}

#[test]
fn sequential_writer_preserves_skipped_positions_and_drops_owners_once() {
    r_test_utils::with_r_thread(|| {
        let dropped = Arc::new(AtomicUsize::new(0));
        let frame = rows(&dropped).into_dataframe().unwrap();
        assert_eq!(dropped.load(Ordering::SeqCst), 5);
        check_frame(&frame);
    });
}

#[cfg(feature = "rayon")]
#[test]
fn parallel_writer_preserves_skipped_positions_and_drops_owners_once() {
    // This shape must support the parallel fill, not merely its fallback.
    fn parallel_capable<T: miniextendr_api::dataframe::ParallelColumns<Event>>() {}
    parallel_capable::<EventDataFrame>();
    r_test_utils::with_r_thread(|| {
        let dropped = Arc::new(AtomicUsize::new(0));
        let frame = rows(&dropped).into_dataframe_par().unwrap();
        assert_eq!(dropped.load(Ordering::SeqCst), 5);
        check_frame(&frame);
    });
}

#[test]
fn split_writer_preserves_payloads_and_counts_all_skipped_rows() {
    r_test_utils::with_r_thread(|| {
        let dropped = Arc::new(AtomicUsize::new(0));
        let DataFrameShape::PerVariantList(parts) = rows(&dropped).into_dataframe_split() else {
            panic!("multiple variants must produce a list");
        };
        assert_eq!(dropped.load(Ordering::SeqCst), 5);
        assert_eq!(
            parts.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            ["leading", "middle", "trailing", "all", "named"]
        );
        for (index, (_, frame)) in parts.iter().enumerate() {
            assert_eq!(frame.nrow(), 1);
            if index == 3 {
                assert_eq!(frame.ncol(), 0);
            } else {
                assert_eq!(
                    frame.column::<Vec<i32>>("value").unwrap(),
                    [if index == 4 {
                        4
                    } else {
                        i32::try_from(index).unwrap() + 1
                    }]
                );
            }
        }
        assert_eq!(parts[1].1.column::<Vec<String>>("extra").unwrap(), ["two"]);
        let DataFrameShape::PerVariantList(empty) = Vec::<Event>::new().into_dataframe_split()
        else {
            panic!("empty input keeps all declared partitions");
        };
        assert_eq!(empty.len(), 5);
        assert!(empty.iter().all(|(_, frame)| frame.nrow() == 0));
    });
}

#[derive(DataFrameRow)]
pub enum NonSend {
    Tuple(#[dataframe(skip)] Rc<()>),
}

#[test]
fn all_skipped_non_send_tuple_keeps_aligned_and_bare_split_counts() {
    r_test_utils::with_r_thread(|| {
        let owner = Rc::new(());
        let make_rows = || {
            vec![
                NonSend::Tuple(Rc::clone(&owner)),
                NonSend::Tuple(Rc::clone(&owner)),
            ]
        };
        let frame = make_rows().into_dataframe().unwrap();
        assert_eq!((frame.nrow(), frame.ncol()), (2, 0));
        assert_eq!(Rc::strong_count(&owner), 1);
        #[cfg(feature = "rayon")]
        {
            let frame = make_rows().into_dataframe_par().unwrap();
            assert_eq!((frame.nrow(), frame.ncol()), (2, 0));
            assert_eq!(Rc::strong_count(&owner), 1);
        }
        let DataFrameShape::Bare(frame) = make_rows().into_dataframe_split() else {
            panic!("one variant must produce a bare dataframe");
        };
        assert_eq!((frame.nrow(), frame.ncol()), (2, 0));
        assert_eq!(Rc::strong_count(&owner), 1);
    });
}
