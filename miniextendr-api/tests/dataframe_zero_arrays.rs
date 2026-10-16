//! Zero-length arrays contribute rows without columns or element conversions.
mod r_test_utils;

use miniextendr_api::DataFrameRow;
use miniextendr_api::dataframe::{
    DataFrame, DataFrameShape, FromDataFrame, IntoDataFrame, IntoDataFrameSplit,
};

// Deliberately no Clone, Default, IntoR or TryFromSexp.
#[derive(Debug, PartialEq)]
pub struct Opaque(std::convert::Infallible);

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct EmptyRow {
    empty: [Opaque; 0],
    raw: [u8; 0],
}

fn empty_rows(n: usize) -> Vec<EmptyRow> {
    (0..n).map(|_| EmptyRow { empty: [], raw: [] }).collect()
}

fn check_empty_rows(frame: &DataFrame, n: usize) {
    assert_eq!((frame.nrow(), frame.ncol()), (n, 0));
    assert_eq!(
        Vec::<EmptyRow>::from_dataframe(frame).unwrap(),
        empty_rows(n)
    );
    #[cfg(feature = "rayon")]
    assert_eq!(
        Vec::<EmptyRow>::from_dataframe_par(frame).unwrap(),
        empty_rows(n)
    );
}

#[test]
fn column_free_struct_arrays_roundtrip_without_element_traits() {
    r_test_utils::with_r_thread(|| {
        for n in [0, 3] {
            let frame = empty_rows(n).into_dataframe().unwrap();
            check_empty_rows(&frame, n);
            #[cfg(feature = "rayon")]
            {
                let frame = empty_rows(n).into_dataframe_par().unwrap();
                check_empty_rows(&frame, n);
            }
        }
    });
}

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct Inner {
    value: i32,
}

#[derive(Debug, PartialEq, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum Event {
    Named {
        empty: [Opaque; 0],
    },
    Tuple([i32; 0]),
    Raw([u8; 0]),
    Value {
        empty: [i32; 0],
        count: i32,
    },
    Dynamic {
        empty: [i32; 0],
        #[dataframe(expand)]
        values: Vec<i32>,
    },
    Nested {
        empty: [i32; 0],
        inner: Inner,
    },
}

fn events() -> Vec<Event> {
    vec![
        Event::Named { empty: [] },
        Event::Tuple([]),
        Event::Raw([]),
        Event::Named { empty: [] },
        Event::Value {
            empty: [],
            count: 7,
        },
        Event::Dynamic {
            empty: [],
            values: vec![],
        },
        Event::Nested {
            empty: [],
            inner: Inner { value: 9 },
        },
        Event::Dynamic {
            empty: [],
            values: vec![1, 2],
        },
    ]
}

fn check_events(frame: &DataFrame, empty: bool) {
    let expected = if empty { vec![] } else { events() };
    assert_eq!(frame.nrow(), expected.len());
    assert_eq!(Vec::<Event>::from_dataframe(frame).unwrap(), expected);
    #[cfg(feature = "rayon")]
    assert_eq!(Vec::<Event>::from_dataframe_par(frame).unwrap(), expected);
}

#[test]
fn tagged_enum_arrays_roundtrip_with_static_dynamic_and_nested_columns() {
    r_test_utils::with_r_thread(|| {
        for empty in [true, false] {
            let rows = || if empty { vec![] } else { events() };
            let frame = rows().into_dataframe().unwrap();
            check_events(&frame, empty);
            #[cfg(feature = "rayon")]
            {
                let frame = rows().into_dataframe_par().unwrap();
                check_events(&frame, empty);
            }
        }
    });
}

#[test]
fn split_counts_do_not_depend_on_the_first_fields_columns() {
    r_test_utils::with_r_thread(|| {
        for empty in [true, false] {
            let rows = if empty { vec![] } else { events() };
            let DataFrameShape::PerVariantList(parts) = rows.into_dataframe_split() else {
                panic!("multiple variants must produce a list");
            };
            assert_eq!(
                parts
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect::<Vec<_>>(),
                ["named", "tuple", "raw", "value", "dynamic", "nested"]
            );
            for ((_, frame), count) in parts.iter().zip([2, 1, 1, 1, 2, 1]) {
                assert_eq!(frame.nrow(), if empty { 0 } else { count });
            }
            assert!(parts[..3].iter().all(|(_, frame)| frame.ncol() == 0));
            assert_eq!(parts[3].1.ncol(), 1);
            assert_eq!(parts[4].1.ncol(), if empty { 0 } else { 2 });
            assert_eq!(parts[5].1.ncol(), 1);
            if !empty {
                assert_eq!(parts[3].1.column::<Vec<i32>>("count").unwrap(), [7]);
                assert_eq!(
                    parts[4].1.column::<Vec<Option<i32>>>("values_1").unwrap(),
                    [None, Some(1)]
                );
                assert_eq!(
                    parts[4].1.column::<Vec<Option<i32>>>("values_2").unwrap(),
                    [None, Some(2)]
                );
                assert_eq!(parts[5].1.column::<Vec<i32>>("inner_value").unwrap(), [9]);
            }
        }
        let DataFrameShape::PerVariantList(parts) = vec![
            Event::Dynamic {
                empty: [],
                values: vec![],
            },
            Event::Dynamic {
                empty: [],
                values: vec![],
            },
        ]
        .into_dataframe_split() else {
            panic!("empty dynamic columns keep their partition");
        };
        assert_eq!((parts[4].1.nrow(), parts[4].1.ncol()), (2, 0));
    });
}

#[derive(Debug, PartialEq, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum Owned {
    Array([Opaque; 0]),
}

#[derive(DataFrameRow)]
pub enum Borrowed<'a> {
    Array(&'a [Opaque; 0]),
}

#[test]
fn bare_splits_count_owned_and_borrowed_arrays_without_element_traits() {
    // The tag supplies a real parallel column even though the array does not.
    #[cfg(feature = "rayon")]
    {
        fn parallel_capable<T: miniextendr_api::dataframe::ParallelColumns<Owned>>() {}
        parallel_capable::<OwnedDataFrame>();
    }
    r_test_utils::with_r_thread(|| {
        let empty = [];
        for n in [0, 2] {
            let owned = || (0..n).map(|_| Owned::Array([])).collect::<Vec<_>>();
            let borrowed = || (0..n).map(|_| Borrowed::Array(&empty)).collect::<Vec<_>>();
            for shape in [
                owned().into_dataframe_split(),
                borrowed().into_dataframe_split(),
            ] {
                let DataFrameShape::Bare(frame) = shape else {
                    panic!("one variant must produce a bare dataframe");
                };
                assert_eq!((frame.nrow(), frame.ncol()), (n, 0));
            }
            let frame = owned().into_dataframe().unwrap();
            assert_eq!((frame.nrow(), frame.ncol()), (n, 1));
            assert_eq!(Vec::<Owned>::from_dataframe(&frame).unwrap(), owned());
            let frame = borrowed().into_dataframe().unwrap();
            assert_eq!((frame.nrow(), frame.ncol()), (n, 0));
            assert!(Vec::<Borrowed<'_>>::from_dataframe(&frame).is_err());
            #[cfg(feature = "rayon")]
            {
                let frame = owned().into_dataframe_par().unwrap();
                assert_eq!((frame.nrow(), frame.ncol()), (n, 1));
                assert_eq!(Vec::<Owned>::from_dataframe_par(&frame).unwrap(), owned());
                let frame = borrowed().into_dataframe_par().unwrap();
                assert_eq!((frame.nrow(), frame.ncol()), (n, 0));
                assert!(Vec::<Borrowed<'_>>::from_dataframe_par(&frame).is_err());
            }
        }
    });
}

// Empty arrays still inherit their element's auto traits.
#[derive(Debug, PartialEq, DataFrameRow)]
pub struct NonSendRow {
    empty: [std::rc::Rc<()>; 0],
}

#[derive(Debug, PartialEq, DataFrameRow)]
#[dataframe(tag = "kind")]
pub enum NonSendEvent {
    Array([std::rc::Rc<()>; 0]),
}

#[test]
fn non_send_empty_arrays_keep_sequential_readers_with_rayon_enabled() {
    r_test_utils::with_r_thread(|| {
        let rows = || vec![NonSendRow { empty: [] }, NonSendRow { empty: [] }];
        let events = || vec![NonSendEvent::Array([]), NonSendEvent::Array([])];
        let frame = rows().into_dataframe().unwrap();
        assert_eq!(Vec::<NonSendRow>::from_dataframe(&frame).unwrap(), rows());
        let frame = events().into_dataframe().unwrap();
        assert_eq!(
            Vec::<NonSendEvent>::from_dataframe(&frame).unwrap(),
            events()
        );
        #[cfg(feature = "rayon")]
        {
            let frame = rows().into_dataframe_par().unwrap();
            assert_eq!(
                Vec::<NonSendRow>::from_dataframe_par(&frame).unwrap(),
                rows()
            );
            let frame = events().into_dataframe_par().unwrap();
            assert_eq!(
                Vec::<NonSendEvent>::from_dataframe_par(&frame).unwrap(),
                events()
            );
        }
    });
}
