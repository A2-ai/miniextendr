use miniextendr_lint::{lint_enabled, run};
use std::fs;
use std::path::PathBuf;
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
fn reports_missing_module_for_miniextendr_items() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    let file = src_dir.join("lib.rs");
    fs::write(
        &file,
        r#"
            #[miniextendr]
            fn foo() {}
            "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint run should succeed");
    let errors = report.errors;
    assert!(
        errors.iter().any(|e| e.contains("no miniextendr_module!")),
        "expected missing module error, got: {:?}",
        errors
    );
}

#[test]
fn succeeds_when_items_match_module() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    let file = src_dir.join("lib.rs");
    fs::write(
        &file,
        r#"
        #[miniextendr]
        fn foo() {}
        
        miniextendr_module! {
            mod lib;
            fn foo;
            }
            "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert_eq!(report.errors.len(), 0, "expected no lint errors");
    assert_eq!(
        report.files,
        vec![PathBuf::from(&file)],
        "should report the scanned file"
    );
}

#[test]
fn reports_missing_external_ptr_derive() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Struct used in `impl Counter;` but missing #[derive(ExternalPtr)]
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        struct Counter {
            value: i32,
        }

        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        miniextendr_module! {
            mod lib;
            impl Counter;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint run should succeed");
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("does not derive ExternalPtr")),
        "expected missing ExternalPtr derive error, got: {:?}",
        report.errors
    );
}

#[test]
fn no_warning_when_external_ptr_derived() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[derive(ExternalPtr)]
        struct Counter {
            value: i32,
        }

        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        miniextendr_module! {
            mod lib;
            impl Counter;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    let derive_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.contains("ExternalPtr"))
        .collect();
    assert!(
        derive_errors.is_empty(),
        "should not warn when ExternalPtr is derived, got: {:?}",
        derive_errors
    );
}

#[test]
fn no_warning_when_qualified_external_ptr_derived() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Using fully-qualified derive path
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[derive(miniextendr_api::ExternalPtr)]
        struct Counter {
            value: i32,
        }

        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        miniextendr_module! {
            mod lib;
            impl Counter;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    let derive_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.contains("ExternalPtr"))
        .collect();
    assert!(
        derive_errors.is_empty(),
        "should not warn when miniextendr_api::ExternalPtr is derived, got: {:?}",
        derive_errors
    );
}

#[test]
fn no_warning_when_typed_external_implemented() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Manual TypedExternal impl instead of derive
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        struct Counter {
            value: i32,
        }

        impl TypedExternal for Counter {
            fn type_tag() -> &'static str { "Counter" }
        }

        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        miniextendr_module! {
            mod lib;
            impl Counter;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    let derive_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.contains("ExternalPtr"))
        .collect();
    assert!(
        derive_errors.is_empty(),
        "should not warn when TypedExternal is manually implemented, got: {:?}",
        derive_errors
    );
}

#[test]
fn no_warning_for_fn_only_modules() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Only fn entries, no impl entries -- should NOT warn about ExternalPtr
    fs::write(
        src_dir.join("lib.rs"),
        r#"
        #[miniextendr]
        fn hello() -> String { "world".to_string() }

        miniextendr_module! {
            mod lib;
            fn hello;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    assert!(
        report.errors.is_empty(),
        "fn-only modules should not trigger ExternalPtr warnings, got: {:?}",
        report.errors
    );
}

#[test]
fn reports_missing_derive_across_files() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Struct defined in one file, module in another
    fs::write(
        src_dir.join("counter.rs"),
        r#"
        struct Counter { value: i32 }

        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        miniextendr_module! {
            mod counter;
            impl Counter;
        }
        "#,
    )
    .unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        mod counter;
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint run should succeed");
    assert!(
        report
            .errors
            .iter()
            .any(|e| e.contains("does not derive ExternalPtr")),
        "expected missing ExternalPtr derive across files, got: {:?}",
        report.errors
    );
}

#[test]
fn no_warning_derive_in_different_file() {
    let dir = tempfile::tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    // Derive in one file, module in another
    fs::write(
        src_dir.join("types.rs"),
        r#"
        #[derive(ExternalPtr)]
        struct Counter { value: i32 }
        "#,
    )
    .unwrap();

    fs::write(
        src_dir.join("lib.rs"),
        r#"
        mod types;

        #[miniextendr]
        impl Counter {
            fn new() -> Self { Counter { value: 0 } }
        }

        miniextendr_module! {
            mod lib;
            impl Counter;
        }
        "#,
    )
    .unwrap();

    let report = run(dir.path()).expect("lint should succeed");
    let derive_errors: Vec<_> = report
        .errors
        .iter()
        .filter(|e| e.contains("ExternalPtr"))
        .collect();
    assert!(
        derive_errors.is_empty(),
        "should find derive from different file, got: {:?}",
        derive_errors
    );
}
