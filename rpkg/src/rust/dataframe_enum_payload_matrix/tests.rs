use super::*;
use miniextendr_api::dataframe::ColumnarFrame;

#[test]
fn vec_width_align_pinned_columns() {
    let df = VecWidthEvent::to_dataframe(vec![
        VecWidthEvent::Score {
            label: "a".into(),
            scores: vec![1.0, 2.0, 3.0],
        },
        VecWidthEvent::NoScore { label: "b".into() },
        VecWidthEvent::Score {
            label: "c".into(),
            scores: vec![4.0],
        },
    ]);
    assert_eq!(df.scores_1, vec![Some(1.0), None, Some(4.0)]);
    assert_eq!(df.scores_2, vec![Some(2.0), None, None]);
    assert_eq!(df.scores_3, vec![Some(3.0), None, None]);
}

#[test]
fn vec_expand_align_runtime_columns() {
    let df = VecExpandEvent::to_dataframe(vec![
        VecExpandEvent::Vals {
            label: "a".into(),
            vals: vec![1.0, 2.0],
        },
        VecExpandEvent::NoVals { label: "b".into() },
        VecExpandEvent::Vals {
            label: "c".into(),
            vals: vec![3.0],
        },
    ]);
    // expand stores Vec<Option<Vec<T>>> in companion struct
    assert_eq!(df.vals[0], Some(vec![1.0, 2.0]));
    assert_eq!(df.vals[1], None);
    assert_eq!(df.vals[2], Some(vec![3.0]));
}

#[test]
fn array_align_expanded_columns() {
    let df = ArrayEvent::to_dataframe(vec![
        ArrayEvent::Coords {
            id: 1,
            coords: [10.0, 20.0],
        },
        ArrayEvent::NoCoords { id: 2 },
        ArrayEvent::Coords {
            id: 3,
            coords: [30.0, 40.0],
        },
    ]);
    assert_eq!(df.coords_1, vec![Some(10.0), None, Some(30.0)]);
    assert_eq!(df.coords_2, vec![Some(20.0), None, Some(40.0)]);
}

#[test]
fn boxed_slice_expand_companion_shape() {
    let df = BoxedSliceEvent::to_dataframe(vec![
        BoxedSliceEvent::Buffer {
            name: "a".into(),
            data: vec![1.0, 2.0, 3.0].into_boxed_slice(),
        },
        BoxedSliceEvent::NoBuffer { name: "b".into() },
    ]);
    // expand on Box<[T]> in enum stores Vec<Option<Box<[T]>>> in companion
    assert_eq!(df.data.len(), 2);
    assert_eq!(df.data[0].as_deref(), Some(&[1.0, 2.0, 3.0][..]));
    assert!(df.data[1].is_none());
}

#[test]
fn singleton_split_returns_bare_dataframe_shape() {
    // Single-variant split returns DataFrameShape::Bare (a data.frame in R),
    // not a list-of-lists. Companion check: from_rows lays out as expected.
    let df = SingletonRowDataFrame::from_rows(vec![
        SingletonRow::Only {
            id: 1,
            label: "alpha".into(),
        },
        SingletonRow::Only {
            id: 2,
            label: "beta".into(),
        },
    ]);
    assert_eq!(df.id.len(), 2);
    assert_eq!(df.label.len(), 2);
}

// region: Map fields Rust unit tests

#[test]
fn test_hashmap_event_align_companion_shape() {
    // Verify companion struct has tally_keys and tally_values with correct option shape.
    let df = HashMapEvent::to_dataframe(vec![
        HashMapEvent::Tally {
            label: "a".into(),
            tally: HashMap::from([("x".to_string(), 1i32)]),
        },
        HashMapEvent::Empty { label: "b".into() },
    ]);
    assert_eq!(df.tally_keys.len(), 2);
    assert_eq!(df.tally_values.len(), 2);
    assert!(df.tally_keys[0].is_some());
    assert!(df.tally_values[0].is_some());
    assert!(df.tally_keys[1].is_none());
    assert!(df.tally_values[1].is_none());
    // Pairwise alignment: same length within a row.
    let k = df.tally_keys[0].as_ref().unwrap();
    let v = df.tally_values[0].as_ref().unwrap();
    assert_eq!(k.len(), v.len());
}

#[test]
fn test_hashmap_empty_map_row() {
    // An empty HashMap produces Some(vec![]) (not None) in both columns.
    let df = HashMapEvent::to_dataframe(vec![HashMapEvent::Tally {
        label: "a".into(),
        tally: HashMap::new(),
    }]);
    assert_eq!(df.tally_keys[0], Some(vec![]));
    assert_eq!(df.tally_values[0], Some(vec![]));
}

#[test]
fn test_btreemap_keys_sorted() {
    // BTreeMap preserves sorted order: keys should be ["a", "z"], values [1, 3].
    let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]),
    }]);
    assert_eq!(
        df.tally_keys[0].as_deref(),
        Some(vec!["a".to_string(), "z".to_string()].as_slice())
    );
    assert_eq!(
        df.tally_values[0].as_deref(),
        Some(vec![1i32, 3i32].as_slice())
    );
}

#[test]
fn test_btreemap_empty_map_row() {
    let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Tally {
        label: "a".into(),
        tally: BTreeMap::new(),
    }]);
    assert_eq!(df.tally_keys[0], Some(vec![]));
    assert_eq!(df.tally_values[0], Some(vec![]));
}

#[test]
fn test_btreemap_absent_variant_is_none() {
    let df = BTreeMapEvent::to_dataframe(vec![BTreeMapEvent::Empty { label: "b".into() }]);
    assert!(df.tally_keys[0].is_none());
    assert!(df.tally_values[0].is_none());
}

// endregion: Map fields Rust unit tests

mod struct_field_tests;

mod nested_enum_field_tests;
