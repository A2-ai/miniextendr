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

#[test]
fn mxl303_case_fold_vtable_collision() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Two distinct traits differing only in case both upper-case to `COUNTER`,
    // so both emit `__VTABLE_COUNTER_FOR_FOO` → duplicate #[no_mangle] symbol.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        impl Counter for Foo {
            fn value(&self) -> i32 { 0 }
        }

        #[miniextendr]
        impl counter for Foo {
            fn value(&self) -> i32 { 0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    let hits: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| format!("{}", d.code) == "MXL303")
        .collect();
    assert_eq!(
        hits.len(),
        2,
        "expected MXL303 to fire on both colliding impls, got: {:?}",
        report.diagnostics
    );
    assert!(
        hits.iter()
            .any(|d| d.message.contains("__VTABLE_COUNTER_FOR_FOO")),
        "expected the collided vtable symbol in the message, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl303_no_collision_for_distinct_names() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Distinct trait names and distinct type names → distinct vtable symbols.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        impl Counter for Foo {
            fn value(&self) -> i32 { 0 }
        }

        #[miniextendr]
        impl Resettable for Foo {
            fn reset(&mut self) {}
        }

        #[miniextendr]
        impl Counter for Bar {
            fn value(&self) -> i32 { 0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL303"),
        "MXL303 must not fire on distinct trait/type names, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl303_suppressed_by_allow_comment() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Same collision as the positive case, but the second impl carries the
    // escape-hatch comment, so only the first impl reports.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        impl Counter for Foo {
            fn value(&self) -> i32 { 0 }
        }

        // mxl::allow(MXL303)
        #[miniextendr]
        impl counter for Foo {
            fn value(&self) -> i32 { 0 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    let hits = report
        .diagnostics
        .iter()
        .filter(|d| format!("{}", d.code) == "MXL303")
        .count();
    assert_eq!(
        hits, 1,
        "the allow-comment should suppress one of the two colliding impls, got: {:?}",
        report.diagnostics
    );
}

// endregion

// region: MXL302 — into_sexp() inside a vec!/array literal (use-after-free idiom)

#[test]
fn mxl302_into_sexp_inside_vec_literal_fires() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let _ = List::from_raw_pairs(vec![
                ("a", a.into_sexp()),
                ("b", b.into_sexp()),
            ]);
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL302"),
        "expected MXL302 for into_sexp() inside vec! literal, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_single_line_pair_literal_fires() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // The minimal positive shape from the issue: `vec![ (k, into_sexp(v)) ]`.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let _ = List::from_raw_pairs(vec![("a", self.a.into_sexp()), ("b", self.b.into_sexp())]);
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL302"),
        "expected MXL302 for single-line pair literal with into_sexp, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_unchecked_variant_fires() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let _ = vec![unsafe { a.into_sexp_unchecked() }, unsafe { b.into_sexp_unchecked() }];
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL302"),
        "expected MXL302 for into_sexp_unchecked inside vec! literal, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_whole_vec_into_sexp_does_not_fire() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // SAFE: into_sexp() is called on the *whole* vec, after the literal closes — the entire
    // Vec is converted as a single SEXP, with no sibling unprotected SEXPs.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let _sexp = vec![self.start, self.end].into_sexp();
            let _other = vec![1i32, 2, 3].into_sexp();
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL302"),
        "MXL302 must NOT fire on the safe `vec![..].into_sexp()` whole-vec form, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_no_literal_does_not_fire() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // SAFE: plain `into_sexp()` outside any literal.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let s = self.value.into_sexp();
            let _ = s;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL302"),
        "MXL302 must NOT fire on a bare into_sexp() outside any literal, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_protected_builder_does_not_fire() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // SAFE-by-construction: this is the exact form the `IntoList` / `DataFrameRow` derives
    // emit — every element's `into_sexp()` is wrapped in `__scope.protect_raw(...)`, so the
    // value is rooted as it is built. MXL302 must treat this as a true negative.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            unsafe {
                let __scope = ProtectScope::new();
                let _ = List::from_raw_pairs(vec![
                    ("a", __scope.protect_raw(self.a.into_sexp())),
                    ("b", __scope.protect_raw(self.b.into_sexp())),
                ]);
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
            .all(|d| format!("{}", d.code) != "MXL302"),
        "MXL302 must NOT fire on the protected builder form (protect_raw-wrapped into_sexp), got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_hoisted_protected_vars_does_not_fire() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // SAFE: into_sexp() is hoisted out of the literal entirely into protected variables.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let scope = ProtectScope::new();
            let a = scope.protect_raw(self.a.into_sexp());
            let b = scope.protect_raw(self.b.into_sexp());
            let _ = List::from_raw_pairs(vec![("a", a), ("b", b)]);
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL302"),
        "MXL302 must NOT fire when into_sexp() is hoisted out of the literal into protected vars, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn mxl302_suppressed_by_allow_comment() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            // mxl::allow(MXL302)
            let _ = List::from_raw_pairs(vec![("a", a.into_sexp()), ("b", b.into_sexp())]);
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| format!("{}", d.code) != "MXL302"),
        "MXL302 must be suppressed by `// mxl::allow(MXL302)`, got: {:?}",
        report.diagnostics
    );
}

// endregion
