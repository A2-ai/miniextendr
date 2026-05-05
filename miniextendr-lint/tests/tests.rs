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
            unsafe { ffi::Rf_allocVector_unchecked(INTSXP, 10) };
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
