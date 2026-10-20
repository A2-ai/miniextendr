//! Integration tests for ProtectPool against R's GC.

mod r_test_utils;

use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::protect_pool::ProtectPool;
use miniextendr_api::{IntoR, Preserved, ProtectScope, Protector, sys};

/// Run R's collector, which also runs the finalizers it made ready.
fn gc() {
    unsafe { r_eval_str_global("invisible(gc())").unwrap() };
}

/// A new environment held by nothing but the returned handle. Its finalizer
/// sets the global logical `flag`, which starts `FALSE`.
fn watched_env(flag: &str) -> Preserved {
    Preserved::new(unsafe {
        r_eval_str_global(&format!(
            "{flag} <- FALSE
             local({{
                 e <- new.env()
                 reg.finalizer(e, function(e) {flag} <<- TRUE)
                 e
             }})"
        ))
        .unwrap()
    })
}

/// Whether the finalizer of the [`watched_env`] for `flag` has run.
fn finalized(flag: &str) -> bool {
    unsafe { r_eval_str_global(&format!("isTRUE({flag})")).unwrap() }
        .as_logical()
        .unwrap()
}

/// Remove the global `flag` a [`watched_env`] set.
fn remove_flag(flag: &str) {
    unsafe { r_eval_str_global(&format!("rm({flag})")).unwrap() };
}

#[test]
fn pool_protects_from_gc() {
    r_test_utils::with_r_thread(|| {
        let mut pool = ProtectPool::new(4);

        // Insert some SEXPs
        let s1 = SEXP::scalar_integer(42);
        let s2 = SEXP::scalar_integer(99);
        let k1 = pool.insert(s1);
        let k2 = pool.insert(s2);

        // Force GC
        unsafe { sys::R_gc() };

        // Values should survive GC
        let got1 = pool.get(k1).expect("k1 should be valid after GC");
        let got2 = pool.get(k2).expect("k2 should be valid after GC");
        assert_eq!(got1.integer_elt(0), 42);
        assert_eq!(got2.integer_elt(0), 99);

        // Release k1, GC again
        pool.release(k1);
        unsafe { sys::R_gc() };

        // k1 should be stale, k2 still valid
        assert!(pool.get(k1).is_none(), "k1 should be stale after release");
        let got2 = pool
            .get(k2)
            .expect("k2 should survive after k1 release + GC");
        assert_eq!(got2.integer_elt(0), 99);

        pool.release(k2);
        assert!(pool.is_empty());
    });
}

#[test]
fn pool_replace_survives_gc() {
    r_test_utils::with_r_thread(|| {
        let mut pool = ProtectPool::new(4);

        let k = pool.insert(SEXP::scalar_integer(1));
        pool.replace(k, SEXP::scalar_integer(2));

        unsafe { sys::R_gc() };

        let got = pool.get(k).expect("replaced value should survive GC");
        assert_eq!(got.integer_elt(0), 2);

        pool.release(k);
    });
}

#[test]
fn pool_growth_survives_gc() {
    r_test_utils::with_r_thread(|| {
        let mut pool = ProtectPool::new(2); // tiny, must grow

        let mut keys = Vec::new();
        for i in 0..100 {
            keys.push(pool.insert(SEXP::scalar_integer(i)));
        }

        unsafe { sys::R_gc() };

        // All 100 values should survive
        for (i, &k) in keys.iter().enumerate() {
            let got = pool
                .get(k)
                .unwrap_or_else(|| panic!("key {i} should be valid"));
            assert_eq!(got.integer_elt(0), i32::try_from(i).unwrap(),);
        }

        for k in keys {
            pool.release(k);
        }
        assert!(pool.is_empty());
    });
}

#[test]
fn pool_stale_key_after_reuse() {
    r_test_utils::with_r_thread(|| {
        let mut pool = ProtectPool::new(4);

        let k1 = pool.insert(SEXP::scalar_integer(10));
        pool.release(k1);

        // Reuse the slot
        let k2 = pool.insert(SEXP::scalar_integer(20));

        // k1 is stale — different generation
        assert!(pool.get(k1).is_none());
        assert_eq!(pool.get(k2).unwrap().integer_elt(0), 20);

        // Releasing k1 should be a no-op (stale)
        pool.release(k1);
        assert_eq!(pool.len(), 1); // k2 still there

        pool.release(k2);
    });
}

#[test]
fn pool_double_release_is_noop() {
    r_test_utils::with_r_thread(|| {
        let mut pool = ProtectPool::new(4);

        let k = pool.insert(SEXP::scalar_integer(77));
        assert_eq!(pool.len(), 1);

        // First release — valid
        pool.release(k);
        assert_eq!(pool.len(), 0);

        // Second release — stale key, should be a no-op
        pool.release(k);
        assert_eq!(pool.len(), 0);

        // Third release — still no-op
        pool.release(k);
        assert_eq!(pool.len(), 0);

        // Pool should still be functional
        let k2 = pool.insert(SEXP::scalar_integer(88));
        assert_eq!(pool.len(), 1);
        assert_eq!(pool.get(k2).unwrap().integer_elt(0), 88);
        pool.release(k2);
    });
}

#[test]
fn root_returns_its_argument() {
    r_test_utils::with_r_thread(|| {
        let mut pool = ProtectPool::new(1);

        let sexp = SEXP::scalar_integer(7);
        assert_eq!(pool.root(sexp), sexp);

        // A Rust value is converted first; the converted object comes back.
        let ints = pool.root(vec![1, 2, 3]);
        assert_eq!(ints.type_of(), miniextendr_api::SEXPTYPE::INTSXP);
        assert_eq!(unsafe { ints.as_slice::<i32>() }, &[1, 2, 3]);
        assert_eq!(pool.len(), 2);
    });
}

#[test]
fn rooted_value_lives_until_the_pool_drops() {
    r_test_utils::with_r_thread(|| {
        let flag = "mx_pool_finalized_lives";
        let mut pool = ProtectPool::new(4);
        // `root` takes the handle over: the pool is the only root left.
        let env = pool.root(watched_env(flag));

        unsafe { sys::R_gc() };
        gc();
        assert!(!finalized(flag), "the pool roots the value");
        assert!(env.is_environment());

        drop(pool);
        gc();
        assert!(finalized(flag), "dropping the pool releases the value");
        remove_flag(flag);
    });
}

#[test]
fn pools_drop_in_either_order() {
    r_test_utils::with_r_thread(|| {
        for older_first in [true, false] {
            let (older_flag, newer_flag) = ("mx_pool_finalized_older", "mx_pool_finalized_newer");
            let mut older = ProtectPool::new(1);
            let mut newer = ProtectPool::new(1);
            // Interleave the roots, and grow both pools, so neither pool's
            // values sit together anywhere.
            older.root(watched_env(older_flag));
            newer.root(watched_env(newer_flag));
            for i in 0..8 {
                older.root(vec![i; 8]);
                newer.root(vec![i; 8]);
            }

            let (first, first_flag, second, second_flag) = if older_first {
                (older, older_flag, newer, newer_flag)
            } else {
                (newer, newer_flag, older, older_flag)
            };
            drop(first);
            gc();
            assert!(
                finalized(first_flag),
                "the dropped pool's value is released"
            );
            assert!(
                !finalized(second_flag),
                "the other pool keeps its value (older first: {older_first})"
            );
            drop(second);
            gc();
            assert!(finalized(second_flag));
            remove_flag(older_flag);
            remove_flag(newer_flag);
        }
    });
}

#[test]
fn insert_keeps_a_fresh_value_alive_while_the_pool_grows() {
    r_test_utils::with_r_thread(|| {
        let flag = "mx_pool_finalized_growth";
        let env = watched_env(flag);
        let mut pool = ProtectPool::new(1);
        pool.insert(SEXP::scalar_integer(0));
        assert_eq!(pool.capacity(), 1, "the pool is full");

        // Under gctorture every allocation collects, including the one that
        // grows the pool. `into_sexp` releases the handle's root without
        // allocating, so `fresh` is rooted by nothing until the pool holds it.
        unsafe { r_eval_str_global("gctorture(TRUE)").unwrap() };
        let fresh = env.into_sexp();
        let key = pool.insert(fresh);
        unsafe { r_eval_str_global("gctorture(FALSE)").unwrap() };

        unsafe { sys::R_gc() };
        gc();
        assert!(pool.capacity() > 1, "the insert grew the pool");
        assert!(!finalized(flag), "the value survived the pool's growth");
        assert_eq!(pool.get(key), Some(fresh));
        assert!(fresh.is_environment());

        drop(pool);
        gc();
        assert!(finalized(flag));
        remove_flag(flag);
    });
}

/// Root a fresh value through any backend.
fn keep(protector: &mut impl Protector, flag: &str) -> SEXP {
    protector.protect(watched_env(flag).into_sexp())
}

#[test]
fn protector_roots_until_it_drops() {
    r_test_utils::with_r_thread(|| {
        let flag = "mx_pool_finalized_protector";

        let mut scope = unsafe { ProtectScope::new() };
        let env = keep(&mut scope, flag);
        gc();
        assert!(!finalized(flag), "the scope roots the value");
        assert!(env.is_environment());
        drop(scope);
        gc();
        assert!(finalized(flag), "dropping the scope releases the value");

        let mut pool = ProtectPool::new(1);
        pool.root(SEXP::scalar_integer(0));
        let env = keep(&mut pool, flag);
        gc();
        assert!(!finalized(flag), "the pool roots the value");
        assert!(env.is_environment());
        drop(pool);
        gc();
        assert!(finalized(flag), "dropping the pool releases the value");
        remove_flag(flag);
    });
}
