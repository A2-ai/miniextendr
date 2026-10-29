use super::super::*;

#[test]
fn test_direction_is_unit_only() {
    // DataFrameRow derive emits IntoR for Direction; Direction must compile.
    // Verify all variants can be constructed (no payload).
    let _ = Direction::North;
    let _ = Direction::South;
    let _ = Direction::East;
    let _ = Direction::West;
}

/// Companion struct for `NestedFlattenEvent` has `status: Vec<Option<Status>>`.
/// `Other` rows produce `None`; `Tracked` rows produce `Some(Status::*)`.
#[test]
fn test_nested_flatten_companion_struct_shape() {
    let df = NestedFlattenEvent::to_dataframe(vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
        NestedFlattenEvent::Tracked {
            id: 3,
            status: Status::Err { code: 404 },
        },
    ]);
    assert_eq!(df.id[0], Some(1i32));
    assert_eq!(df.id[1], Some(2i32));
    assert_eq!(df.id[2], Some(3i32));
    // status field holds Option<Status>
    assert!(df.status[0].is_some()); // Tracked → Some(Status::Ok)
    assert!(df.status[1].is_none()); // Other → None (absent variant)
    assert!(df.status[2].is_some()); // Tracked → Some(Status::Err)
    // Verify Status discriminator via DataFrameRow
    let inner_df = Status::to_dataframe(vec![Status::Ok, Status::Err { code: 500 }]);
    assert!(inner_df.code[0].is_none()); // Ok has no code
    assert_eq!(inner_df.code[1], Some(500i32)); // Err has code
}

/// Verify payload-flatten partition: inner Status columns are correctly populated.
/// `Status::Ok` has no `code` field; `Status::Err` does.
#[test]
fn test_nested_flatten_status_payload_columns() {
    // Verify inner Status companion struct has the expected column layout.
    let inner_df = Status::to_dataframe(vec![Status::Ok, Status::Err { code: 404 }, Status::Ok]);
    // `code` column: None for Ok rows, Some(i32) for Err rows.
    assert!(inner_df.code[0].is_none()); // Ok has no code
    assert_eq!(inner_df.code[1], Some(404i32));
    assert!(inner_df.code[2].is_none());
}

#[test]
fn test_nested_factor_align_col_types() {
    let df = NestedFactorEvent::to_dataframe(vec![
        NestedFactorEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedFactorEvent::Stop { id: 2 },
    ]);
    // The companion struct has id: Vec<Option<i32>> and dir: Vec<Option<Direction>>
    assert_eq!(df.id[0], Some(1i32));
    assert_eq!(df.id[1], Some(2i32));
    assert!(df.dir[0].is_some());
    assert!(df.dir[1].is_none()); // Stop variant has no dir field
}

/// Factor levels must match Direction's variant order: North, South, East, West.
#[test]
fn test_direction_factor_levels_match_variant_order() {
    use miniextendr_api::UnitEnumFactor;
    // Direction is unit-only → DataFrameRow derive auto-emits UnitEnumFactor.
    // FACTOR_LEVELS must be in declaration order.
    let levels = Direction::FACTOR_LEVELS;
    assert_eq!(levels, &["North", "South", "East", "West"]);
}

#[test]
fn test_nested_list_align_col_types() {
    let df = NestedListEvent::to_dataframe(vec![
        NestedListEvent::Move {
            id: 1,
            dir: Direction::North,
        },
        NestedListEvent::Stop { id: 2 },
    ]);
    assert_eq!(df.id[0], Some(1i32));
    assert!(df.dir[0].is_some());
    assert!(df.dir[1].is_none());
}

#[test]
fn test_nested_flatten_align_col_types() {
    let df = NestedFlattenEvent::to_dataframe(vec![
        NestedFlattenEvent::Tracked {
            id: 1,
            status: Status::Ok,
        },
        NestedFlattenEvent::Other { id: 2 },
    ]);
    assert_eq!(df.id[0], Some(1i32));
    // status flattened → status column holds Option<Status>
    assert!(df.status[0].is_some());
    assert!(df.status[1].is_none()); // Other row: status is None
}
