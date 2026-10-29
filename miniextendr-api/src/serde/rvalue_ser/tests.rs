use super::*;
use serde::Serialize;
use std::collections::BTreeMap;

fn named(v: &RValue) -> Vec<(&str, &RValue)> {
    match v {
        RValue::List(pairs) => pairs
            .iter()
            .map(|(n, v)| (n.as_deref().unwrap_or(""), v))
            .collect(),
        other => panic!("expected a list, got {other:?}"),
    }
}

#[derive(Serialize)]
struct Typed {
    #[serde(serialize_with = "as_character")]
    names: Vec<String>,
    #[serde(serialize_with = "as_character")]
    holes: Vec<Option<String>>,
    #[serde(serialize_with = "as_character")]
    absent: Option<String>,
    #[serde(serialize_with = "as_integer")]
    counts: Vec<i64>,
    #[serde(serialize_with = "as_double")]
    values: Vec<Option<i32>>,
    #[serde(serialize_with = "as_logical")]
    flags: Vec<bool>,
}

#[test]
fn atomic_helpers_type_empty_and_holed_sequences() {
    let empty = Typed {
        names: vec![],
        holes: vec![],
        absent: None,
        counts: vec![],
        values: vec![],
        flags: vec![],
    };
    let v = to_rvalue(&empty).unwrap();
    let fields = named(&v);
    assert!(matches!(fields[0].1, RValue::Character(c) if c.is_empty()));
    assert!(matches!(fields[1].1, RValue::Character(c) if c.is_empty()));
    assert!(matches!(fields[2].1, RValue::Character(c) if c == &[None]));
    assert!(matches!(fields[3].1, RValue::Integer(c) if c.is_empty()));
    assert!(matches!(fields[4].1, RValue::Double(c) if c.is_empty()));
    assert!(matches!(fields[5].1, RValue::Logical(c) if c.is_empty()));

    let full = Typed {
        names: vec!["a".into(), "b".into()],
        holes: vec![Some("a".into()), None],
        absent: Some("x".into()),
        counts: vec![1, 2],
        values: vec![Some(1), None],
        flags: vec![true],
    };
    let v = to_rvalue(&full).unwrap();
    let fields = named(&v);
    assert!(
        matches!(fields[0].1, RValue::Character(c) if c == &[Some("a".to_string()), Some("b".to_string())])
    );
    assert!(matches!(fields[1].1, RValue::Character(c) if c == &[Some("a".to_string()), None]));
    assert!(matches!(fields[2].1, RValue::Character(c) if c == &[Some("x".to_string())]));
    assert!(matches!(fields[3].1, RValue::Integer(c) if c == &[Some(1), Some(2)]));
    match fields[4].1 {
        RValue::Double(c) => {
            assert_eq!(c[0], 1.0);
            assert!(c[1].is_na());
        }
        other => panic!("expected a double vector, got {other:?}"),
    }
    assert!(matches!(fields[5].1, RValue::Logical(c) if c == &[Some(true)]));
}

#[test]
fn atomic_helpers_refuse_other_elements() {
    #[derive(Serialize)]
    struct Wide {
        #[serde(serialize_with = "as_integer")]
        counts: Vec<i64>,
    }
    let err = to_rvalue(&Wide {
        counts: vec![1, 1 << 40],
    })
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("`as_integer`: element 2 of the field is not an integer"),
        "{err}"
    );

    #[derive(Serialize)]
    struct Nested {
        #[serde(serialize_with = "as_character")]
        groups: Vec<Vec<String>>,
    }
    let err = to_rvalue(&Nested {
        groups: vec![vec!["a".into(), "b".into()]],
    })
    .unwrap_err();
    assert!(
        err.to_string()
            .contains("element 1 of the field is not a string"),
        "{err}"
    );

    #[derive(Serialize)]
    struct Numbers {
        #[serde(serialize_with = "as_character")]
        ids: Vec<i32>,
    }
    let err = to_rvalue(&Numbers { ids: vec![1] }).unwrap_err();
    assert!(
        err.to_string()
            .contains("`as_character`: the field is not a string"),
        "{err}"
    );
}

#[test]
fn scalars_map_like_the_sexp_serializer() {
    assert!(matches!(to_rvalue(&true).unwrap(), RValue::Logical(v) if v == vec![Some(true)]));
    assert!(matches!(to_rvalue(&7u8).unwrap(), RValue::Integer(v) if v == vec![Some(7)]));
    assert!(matches!(to_rvalue(&1.5f32).unwrap(), RValue::Double(v) if v == vec![1.5]));
    assert!(
        matches!(to_rvalue(&'x').unwrap(), RValue::Character(v) if v == vec![Some("x".to_string())])
    );
    assert!(matches!(to_rvalue(&()).unwrap(), RValue::Null));
    assert!(matches!(to_rvalue(&None::<i32>).unwrap(), RValue::Null));
    assert!(matches!(to_rvalue(&Some(3i32)).unwrap(), RValue::Integer(v) if v == vec![Some(3)]));
}

#[test]
fn wide_integers_follow_the_i64_policy() {
    assert!(matches!(to_rvalue(&5i64).unwrap(), RValue::Integer(v) if v == vec![Some(5)]));
    assert!(
        matches!(to_rvalue(&(1i64 << 40)).unwrap(), RValue::Double(v) if v == vec![(1i64 << 40) as f64])
    );
    // NA_integer_ must not be produced from a real value.
    assert!(matches!(
        to_rvalue(&i64::from(i32::MIN)).unwrap(),
        RValue::Double(_)
    ));
    assert!(matches!(to_rvalue(&u64::MAX).unwrap(), RValue::Double(_)));
    assert!(matches!(to_rvalue(&u32::MAX).unwrap(), RValue::Double(_)));
}

#[test]
fn homogeneous_sequences_coalesce_mixed_ones_stay_lists() {
    assert!(
        matches!(to_rvalue(&vec![1i32, 2, 3]).unwrap(), RValue::Integer(v) if v == vec![Some(1), Some(2), Some(3)])
    );
    assert!(matches!(to_rvalue(&["a", "b"][..]).unwrap(), RValue::Character(v) if v.len() == 2));
    // Fixed-size arrays are tuples to serde, so they stay lists.
    assert!(matches!(to_rvalue(&["a", "b"]).unwrap(), RValue::List(v) if v.len() == 2));
    match to_rvalue(&vec![Some(1i32), None, Some(3)]).unwrap() {
        RValue::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(items.iter().all(|(n, _)| n.is_none()));
            assert!(matches!(items[1].1, RValue::Null));
        }
        other => panic!("expected a list, got {other:?}"),
    }
    // Tuples never coalesce.
    assert!(matches!(to_rvalue(&(1i32, 2i32)).unwrap(), RValue::List(v) if v.len() == 2));
    assert!(matches!(to_rvalue(&Vec::<i32>::new()).unwrap(), RValue::List(v) if v.is_empty()));
}

#[derive(Serialize)]
struct Inner {
    flag: bool,
}

#[derive(Serialize)]
struct Outer {
    id: i32,
    name: String,
    inner: Inner,
    weights: Vec<f64>,
}

#[test]
fn structs_become_named_lists_in_field_order() {
    let v = to_rvalue(&Outer {
        id: 1,
        name: "n".into(),
        inner: Inner { flag: false },
        weights: vec![0.5, 1.5],
    })
    .unwrap();
    let fields = named(&v);
    assert_eq!(
        fields.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        ["id", "name", "inner", "weights"]
    );
    assert!(matches!(fields[2].1, RValue::List(inner) if inner[0].0.as_deref() == Some("flag")));
    assert!(matches!(fields[3].1, RValue::Double(w) if w == &[0.5, 1.5]));
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Shape {
    Unit,
    Newtype(i32),
    Tuple(i32, String),
    Struct { w: f64 },
}

#[test]
fn enum_variants_follow_the_external_tagging_shapes() {
    assert!(
        matches!(to_rvalue(&Shape::Unit).unwrap(), RValue::Character(v) if v == vec![Some("unit".to_string())])
    );
    let nt = to_rvalue(&Shape::Newtype(4)).unwrap();
    assert!(matches!(&named(&nt)[..], [("newtype", RValue::Integer(v))] if v == &vec![Some(4)]));
    let tup = to_rvalue(&Shape::Tuple(1, "a".into())).unwrap();
    assert!(matches!(&named(&tup)[..], [("tuple", RValue::List(items))] if items.len() == 2));
    let st = to_rvalue(&Shape::Struct { w: 2.0 }).unwrap();
    assert!(
        matches!(&named(&st)[..], [("struct", RValue::List(f))] if f[0].0.as_deref() == Some("w"))
    );
}

#[test]
fn maps_need_string_keys() {
    let mut m = BTreeMap::new();
    m.insert("b".to_string(), 2i32);
    m.insert("a".to_string(), 1i32);
    let v = to_rvalue(&m).unwrap();
    assert_eq!(
        named(&v).iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        ["a", "b"]
    );

    let mut bad = BTreeMap::new();
    bad.insert(1i32, "x");
    assert!(matches!(to_rvalue(&bad), Err(RSerdeError::NonStringKey)));
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Tagged {
    MissingField { field: String },
    Io,
}

#[derive(Serialize)]
enum External {
    Bad { code: i32 },
    Plain(String),
    Wrapped(Inner),
    Unit,
}

#[test]
fn tagged_parts_internal_tag_is_consumed() {
    let (variant, fields) =
        tagged_parts(&Tagged::MissingField { field: "id".into() }, "kind").unwrap();
    assert_eq!(variant.as_deref(), Some("missing_field"));
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "field");
    assert!(matches!(&fields[0].1, RValue::Character(v) if v == &vec![Some("id".to_string())]));

    let (variant, fields) = tagged_parts(&Tagged::Io, "kind").unwrap();
    assert_eq!(variant.as_deref(), Some("io"));
    assert!(fields.is_empty());
}

#[test]
fn tagged_parts_external_variants() {
    let (variant, fields) = tagged_parts(&External::Bad { code: 7 }, "kind").unwrap();
    assert_eq!(variant.as_deref(), Some("Bad"));
    assert_eq!(fields[0].0, "code");

    let (variant, fields) = tagged_parts(&External::Plain("x".into()), "kind").unwrap();
    assert_eq!(variant.as_deref(), Some("Plain"));
    assert_eq!(fields[0].0, "value");

    let (variant, fields) = tagged_parts(&External::Wrapped(Inner { flag: true }), "kind").unwrap();
    assert_eq!(variant.as_deref(), Some("Wrapped"));
    assert_eq!(fields[0].0, "flag");

    let (variant, fields) = tagged_parts(&External::Unit, "kind").unwrap();
    assert_eq!(variant.as_deref(), Some("Unit"));
    assert!(fields.is_empty());
}

#[test]
fn tagged_parts_without_variant_information() {
    let (variant, fields) = tagged_parts(&Inner { flag: true }, "kind").unwrap();
    assert!(variant.is_none());
    assert_eq!(fields[0].0, "flag");

    let (variant, fields) = tagged_parts(&String::from("boom"), "kind").unwrap();
    assert!(variant.is_none());
    assert!(fields.is_empty());

    // A different tag name leaves the `kind` field as data.
    let (variant, fields) = tagged_parts(&Tagged::Io, "type").unwrap();
    assert!(variant.is_none());
    assert_eq!(fields[0].0, "kind");
}
