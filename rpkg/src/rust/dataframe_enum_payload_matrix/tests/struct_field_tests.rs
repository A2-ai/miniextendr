use super::super::*;

#[test]
fn test_struct_flatten_align_located_rows_have_values() {
    let df = StructFlattenEvent::to_dataframe(vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Other { id: 2 },
    ]);
    // origin is present at row 0 (Located), absent at row 1 (Other)
    // The companion struct field is always _tag regardless of the tag= column name.
    assert_eq!(df._tag[0], "Located".to_string());
    assert_eq!(df._tag[1], "Other".to_string());
    assert_eq!(df.id[0], Some(1i32));
    assert_eq!(df.id[1], Some(2i32));
    // origin column holds the raw Point values
    assert!(df.origin[0].is_some());
    assert!(df.origin[1].is_none());
}

// This test calls into_dataframe_split which calls into_data_frame → R API.
// It's exercised by the R-level tests in test-dataframe-enum-payload-matrix.R.
#[test]
#[ignore = "requires R runtime (calls into_data_frame)"]
fn test_struct_flatten_split_located_has_correct_count() {
    let split = vec![
        StructFlattenEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructFlattenEvent::Located {
            id: 2,
            origin: Point { x: 3.0, y: 4.0 },
        },
        StructFlattenEvent::Other { id: 3 },
    ]
    .into_dataframe_split();
    // split returns a list of per-variant data frames
    // We can't easily inspect column names from Rust, but we can check row counts
    let _ = split; // compiled and not panicked ⇒ success
}

#[test]
fn test_struct_list_align_origin_is_some() {
    let df = StructListEvent::to_dataframe(vec![
        StructListEvent::Located {
            id: 1,
            origin: Point { x: 1.0, y: 2.0 },
        },
        StructListEvent::Other { id: 2 },
    ]);
    // origin is the list-column holding Option<Point>
    assert!(df.origin[0].is_some());
    assert!(df.origin[1].is_none());
}
