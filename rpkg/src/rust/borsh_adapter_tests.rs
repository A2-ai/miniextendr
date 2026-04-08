//! Borsh adapter tests -- binary serialization via R raw vectors.

use miniextendr_api::borsh_impl::{borsh_from_raw, borsh_to_raw};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::{Borsh, IntoR, RBorshOps, TryFromSexp, miniextendr};

/// Test roundtripping a double vector through Borsh serialization and deserialization.
/// @param values Numeric vector to serialize.
#[miniextendr]
pub fn borsh_roundtrip_doubles(values: Vec<f64>) -> Vec<f64> {
    let raw: SEXP = borsh_to_raw(&values);
    borsh_from_raw::<Vec<f64>>(raw).unwrap()
}

/// Test roundtripping a string through the Borsh SEXP wrapper conversion path.
/// @param input String to serialize.
#[miniextendr]
pub fn borsh_roundtrip_string(input: String) -> String {
    let sexp = Borsh(input.clone()).into_sexp();
    let Borsh(recovered) = Borsh::<String>::try_from_sexp(sexp).unwrap();
    recovered
}

/// Test Borsh serialized byte length of a tuple (i32, String, bool).
#[miniextendr]
pub fn borsh_tuple_size() -> i32 {
    let value: (i32, String, bool) = (42, "hello".to_string(), true);
    value.borsh_size() as i32
}

/// Test roundtripping a nested structure through Borsh serialization.
#[miniextendr]
pub fn borsh_nested_roundtrip() -> bool {
    let original: Vec<(String, Vec<u8>)> = vec![
        ("alpha".to_string(), vec![1, 2, 3]),
        ("beta".to_string(), vec![4, 5]),
    ];
    let bytes = original.borsh_serialize();
    let recovered: Vec<(String, Vec<u8>)> = RBorshOps::borsh_deserialize(&bytes).unwrap();
    original == recovered
}

/// Test that deserializing invalid bytes returns an error message.
#[miniextendr]
pub fn borsh_invalid_data() -> String {
    let bad: &[u8] = &[0xff, 0xff];
    match <String as RBorshOps>::borsh_deserialize(bad) {
        Ok(_) => "unexpected success".to_string(),
        Err(e) => e,
    }
}

/// Test roundtripping Option<i32> values (Some and None) through Borsh.
#[miniextendr]
pub fn borsh_option_roundtrip() -> bool {
    let some_val: Option<i32> = Some(99);
    let none_val: Option<i32> = None;
    let some_bytes = some_val.borsh_serialize();
    let none_bytes = none_val.borsh_serialize();
    let r_some: Option<i32> = RBorshOps::borsh_deserialize(&some_bytes).unwrap();
    let r_none: Option<i32> = RBorshOps::borsh_deserialize(&none_bytes).unwrap();
    r_some == some_val && r_none == none_val
}

// region: Upstream example-derived fixtures

/// Test roundtripping a HashMap<String, i32> through Borsh.
#[miniextendr]
pub fn borsh_hashmap_roundtrip() -> bool {
    use std::collections::HashMap;
    let mut original = HashMap::new();
    original.insert("alpha".to_string(), 1i32);
    original.insert("beta".to_string(), 2);
    original.insert("gamma".to_string(), 3);
    let bytes = original.borsh_serialize();
    let recovered: HashMap<String, i32> = RBorshOps::borsh_deserialize(&bytes).unwrap();
    original == recovered
}

/// Test roundtripping a Vec<bool> through Borsh.
#[miniextendr]
pub fn borsh_vec_bool_roundtrip() -> bool {
    let original = vec![true, false, true, true, false];
    let bytes = original.borsh_serialize();
    let recovered: Vec<bool> = RBorshOps::borsh_deserialize(&bytes).unwrap();
    original == recovered
}

// endregion
