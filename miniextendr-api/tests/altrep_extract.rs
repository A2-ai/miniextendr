//! Compile-time verification that AltrepExtract is implemented for key types.
//!
//! AltrepExtract has a blanket impl for all T: TypedExternal. This test ensures
//! that the blanket impl correctly covers the standard container types that users
//! would store in ALTREP data1 slots.

use miniextendr_api::altrep_data::AltrepExtract;

fn assert_altrep_extract<T: AltrepExtract>() {}

#[test]
fn altrep_extract_implemented_for_vec_types() {
    assert_altrep_extract::<Vec<i32>>();
    assert_altrep_extract::<Vec<f64>>();
    assert_altrep_extract::<Vec<u8>>();
    assert_altrep_extract::<Vec<bool>>();
    assert_altrep_extract::<Vec<String>>();
    assert_altrep_extract::<Vec<Option<String>>>();
    assert_altrep_extract::<Vec<Option<i32>>>();
    assert_altrep_extract::<Vec<Option<f64>>>();
}

#[test]
fn altrep_extract_implemented_for_box_slice_types() {
    assert_altrep_extract::<Box<[i32]>>();
    assert_altrep_extract::<Box<[f64]>>();
    assert_altrep_extract::<Box<[u8]>>();
    assert_altrep_extract::<Box<[bool]>>();
    assert_altrep_extract::<Box<[String]>>();
}

#[test]
fn altrep_extract_implemented_for_array_types() {
    assert_altrep_extract::<[i32; 5]>();
    assert_altrep_extract::<[f64; 10]>();
    assert_altrep_extract::<[u8; 3]>();
    assert_altrep_extract::<[bool; 1]>();
    assert_altrep_extract::<[String; 2]>();
}

#[test]
fn altrep_extract_implemented_for_static_slices() {
    assert_altrep_extract::<&'static [i32]>();
    assert_altrep_extract::<&'static [f64]>();
    assert_altrep_extract::<&'static [u8]>();
    assert_altrep_extract::<&'static [bool]>();
}

#[test]
fn altrep_extract_implemented_for_range_types() {
    assert_altrep_extract::<std::ops::Range<i32>>();
    assert_altrep_extract::<std::ops::Range<i64>>();
    assert_altrep_extract::<std::ops::Range<f64>>();
}

#[test]
fn altrep_extract_implemented_for_primitives() {
    // Scalars also implement TypedExternal, so AltrepExtract works
    assert_altrep_extract::<i32>();
    assert_altrep_extract::<f64>();
    assert_altrep_extract::<u8>();
    assert_altrep_extract::<bool>();
    assert_altrep_extract::<String>();
}
