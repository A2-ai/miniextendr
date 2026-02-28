//! Borsh adapter tests — binary serialization via R raw vectors.

use miniextendr_api::borsh_impl::{borsh_from_raw, borsh_to_raw};
use miniextendr_api::ffi::SEXP;
use miniextendr_api::{Borsh, IntoR, RBorshOps, TryFromSexp, miniextendr, miniextendr_module};

/// Roundtrip a Vec<f64> through borsh serialization → R raw → borsh deserialization.
/// @noRd
#[miniextendr]
pub fn borsh_roundtrip_doubles(values: Vec<f64>) -> Vec<f64> {
    let raw: SEXP = borsh_to_raw(&values);
    borsh_from_raw::<Vec<f64>>(raw).unwrap()
}

/// Roundtrip a string through Borsh<T> wrapper (SEXP conversion path).
/// @noRd
#[miniextendr]
pub fn borsh_roundtrip_string(input: String) -> String {
    let sexp = Borsh(input.clone()).into_sexp();
    let Borsh(recovered) = Borsh::<String>::try_from_sexp(sexp).unwrap();
    recovered
}

/// Serialize a tuple (i32, String, bool) and return the raw byte length.
/// @noRd
#[miniextendr]
pub fn borsh_tuple_size() -> i32 {
    let value: (i32, String, bool) = (42, "hello".to_string(), true);
    value.borsh_size() as i32
}

/// Serialize then deserialize a nested structure.
/// Returns TRUE if roundtrip preserves the data.
/// @noRd
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

/// Attempt to deserialize invalid bytes — should return an error string.
/// @noRd
#[miniextendr]
pub fn borsh_invalid_data() -> String {
    let bad: &[u8] = &[0xff, 0xff];
    match <String as RBorshOps>::borsh_deserialize(bad) {
        Ok(_) => "unexpected success".to_string(),
        Err(e) => e,
    }
}

/// Roundtrip Option<i32> values (Some + None).
/// Returns TRUE if both roundtrip correctly.
/// @noRd
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

miniextendr_module! {
    mod borsh_adapter_tests;
    fn borsh_roundtrip_doubles;
    fn borsh_roundtrip_string;
    fn borsh_tuple_size;
    fn borsh_nested_roundtrip;
    fn borsh_invalid_data;
    fn borsh_option_roundtrip;
}
