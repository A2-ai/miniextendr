use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use miniextendr_lint::{lint_enabled, run};

// Tests that mutate env vars should not run in parallel.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

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
}
