use super::*;

#[test]
fn group_rows_partitions_and_orders_by_key() {
    let rows = vec![("b", 1), ("a", 2), ("b", 3), ("c", 4)];
    let grouped = group_rows(rows, |r| r.0);
    let keys: Vec<&str> = grouped.keys().copied().collect();
    assert_eq!(keys, vec!["a", "b", "c"]);
    assert_eq!(grouped["b"], vec![("b", 1), ("b", 3)]);
}

#[test]
fn group_rows_option_key_gives_na_a_home() {
    let rows = vec![(Some(2), "x"), (None, "y"), (Some(1), "z")];
    let grouped = group_rows(rows, |r| r.0);
    let keys: Vec<Option<i32>> = grouped.keys().copied().collect();
    assert_eq!(keys, vec![None, Some(1), Some(2)]);
}

#[test]
fn group_key_labels_match_r_printing() {
    assert_eq!(GroupKey::Str("a".into()).label(), "a");
    assert_eq!(GroupKey::Int(-3).label(), "-3");
    assert_eq!(GroupKey::Bool(true).label(), "TRUE");
    assert_eq!(GroupKey::Bool(false).label(), "FALSE");
    assert_eq!(GroupKey::Na.label(), "NA");
    assert_eq!(GroupKey::Na.to_string(), "NA");
}

#[test]
fn tuple_key_labels_join_with_dot() {
    let key = GroupKey::Tuple(vec![
        GroupKey::Str("a".into()),
        GroupKey::Int(2),
        GroupKey::Real(RealKey::new(2.5, "2.5")),
        GroupKey::Bool(true),
        GroupKey::Na,
    ]);
    assert_eq!(key.label(), "a.2.2.5.TRUE.NA");
    assert_eq!(key.to_string(), "a.2.2.5.TRUE.NA");
}

fn hash_of(key: &RealKey) -> u64 {
    use std::hash::{BuildHasher, BuildHasherDefault};
    BuildHasherDefault::<std::collections::hash_map::DefaultHasher>::default().hash_one(key)
}

#[test]
fn real_key_equality_follows_vctrs() {
    // -0 and 0 are one key (and hash alike).
    let neg_zero = RealKey::new(-0.0, "0");
    let zero = RealKey::new(0.0, "0");
    assert_eq!(neg_zero, zero);
    assert_eq!(hash_of(&neg_zero), hash_of(&zero));
    assert!(neg_zero.value().is_sign_positive());

    // Every NaN is one key, stored as the canonical positive NaN —
    // including x86's negative `0/0` NaN.
    let nan = RealKey::new(f64::NAN, "NaN");
    let neg_nan = RealKey::new(f64::from_bits(0xFFF8_0000_0000_0000), "NaN");
    assert_eq!(nan, neg_nan);
    assert_eq!(hash_of(&nan), hash_of(&neg_nan));
    assert_eq!(neg_nan.value().to_bits(), CANONICAL_NAN_BITS);
    assert!(!nan.value().is_na());

    // The label plays no part in equality; distinct values differ.
    assert_eq!(RealKey::new(1.5, "x"), RealKey::new(1.5, "y"));
    assert_ne!(RealKey::new(1.5, "1.5"), RealKey::new(2.5, "2.5"));
}

#[test]
fn real_key_label_and_order() {
    let key = GroupKey::Real(RealKey::new(19724.0, "2024-01-02"));
    assert_eq!(key.label(), "2024-01-02");
    assert_eq!(key.to_string(), "2024-01-02");

    // Normalised values sort -Inf < numbers < Inf < NaN under total_cmp.
    let mut values: Vec<f64> = [f64::NAN, 1.0, f64::INFINITY, -0.0, f64::NEG_INFINITY]
        .into_iter()
        .map(normalize_real)
        .collect();
    values.sort_by(f64::total_cmp);
    assert_eq!(values[0], f64::NEG_INFINITY);
    assert_eq!(values[1].to_bits(), 0.0f64.to_bits());
    assert_eq!(values[2], 1.0);
    assert_eq!(values[3], f64::INFINITY);
    assert!(values[4].is_nan());
}
