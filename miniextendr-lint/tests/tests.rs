use miniextendr_lint::{lint_enabled, run};
use std::fs;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn lint_enabled_respects_env() {
    let _guard = ENV_LOCK.lock().unwrap();

    unsafe { std::env::set_var("MINIEXTENDR_LINT", "0") };
    assert!(!lint_enabled("MINIEXTENDR_LINT").unwrap());

    unsafe { std::env::set_var("MINIEXTENDR_LINT", "off") };
    assert!(!lint_enabled("MINIEXTENDR_LINT").unwrap());

    unsafe { std::env::set_var("MINIEXTENDR_LINT", "1") };
    assert!(lint_enabled("MINIEXTENDR_LINT").unwrap());

    unsafe { std::env::set_var("MINIEXTENDR_LINT", "yes") };
    assert!(lint_enabled("MINIEXTENDR_LINT").unwrap());
}

#[test]
fn no_errors_for_simple_miniextendr_fn() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        pub fn hello() -> String { "world".to_string() }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.is_empty(),
        "simple #[miniextendr] fn should have no errors, got: {:?}",
        report.errors
    );
}

#[test]
fn mxl106_non_pub_fn_with_export_tag() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        /// @export
        #[miniextendr]
        fn not_pub() -> i32 { 42 }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL106"),
        "expected MXL106 warning for non-pub fn with @export, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl009_multiple_impl_blocks_without_labels() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        #[miniextendr]
        impl Counter {
            fn get_value(&self) -> i32 { self.value }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.iter().any(|e| e.contains("missing labels")),
        "expected MXL009 error for missing labels, got: {:?}",
        report.errors
    );
}

#[test]
fn mxl010_duplicate_labels() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(label = "ops")]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        #[miniextendr(label = "ops")]
        impl Counter {
            fn get_value(&self) -> i32 { self.value }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.iter().any(|e| e.contains("duplicate label")),
        "expected MXL010 error for duplicate labels, got: {:?}",
        report.errors
    );
}

#[test]
fn mxl203_internal_plus_noexport_redundancy() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(internal, noexport)]
        pub fn helper() -> i32 { 42 }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL203"),
        "expected MXL203 for internal+noexport redundancy, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn no_errors_for_labeled_impl_blocks() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(label = "constructors")]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        #[miniextendr(label = "methods")]
        impl Counter {
            fn get_value(&self) -> i32 { self.value }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.is_empty(),
        "properly labeled impl blocks should not error, got: {:?}",
        report.errors
    );
}

#[test]
fn follows_mod_declarations() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        mod child;
        "#,
    )
    .unwrap();

    fs::write(
        src_dir.join("child.rs"),
        r#"
        #[miniextendr]
        pub fn from_child() -> i32 { 1 }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert_eq!(
        report.files.len(),
        2,
        "should scan both lib.rs and child.rs"
    );
    assert!(
        report.errors.is_empty(),
        "child module with #[miniextendr] should be fine, got: {:?}",
        report.errors
    );
}

#[test]
fn mxl008_class_system_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(r6)]
        impl MyType {
            fn new() -> Self { MyType }
        }

        #[miniextendr]
        impl MyTrait for MyType {
            fn method(&self) -> i32 { 42 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("MXL008") || e.contains("class system")),
        "expected MXL008 error for class system mismatch, got: {:?}",
        report.errors
    );
}

#[test]
fn mxl008_s3_trait_on_s4_inherent_is_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(s4)]
        impl MyType {
            fn new() -> Self { MyType }
        }

        #[miniextendr(s3)]
        impl MyTrait for MyType {
            fn method(&self) -> i32 { 42 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.is_empty(),
        "S3 trait on S4 inherent should be allowed (S3 dispatch works on any class), got: {:?}",
        report.errors
    );
}

#[test]
fn mxl300_rf_error_usage() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        pub fn bad_error() {
            unsafe { Rf_error(c"oops".as_ptr()) };
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL300"),
        "expected MXL300 warning for direct Rf_error call, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl111_s4_prefixed_method_fires() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(s4)]
        impl Foo {
            pub fn s4_compute(&self) -> i32 { 0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL111"),
        "expected MXL111 warning for s4_-prefixed method on s4 impl, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl111_no_fire_for_non_s4_impl() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(r6)]
        impl Bar {
            pub fn s4_compute(&self) -> i32 { 0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL111"),
        "MXL111 must not fire on r6 impl with s4_-prefixed method, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl111_no_fire_for_standalone_fn() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        pub fn s4_helper() -> i32 { 0 }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL111"),
        "MXL111 must not fire on standalone fn named s4_*, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl111_no_fire_for_s4_constructor() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(s4)]
        impl Foo {
            pub fn new() -> Self { Foo {} }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL111"),
        "MXL111 must not fire on `new` constructor of s4 impl, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl301_ffi_unchecked_usage() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        pub fn bad_unchecked() {
            unsafe { sys::Rf_allocVector_unchecked(INTSXP, 10) };
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL301"),
        "expected MXL301 warning for _unchecked FFI call, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl110_r_reserved_word_as_param() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        pub fn bad(repeat: i32) -> i32 { repeat }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL110"),
        "expected MXL110 error for R reserved word `repeat` as param, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl110_good_param_name_no_error() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        pub fn good(n: i32) -> i32 { n }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL110"),
        "expected no MXL110 for ordinary param name, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl110_r_reserved_word_as_method_param() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(r6)]
        impl Foo {
            pub fn n(&self, repeat: i32) -> i32 { repeat }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL110"),
        "expected MXL110 error for R reserved word `repeat` as method param, got: {:?}",
        report.diagnostics
    );
}

// region: MXL120 — vctrs Self-returning constructor / instance receiver

#[test]
fn mxl120_vctrs_ctor_returns_self() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Self { MyVec { values } }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 error for vctrs ctor returning Self, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_ctor_returns_result_self() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Result<Self, String> {
                Ok(MyVec { values })
            }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 error for vctrs ctor returning Result<Self, _>, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_instance_method_ref_self() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Vec<f64> { values }
            pub fn value(&self) -> f64 { 0.0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 error for vctrs instance method &self, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_instance_method_external_ptr() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Vec<f64> { values }
            pub fn value(self: &ExternalPtr<Self>) -> f64 { 0.0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 error for vctrs instance method with ExternalPtr<Self> receiver, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_no_fire_for_valid_vctrs_impl() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // All-static methods, constructor returns Vec<f64> — clean.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Vec<f64> { values }
            pub fn scale(amounts: Vec<f64>, factor: f64) -> Vec<f64> {
                amounts.into_iter().map(|v| v * factor).collect()
            }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL120"),
        "MXL120 must not fire on valid vctrs impl, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_no_fire_for_r6_ctor_returns_self() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Non-vctrs class system: MXL120 must NOT fire even if constructor returns Self.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(r6)]
        impl MyR6 {
            pub fn new() -> Self { MyR6 {} }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL120"),
        "MXL120 must not fire on non-vctrs impl (r6), got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_ctor_returns_box_self() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Box<Self> { Box::new(MyVec { values }) }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 error for vctrs ctor returning Box<Self>, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_ctor_returns_named_type() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Named-type return (`fn new() -> MyVec`) exercises the `last.ident == type_name` branch.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> MyVec { MyVec { values } }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 error for vctrs ctor returning named type MyVec, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_instance_method_externalptr_value_receiver() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // `self: ExternalPtr<Self>` (by-value) — distinct from `self: &ExternalPtr<Self>` (ref).
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Vec<f64> { values }
            pub fn consume(self: ExternalPtr<Self>) -> f64 { 0.0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 for vctrs method with ExternalPtr<Self> by-value receiver, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_vctrs_constructor_tag_on_non_new() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // `#[miniextendr(constructor)]` on a non-`new` method returning Self — exercises
    // the `has_constructor_attr` path at `vctrs_self_ctor.rs`.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Vec<f64> { values }
            #[miniextendr(constructor)]
            pub fn from_raw(values: Vec<f64>) -> Self { MyVec { values } }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL120"),
        "expected MXL120 for vctrs #[miniextendr(constructor)] method returning Self, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl120_no_false_positive_for_consuming_self_on_vctrs() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Consuming `self` (Value) is NOT an instance receiver per the macro's `is_instance()`.
    // With `#[miniextendr(constructor)]` and a non-Self return, the macro allows it.
    // MXL120 Check 2 must NOT fire here — this verifies `is_instance()` parity with
    // `ReceiverKind::is_instance` in `miniextendr-macros` (which excludes `Value`).
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(vctrs(vctr))]
        impl MyVec {
            pub fn new(values: Vec<f64>) -> Vec<f64> { values }
            #[miniextendr(constructor)]
            pub fn convert(self) -> Vec<f64> { vec![] }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    // Check 2 (instance receiver) must NOT fire.  Check 1 (ctor return type) also must
    // not fire because `Vec<f64>` is not Self/named-type.
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL120"),
        "MXL120 must not fire on consuming `self` with constructor attr + Vec<f64> return, got: {:?}",
        report.diagnostics
    );
}

// endregion
