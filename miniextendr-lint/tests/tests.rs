use miniextendr_lint::{build_directives, lint_enabled, run};
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

/// Bare impl flags after the class system (`no_preconditions`, `blanket`,
/// ...) are not class systems: an R6 inherent impl with one and an R6 trait
/// impl without it agree.
#[test]
fn mxl008_ignores_bare_flags_after_the_class_system() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(r6, noexport, no_preconditions)]
        impl MyType {
            fn new() -> Self { MyType }
        }

        #[miniextendr(r6, preconditions)]
        impl MyTrait for MyType {
            fn method(&self) -> i32 { 42 }
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.is_empty(),
        "both impls are R6, got: {:?}",
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

#[test]
fn mxl302_scanner_survives_multibyte_source() {
    // Regression: the raw-text scanner walked byte indices but sliced &str,
    // panicking mid-codepoint on non-ASCII source ("start byte index N is
    // not a char boundary"). Non-ASCII string literals inside a vec! literal
    // must scan cleanly and still flag the unprotected into_sexp element.
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        pub fn build() {
            let labels = vec![Some("α".to_string()), Some("β-日本語".to_string())];
            let _ = List::from_raw_pairs(vec![("α", a.into_sexp()), ("β", b.into_sexp())]);
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint must not panic on multibyte source");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| format!("{}", d.code) == "MXL302"),
        "expected MXL302 for into_sexp() inside a multibyte vec! literal, got: {:?}",
        report.diagnostics
    );
}

// endregion

// region: MXL304 — non-API ATTRIB / SET_ATTRIB

/// Lint `src` as a one-file crate and return the 1-based lines MXL304 flags.
fn mxl304_lines(src: &str) -> Vec<usize> {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), src).unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    report
        .diagnostics
        .iter()
        .filter(|d| format!("{}", d.code) == "MXL304")
        .map(|d| d.line)
        .collect()
}

#[test]
fn mxl304_attrib_call_fires() {
    let lines = mxl304_lines(
        r#"
pub fn has_any(x: SEXP) -> bool {
    unsafe { miniextendr_api::sys::ATTRIB(x) != R_NilValue }
}
"#,
    );
    assert_eq!(lines, vec![3], "expected MXL304 on the ATTRIB() call");
}

#[test]
fn mxl304_set_attrib_and_unchecked_forms_fire() {
    let lines = mxl304_lines(
        r#"
pub fn strip(x: SEXP) {
    unsafe { SET_ATTRIB(x, R_NilValue) };
    unsafe { SET_ATTRIB_unchecked(x, R_NilValue) };
    let _ = unsafe { ATTRIB_unchecked (x) };
}
"#,
    );
    assert_eq!(lines, vec![3, 4, 5]);
}

#[test]
fn mxl304_extern_declaration_fires() {
    let lines = mxl304_lines(
        r#"
unsafe extern "C" {
    fn ATTRIB(x: SEXP) -> SEXP;
}
"#,
    );
    assert_eq!(
        lines,
        vec![3],
        "a hand-written ATTRIB declaration must be flagged"
    );
}

#[test]
fn mxl304_api_attribute_functions_do_not_fire() {
    let lines = mxl304_lines(
        r#"
pub fn ok(x: SEXP, y: SEXP) -> bool {
    unsafe { DUPLICATE_ATTRIB(y, x) };
    unsafe { SHALLOW_DUPLICATE_ATTRIB(y, x) };
    unsafe { CLEAR_ATTRIB(y) };
    let any = unsafe { ANY_ATTRIB(x) } != 0;
    // ATTRIB(x) in a comment is fine
    let note = "ATTRIB";
    any && x.has_attributes() && !note.is_empty()
}
"#,
    );
    assert!(
        lines.is_empty(),
        "API functions must not trip MXL304, got lines {lines:?}"
    );
}

#[test]
fn mxl304_suppressed_by_allow_comment() {
    let lines = mxl304_lines(
        r#"
pub fn has_any(x: SEXP) -> bool {
    // mxl::allow(MXL304)
    unsafe { ATTRIB(x) != R_NilValue }
}
"#,
    );
    assert!(
        lines.is_empty(),
        "MXL304 must honour `// mxl::allow(MXL304)`, got {lines:?}"
    );
}

#[test]
fn mxl304_scanner_survives_multibyte_source() {
    let lines = mxl304_lines(
        r#"
pub fn has_any(x: SEXP) -> bool {
    let label = "α-日本語"; unsafe { ATTRIB(x) != R_NilValue }
}
"#,
    );
    assert_eq!(lines, vec![3]);
}

// endregion

// region: parse failures — `&Dots` hint (#1737) and rerun directives (#1738)

/// Writes each `(relative path, source)` of `files` into `dir`, creating the
/// directories they need.
fn write_crate(dir: &std::path::Path, files: &[(&str, &str)]) {
    fs::create_dir_all(dir).unwrap();
    for (name, body) in files {
        let path = dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
}

/// The `cargo::rerun-if-changed` directive for `path`.
fn rerun(path: &std::path::Path) -> String {
    format!("cargo::rerun-if-changed={}", path.display())
}

const BAD_DOTS: &str = "\nuse miniextendr_api::prelude::*;\n\n#[miniextendr]\npub fn f(sources: ..., dosing_type: i32) -> i32 {\n    dosing_type\n}\n";

#[test]
fn parse_failure_names_the_dots_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    write_crate(&src, &[("lib.rs", "mod bad;\n"), ("bad.rs", BAD_DOTS)]);

    let err = run(dir.path()).expect_err("bad.rs must not parse");
    assert!(
        err.message.contains("bad.rs: failed to parse: "),
        "got: {}",
        err.message
    );
    assert!(
        err.message.contains(
            "line 5: Rust's `...` is only valid as the last parameter; write a dots parameter \
             that is not last as `sources: &Dots`"
        ),
        "the parse error must name the `&Dots` spelling, got: {}",
        err.message
    );
}

/// `_: ...` (rustc's fix for a bare `...`, #1743) parses like `name: ...`, on a
/// function and on a method.
#[test]
fn wild_dots_parse() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    write_crate(
        &src,
        &[(
            "lib.rs",
            "use miniextendr_api::prelude::*;\n\n\
             #[miniextendr]\npub fn ignores(x: i32, _: ...) -> i32 { x }\n\n\
             #[derive(miniextendr_api::ExternalPtr)]\npub struct T;\n\n\
             #[miniextendr(env)]\nimpl T {\n    pub fn m(&self, _: ...) -> i32 { 1 }\n}\n",
        )],
    );

    let report = run(dir.path()).expect("`_: ...` must parse");
    assert!(report.errors.is_empty(), "got: {:?}", report.errors);
}

#[test]
fn parse_failure_without_misplaced_dots_gets_no_hint() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    write_crate(
        &src,
        &[(
            "lib.rs",
            "// fn commented(rest: ..., x: i32) {}\npub fn f( -> i32 { 1 }\n",
        )],
    );

    let err = run(dir.path()).expect_err("lib.rs must not parse");
    assert!(
        err.message.contains("failed to parse"),
        "got: {}",
        err.message
    );
    assert!(!err.message.contains("&Dots"), "got: {}", err.message);
}

#[test]
fn build_directives_watch_an_unparseable_file_in_src() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    write_crate(&src, &[("lib.rs", "mod bad;\n"), ("bad.rs", BAD_DOTS)]);

    let directives = build_directives(dir.path());
    for expected in [
        rerun(&src),
        rerun(&src.join("lib.rs")),
        rerun(&src.join("bad.rs")),
    ] {
        assert!(
            directives.contains(&expected),
            "missing `{expected}` in {directives:#?}"
        );
    }
    assert!(
        directives
            .iter()
            .any(|d| d.starts_with("cargo::warning=") && d.contains("failed to parse")),
        "the parse error must still be reported, got {directives:#?}"
    );
}

#[test]
fn build_directives_watch_files_not_the_manifest_dir_without_src() {
    // The scaffolded layout: `lib.rs` next to `Cargo.toml`, no `src/`.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(root, &[("lib.rs", "mod bad;\n"), ("bad.rs", BAD_DOTS)]);
    fs::create_dir(root.join("target")).unwrap();

    let directives = build_directives(root);
    assert!(
        directives.contains(&rerun(&root.join("bad.rs"))),
        "{directives:#?}"
    );
    assert!(
        directives.contains(&rerun(&root.join("lib.rs"))),
        "{directives:#?}"
    );
    assert!(
        !directives.contains(&rerun(root)),
        "the manifest dir (with its target/) must not be watched: {directives:#?}"
    );

    // Fixed, the same files stay watched and the warning is gone.
    fs::write(root.join("bad.rs"), "pub fn f() -> i32 { 1 }\n").unwrap();
    let directives = build_directives(root);
    assert!(
        directives.contains(&rerun(&root.join("bad.rs"))),
        "{directives:#?}"
    );
    assert!(!directives.contains(&rerun(root)), "{directives:#?}");
    assert!(
        !directives.iter().any(|d| d.starts_with("cargo::warning=")),
        "{directives:#?}"
    );
}

#[test]
fn build_directives_watch_an_unparseable_root_lib_rs() {
    // Root-`lib.rs` layout where `lib.rs` itself fails, so no `mod` list is read.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("lib.rs", &format!("mod sources;\n{BAD_DOTS}")),
            ("sources.rs", ""),
        ],
    );

    let directives = build_directives(root);
    assert!(
        directives.contains(&rerun(&root.join("lib.rs"))),
        "{directives:#?}"
    );
    assert!(
        directives
            .iter()
            .any(|d| d.starts_with("cargo::warning=") && d.contains("lib.rs: failed to parse")),
        "{directives:#?}"
    );

    // Fixed, the lint reruns, reads the `mod` list again and the warning is gone.
    fs::write(root.join("lib.rs"), "mod sources;\n").unwrap();
    let directives = build_directives(root);
    for file in ["lib.rs", "sources.rs"] {
        assert!(
            directives.contains(&rerun(&root.join(file))),
            "{directives:#?}"
        );
    }
    assert!(
        !directives.iter().any(|d| d.starts_with("cargo::warning=")),
        "{directives:#?}"
    );
}

// endregion

// region: crate root from `[lib] path` in Cargo.toml (#1745)

/// A one-function source that MXL304 flags (a raw-text rule, no attributes needed).
const ATTRIB_CALL: &str =
    "pub fn has_any(x: SEXP) -> bool {\n    unsafe { ATTRIB(x) != R_NilValue }\n}\n";

/// `root` joined with the `/`-separated `rel`, one component at a time, so it
/// compares equal to the paths the walk builds on every platform.
fn path_in(root: &std::path::Path, rel: &str) -> std::path::PathBuf {
    rel.split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

/// The files MXL304 flags in the crate whose manifest dir is `root`.
fn mxl304_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let report = run(root).expect("lint should succeed");
    report
        .diagnostics
        .iter()
        .filter(|d| format!("{}", d.code) == "MXL304")
        .map(|d| d.path.clone())
        .collect()
}

#[test]
fn lib_path_outside_src_is_linted_and_watched() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"pkg\"\n\n[lib]\npath = \"rust/lib.rs\"\ncrate-type = [\"staticlib\"]\n",
            ),
            ("rust/lib.rs", "mod child;\n"),
            ("rust/child.rs", ATTRIB_CALL),
        ],
    );
    fs::create_dir(root.join("target")).unwrap();

    let report = run(root).expect("lint should succeed");
    assert_eq!(
        report.files,
        vec![path_in(root, "rust/child.rs"), path_in(root, "rust/lib.rs")]
    );
    assert_eq!(mxl304_files(root), vec![path_in(root, "rust/child.rs")]);

    let directives = build_directives(root);
    for file in ["Cargo.toml", "rust/lib.rs", "rust/child.rs"] {
        assert!(
            directives.contains(&rerun(&path_in(root, file))),
            "missing {file} in {directives:#?}"
        );
    }
    for watched_dir in [root.to_path_buf(), root.join("rust"), root.join("src")] {
        assert!(
            !directives.contains(&rerun(&watched_dir)),
            "{} must not be watched as a directory: {directives:#?}",
            watched_dir.display()
        );
    }
    assert!(
        directives
            .iter()
            .any(|d| d.starts_with("cargo::warning=[MXL304]")),
        "{directives:#?}"
    );
}

#[test]
fn lib_path_root_not_named_lib_rs_keeps_children_beside_it() {
    // A crate root is a "mod-rs" file whatever its name: `mod child;` in
    // `rust/entry.rs` is `rust/child.rs`, not `rust/entry/child.rs`.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("Cargo.toml", "[lib]\npath = \"rust/entry.rs\"\n"),
            ("rust/entry.rs", "mod child;\n"),
            ("rust/child.rs", "mod grandchild;\n"),
            ("rust/child/grandchild.rs", ATTRIB_CALL),
            ("rust/entry/child.rs", "pub fn decoy() {}\n"),
        ],
    );

    let report = run(root).expect("lint should succeed");
    assert_eq!(
        report.files,
        vec![
            path_in(root, "rust/child/grandchild.rs"),
            path_in(root, "rust/child.rs"),
            path_in(root, "rust/entry.rs"),
        ]
    );
    assert_eq!(
        mxl304_files(root),
        vec![path_in(root, "rust/child/grandchild.rs")]
    );
}

#[test]
fn lib_path_at_the_manifest_root_ignores_an_unrelated_src_dir() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("Cargo.toml", "[lib]\npath = \"lib.rs\"\n"),
            ("lib.rs", "mod child;\n"),
            ("child.rs", "pub fn f() -> i32 { 1 }\n"),
            ("src/lib.rs", ATTRIB_CALL),
        ],
    );

    let report = run(root).expect("lint should succeed");
    assert_eq!(
        report.files,
        vec![root.join("child.rs"), root.join("lib.rs")]
    );
    assert!(mxl304_files(root).is_empty());

    let directives = build_directives(root);
    assert!(
        !directives.contains(&rerun(&root.join("src"))),
        "an unrelated src/ must not be watched: {directives:#?}"
    );
    assert!(!directives.contains(&rerun(root)), "{directives:#?}");
    assert!(
        !directives.iter().any(|d| d.starts_with("cargo::warning=")),
        "{directives:#?}"
    );
}

#[test]
fn lib_path_with_a_dot_prefix_names_the_same_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("Cargo.toml", "[lib]\npath = \"./lib.rs\"\n"),
            ("lib.rs", ""),
        ],
    );

    let report = run(root).expect("lint should succeed");
    assert_eq!(report.files, vec![root.join("lib.rs")]);
    assert!(
        build_directives(root).contains(&rerun(&root.join("lib.rs"))),
        "the directive must name the file without `./`"
    );
}

#[test]
fn no_lib_path_keeps_the_src_lib_rs_guess() {
    // A `[lib]` table without `path` (rpkg's shape minus one line) and a
    // manifest without `[lib]` both fall back to the guess.
    for manifest in [
        "[package]\nname = \"pkg\"\n",
        "[lib]\ncrate-type = [\"staticlib\"]\n",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_crate(
            root,
            &[
                ("Cargo.toml", manifest),
                ("src/lib.rs", "mod child;\n"),
                ("src/child.rs", ATTRIB_CALL),
            ],
        );

        assert_eq!(
            mxl304_files(root),
            vec![path_in(root, "src/child.rs")],
            "{manifest}"
        );
        let directives = build_directives(root);
        assert!(
            directives.contains(&rerun(&root.join("src"))),
            "{manifest}: {directives:#?}"
        );
    }
}

#[test]
fn no_lib_path_keeps_the_root_lib_rs_guess() {
    for manifest in [
        "[package]\nname = \"pkg\"\n",
        "[lib]\ncrate-type = [\"staticlib\"]\n",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_crate(
            root,
            &[
                ("Cargo.toml", manifest),
                ("lib.rs", "mod child;\n"),
                ("child.rs", ATTRIB_CALL),
            ],
        );

        assert_eq!(
            mxl304_files(root),
            vec![root.join("child.rs")],
            "{manifest}"
        );
        let directives = build_directives(root);
        assert!(
            !directives.contains(&rerun(root)),
            "{manifest}: {directives:#?}"
        );
    }
}

#[test]
fn lib_path_to_a_missing_file_reports_that_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("Cargo.toml", "[lib]\npath = \"rust/lib.rs\"\n"),
            ("src/lib.rs", ATTRIB_CALL),
        ],
    );

    let err = run(root).expect_err("rust/lib.rs does not exist");
    assert_eq!(
        err.message,
        format!(
            "miniextendr-lint: cannot find crate root {} ([lib] path in Cargo.toml)",
            path_in(root, "rust/lib.rs").display()
        )
    );

    // Fixing the manifest must rerun the lint, so it is watched on this path too.
    let directives = build_directives(root);
    assert!(
        directives.contains(&rerun(&root.join("Cargo.toml"))),
        "{directives:#?}"
    );
    assert!(
        !directives.contains(&rerun(&root.join("src"))),
        "{directives:#?}"
    );
}

#[test]
fn missing_default_root_names_the_guess() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(root, &[("Cargo.toml", "[package]\nname = \"pkg\"\n")]);
    fs::create_dir(root.join("src")).unwrap();

    let err = run(root).expect_err("src/lib.rs does not exist");
    assert_eq!(
        err.message,
        format!(
            "miniextendr-lint: cannot find crate root {} (no [lib] path in Cargo.toml)",
            root.join("src").join("lib.rs").display()
        )
    );
}

#[test]
fn path_keys_of_other_tables_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            (
                "Cargo.toml",
                concat!(
                    "path = \"top/lib.rs\"\n",
                    "[package]\nname = \"pkg\"\npath = \"package/lib.rs\"\n",
                    "[[bin]]\nname = \"tool\"\npath = \"bin/main.rs\"\n",
                    "[lib]\ncrate-type = [\"staticlib\"]\n",
                    "[[bin]]\nname = \"other\"\npath = \"bin/other.rs\"\n",
                    "[dependencies]\nfoo = { path = \"../foo\" }\n",
                    "[dependencies.lib]\npath = \"../lib\"\n",
                    "[package.metadata.lib]\npath = \"metadata/lib.rs\"\n",
                ),
            ),
            ("src/lib.rs", ATTRIB_CALL),
            ("bin/main.rs", ATTRIB_CALL),
            ("bin/other.rs", ATTRIB_CALL),
        ],
    );

    assert_eq!(mxl304_files(root), vec![path_in(root, "src/lib.rs")]);
}

#[test]
fn lib_path_spellings_with_comments_and_whitespace_parse() {
    for manifest in [
        "# [lib]\n# path = \"wrong.rs\"\n[lib]\npath = \"rust/lib.rs\"\n",
        "[ lib ]   # the R package's library\n  path   =   'rust/lib.rs'   # moved\n",
        "[lib]\n# path = \"wrong.rs\"\n\n\tpath=\"rust/lib.rs\"\ncrate-type = [\"staticlib\"]\n",
        "[\"lib\"]\n\"path\" = \"rust/lib.rs\"\n",
        "lib.path = \"rust/lib.rs\"\n[package]\nname = \"pkg\"\n",
        "[package]\r\nname = \"pkg\"\r\n\r\n[lib]\r\npath = \"rust/lib.rs\"\r\n",
        "[package.metadata.grid]\ncells = [\n  [\n    1,\n  ],\n]\n[lib]\npath = \"rust/lib.rs\"\n",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_crate(
            root,
            &[
                ("Cargo.toml", manifest),
                ("rust/lib.rs", ATTRIB_CALL),
                ("src/lib.rs", ""),
                ("wrong.rs", ""),
            ],
        );

        let report = run(root).unwrap_or_else(|err| panic!("{manifest:?}: {err}"));
        assert_eq!(
            report.files,
            vec![path_in(root, "rust/lib.rs")],
            "{manifest:?}"
        );
    }
}

#[test]
fn unreadable_lib_path_is_an_error_naming_the_manifest() {
    for (manifest, expected) in [
        (
            "[lib]\npath = \"rust\\\\lib.rs\"\n",
            "cannot read [lib] path `\"rust\\\\lib.rs\"`",
        ),
        (
            "[lib]\npath = \"\"\"rust/lib.rs\"\"\"\n",
            "cannot read [lib] path",
        ),
        ("[lib]\npath = rust/lib.rs\n", "cannot read [lib] path"),
        (
            "lib = { path = \"rust/lib.rs\", crate-type = [\"staticlib\"] }\n",
            "an inline `lib = { path = ... }` table is not read",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_crate(
            root,
            &[
                ("Cargo.toml", manifest),
                ("rust/lib.rs", ""),
                ("src/lib.rs", ""),
            ],
        );

        let err = run(root).expect_err(manifest);
        let prefix = format!("miniextendr-lint: {}: ", root.join("Cargo.toml").display());
        assert!(
            err.message.starts_with(&prefix) && err.message.contains(expected),
            "{manifest:?}: {}",
            err.message
        );

        // The guessed `src/` tree is not watched in its place; the manifest is.
        let directives = build_directives(root);
        assert!(
            !directives.contains(&rerun(&root.join("src"))),
            "{directives:#?}"
        );
        assert!(
            directives.contains(&rerun(&root.join("Cargo.toml"))),
            "{directives:#?}"
        );
    }
}

#[test]
fn inline_lib_table_without_path_falls_back() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("Cargo.toml", "lib = { crate-type = [\"staticlib\"] }\n"),
            ("src/lib.rs", ATTRIB_CALL),
        ],
    );

    assert_eq!(mxl304_files(root), vec![path_in(root, "src/lib.rs")]);
}

// endregion

// region: module files resolve as rustc resolves them

/// The files the walk visits in the crate whose manifest dir is `root`, as
/// `/`-separated paths relative to it.
fn walked_files(root: &std::path::Path) -> Vec<String> {
    let report = run(root).expect("lint should succeed");
    report
        .files
        .iter()
        .map(|path| {
            let rel = path.strip_prefix(root).unwrap();
            rel.components()
                .map(|c| c.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

#[test]
fn path_attr_in_a_non_mod_rs_file_is_relative_to_its_directory() {
    // `#[path]` outside inline modules is relative to the declaring file's own
    // directory, and the file it names keeps its children beside it.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("src/lib.rs", "mod a;\n"),
            ("src/a.rs", "#[path = \"shared.rs\"]\nmod s;\n"),
            ("src/shared.rs", "mod leaf;\n"),
            ("src/leaf.rs", ATTRIB_CALL),
            ("src/a/shared.rs", ATTRIB_CALL),
            ("src/shared/leaf.rs", ATTRIB_CALL),
        ],
    );

    assert_eq!(
        walked_files(root),
        ["src/a.rs", "src/leaf.rs", "src/lib.rs", "src/shared.rs"]
    );
}

#[test]
fn inline_modules_add_their_name_to_the_child_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("src/lib.rs", "mod a;\nmod outer {\n    mod inner;\n}\n"),
            ("src/outer/inner.rs", ""),
            (
                "src/a.rs",
                "mod nested {\n    mod deep;\n    #[path = \"other.rs\"]\n    mod p;\n}\n",
            ),
            ("src/a/nested/deep.rs", ""),
            ("src/a/nested/other.rs", ""),
            ("src/inner.rs", ATTRIB_CALL),
            ("src/a/deep.rs", ATTRIB_CALL),
            ("src/other.rs", ATTRIB_CALL),
        ],
    );

    assert_eq!(
        walked_files(root),
        [
            "src/a/nested/deep.rs",
            "src/a/nested/other.rs",
            "src/a.rs",
            "src/lib.rs",
            "src/outer/inner.rs",
        ]
    );
}

#[test]
fn path_attr_on_an_inline_module_names_its_directory() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            (
                "src/lib.rs",
                "#[path = \"elsewhere\"]\nmod m {\n    mod x;\n}\n",
            ),
            ("src/elsewhere/x.rs", ""),
            ("src/m/x.rs", ATTRIB_CALL),
        ],
    );

    assert_eq!(walked_files(root), ["src/elsewhere/x.rs", "src/lib.rs"]);
}

#[test]
fn cfg_gated_inline_modules_follow_the_gate() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            (
                "src/lib.rs",
                concat!(
                    "#[cfg(feature = \"mxl-test-never-on\")]\nmod gated {\n    mod hidden;\n}\n",
                    "#[cfg(not(feature = \"mxl-test-never-on\"))]\nmod shown {\n    mod visible;\n}\n",
                ),
            ),
            ("src/gated/hidden.rs", ATTRIB_CALL),
            ("src/shown/visible.rs", ""),
        ],
    );

    assert_eq!(walked_files(root), ["src/lib.rs", "src/shown/visible.rs"]);
}

#[test]
fn raw_identifier_modules_drop_the_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write_crate(
        root,
        &[
            ("src/lib.rs", "mod r#type;\n"),
            ("src/type.rs", "mod r#match;\n"),
            ("src/type/match.rs", ATTRIB_CALL),
        ],
    );

    assert_eq!(
        walked_files(root),
        ["src/lib.rs", "src/type/match.rs", "src/type.rs"]
    );
    assert_eq!(mxl304_files(root), vec![path_in(root, "src/type/match.rs")]);
}

// endregion
