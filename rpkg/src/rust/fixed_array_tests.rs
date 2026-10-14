//! Fixed-array returns use the same conversion as slices (#2058).

use miniextendr_api::miniextendr;

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_native() -> [i32; 2] {
    [1, 2]
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_native_vec() -> Vec<i32> {
    fixed_array_native().to_vec()
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_i64(big: bool) -> [i64; 2] {
    [1, if big { 3_000_000_000 } else { 2 }]
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_i64_vec(big: bool) -> Vec<i64> {
    fixed_array_i64(big).to_vec()
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_strings() -> [String; 2] {
    ["one".into(), "two".into()]
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_strings_vec() -> Vec<String> {
    Vec::from(fixed_array_strings())
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_bools() -> [bool; 2] {
    [true, false]
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_bools_vec() -> Vec<bool> {
    fixed_array_bools().to_vec()
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_empty_strings() -> [String; 0] {
    []
}

/// @noRd
#[miniextendr(noexport)]
pub fn fixed_array_optional_i64(present: bool) -> Option<[i64; 2]> {
    present.then_some([1, 3_000_000_000])
}
