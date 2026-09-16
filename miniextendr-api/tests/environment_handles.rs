//! Environment receivers must work on the minimum supported R version.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::externalptr::resolve_receiver;
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::{SEXP, SexpExt};

unsafe fn fixture(scope: &ProtectScope, code: &str) -> SEXP {
    unsafe { scope.protect_raw(r_eval_str_global(code).unwrap()) }
}

#[test]
fn direct_binding_ignores_masked_get0() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let pair = fixture(
            &scope,
            r#"local({
            p <- new("externalptr")
            e <- new.env(parent = emptyenv())
            e$.ptr <- p
            e$get0 <- function(...) stop("masked get0")
            list(p, e)
        })"#,
        );
        assert_eq!(
            resolve_receiver::<()>(pair.vector_elt(1)),
            pair.vector_elt(0)
        );
    });
}

#[test]
fn missing_null_wrong_type_and_inherited_bindings_are_rejected() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let envs = fixture(
            &scope,
            r#"local({
            parent <- new.env(parent = emptyenv())
            parent$.ptr <- new("externalptr")
            list(new.env(parent = emptyenv()),
                 list2env(list(.ptr = NULL), parent = emptyenv()),
                 list2env(list(.ptr = 1L), parent = emptyenv()),
                 new.env(parent = parent))
        })"#,
        );
        for i in 0..4 {
            let env = envs.vector_elt(i);
            assert!(std::panic::catch_unwind(|| resolve_receiver::<()>(env)).is_err());
        }
    });
}

#[test]
fn delayed_binding_is_forced_once() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let pair = fixture(
            &scope,
            r#"local({
            p <- new("externalptr")
            e <- new.env(parent = emptyenv())
            state <- new.env()
            state$n <- 0L
            delayedAssign(".ptr", { state$n <- state$n + 1L; stopifnot(state$n == 1L); p },
                          eval.env = environment(), assign.env = e)
            list(p, e)
        })"#,
        );
        for _ in 0..2 {
            assert_eq!(
                resolve_receiver::<()>(pair.vector_elt(1)),
                pair.vector_elt(0)
            );
        }
    });
}

#[test]
fn active_binding_is_evaluated_once_per_lookup() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let pair = fixture(
            &scope,
            r#"local({
            p <- new("externalptr")
            e <- new.env(parent = emptyenv())
            n <- 0L
            makeActiveBinding(".ptr", function() {
                n <<- n + 1L
                if (n == 1L) p else stop("binding read more than once")
            }, e)
            list(p, e)
        })"#,
        );
        assert_eq!(
            resolve_receiver::<()>(pair.vector_elt(1)),
            pair.vector_elt(0)
        );
    });
}

#[test]
fn active_r6_environments_survive_gc_during_the_next_lookup() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let pair = fixture(
            &scope,
            r#"local({
            p <- new("externalptr")
            e <- new.env(parent = emptyenv())
            makeActiveBinding(".__enclos_env__", function() {
                enclos <- new.env(parent = emptyenv())
                makeActiveBinding("private", function() {
                    list2env(list(.ptr = p), parent = emptyenv())
                }, enclos)
                enclos
            }, e)
            list(p, e)
        })"#,
        );
        struct ResetGc;
        impl Drop for ResetGc {
            fn drop(&mut self) {
                unsafe {
                    r_eval_str_global("gctorture(FALSE)").unwrap();
                }
            }
        }
        let _reset = ResetGc;
        r_eval_str_global("gctorture(TRUE)").unwrap();
        assert_eq!(
            resolve_receiver::<()>(pair.vector_elt(1)),
            pair.vector_elt(0)
        );
    });
}
