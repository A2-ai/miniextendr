//! Integration tests for ProtectPool against R's GC.

mod r_test_utils;

use miniextendr_api::ffi::{self, Rf_ScalarInteger};
use miniextendr_api::protect_pool::ProtectPool;

#[test]
#[ignore = "Requires R runtime; run with --ignored"]
fn pool_protects_from_gc() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut pool = ProtectPool::new(4);

        // Insert some SEXPs
        let s1 = Rf_ScalarInteger(42);
        let s2 = Rf_ScalarInteger(99);
        let k1 = pool.insert(s1);
        let k2 = pool.insert(s2);

        // Force GC
        ffi::R_gc();

        // Values should survive GC
        let got1 = pool.get(k1).expect("k1 should be valid after GC");
        let got2 = pool.get(k2).expect("k2 should be valid after GC");
        assert_eq!(ffi::INTEGER_ELT(got1, 0), 42);
        assert_eq!(ffi::INTEGER_ELT(got2, 0), 99);

        // Release k1, GC again
        pool.release(k1);
        ffi::R_gc();

        // k1 should be stale, k2 still valid
        assert!(pool.get(k1).is_none(), "k1 should be stale after release");
        let got2 = pool
            .get(k2)
            .expect("k2 should survive after k1 release + GC");
        assert_eq!(ffi::INTEGER_ELT(got2, 0), 99);

        pool.release(k2);
        assert!(pool.is_empty());
    });
}

#[test]
#[ignore = "Requires R runtime; run with --ignored"]
fn pool_replace_survives_gc() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut pool = ProtectPool::new(4);

        let k = pool.insert(Rf_ScalarInteger(1));
        pool.replace(k, Rf_ScalarInteger(2));

        ffi::R_gc();

        let got = pool.get(k).expect("replaced value should survive GC");
        assert_eq!(ffi::INTEGER_ELT(got, 0), 2);

        pool.release(k);
    });
}

#[test]
#[ignore = "Requires R runtime; run with --ignored"]
fn pool_growth_survives_gc() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut pool = ProtectPool::new(2); // tiny, must grow

        let mut keys = Vec::new();
        for i in 0..100 {
            keys.push(pool.insert(Rf_ScalarInteger(i)));
        }

        ffi::R_gc();

        // All 100 values should survive
        for (i, &k) in keys.iter().enumerate() {
            let got = pool
                .get(k)
                .unwrap_or_else(|| panic!("key {i} should be valid"));
            assert_eq!(ffi::INTEGER_ELT(got, 0), i32::try_from(i).unwrap(),);
        }

        for k in keys {
            pool.release(k);
        }
        assert!(pool.is_empty());
    });
}

#[test]
#[ignore = "Requires R runtime; run with --ignored"]
fn pool_stale_key_after_reuse() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut pool = ProtectPool::new(4);

        let k1 = pool.insert(Rf_ScalarInteger(10));
        pool.release(k1);

        // Reuse the slot
        let k2 = pool.insert(Rf_ScalarInteger(20));

        // k1 is stale — different generation
        assert!(pool.get(k1).is_none());
        assert_eq!(ffi::INTEGER_ELT(pool.get(k2).unwrap(), 0), 20);

        // Releasing k1 should be a no-op (stale)
        pool.release(k1);
        assert_eq!(pool.len(), 1); // k2 still there

        pool.release(k2);
    });
}

#[test]
#[ignore = "Requires R runtime; run with --ignored"]
fn pool_double_release_is_noop() {
    r_test_utils::with_r_thread(|| unsafe {
        let mut pool = ProtectPool::new(4);

        let k = pool.insert(Rf_ScalarInteger(77));
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
        let k2 = pool.insert(Rf_ScalarInteger(88));
        assert_eq!(pool.len(), 1);
        assert_eq!(ffi::INTEGER_ELT(pool.get(k2).unwrap(), 0), 88);
        pool.release(k2);
    });
}
