//! Regression tests for nested-Vec serde round-tripping.
//!
//! `SeqSerializer::end()` calls `List::from_scalars_or_list`, which coalesces
//! length-1 same-typed scalars into an R atomic vector. That means
//! `Vec<Vec<i32>>` with all-length-1 inner vecs serializes to a flat INTSXP.
//! Without symmetric handling on the deserializer side, the inner
//! `deserialize_seq` is called on a scalar `VectorElementDeserializer` and
//! fails with "invalid type: integer N, expected a sequence".
//!
//! These tests exercise the round-trip for that and related shapes.

#![cfg(feature = "serde")]

mod r_test_utils;

use miniextendr_api::serde::{from_r, to_r};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithNestedVecUsize {
    nested: Vec<Vec<usize>>,
}

#[test]
fn nested_vec_usize_all_singletons() {
    r_test_utils::with_r_thread(|| {
        let value = WithNestedVecUsize {
            nested: vec![vec![56], vec![73], vec![158]],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithNestedVecUsize = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[test]
fn nested_vec_usize_singleton_outer() {
    r_test_utils::with_r_thread(|| {
        let value = WithNestedVecUsize {
            nested: vec![vec![42]],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithNestedVecUsize = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[test]
fn nested_vec_usize_mixed_lengths() {
    r_test_utils::with_r_thread(|| {
        let value = WithNestedVecUsize {
            nested: vec![vec![1], vec![2, 3], vec![4, 5, 6]],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithNestedVecUsize = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[test]
fn nested_vec_usize_empty_inner() {
    r_test_utils::with_r_thread(|| {
        let value = WithNestedVecUsize {
            nested: vec![vec![], vec![1], vec![]],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithNestedVecUsize = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithNestedVecF64 {
    nested: Vec<Vec<f64>>,
}

#[test]
fn nested_vec_f64_all_singletons() {
    r_test_utils::with_r_thread(|| {
        let value = WithNestedVecF64 {
            nested: vec![vec![1.5], vec![2.5], vec![3.5]],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithNestedVecF64 = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithNestedVecString {
    nested: Vec<Vec<String>>,
}

#[test]
fn nested_vec_string_all_singletons() {
    r_test_utils::with_r_thread(|| {
        let value = WithNestedVecString {
            nested: vec![
                vec!["a".to_string()],
                vec!["b".to_string()],
                vec!["c".to_string()],
            ],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithNestedVecString = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithTriplyNested {
    deep: Vec<Vec<Vec<i32>>>,
}

#[test]
fn triply_nested_vec_all_singletons() {
    r_test_utils::with_r_thread(|| {
        let value = WithTriplyNested {
            deep: vec![vec![vec![1]], vec![vec![2]]],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithTriplyNested = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct WithTupleOfScalars {
    pairs: Vec<(i32, i32)>,
}

#[test]
fn vec_of_tuple_pairs_roundtrips() {
    r_test_utils::with_r_thread(|| {
        let value = WithTupleOfScalars {
            pairs: vec![(1, 2), (3, 4), (5, 6)],
        };
        let sexp = to_r(&value).expect("serialize");
        let back: WithTupleOfScalars = from_r(sexp).expect("deserialize");
        assert_eq!(back, value);
    });
}
