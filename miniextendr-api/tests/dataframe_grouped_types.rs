//! Grouped type fragments retain the same dataframe schema and readers.
mod r_test_utils;

use miniextendr_api::DataFrameRow;
use miniextendr_api::dataframe::{DataFrameRowConvert, FromDataFrame, IntoDataFrame};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

#[derive(Debug, PartialEq, DataFrameRow)]
pub struct Child {
    value: i32,
}

// Each $ty:ty arrives at the derive inside a rustc-created invisible group.
// Include fragments nested inside a reference, Box, and map arguments as well
// as whole fields; parsing handwritten source alone doesn't exercise these.
macro_rules! rows {
    ($scalar:ty, $nullable:ty, $seq:ty, $slice:ty, $array:ty, $key:ty, $child:ty, $borrowed:ty) => {
        #[derive(Debug, PartialEq, DataFrameRow)]
        pub struct Row {
            id: $scalar,
            maybe: $nullable,
            #[dataframe(expand)]
            values: $seq,
            #[dataframe(width = 2)]
            boxed: Box<$slice>,
            pair: $array,
            listed: Vec<$scalar>,
            named: HashMap<$key, $scalar>,
            keyed: BTreeMap<$scalar, $scalar>,
            child: $child,
        }

        #[derive(Debug, PartialEq, DataFrameRow)]
        pub struct Tuple($scalar, $nullable, #[dataframe(expand)] $seq);

        #[derive(Debug, PartialEq, DataFrameRow)]
        #[dataframe(align, tag = "kind")]
        pub enum Event {
            Generated {
                id: $scalar,
                #[dataframe(expand)]
                values: $seq,
                pair: $array,
            },
            Written {
                id: i32,
                #[dataframe(expand)]
                values: Vec<i32>,
                pair: [i32; 2],
            },
            Empty,
        }

        #[derive(Debug, PartialEq, DataFrameRow)]
        #[dataframe(align, tag = "kind")]
        pub enum TupleEvent {
            Generated($scalar, #[dataframe(expand)] $seq),
            Written(i32, #[dataframe(expand)] Vec<i32>),
            Empty,
        }

        #[derive(Debug, DataFrameRow)]
        pub struct Borrowed {
            #[dataframe(width = 2)]
            slice: $borrowed,
            pair: &'static $array,
        }
    };
}

rows!(
    i32,
    Option<i32>,
    Vec<i32>,
    [i32],
    [i32; 2],
    String,
    Child,
    &'static [i32]
);

fn check<T: DataFrameRowConvert + Debug + PartialEq>(rows: impl Fn() -> Vec<T>) {
    let expected = rows();
    let frame = rows().into_dataframe().unwrap();
    assert_eq!(frame.nrow(), expected.len());
    assert_eq!(Vec::<T>::from_dataframe(&frame).unwrap(), expected);
    #[cfg(feature = "rayon")]
    {
        assert_eq!(Vec::<T>::from_dataframe_par(&frame).unwrap(), expected);
        let frame = rows().into_dataframe_par().unwrap();
        assert_eq!(Vec::<T>::from_dataframe_par(&frame).unwrap(), expected);
    }
}

fn records(n: i32) -> Vec<Row> {
    (0..n)
        .map(|i| Row {
            id: i,
            maybe: (i == 0).then_some(7),
            values: (0..i).collect(),
            boxed: vec![i, i + 1].into_boxed_slice(),
            pair: [i, i + 2],
            listed: vec![i, i + 3],
            named: HashMap::from([("key".into(), i)]),
            keyed: BTreeMap::from([(i, i + 4)]),
            child: Child { value: i + 5 },
        })
        .collect()
}

#[test]
fn grouped_struct_fields_keep_their_schema_and_roundtrip() {
    r_test_utils::with_r_thread(|| {
        for n in [0, 3] {
            check(|| records(n));
        }
        let frame = records(3).into_dataframe().unwrap();
        assert_eq!(frame.column::<Vec<i32>>("id").unwrap(), [0, 1, 2]);
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("maybe").unwrap(),
            [Some(7), None, None]
        );
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("values_2").unwrap(),
            [None, None, Some(1)]
        );
        assert_eq!(frame.column::<Vec<i32>>("child_value").unwrap(), [5, 6, 7]);
    });
}

#[test]
fn grouped_tuple_fields_keep_the_write_only_schema() {
    r_test_utils::with_r_thread(|| {
        let frame = vec![Tuple(1, None, vec![]), Tuple(2, Some(3), vec![4, 5])]
            .into_dataframe()
            .unwrap();
        assert_eq!(frame.column::<Vec<i32>>("_0").unwrap(), [1, 2]);
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("_1").unwrap(),
            [None, Some(3)]
        );
        assert_eq!(
            frame.column::<Vec<Option<i32>>>("_2_2").unwrap(),
            [None, Some(5)]
        );
        // Tuple structs have no reader, independently of type grouping.
        assert!(Vec::<Tuple>::from_dataframe(&frame).is_err());
    });
}

#[test]
fn grouped_enum_fields_share_columns_with_handwritten_types() {
    r_test_utils::with_r_thread(|| {
        check(Vec::<Event>::new);
        check(|| vec![Event::Empty]);
        check(|| {
            vec![
                Event::Empty,
                Event::Generated {
                    id: 1,
                    values: vec![2, 3],
                    pair: [4, 5],
                },
                Event::Written {
                    id: 6,
                    values: vec![],
                    pair: [7, 8],
                },
            ]
        });
        check(Vec::<TupleEvent>::new);
        check(|| {
            vec![
                TupleEvent::Empty,
                TupleEvent::Generated(1, vec![2, 3]),
                TupleEvent::Written(4, vec![]),
            ]
        });
    });
}

#[test]
fn grouping_does_not_make_borrowed_fields_readable() {
    r_test_utils::with_r_thread(|| {
        let frame = vec![Borrowed {
            slice: &[1, 2],
            pair: &[3, 4],
        }]
        .into_dataframe()
        .unwrap();
        assert_eq!(frame.column::<Vec<i32>>("pair_2").unwrap(), [4]);
        let error = Vec::<Borrowed>::from_dataframe(&frame)
            .unwrap_err()
            .to_string();
        assert!(error.contains("no R→Rust reader"), "{error}");
        #[cfg(feature = "rayon")]
        assert!(Vec::<Borrowed>::from_dataframe_par(&frame).is_err());
    });
}
