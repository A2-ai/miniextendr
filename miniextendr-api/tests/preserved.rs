//! `Preserved`: one precious-list root per handle, released on drop.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::{IntoR, Preserved, SexpExt};

/// Run R's collector, which also runs the finalizers it made ready.
fn gc() {
    unsafe { r_eval_str_global("invisible(gc())").unwrap() };
}

/// Whether the finalizer registered by [`watched_env`] has run.
fn finalized() -> bool {
    unsafe { r_eval_str_global("isTRUE(mx_preserved_finalized)").unwrap() }
        .as_logical()
        .unwrap()
}

/// A new environment whose finalizer sets `mx_preserved_finalized`, held by
/// nothing but the returned handle.
fn watched_env() -> Preserved {
    Preserved::new(unsafe {
        r_eval_str_global(
            "mx_preserved_finalized <- FALSE
             local({
                 e <- new.env()
                 reg.finalizer(e, function(e) mx_preserved_finalized <<- TRUE)
                 e
             })",
        )
        .unwrap()
    })
}

#[test]
fn every_handle_keeps_the_object_until_it_drops() {
    r_test_utils::with_r_thread(|| {
        let env = watched_env();
        gc();
        assert!(!finalized(), "the handle roots the object");

        // Each clone holds its own entry, so the handles drop in any order.
        let first = env.clone();
        let second = first.clone();
        assert_eq!(*second, *env);
        drop(env);
        gc();
        assert!(!finalized(), "two clones still root the object");
        drop(second);
        gc();
        assert!(!finalized(), "the last clone still roots the object");
        drop(first);
        gc();
        assert!(
            finalized(),
            "the object is collected once every handle drops"
        );
        unsafe { r_eval_str_global("rm(mx_preserved_finalized)").unwrap() };
    });
}

#[test]
fn into_sexp_hands_over_the_object() {
    r_test_utils::with_r_thread(|| {
        let env = watched_env();
        let sexp = env.as_sexp();
        let protected = unsafe { miniextendr_api::OwnedProtect::new(env.into_sexp()) };
        assert_eq!(*protected, sexp);
        gc();
        assert!(!finalized(), "the new owner roots the object");
        drop(protected);
        gc();
        assert!(finalized(), "into_sexp released the handle's entry");
        unsafe { r_eval_str_global("rm(mx_preserved_finalized)").unwrap() };
    });
}

#[test]
fn handles_hold_values_across_allocations() {
    r_test_utils::with_r_thread(|| {
        unsafe { r_eval_str_global("gctorture(TRUE)").unwrap() };
        let mut held: Vec<Preserved> = (0..16).map(|i| Preserved::new(vec![i; 32])).collect();
        // Release out of order, and clone some, while R allocates.
        held.retain(|value| ints(value)[0] % 3 != 0);
        let clones: Vec<Preserved> = held.iter().step_by(2).cloned().collect();
        for i in 0..64 {
            let _ = vec![i; 64].into_sexp();
        }
        held.reverse();
        drop(held.split_off(3));
        unsafe { r_eval_str_global("gctorture(FALSE)").unwrap() };

        for value in held.iter().chain(&clones) {
            let first = ints(value)[0];
            assert_ne!(first % 3, 0);
            assert_eq!(ints(value), &[first; 32]);
            assert_eq!(value.type_of(), miniextendr_api::SEXPTYPE::INTSXP);
        }
    });
}

/// The values of a preserved integer vector.
fn ints(value: &Preserved) -> &[i32] {
    unsafe { value.as_slice::<i32>() }
}
