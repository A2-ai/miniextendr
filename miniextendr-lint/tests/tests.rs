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
    let mxl106 = report
        .diagnostics
        .iter()
        .find(|d| format!("{}", d.code) == "MXL106")
        .unwrap_or_else(|| {
            panic!(
                "expected MXL106 warning for non-pub fn with @export, got: {:?}",
                report.diagnostics
            )
        });
    // The tag is passed through to the R wrapper, so it does export (#2026).
    assert!(
        mxl106.message.contains("still exports"),
        "MXL106 should say the tag still exports the wrapper, got: {}",
        mxl106.message
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

/// A type's methods may be split over several impl blocks (#1776): the
/// wrapper writer merges them into one class.
#[test]
fn no_errors_for_several_impl_blocks_of_one_type() {
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
        report.errors.is_empty(),
        "several impl blocks of one type should not error, got: {:?}",
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
fn mxl111_no_fire_for_s4_receiverless_methods() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // `s4_from_value` is the inferred constructor and `s4_default` a static
    // method: neither becomes an `s4_`-prefixed generic.
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr(s4)]
        impl Foo {
            pub fn s4_from_value(x: i32) -> Self { Foo { x } }
            pub fn s4_default() -> i32 { 0 }
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
        "MXL111 must not fire on receiver-less methods of an s4 impl, got: {:?}",
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
            pub fn value(self: ExternalPtrRef<'_, Self>) -> f64 { 0.0 }
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
        "expected MXL120 error for vctrs instance method with ExternalPtrRef<'_, Self> receiver, got: {:?}",
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

    // `self: ExternalPtr<Self>` (by-value) — distinct from `self: ExternalPtrRef<'_, Self>` (view).
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
fn case_distinct_trait_impls_are_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        impl Counter for Foo { fn value(&self) -> i32 { 1 } }
        #[miniextendr]
        impl counter for Foo { fn value(&self) -> i32 { 2 } }
        "#,
    )
    .unwrap();
    let report = run(dir.path()).expect("lint should succeed");
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
}

// endregion

// region: MXL121 — a class R code can never create

/// The MXL121 diagnostics of a crate made of `files` (`(name, source)` under
/// `src/`, `lib.rs` first).
fn mxl121(files: &[(&str, &str)]) -> Vec<miniextendr_lint::Diagnostic> {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    for (name, source) in files {
        fs::write(src_dir.join(name), source).unwrap();
    }
    let report = run(dir.path()).expect("lint should succeed");
    report
        .diagnostics
        .into_iter()
        .filter(|d| d.code.to_string() == "MXL121")
        .collect()
}

/// No constructor and no factory: an instance method returning `Self` needs
/// a `Widget` already, and `id_of` returns no `Widget`.
#[test]
fn mxl121_fires_without_constructor_or_factory() {
    let diags = mxl121(&[(
        "lib.rs",
        r#"
        #[derive(ExternalPtr)]
        pub struct Widget { id: i32 }

        /// Widgets.
        #[miniextendr(r6)]
        impl Widget {
            pub fn id(&self) -> i32 { self.id }
            pub fn copy(&self) -> Self { Widget { id: self.id } }
            pub fn id_of(text: &str) -> i32 { text.len() as i32 }
        }
        "#,
    )]);
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].line, 7, "{diags:?}");
    assert!(
        diags[0]
            .message
            .starts_with("R code can never create a `Widget`"),
        "{diags:?}"
    );
    assert!(
        diags[0]
            .help
            .as_deref()
            .is_some_and(|help| help.contains("`// mxl::allow(MXL121)`")),
        "{diags:?}"
    );
}

/// The constructor the macro infers may be in another impl block of the
/// type, in another file.
#[test]
fn mxl121_constructor_in_a_later_block() {
    let diags = mxl121(&[
        (
            "lib.rs",
            r#"
            mod more;

            #[miniextendr(s3)]
            impl Widget {
                pub fn id(&self) -> i32 { self.id }
            }
            "#,
        ),
        (
            "more.rs",
            r#"
            #[miniextendr(s3)]
            impl Widget {
                pub fn from_id(id: i32) -> Self { Widget { id } }
            }
            "#,
        ),
    ]);
    assert!(diags.is_empty(), "{diags:?}");
}

/// A marked constructor, whatever it returns.
#[test]
fn mxl121_marked_constructor() {
    let diags = mxl121(&[(
        "lib.rs",
        r#"
        #[miniextendr(env)]
        impl Widget {
            #[miniextendr(env(constructor))]
            pub fn parse(text: &str) -> Result<Self, String> { todo!() }
        }
        "#,
    )]);
    assert!(diags.is_empty(), "{diags:?}");
}

/// A `#[miniextendr]` function returning the type, here as an `ExternalPtr`
/// inside a `Result`, or another class's method returning it.
#[test]
fn mxl121_factory_function() {
    let free_fn = mxl121(&[(
        "lib.rs",
        r#"
        #[miniextendr(s4)]
        impl Widget {
            pub fn id(&self) -> i32 { self.id }
        }

        #[miniextendr]
        pub fn widget_new(id: i32) -> Result<ExternalPtr<Widget>, String> { todo!() }
        "#,
    )]);
    assert!(free_fn.is_empty(), "{free_fn:?}");

    let other_class = mxl121(&[(
        "lib.rs",
        r#"
        #[miniextendr(s7)]
        impl Widget {
            pub fn id(&self) -> i32 { self.id }
        }

        #[miniextendr(s7)]
        impl Factory {
            pub fn new() -> Self { Factory }
            pub fn widget(&self, id: i32) -> Option<Widget> { None }
        }
        "#,
    )]);
    assert!(other_class.is_empty(), "{other_class:?}");
}

/// `// mxl::allow(MXL121)` on the impl's header, here above its doc comments.
#[test]
fn mxl121_allow_on_the_impl() {
    let diags = mxl121(&[(
        "lib.rs",
        r#"
        // Objects come from a function returning them as a `SEXP`.
        // mxl::allow(MXL121)
        /// Widgets.
        #[miniextendr(env)]
        impl Widget {
            pub fn id(&self) -> i32 { self.id }
        }
        "#,
    )]);
    assert!(diags.is_empty(), "{diags:?}");
}

/// vctrs vectors are built in R from their payload, not from a `T`.
#[test]
fn mxl121_skips_vctrs() {
    let diags = mxl121(&[(
        "lib.rs",
        r#"
        #[miniextendr(vctrs(kind = "vctr", base = "double"))]
        impl Temp {
            pub fn total(x: Vec<f64>) -> f64 { x.iter().sum() }
        }
        "#,
    )]);
    assert!(diags.is_empty(), "{diags:?}");
}

/// The MXL121 diagnostics of a crate holding `Record`, an S3 class with an
/// accessor-only impl block, and `rest`.
fn mxl121_record(rest: &str) -> Vec<miniextendr_lint::Diagnostic> {
    let source = format!(
        r#"
        #[derive(ExternalPtr)]
        #[externalptr(s3)]
        pub struct Record {{
            inner: Inner,
            #[r_data]
            _r: RSidecar,
        }}

        /// @rdname my_class_methods
        #[miniextendr(s3(r_data_accessors = "get"), class = "my_class")]
        impl Record {{}}
        {rest}
        "#
    );
    mxl121(&[("lib.rs", &source)])
}

/// Asserts `diags` is the one MXL121 on `Record`.
fn assert_record_flagged(diags: &[miniextendr_lint::Diagnostic]) {
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert!(
        diags[0]
            .message
            .starts_with("R code can never create a `Record`"),
        "{diags:?}"
    );
}

/// Every entry point returns a wrapper whose hand-written `IntoR` builds the
/// pointer from a struct literal.
#[test]
fn mxl121_hand_built_by_into_r_wrapper() {
    let diags = mxl121_record(
        r#"
        pub struct RecordObject(pub(crate) Inner);

        impl RecordObject {
            fn build(self) -> SEXP {
                let ptr = ExternalPtr::new(Record { inner: self.0, _r: RSidecar::default() });
                set_class(ptr.into_sexp(), "my_class")
            }
        }

        impl IntoR for RecordObject {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                Ok(self.build())
            }
        }

        #[miniextendr]
        pub fn make_one(n: i32) -> RecordObject { RecordObject(Inner::new(n)) }
        "#,
    );
    assert!(diags.is_empty(), "{diags:?}");
}

/// `ExternalPtr::<T>::new`, whatever its argument.
#[test]
fn mxl121_hand_built_turbofish() {
    let diags = mxl121_record(
        r#"
        fn build(x: Inner) -> SEXP {
            miniextendr_api::ExternalPtr::<Record>::new(record_from(x)).into_sexp()
        }
        "#,
    );
    assert!(diags.is_empty(), "{diags:?}");
}

/// A function the crate keeps to itself returning `ExternalPtr<T>`.
#[test]
fn mxl121_hand_built_by_plain_fn() {
    let diags = mxl121_record(
        r#"
        fn build(x: Inner) -> Result<ExternalPtr<crate::Record>, String> { todo!() }
        "#,
    );
    assert!(diags.is_empty(), "{diags:?}");
}

/// A `let` annotated `ExternalPtr<T>`.
#[test]
fn mxl121_hand_built_let_annotation() {
    let diags = mxl121_record(
        r#"
        fn build(x: Inner) -> SEXP {
            let ptr: ExternalPtr<Record> = make(x);
            ptr.into_sexp()
        }
        "#,
    );
    assert!(diags.is_empty(), "{diags:?}");
}

/// `Self { .. }` and `Self(..)` inside `impl T`.
#[test]
fn mxl121_hand_built_from_self() {
    let named = mxl121_record(
        r#"
        impl Record {
            fn into_sexp(inner: Inner) -> SEXP {
                ExternalPtr::new(Self { inner, _r: RSidecar::default() }).into_sexp()
            }
        }
        "#,
    );
    assert!(named.is_empty(), "{named:?}");

    let tuple = mxl121(&[(
        "lib.rs",
        r#"
        #[miniextendr(s3, class = "my_class")]
        impl Tagged {}

        impl Tagged {
            fn into_sexp(inner: Inner) -> SEXP { ExternalPtr::new(Self(inner)).into_sexp() }
        }
        "#,
    )]);
    assert!(tuple.is_empty(), "{tuple:?}");
}

/// A struct literal of `T` that never goes into an `ExternalPtr`.
#[test]
fn mxl121_struct_literal_alone_still_fires() {
    let diags = mxl121_record(
        r#"
        fn build(inner: Inner) -> Box<Record> {
            Box::new(Record { inner, _r: RSidecar::default() })
        }
        "#,
    );
    assert_record_flagged(&diags);
}

/// A struct literal bound by `let` and put in an `ExternalPtr` later in the
/// same function body, as an argument to another call.
#[test]
fn mxl121_hand_built_let_bound_literal() {
    let diags = mxl121(&[(
        "lib.rs",
        r#"
        #[derive(ExternalPtr)]
        #[externalptr(s3)]
        pub struct StoredRecord {
            engine: Engine,
            #[r_data]
            _r: RSidecar,
            #[r_data(name = "steps")]
            pub r_steps: Sidecar<SEXP>,
        }

        /// @rdname my_class_methods
        #[miniextendr(s3(r_data_accessors = "get"), class = "my_class")]
        impl StoredRecord {}

        impl StoredRecord {
            const CLASS: &str = "my_class";
        }

        pub struct RecordObject(pub(crate) Engine, SEXP);

        impl RecordObject {
            fn build(self, steps: SEXP) -> SEXP {
                let stored = StoredRecord {
                    engine: self.0,
                    _r: RSidecar,
                    r_steps: Sidecar::new(steps),
                };
                classed(ExternalPtr::new(stored), StoredRecord::CLASS)
            }
        }

        impl IntoR for RecordObject {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                let steps = self.1;
                Ok(self.build(steps))
            }
        }

        #[miniextendr]
        pub fn make_one(n: i32) -> RecordObject { todo!() }
        "#,
    )]);
    assert!(diags.is_empty(), "{diags:?}");
}

/// A `let` whose annotation names the type, whatever its value, here in a
/// trait-impl method.
#[test]
fn mxl121_hand_built_from_annotated_let() {
    let diags = mxl121_record(
        r#"
        pub struct RecordObject(Inner);

        impl IntoR for RecordObject {
            type Error = std::convert::Infallible;
            fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
                let s: Record = make_record(self.0);
                Ok(ExternalPtr::new(s).into_sexp())
            }
        }
        "#,
    );
    assert!(diags.is_empty(), "{diags:?}");
}

/// The documented limit: `ExternalPtr::new(local)` where the local is a value
/// from a call whose type isn't written.
#[test]
fn mxl121_unnamed_local_still_fires() {
    let diags = mxl121_record(
        r#"
        fn build(inner: Inner) -> SEXP {
            let record = make_record(inner);
            ExternalPtr::new(record).into_sexp()
        }
        "#,
    );
    assert_record_flagged(&diags);
}

/// A `let` binding is known only in its own function body, and the last
/// `let` of a name wins.
#[test]
fn mxl121_let_elsewhere_still_fires() {
    let other_fn = mxl121_record(
        r#"
        fn stored(inner: Inner) -> Inner {
            let s = Record { inner, _r: RSidecar::default() };
            s.inner
        }

        fn build(s: Wrapped) -> SEXP {
            ExternalPtr::new(s).into_sexp()
        }
        "#,
    );
    assert_record_flagged(&other_fn);

    let shadowed = mxl121_record(
        r#"
        fn build(inner: Inner) -> SEXP {
            let s = Record { inner, _r: RSidecar::default() };
            let s = wrap(s);
            ExternalPtr::new(s).into_sexp()
        }
        "#,
    );
    assert_record_flagged(&shadowed);
}

/// An `ExternalPtr` of another type.
#[test]
fn mxl121_other_type_still_fires() {
    let diags = mxl121_record(
        r#"
        fn build(inner: Inner) -> ExternalPtr<Other> {
            let ptr: ExternalPtr<Other> = ExternalPtr::new(Other { inner });
            ptr
        }
        "#,
    );
    assert_record_flagged(&diags);
}

/// Receivers and parameters are objects the caller has already.
#[test]
fn mxl121_inputs_still_fire() {
    let diags = mxl121_record(
        r#"
        #[miniextendr(s3, class = "my_class")]
        impl Record {
            pub fn id(self: ExternalPtrRef<'_, Self>) -> i32 { 1 }
            pub fn same(&self, other: &Self) -> bool { true }
            pub fn keep(self: ExternalPtr<Self>, other: ExternalPtr<Self>) -> bool { true }
        }

        #[miniextendr]
        pub fn record_id(x: &Record, y: Option<ExternalPtr<Record>>) -> i32 { 1 }

        fn helper(x: ExternalPtr<Record>) -> i32 { 1 }
        "#,
    );
    assert_record_flagged(&diags);
}

/// A method of the type's own `#[miniextendr]` impl taking `self` needs an
/// object already, whatever it builds.
#[test]
fn mxl121_own_method_with_receiver_still_fires() {
    let diags = mxl121_record(
        r#"
        #[miniextendr(s3, class = "my_class")]
        impl Record {
            pub fn copy(&self) -> ExternalPtr<Self> {
                ExternalPtr::new(Self { inner: self.inner.clone(), _r: RSidecar::default() })
            }
        }
        "#,
    );
    assert_record_flagged(&diags);
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

// region: MXL305 — raw R evaluation in a #[miniextendr] body

/// Lint `src` as a one-file crate and return the MXL305 diagnostics.
fn mxl305(src: &str) -> Vec<miniextendr_lint::Diagnostic> {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("lib.rs"), src).unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    report
        .diagnostics
        .into_iter()
        .filter(|d| format!("{}", d.code) == "MXL305")
        .collect()
}

/// The 1-based lines MXL305 flags in `src`.
fn mxl305_lines(src: &str) -> Vec<usize> {
    mxl305(src).iter().map(|d| d.line).collect()
}

#[test]
fn mxl305_rf_eval_in_free_fn_fires() {
    let diags = mxl305(
        r#"
#[miniextendr]
pub fn call_back(f: SEXP) -> SEXP {
    unsafe { sys::Rf_eval(f, sys::R_GlobalEnv) }
}
"#,
    );
    assert_eq!(diags.iter().map(|d| d.line).collect::<Vec<_>>(), vec![4]);
    assert!(
        diags[0].message.contains("`Rf_eval()`") && diags[0].message.contains("stays borrowed"),
        "got: {}",
        diags[0].message
    );
    let help = diags[0].help.as_deref().unwrap_or_default();
    assert!(
        help.contains("eval_with_handlers") && help.contains("RCall::eval"),
        "help should name the evaluators, got: {help}"
    );
}

#[test]
fn mxl305_any_path_fires() {
    let lines = mxl305_lines(
        r#"
use miniextendr_api::sys::Rf_eval;

#[miniextendr]
pub fn paths(call: SEXP, env: SEXP) {
    unsafe { miniextendr_api::sys::Rf_eval(call, env) };
    unsafe { Rf_eval (call, env) };
    unsafe { ::miniextendr_api::sys::R_forceAndCall(call, 1, env) };
    let _ = vec![unsafe { sys::Rf_eval(call, env) }];
    unsafe { sys::Rf_applyClosure(call, call, R_NilValue, env, R_NilValue, Rboolean::TRUE) };
}
"#,
    );
    assert_eq!(lines, vec![6, 7, 8, 9, 10]);
}

#[test]
fn mxl305_check_user_interrupt_fires() {
    let diags = mxl305(
        r#"
#[miniextendr]
pub fn long_loop(n: i32) -> i32 {
    for _ in 0..n {
        unsafe { R_CheckUserInterrupt() };
    }
    n
}
"#,
    );
    assert_eq!(diags.iter().map(|d| d.line).collect::<Vec<_>>(), vec![5]);
    assert!(
        diags[0].message.contains("a pending interrupt"),
        "got: {}",
        diags[0].message
    );
    let help = diags[0].help.as_deref().unwrap_or_default();
    assert!(
        help.contains("#[miniextendr(check_interrupt)]") && help.contains("#2101"),
        "help should name check_interrupt and the helper issue, got: {help}"
    );
}

#[test]
fn mxl305_unchecked_forms_fire() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
pub fn unchecked(call: SEXP, env: SEXP) {
    unsafe { sys::Rf_eval_unchecked(call, env) };
    unsafe { sys::R_forceAndCall_unchecked(call, 1, env) };
    unsafe { sys::R_CheckUserInterrupt_unchecked() };
}
"#,
    );
    assert_eq!(lines, vec![4, 5, 6]);
}

#[test]
fn mxl305_one_line_body_fires() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
pub fn f(x: SEXP) -> SEXP { unsafe { Rf_eval(x, R_GlobalEnv) } }
"#,
    );
    assert_eq!(lines, vec![3]);
}

#[test]
fn mxl305_inherent_method_fires() {
    let lines = mxl305_lines(
        r#"
#[derive(ExternalPtr)]
pub struct Counter { n: i32 }

#[miniextendr]
impl Counter {
    pub fn new() -> Self { Counter { n: 0 } }

    pub fn each(&mut self, f: SEXP) {
        self.n += 1;
        unsafe { sys::Rf_eval(f, sys::R_GlobalEnv) };
    }
}
"#,
    );
    assert_eq!(lines, vec![11]);
}

#[test]
fn mxl305_trait_impl_method_fires() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
impl Runner for Counter {
    fn run(&self) {
        unsafe { R_CheckUserInterrupt() };
    }
}
"#,
    );
    assert_eq!(lines, vec![5]);
}

#[test]
fn mxl305_trait_default_method_fires() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
pub trait Runner {
    fn step(&mut self);
    fn run(&mut self) {
        self.step();
        unsafe { R_CheckUserInterrupt() };
    }
}
"#,
    );
    assert_eq!(lines, vec![7]);
}

/// `with_r_unwind_protect` resumes an R exit with `R_ContinueUnwind`, which
/// jumps over the method's frames all the same: no exemption.
#[test]
fn mxl305_inside_with_r_unwind_protect_still_fires() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
impl Counter {
    pub fn each(&mut self, f: SEXP) -> SEXP {
        with_r_unwind_protect(|| unsafe { sys::Rf_eval(f, sys::R_GlobalEnv) }, None)
    }
}
"#,
    );
    assert_eq!(lines, vec![5]);
}

#[test]
fn mxl305_outside_miniextendr_bodies_does_not_fire() {
    let lines = mxl305_lines(
        r#"
pub fn helper(f: SEXP) -> SEXP {
    unsafe { sys::Rf_eval(f, sys::R_GlobalEnv) }
}

impl Plain {
    pub fn each(&self, f: SEXP) {
        unsafe { R_CheckUserInterrupt() };
    }
}

pub trait PlainTrait {
    fn run(&self) {
        unsafe { R_CheckUserInterrupt() };
    }
}

/// Calls `Rf_eval(f, env)` through a helper the lint doesn't follow.
#[miniextendr]
pub fn via_helper(f: SEXP) -> SEXP {
    helper(f)
}
"#,
    );
    assert!(
        lines.is_empty(),
        "only #[miniextendr] bodies are scanned, got lines {lines:?}"
    );
}

#[test]
fn mxl305_safe_evaluators_and_mentions_do_not_fire() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
pub fn safe(f: SEXP, expr: Quoted) -> SEXP {
    use miniextendr_api::sys::R_CheckUserInterrupt;
    // Rf_eval(f, env) here would jump over this frame.
    let _ = unsafe { RCall::from_sexp(f).eval_with_handlers(R_GlobalEnv) };
    let _ = unsafe { eval_with_handlers(f, R_GlobalEnv) };
    let _ = unsafe { try_eval_with_handlers(f, R_GlobalEnv) };
    let _ = unsafe { RCall::from_sexp(f).eval_base() };
    let mut err = 0;
    let _ = unsafe { R_tryEval(f, R_GlobalEnv, &mut err) };
    let _ = R_CheckUserInterrupt;
    expr.eval()
}
"#,
    );
    assert!(
        lines.is_empty(),
        "safe evaluators, imports and comments must not trip MXL305, got lines {lines:?}"
    );
}

#[test]
fn mxl305_suppressed_by_allow_comment() {
    let lines = mxl305_lines(
        r#"
#[miniextendr]
pub fn fixture(f: SEXP) {
    // mxl::allow(MXL305): deliberately raises over this frame
    unsafe { sys::Rf_eval(f, sys::R_GlobalEnv) };
    unsafe { R_CheckUserInterrupt() }; // mxl::allow(MXL305)
}
"#,
    );
    assert!(
        lines.is_empty(),
        "MXL305 must honour `// mxl::allow(MXL305)`, got {lines:?}"
    );
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
    for file in ["rust/lib.rs", "rust/child.rs"] {
        assert!(
            directives.contains(&rerun(&path_in(root, file))),
            "missing {file} in {directives:#?}"
        );
    }
    // Scaffolded `configure` touches Cargo.toml on every run; watching it here
    // would rebuild the crate on every install.
    assert!(
        !directives.contains(&rerun(&root.join("Cargo.toml"))),
        "{directives:#?}"
    );
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

    // No source file is watched, so the manifest is: fixing it reruns the lint.
    let directives = build_directives(root);
    assert!(
        directives.contains(&rerun(&root.join("Cargo.toml"))),
        "{directives:#?}"
    );
    assert!(
        !directives.contains(&rerun(&root.join("src"))),
        "{directives:#?}"
    );

    // Fixed, the root's files are watched instead and the warning is gone.
    write_crate(root, &[("Cargo.toml", "[lib]\npath = \"src/lib.rs\"\n")]);
    let directives = build_directives(root);
    assert!(
        !directives.contains(&rerun(&root.join("Cargo.toml"))),
        "{directives:#?}"
    );
    assert!(
        directives.contains(&rerun(&root.join("src"))),
        "{directives:#?}"
    );
    assert!(
        directives.contains(&rerun(&path_in(root, "src/lib.rs"))),
        "{directives:#?}"
    );
    assert!(
        !directives
            .iter()
            .any(|d| d.contains("cannot find crate root")),
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

// region: MXL204 — doc links into undeclared R packages

/// An R package at `<tmp>/pkg` with its crate at `src/rust`; returns the crate
/// directory and keeps the tempdir alive.
fn r_package(
    description: Option<&str>,
    cargo_toml: &str,
    lib_rs: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let crate_dir = dir.path().join("pkg/src/rust");
    fs::create_dir_all(&crate_dir).unwrap();
    if let Some(text) = description {
        fs::write(dir.path().join("pkg/DESCRIPTION"), text).unwrap();
    }
    fs::write(crate_dir.join("Cargo.toml"), cargo_toml).unwrap();
    fs::write(crate_dir.join("lib.rs"), lib_rs).unwrap();
    (dir, crate_dir)
}

const DESCRIPTION: &str = "Package: mypkg\nImports: cli (>= 3.0),\n    rlang\nSuggests: testthat\n";

fn mxl204_links(report: &miniextendr_lint::LintReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .filter(|d| d.code == miniextendr_lint::LintCode::MXL204)
        .map(|d| d.message.as_str())
        .collect()
}

#[test]
fn mxl204_undeclared_package_warns() {
    let (_dir, crate_dir) = r_package(
        Some(DESCRIPTION),
        "[package]\nname = \"x\"\n",
        r#"
/// @details See [`Sources::prepare`] and [text][dplyr::bind_rows].
#[miniextendr]
pub fn f() {}
"#,
    );
    let report = run(&crate_dir).unwrap();
    let found = mxl204_links(&report);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].contains("`Sources`") && found[1].contains("`dplyr`"));
    let help = report.diagnostics[0].help.as_deref().unwrap();
    assert!(help.contains("[text][crate::path]"));
    assert!(report.errors.is_empty(), "a warning, not an error");
}

#[test]
fn mxl204_declared_base_and_own_packages_are_silent() {
    let (_dir, crate_dir) = r_package(
        Some(DESCRIPTION),
        "[package]\nname = \"x\"\n",
        r#"
/// @details [cli::cli_abort], [rlang::abort()], [testthat::expect_true],
/// [stats::median], [MASS::mvrnorm], [mypkg::f], [crate::x], [Self::y],
/// [`a::b::c`]
#[miniextendr]
pub fn f() {}
"#,
    );
    let report = run(&crate_dir).unwrap();
    assert!(mxl204_links(&report).is_empty(), "{:?}", report.diagnostics);
}

#[test]
fn mxl204_leading_prose_counts_only_under_keep() {
    let lib = r#"
/// See [`Sources::prepare`].
#[miniextendr]
pub fn f() {}
"#;
    let (_dir, crate_dir) = r_package(Some(DESCRIPTION), "[package]\nname = \"x\"\n", lib);
    assert!(mxl204_links(&run(&crate_dir).unwrap()).is_empty());

    let (_dir, crate_dir) = r_package(
        Some(DESCRIPTION),
        "[package]\nname = \"x\"\n[package.metadata.miniextendr]\nroxygen_prose_links = \"keep\"\n",
        lib,
    );
    assert_eq!(mxl204_links(&run(&crate_dir).unwrap()).len(), 1);
}

/// A block that renders no help page hands roxygen2 no links (#1818): the
/// macro strips them all, so there is nothing to report there.
#[test]
fn mxl204_skips_blocks_that_render_no_page() {
    let (_dir, crate_dir) = r_package(
        Some(DESCRIPTION),
        "[package]\nname = \"x\"\n",
        r#"
/// @details See [`Sources::prepare`].
#[miniextendr(noexport)]
pub fn hidden() {}

/// @details See [`Sources::prepare`].
/// @noRd
#[miniextendr]
pub fn no_rd() {}

/// @details See [`Sources::prepare`].
#[miniextendr(r6, noexport)]
impl Hidden {
    /// @details See [`Sources::method`].
    pub fn method(&self) {}
}

#[miniextendr(r6)]
impl Shown {
    /// @details See [`Sources::own_no_rd`].
    /// @noRd
    pub fn method(&self) {}
    /// @details See [`Sources::shown`].
    pub fn other(&self) {}
}

/// @details See [`Sources::internal`].
#[miniextendr(r6, noexport, internal)]
impl Internal {}

/// @details See [`Sources::tagged_internal`].
/// @keywords internal
#[miniextendr(r6, noexport)]
impl TaggedInternal {}
"#,
    );
    let report = run(&crate_dir).unwrap();
    let found = mxl204_links(&report);
    assert_eq!(found.len(), 3, "{found:?}");
    for (message, link) in found.iter().zip([
        "Sources::shown",
        "Sources::internal",
        "Sources::tagged_internal",
    ]) {
        assert!(message.contains(link), "{message} names {link}");
    }
}

#[test]
fn mxl204_skips_without_a_description() {
    let (_dir, crate_dir) = r_package(
        None,
        "[package]\nname = \"x\"\n",
        r#"
/// @details [`Sources::prepare`]
#[miniextendr]
pub fn f() {}
"#,
    );
    assert!(mxl204_links(&run(&crate_dir).unwrap()).is_empty());
}

#[test]
fn mxl204_description_is_watched_when_present() {
    let (_dir, crate_dir) = r_package(Some(DESCRIPTION), "[package]\nname = \"x\"\n", "");
    let watched = build_directives(&crate_dir)
        .into_iter()
        .any(|d| d.starts_with("cargo::rerun-if-changed=") && d.ends_with("DESCRIPTION"));
    assert!(watched);
    let (_dir, crate_dir) = r_package(None, "[package]\nname = \"x\"\n", "");
    let watched = build_directives(&crate_dir)
        .into_iter()
        .any(|d| d.ends_with("DESCRIPTION"));
    assert!(!watched);
}

// endregion
