//! The snapshot encoding's choices, on the Rust-side tree (no R needed). The
//! round trips through R are in `tests/serde_exact.rs`.

use std::borrow::Cow;
use std::collections::BTreeMap;

use super::ser::{Node, NodeSerializer, Seq};
use serde::Serialize;

fn node<T: Serialize>(value: &T) -> Node {
    value.serialize(NodeSerializer).expect("encodes")
}

fn named(fields: Vec<(&'static str, Node)>) -> Node {
    Node::Named(
        fields
            .into_iter()
            .map(|(name, node)| (Cow::Borrowed(name), node))
            .collect(),
    )
}

#[test]
fn a_sequence_of_one_kind_of_scalar_is_one_vector_at_any_length() {
    assert_eq!(
        node(&vec!["1".to_owned()]),
        Node::Seq(Seq::Strs(vec!["1".to_owned()]))
    );
    assert_eq!(
        node(&vec![1.5, -0.0]),
        Node::Seq(Seq::Reals(vec![1.5, -0.0]))
    );
    assert_eq!(node(&vec![true]), Node::Seq(Seq::Bools(vec![true])));
    assert_eq!(node(&vec![1u8, 2]), Node::Seq(Seq::Ints(vec![1, 2])));
    assert_eq!(node(&vec![i32::MIN]), Node::Seq(Seq::Ints(vec![i32::MIN])));
    // A string and a scalar are not a sequence.
    assert_eq!(node(&"1"), Node::Str("1".to_owned()));
    assert_eq!(node(&2.5f32), Node::Real(2.5));
}

#[test]
fn a_long_vector_of_floats_collects_into_one_rust_vector() {
    let values: Vec<f64> = (0..100_000).map(f64::from).collect();
    match node(&values) {
        Node::Seq(Seq::Reals(v)) => assert_eq!(v, values),
        other => panic!("{other:?}"),
    }
}

#[test]
fn integers_outside_i32_are_decimal_and_a_sequence_of_them_stays_one_vector() {
    assert_eq!(node(&i32::MIN), Node::Int(i32::MIN));
    assert_eq!(node(&5u64), Node::Int(5));
    assert_eq!(node(&u32::MAX), Node::Wide("4294967295".to_owned()));
    assert_eq!(
        node(&9_007_199_254_740_993u64),
        Node::Wide("9007199254740993".to_owned())
    );
    assert_eq!(
        node(&i128::MIN),
        Node::Wide("-170141183460469231731687303715884105728".to_owned())
    );
    assert_eq!(node(&u128::MAX), Node::Wide(u128::MAX.to_string()));
    assert_eq!(node(&vec![1i64, 2]), Node::Seq(Seq::Ints(vec![1, 2])));
    assert_eq!(
        node(&vec![1i64, i64::MAX, -3]),
        Node::Seq(Seq::Wides(vec![
            "1".to_owned(),
            i64::MAX.to_string(),
            "-3".to_owned()
        ]))
    );
    assert_eq!(
        node(&vec![u64::MAX]),
        Node::Seq(Seq::Wides(vec![u64::MAX.to_string()]))
    );
}

#[test]
fn a_none_a_nested_value_or_a_mix_makes_the_sequence_a_list() {
    assert_eq!(node(&Vec::<f64>::new()), Node::Seq(Seq::List(vec![])));
    // Every element `Some`: the elements are scalars.
    assert_eq!(
        node(&vec![Some(1.0), Some(2.0)]),
        Node::Seq(Seq::Reals(vec![1.0, 2.0]))
    );
    assert_eq!(
        node(&vec![Some(1.0), None]),
        Node::Seq(Seq::List(vec![Node::Real(1.0), Node::Null]))
    );
    assert_eq!(
        node(&vec![vec![1.0]]),
        Node::Seq(Seq::List(vec![Node::Seq(Seq::Reals(vec![1.0]))]))
    );
    assert_eq!(
        node(&vec!["a".to_owned(), "b\0c".to_owned()]),
        Node::Seq(Seq::List(vec![
            Node::Str("a".to_owned()),
            Node::NulStr(b"b\0c".to_vec())
        ]))
    );
    // A list of integers, one outside `i32`, keeps each as its own node.
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Number {
        Int(i64),
        Text(&'static str),
    }
    assert_eq!(
        node(&vec![
            Number::Int(1),
            Number::Int(i64::MAX),
            Number::Text("x")
        ]),
        Node::Seq(Seq::List(vec![
            Node::Int(1),
            Node::Wide(i64::MAX.to_string()),
            Node::Str("x".to_owned())
        ]))
    );
}

#[test]
fn some_is_marked_only_where_its_value_is_the_none_encoding() {
    assert_eq!(node(&None::<Option<f64>>), Node::Null);
    assert_eq!(node(&Some(None::<f64>)), Node::Some(Box::new(Node::Null)));
    assert_eq!(node(&Some(Some(1.0))), Node::Real(1.0));
    assert_eq!(
        node(&Some(Some(None::<f64>))),
        Node::Some(Box::new(Node::Some(Box::new(Node::Null))))
    );
    assert_eq!(node(&Some(())), Node::Some(Box::new(Node::Null)));
}

#[test]
fn a_map_is_a_named_list_only_when_every_key_is_a_string() {
    let by_name: BTreeMap<String, u8> = [("a".to_owned(), 1)].into();
    assert_eq!(node(&by_name), named(vec![("a", Node::Int(1))]));
    assert_eq!(node(&BTreeMap::<String, u8>::new()), Node::Named(vec![]));

    let by_id: BTreeMap<u32, f64> = [(1, 0.5), (u32::MAX, 1.5)].into();
    assert_eq!(
        node(&by_id),
        Node::Map {
            keys: Seq::Wides(vec!["1".to_owned(), "4294967295".to_owned()]),
            values: Seq::Reals(vec![0.5, 1.5]),
        }
    );

    let by_profile: BTreeMap<Vec<(String, String)>, usize> =
        [(vec![("ID".to_owned(), "1".to_owned())], 3)].into();
    assert_eq!(
        node(&by_profile),
        Node::Map {
            keys: Seq::List(vec![Node::Seq(Seq::List(vec![Node::List(vec![
                Node::Str("ID".to_owned()),
                Node::Str("1".to_owned())
            ])]))]),
            values: Seq::Ints(vec![3]),
        }
    );

    let with_nul: BTreeMap<String, u8> = [("a\0b".to_owned(), 1)].into();
    assert_eq!(
        node(&with_nul),
        Node::Map {
            keys: Seq::List(vec![Node::NulStr(b"a\0b".to_vec())]),
            values: Seq::Ints(vec![1]),
        }
    );
}

#[test]
fn variants_and_tuples() {
    #[derive(Serialize)]
    enum Shape {
        Empty,
        Circle(f64),
        Pair(u8, u8),
        Rect { w: f64 },
    }
    assert_eq!(node(&Shape::Empty), Node::Str("Empty".to_owned()));
    assert_eq!(
        node(&Shape::Circle(1.0)),
        named(vec![("Circle", Node::Real(1.0))])
    );
    assert_eq!(
        node(&Shape::Pair(1, 2)),
        named(vec![("Pair", Node::List(vec![Node::Int(1), Node::Int(2)]))])
    );
    assert_eq!(
        node(&Shape::Rect { w: 2.0 }),
        named(vec![("Rect", named(vec![("w", Node::Real(2.0))]))])
    );
    assert_eq!(
        node(&(1u8, "a")),
        Node::List(vec![Node::Int(1), Node::Str("a".to_owned())])
    );
}

#[test]
fn a_name_holding_a_nul_byte_is_refused() {
    #[derive(Serialize)]
    struct Odd {
        #[serde(rename = "a\0b")]
        field: u8,
    }
    let err = Odd { field: 1 }
        .serialize(NodeSerializer)
        .expect_err("an R name can't hold a NUL byte");
    assert!(err.to_string().contains("holds a NUL byte"), "{err}");
}
