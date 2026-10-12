//! `REnv::get` / `get_inherits` / `define`: the same answers on every R the
//! crate supports (`R_getVarEx` from R 4.5, `base::get0()` before).

mod r_test_utils;

use miniextendr_api::expression::{REnv, r_eval_str_global};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{IntoR, SEXP, SexpExt};

/// Evaluate `code` in the global environment and root the value in `scope`.
unsafe fn fixture(scope: &ProtectScope, code: &str) -> SEXP {
    unsafe { scope.protect_raw(r_eval_str_global(code).unwrap()) }
}

/// Evaluate `code`, an R expression giving `TRUE` or `FALSE`.
fn r_flag(code: &str) -> bool {
    unsafe { r_eval_str_global(code).unwrap() }
        .as_logical()
        .unwrap_or_else(|| panic!("`{code}` returned NA"))
}

#[test]
fn get_reads_the_frame_only() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let envs = fixture(
            &scope,
            r#"local({
            parent <- new.env(parent = emptyenv())
            parent$x <- "from parent"
            parent$y <- 2
            parent$z <- 1
            child <- new.env(parent = parent)
            child$y <- 1
            child$z <- NULL
            list(parent, child)
        })"#,
        );
        let parent = REnv::from_sexp(envs.vector_elt(0));
        let child = REnv::from_sexp(envs.vector_elt(1));

        assert_eq!(child.get("x"), None);
        assert_eq!(child.get("absent"), None);
        assert_eq!(child.get("y").and_then(|v| v.as_real()), Some(1.0));
        // A binding to NULL is a binding.
        assert_eq!(child.get("z"), Some(SEXP::nil()));

        let x = child.get_inherits("x").unwrap();
        assert_eq!(Some(x), parent.get("x"), "the parent's own value");
        assert_eq!(x.string_elt_str(0), Some("from parent"));
        // The nearest binding wins, even a NULL one.
        assert_eq!(child.get_inherits("y").and_then(|v| v.as_real()), Some(1.0));
        assert_eq!(child.get_inherits("z"), Some(SEXP::nil()));
        assert_eq!(child.get_inherits("absent"), None);
    });
}

#[test]
fn get_forces_a_promise_once_and_calls_an_active_binding() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let env = fixture(
            &scope,
            r#"local({
            e <- new.env(parent = emptyenv())
            mx_renv_forced <<- 0L
            mx_renv_calls <<- 0L
            delayedAssign("lazy", { mx_renv_forced <<- mx_renv_forced + 1L; 42L },
                          assign.env = e)
            makeActiveBinding("active", function() {
                mx_renv_calls <<- mx_renv_calls + 1L
                mx_renv_calls
            }, e)
            e
        })"#,
        );
        let env = REnv::from_sexp(env);
        for _ in 0..2 {
            assert_eq!(env.get("lazy").and_then(|v| v.as_integer()), Some(42));
        }
        assert!(r_flag("mx_renv_forced == 1L"), "the promise is forced once");

        assert_eq!(env.get("active").and_then(|v| v.as_integer()), Some(1));
        assert_eq!(
            env.get_inherits("active").and_then(|v| v.as_integer()),
            Some(2)
        );
        r_eval_str_global("rm(mx_renv_forced, mx_renv_calls)").unwrap();
    });
}

#[test]
fn get_finds_namespace_and_base_functions() {
    r_test_utils::with_r_thread(|| unsafe {
        // A lazy-loaded namespace binds its functions to promises.
        let stats = REnv::package_namespace("stats").unwrap();
        assert!(stats.get("median").is_some_and(|f| f.is_function()));
        assert_eq!(stats.get("no_such_function_here"), None);
        // `sum` lives in base, an enclosure of the namespace.
        assert_eq!(stats.get("sum"), None);
        assert!(stats.get_inherits("sum").is_some_and(|f| f.is_function()));
        assert!(REnv::base().get("sum").is_some_and(|f| f.is_function()));
        assert!(
            REnv::base_namespace()
                .get("get0")
                .is_some_and(|f| f.is_function())
        );
        assert_eq!(REnv::empty().get_inherits("sum"), None);
    });
}

#[test]
fn define_binds_in_the_frame() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let envs = fixture(
            &scope,
            r#"local({
            parent <- new.env(parent = emptyenv())
            parent$x <- "parent"
            list(parent, new.env(parent = parent))
        })"#,
        );
        let parent = REnv::from_sexp(envs.vector_elt(0));
        let child = REnv::from_sexp(envs.vector_elt(1));

        // A fresh, unrooted value: `define` roots it while it allocates.
        child.define("x", vec![1, 2, 3].into_sexp());
        child.define("n", SEXP::nil());
        let x = child.get("x").unwrap();
        assert_eq!(x.as_slice::<i32>(), &[1, 2, 3]);
        assert_eq!(child.get("n"), Some(SEXP::nil()));
        // The parent's binding is untouched.
        let parent_x = parent.get("x").unwrap();
        assert_eq!(parent_x.string_elt_str(0), Some("parent"));

        // Rebinding replaces the value.
        child.define("x", SEXP::scalar_integer(9));
        assert_eq!(child.get("x").and_then(|v| v.as_integer()), Some(9));
    });
}

#[test]
fn define_roots_its_value_under_gctorture() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let env = REnv::from_sexp(fixture(&scope, "new.env(parent = emptyenv())"));
        r_eval_str_global("gctorture(TRUE)").unwrap();
        for i in 0..8 {
            env.define(&format!("v{i}"), vec![i; 16].into_sexp());
        }
        r_eval_str_global("gctorture(FALSE)").unwrap();
        for i in 0..8 {
            let v = env.get(&format!("v{i}")).unwrap();
            assert_eq!(v.as_slice::<i32>(), &[i; 16]);
        }
    });
}
