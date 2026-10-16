//! String conflicts apply to each expanded cell and list element.
mod r_test_utils;
use miniextendr_api::DataFrameRow;
use miniextendr_api::dataframe::{FromDataFrame, IntoDataFrame};
use std::collections::BTreeMap;

#[derive(Debug, DataFrameRow)]
#[dataframe(align, tag = "kind", conflicts = "string")]
pub enum Expanded {
    Int {
        fixed: [i32; 2],
        #[dataframe(width = 2)]
        pinned: Vec<i32>,
        #[dataframe(expand)]
        auto: Vec<i32>,
    },
    Float {
        fixed: [f64; 2],
        #[dataframe(width = 2)]
        pinned: Box<[f64]>,
        #[dataframe(expand)]
        auto: Vec<f64>,
    },
    Text {
        fixed: [String; 2],
        #[dataframe(width = 2)]
        pinned: Vec<String>,
        #[dataframe(expand)]
        auto: Vec<String>,
    },
    Empty,
}

fn rows() -> Vec<Expanded> {
    vec![
        Expanded::Int {
            fixed: [1, 2],
            pinned: vec![3],
            auto: vec![4, 5],
        },
        Expanded::Float {
            fixed: [1.5, 2.5],
            pinned: vec![3.5, 4.5].into_boxed_slice(),
            auto: vec![6.5],
        },
        Expanded::Text {
            fixed: ["x".into(), "y".into()],
            pinned: vec![],
            auto: vec![],
        },
        Expanded::Empty,
    ]
}

fn check_expanded(frame: &miniextendr_api::dataframe::DataFrame) {
    for (name, values) in [
        ("fixed_1", [Some("1"), Some("1.5"), Some("x"), None]),
        ("fixed_2", [Some("2"), Some("2.5"), Some("y"), None]),
        ("pinned_1", [Some("3"), Some("3.5"), None, None]),
        ("pinned_2", [None, Some("4.5"), None, None]),
        ("auto_1", [Some("4"), Some("6.5"), None, None]),
        ("auto_2", [Some("5"), None, None, None]),
    ] {
        let expected: Vec<_> = values.map(|v| v.map(str::to_owned)).into();
        assert_eq!(
            frame.column::<Vec<Option<String>>>(name).unwrap(),
            expected,
            "{name}"
        );
    }
    assert_eq!((frame.nrow(), frame.ncol()), (4, 7));
    assert!(Vec::<Expanded>::from_dataframe(frame).is_err());
}

#[test]
fn all_expansion_modes_stringify_cells_and_preserve_absence() {
    r_test_utils::with_r_thread(|| {
        check_expanded(&rows().into_dataframe().unwrap());
        #[cfg(feature = "rayon")]
        check_expanded(&rows().into_dataframe_par().unwrap());
        let empty = Vec::<Expanded>::new().into_dataframe().unwrap();
        assert_eq!((empty.nrow(), empty.ncol()), (0, 5));
    });
}

#[derive(DataFrameRow)]
#[dataframe(tag = "kind", conflicts = "string")]
pub enum Maps {
    Int {
        entries: BTreeMap<i32, i32>,
        listed: Vec<i32>,
    },
    Text {
        entries: BTreeMap<String, f64>,
        listed: Box<[String]>,
    },
    Empty,
}

fn maps() -> Vec<Maps> {
    vec![
        Maps::Int {
            entries: BTreeMap::from([(2, 20), (1, 10)]),
            listed: vec![1, 2],
        },
        Maps::Text {
            entries: BTreeMap::from([("b".into(), 2.5), ("a".into(), 1.5)]),
            listed: vec!["x".into()].into_boxed_slice(),
        },
        Maps::Int {
            entries: BTreeMap::new(),
            listed: vec![],
        },
        Maps::Empty,
    ]
}

fn check_maps(frame: &miniextendr_api::dataframe::DataFrame) {
    for (name, expected) in [
        (
            "entries_keys",
            vec![
                Some(vec!["1", "2"]),
                Some(vec!["a", "b"]),
                Some(vec![]),
                None,
            ],
        ),
        (
            "entries_values",
            vec![
                Some(vec!["10", "20"]),
                Some(vec!["1.5", "2.5"]),
                Some(vec![]),
                None,
            ],
        ),
        (
            "listed",
            vec![Some(vec!["1", "2"]), Some(vec!["x"]), Some(vec![]), None],
        ),
    ] {
        let expected: Vec<Option<Vec<String>>> = expected
            .into_iter()
            .map(|row| row.map(|values| values.into_iter().map(str::to_owned).collect()))
            .collect();
        let column = frame.column::<miniextendr_api::List>(name).unwrap();
        let actual: Vec<_> = (0..column.len())
            .map(|i| column.get_index::<Option<Vec<String>>>(i).unwrap())
            .collect();
        assert_eq!(actual, expected, "{name}");
    }
}

#[test]
fn map_and_list_conflicts_keep_list_shape_and_map_pairing() {
    r_test_utils::with_r_thread(|| {
        check_maps(&maps().into_dataframe().unwrap());
        #[cfg(feature = "rayon")]
        check_maps(&maps().into_dataframe_par().unwrap());
    });
}

macro_rules! auto_rows {
    ($name:ident, $first:ty, $second:ty) => {
        #[derive(DataFrameRow)]
        #[dataframe(tag = "kind", conflicts = "string")]
        pub enum $name {
            First {
                #[dataframe(expand)]
                values: $first,
            },
            Second {
                #[dataframe(expand)]
                values: $second,
            },
            Empty,
        }
    };
}
auto_rows!(Boxed, Box<[i32]>, Box<[f64]>);
auto_rows!(Borrowed, &'static [i32], &'static [f64]);

#[test]
fn boxed_and_borrowed_auto_expansion_stringify_into_owned_storage() {
    r_test_utils::with_r_thread(|| {
        let boxed = || {
            vec![
                Boxed::First {
                    values: vec![1, 2].into_boxed_slice(),
                },
                Boxed::Second {
                    values: vec![1.5].into_boxed_slice(),
                },
                Boxed::Empty,
            ]
        };
        let borrowed = || {
            vec![
                Borrowed::First { values: &[1, 2] },
                Borrowed::Second { values: &[1.5] },
                Borrowed::Empty,
            ]
        };
        for frame in [
            boxed().into_dataframe().unwrap(),
            borrowed().into_dataframe().unwrap(),
        ] {
            assert_eq!(
                frame.column::<Vec<Option<String>>>("values_2").unwrap(),
                [Some("2".into()), None, None]
            );
        }
        #[cfg(feature = "rayon")]
        {
            for frame in [
                boxed().into_dataframe_par().unwrap(),
                borrowed().into_dataframe_par().unwrap(),
            ] {
                assert_eq!(
                    frame.column::<Vec<Option<String>>>("values_2").unwrap(),
                    [Some("2".into()), None, None]
                );
            }
        }
    });
}
#[derive(DataFrameRow)]
#[dataframe(tag = "kind", conflicts = "string")]
pub enum PartialMaps {
    Integer {
        same_keys: BTreeMap<i32, i32>,
        same_values: BTreeMap<i32, i32>,
    },
    Text {
        same_keys: BTreeMap<i32, String>,
        same_values: BTreeMap<String, i32>,
    },
}

#[test]
fn map_keys_and_values_coerce_independently() {
    r_test_utils::with_r_thread(|| {
        let rows = || {
            vec![
                PartialMaps::Integer {
                    same_keys: BTreeMap::from([(1, 10)]),
                    same_values: BTreeMap::from([(2, 20)]),
                },
                PartialMaps::Text {
                    same_keys: BTreeMap::from([(3, "thirty".into())]),
                    same_values: BTreeMap::from([("four".into(), 40)]),
                },
            ]
        };
        let check = |frame: &miniextendr_api::dataframe::DataFrame| {
            let keys = frame
                .column::<miniextendr_api::List>("same_keys_keys")
                .unwrap();
            assert_eq!(keys.get_index::<Vec<i32>>(1).unwrap(), [3]);
            let values = frame
                .column::<miniextendr_api::List>("same_keys_values")
                .unwrap();
            assert_eq!(values.get_index::<Vec<String>>(0).unwrap(), ["10"]);
            let keys = frame
                .column::<miniextendr_api::List>("same_values_keys")
                .unwrap();
            assert_eq!(keys.get_index::<Vec<String>>(0).unwrap(), ["2"]);
            let values = frame
                .column::<miniextendr_api::List>("same_values_values")
                .unwrap();
            assert_eq!(values.get_index::<Vec<i32>>(1).unwrap(), [40]);
        };
        check(&rows().into_dataframe().unwrap());
        #[cfg(feature = "rayon")]
        check(&rows().into_dataframe_par().unwrap());
    });
}

#[test]
fn variant_splits_keep_original_element_types() {
    use miniextendr_api::dataframe::{DataFrameShape, IntoDataFrameSplit};
    r_test_utils::with_r_thread(|| {
        let DataFrameShape::PerVariantList(parts) = rows().into_dataframe_split() else {
            panic!("multiple variants must split into frames");
        };
        assert_eq!(parts[0].1.column::<Vec<i32>>("fixed_1").unwrap(), [1]);
        assert_eq!(parts[1].1.column::<Vec<f64>>("fixed_1").unwrap(), [1.5]);
        assert_eq!(parts[2].1.column::<Vec<String>>("fixed_1").unwrap(), ["x"]);
        assert_eq!(parts[0].1.column::<Vec<i32>>("auto_2").unwrap(), [5]);
        assert_eq!(parts[1].1.column::<Vec<f64>>("pinned_1").unwrap(), [3.5]);
    });
}
