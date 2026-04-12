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

// region: Custom AltrepExtract (non-TypedExternal) — compile-time proof

/// A type that does NOT implement TypedExternal, but manually implements AltrepExtract.
/// This proves the trait is usable for custom storage strategies (e.g., storing data
/// directly in R SEXPs rather than via ExternalPtr).
struct CustomStorageData {
    #[allow(dead_code)]
    values: Vec<i32>,
}

/// Manual AltrepExtract: the type extracts itself from an ALTREP SEXP without ExternalPtr.
/// This is a compile-time test only — runtime testing requires a full ALTREP registration
/// path that bypasses ExternalPtr, which is future work.
impl AltrepExtract for CustomStorageData {
    unsafe fn altrep_extract_ref(x: miniextendr_api::ffi::SEXP) -> &'static Self {
        // In a real implementation, this would extract data from the SEXP directly
        // (e.g., from an INTSXP stored in data1). For compile-time testing, we just
        // prove the trait is implementable.
        let _ = x;
        panic!("compile-time only — not callable without R runtime")
    }

    unsafe fn altrep_extract_mut(x: miniextendr_api::ffi::SEXP) -> &'static mut Self {
        let _ = x;
        panic!("compile-time only — not callable without R runtime")
    }
}

#[test]
fn custom_altrep_extract_compiles() {
    // Proves AltrepExtract is implementable without TypedExternal
    assert_altrep_extract::<CustomStorageData>();
}

// endregion
