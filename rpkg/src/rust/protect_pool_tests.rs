//! Test fixtures for protect_pool (generational GC pool).

use miniextendr_api::ffi::Rf_ScalarInteger;
use miniextendr_api::prelude::*;
use miniextendr_api::protect_pool::ProtectPool;

/// Create a pool, insert a value, retrieve it, release it, verify stale key returns None.
#[miniextendr]
pub fn protect_pool_roundtrip() -> bool {
    unsafe {
        let mut pool = ProtectPool::new(4);
        let sexp = Rf_ScalarInteger(42);
        let key = pool.insert(sexp);

        // Should retrieve the value
        let got = pool.get(key);
        if got.is_none() {
            return false;
        }

        // Release
        pool.release(key);

        // Stale key should return None
        pool.get(key).is_none()
    }
}

/// Test that pool handles multiple inserts and releases.
#[miniextendr]
pub fn protect_pool_multi() -> i32 {
    unsafe {
        let mut pool = ProtectPool::new(8);
        let s1 = Rf_ScalarInteger(1);
        let s2 = Rf_ScalarInteger(2);
        let s3 = Rf_ScalarInteger(3);

        let k1 = pool.insert(s1);
        let k2 = pool.insert(s2);
        let k3 = pool.insert(s3);

        // Release middle one
        pool.release(k2);

        let mut count = 0;
        if pool.get(k1).is_some() {
            count += 1;
        }
        if pool.get(k2).is_none() {
            count += 1; // released, should be None
        }
        if pool.get(k3).is_some() {
            count += 1;
        }
        count
    }
}
