use miniextendr_api::miniextendr;

// region: Box<[T]> roundtrip tests

/// `Box<[f64]>` roundtrip: R numeric → Rust → R numeric
#[miniextendr]
pub fn box_slice_f64_roundtrip(x: Box<[f64]>) -> Box<[f64]> {
    x
}

/// `Box<[i32]>` roundtrip: R integer → Rust → R integer
#[miniextendr]
pub fn box_slice_i32_roundtrip(x: Box<[i32]>) -> Box<[i32]> {
    x
}

/// `Box<[String]>` roundtrip: R character → Rust → R character
#[miniextendr]
pub fn box_slice_string_roundtrip(x: Box<[String]>) -> Box<[String]> {
    x
}

/// `Box<[bool]>` roundtrip: R logical → Rust → R logical
#[miniextendr]
pub fn box_slice_bool_roundtrip(x: Box<[bool]>) -> Box<[bool]> {
    x
}

/// `Box<[u8]>` roundtrip: R raw → Rust → R raw
#[miniextendr]
pub fn box_slice_raw_roundtrip(x: Box<[u8]>) -> Box<[u8]> {
    x
}

/// Transform: double each element using `Box<[f64]>`
#[miniextendr]
#[allow(clippy::boxed_local)]
pub fn box_slice_double(x: Box<[f64]>) -> Box<[f64]> {
    x.iter().map(|v| v * 2.0).collect()
}

/// `Box<[Option<f64>]>` with NA support
#[miniextendr]
pub fn box_slice_option_f64_roundtrip(x: Box<[Option<f64>]>) -> Vec<Option<f64>> {
    // Return as Vec to verify Box→Vec conversion path
    x.into_vec()
}

/// `Box<[Option<i32>]>` with NA support
#[miniextendr]
pub fn box_slice_option_i32_roundtrip(x: Box<[Option<i32>]>) -> Vec<Option<i32>> {
    x.into_vec()
}

/// `Box<[Option<String>]>` with NA support
#[miniextendr]
pub fn box_slice_option_string_roundtrip(x: Box<[Option<String>]>) -> Vec<Option<String>> {
    x.into_vec()
}

// endregion
