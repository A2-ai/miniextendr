use super::*;

#[test]
fn test_vctrs_build_error_display() {
    assert_eq!(
        VctrsBuildError::NotAVector.to_string(),
        "data is not a vector type (atomic, list, or expression)"
    );
    assert_eq!(
        VctrsBuildError::ListRequiresInheritBaseType.to_string(),
        "list data requires inherit_base_type = true (or None for default)"
    );
    assert_eq!(
        VctrsBuildError::FieldLengthMismatch {
            field: "x".to_string(),
            expected: 3,
            actual: 5
        }
        .to_string(),
        "field 'x' has length 5 but expected 3"
    );
    assert_eq!(
        VctrsBuildError::EmptyRecord.to_string(),
        "record must have at least one field"
    );
    assert_eq!(
        VctrsBuildError::DuplicateFieldName {
            name: "x".to_string()
        }
        .to_string(),
        "duplicate field name: 'x'"
    );
    assert_eq!(
        VctrsBuildError::UnnamedFields.to_string(),
        "record fields must be named"
    );
    assert_eq!(
        VctrsBuildError::MissingPtypeOrSize.to_string(),
        "list_of requires at least one of ptype or size"
    );
    assert_eq!(
        VctrsBuildError::InvalidSize { size: -1 }.to_string(),
        "invalid size: -1 (must be non-negative)"
    );
    assert_eq!(
        VctrsBuildError::EmptyClass.to_string(),
        "class vector must not be empty"
    );
}

// region: Phase C: VctrsKind tests

#[test]
fn test_vctrs_kind_default() {
    assert_eq!(VctrsKind::default(), VctrsKind::Vctr);
}

#[test]
fn test_vctrs_kind_eq() {
    assert_eq!(VctrsKind::Vctr, VctrsKind::Vctr);
    assert_eq!(VctrsKind::Rcrd, VctrsKind::Rcrd);
    assert_eq!(VctrsKind::ListOf, VctrsKind::ListOf);
    assert_ne!(VctrsKind::Vctr, VctrsKind::Rcrd);
    assert_ne!(VctrsKind::Rcrd, VctrsKind::ListOf);
}

#[test]
fn test_vctrs_kind_clone() {
    let kind = VctrsKind::Rcrd;
    let cloned = kind;
    assert_eq!(kind, cloned);
}

#[test]
fn test_vctrs_kind_debug() {
    assert_eq!(format!("{:?}", VctrsKind::Vctr), "Vctr");
    assert_eq!(format!("{:?}", VctrsKind::Rcrd), "Rcrd");
    assert_eq!(format!("{:?}", VctrsKind::ListOf), "ListOf");
}
// endregion

// region: Phase C: VctrsClass trait tests (compile-time verification)

// Test struct implementing VctrsClass
struct TestPercent;

impl VctrsClass for TestPercent {
    const CLASS_NAME: &'static str = "test_percent";
    const KIND: VctrsKind = VctrsKind::Vctr;
    const BASE_TYPE: Option<SEXPTYPE> = Some(SEXPTYPE::REALSXP);
    const INHERIT_BASE_TYPE: bool = false;
    const ABBR: Option<&'static str> = Some("pct");
}

#[test]
fn test_vctrs_class_constants() {
    assert_eq!(TestPercent::CLASS_NAME, "test_percent");
    assert_eq!(TestPercent::KIND, VctrsKind::Vctr);
    assert_eq!(TestPercent::BASE_TYPE, Some(SEXPTYPE::REALSXP));
    const { assert!(!TestPercent::INHERIT_BASE_TYPE) };
    assert_eq!(TestPercent::ABBR, Some("pct"));
}

#[test]
fn test_vctrs_class_default_methods() {
    // Test default implementations
    assert!(TestPercent::additional_classes().is_empty());
    let instance = TestPercent;
    assert!(instance.attrs().is_empty());
}

// Test struct implementing VctrsRecord
struct TestRational;

impl VctrsClass for TestRational {
    const CLASS_NAME: &'static str = "test_rational";
    const KIND: VctrsKind = VctrsKind::Rcrd;
}

impl VctrsRecord for TestRational {
    fn field_names() -> &'static [&'static str] {
        &["n", "d"]
    }
}

#[test]
fn test_vctrs_record_trait() {
    assert_eq!(TestRational::CLASS_NAME, "test_rational");
    assert_eq!(TestRational::KIND, VctrsKind::Rcrd);
    assert_eq!(TestRational::field_names(), &["n", "d"]);
}

// Test struct implementing VctrsListOf
struct TestIntList;

impl VctrsClass for TestIntList {
    const CLASS_NAME: &'static str = "test_int_list";
    const KIND: VctrsKind = VctrsKind::ListOf;
}

impl VctrsListOf for TestIntList {
    fn ptype_expr() -> &'static str {
        "integer()"
    }

    fn fixed_size() -> Option<i32> {
        Some(3)
    }
}

#[test]
fn test_vctrs_list_of_trait() {
    assert_eq!(TestIntList::CLASS_NAME, "test_int_list");
    assert_eq!(TestIntList::KIND, VctrsKind::ListOf);
    assert_eq!(TestIntList::ptype_expr(), "integer()");
    assert_eq!(TestIntList::fixed_size(), Some(3));
}

// Test VctrsListOf with default fixed_size
struct TestStringList;

impl VctrsClass for TestStringList {
    const CLASS_NAME: &'static str = "test_string_list";
    const KIND: VctrsKind = VctrsKind::ListOf;
}

impl VctrsListOf for TestStringList {
    fn ptype_expr() -> &'static str {
        "character()"
    }
}

#[test]
fn test_vctrs_list_of_default_size() {
    assert_eq!(TestStringList::fixed_size(), None);
}

// Test VctrsClass with custom additional_classes
struct TestSubPercent;

impl VctrsClass for TestSubPercent {
    const CLASS_NAME: &'static str = "test_sub_percent";
    const KIND: VctrsKind = VctrsKind::Vctr;

    fn additional_classes() -> &'static [&'static str] {
        &["test_percent"]
    }
}

#[test]
fn test_vctrs_class_with_additional_classes() {
    assert_eq!(TestSubPercent::CLASS_NAME, "test_sub_percent");
    assert_eq!(TestSubPercent::additional_classes(), &["test_percent"]);
}
// endregion
