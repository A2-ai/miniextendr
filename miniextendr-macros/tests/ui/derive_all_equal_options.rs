//! Compile-fail test: the options of `#[derive(AllEqual)]` (#2149). The
//! struct takes the settings only; a field takes the settings, `skip`,
//! `label` and `with`, and `skip` alone; a setting is a number or a bool of
//! its kind, given once; and a struct must have a field to compare.

use miniextendr_api::{AllEqual, ExternalPtr};

#[derive(ExternalPtr, AllEqual)]
struct UnknownFieldOption {
    #[all_equal(tolerence = 0)]
    value: f64,
}

#[derive(ExternalPtr, AllEqual)]
#[all_equal(label = "Struct")]
struct LabelOnStruct {
    value: f64,
}

#[derive(ExternalPtr, AllEqual)]
struct SkipWithLabel {
    #[all_equal(skip, label = "Value")]
    value: f64,
    other: f64,
}

#[derive(ExternalPtr, AllEqual)]
#[all_equal(tolerance = -1)]
struct NegativeTolerance {
    value: f64,
}

#[derive(ExternalPtr, AllEqual)]
struct NameCheckNotBool {
    #[all_equal(check_names = "no")]
    value: f64,
}

#[derive(ExternalPtr, AllEqual)]
struct ToleranceTwice {
    #[all_equal(tolerance = 0, tolerance = 1)]
    value: f64,
}

#[derive(ExternalPtr, AllEqual)]
struct WithNotAPath {
    #[all_equal(with = "not a path")]
    value: f64,
}

#[derive(ExternalPtr, AllEqual)]
struct NothingCompared {
    #[all_equal(skip)]
    value: f64,
}

fn main() {}
