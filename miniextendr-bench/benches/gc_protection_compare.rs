//! Head-to-head GC protection mechanism benchmarks.
//!
//! Compares protect stack, precious list, DLL preserve, and VECSXP pool
//! variants under identical workloads. Uses `Bencher::with_inputs` to
//! pre-allocate SEXPs outside the timed region so we measure ONLY
//! the protection overhead.
//!
//! See `plans/gc-protection-benchmarks.md`.

use miniextendr_api::ffi::{self, R_PreserveObject, R_ReleaseObject, Rf_ScalarInteger, SEXP};
use miniextendr_api::preserve;
use miniextendr_bench::pool_prototypes::{
    BTreeMapPool, DequePool, HashMapPool, IndexMapPool, SlotmapPool, VecPool,
};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: helpers

/// Pre-allocate N SEXPs (protected by R_PreserveObject so they survive across iterations).
/// Returns a Vec of SEXPs that are valid for the lifetime of the benchmark.
unsafe fn prealloc_sexps(n: usize) -> Vec<SEXP> {
    let mut sexps = Vec::with_capacity(n);
    for i in 0..n {
        unsafe {
            let s = Rf_ScalarInteger((i % 1000) as i32);
            R_PreserveObject(s);
            sexps.push(s);
        }
    }
    sexps
}

/// Release pre-allocated SEXPs.
unsafe fn release_prealloc(sexps: &[SEXP]) {
    for &s in sexps {
        unsafe { R_ReleaseObject(s) };
    }
}

/// Deterministic shuffle (xorshift64 Fisher-Yates).
fn shuffle(v: &mut [usize]) {
    let mut state: u64 = 0xDEAD_BEEF_CAFE_1234;
    for i in (1..v.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let j = (state as usize) % (i + 1);
        v.swap(i, j);
    }
}

// endregion

// region: Group 1 — Single protect + release (pure protection cost)

mod single_latency {
    use super::*;
    use divan::Bencher;

    #[divan::bench]
    fn protect_stack(bencher: Bencher) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) }; // keep alive across iterations

        bencher.bench_local(|| unsafe {
            ffi::Rf_protect(sexp);
            ffi::Rf_unprotect(1);
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench]
    fn precious_list(bencher: Bencher) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };

        bencher.bench_local(|| unsafe {
            R_PreserveObject(sexp);
            R_ReleaseObject(sexp);
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench]
    fn dll_preserve(bencher: Bencher) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };

        bencher.bench_local(|| unsafe {
            let cell = preserve::insert_unchecked(sexp);
            preserve::release_unchecked(cell);
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench]
    fn vec_pool(bencher: Bencher) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { VecPool::new(16) };

        bencher.bench_local(|| unsafe {
            let slot = pool.insert(sexp);
            pool.release(slot);
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench]
    fn slotmap_pool(bencher: Bencher) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { SlotmapPool::new(16) };

        bencher.bench_local(|| unsafe {
            let key = pool.insert(sexp);
            pool.release(key);
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench]
    fn deque_pool(bencher: Bencher) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { DequePool::new(16) };

        bencher.bench_local(|| unsafe {
            let slot = pool.insert(sexp);
            pool.release(slot);
        });

        unsafe { R_ReleaseObject(sexp) };
    }
}

// endregion

// region: Group 2 — Batch protect N, release all (pre-allocated SEXPs)

mod batch_throughput {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn protect_stack(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            for &s in &sexps {
                ffi::Rf_protect(s);
            }
            ffi::Rf_unprotect(n as i32);
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [10, 100, 1_000])]
    fn precious_list(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            for &s in &sexps {
                R_PreserveObject(s);
            }
            for &s in &sexps {
                R_ReleaseObject(s);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn dll_preserve(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            let mut cells = Vec::with_capacity(n);
            for &s in &sexps {
                cells.push(preserve::insert_unchecked(s));
            }
            for cell in cells {
                preserve::release_unchecked(cell);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn vec_pool(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut pool = unsafe { VecPool::new(n.max(16)) };

        bencher.bench_local(|| unsafe {
            let mut slots = Vec::with_capacity(n);
            for &s in &sexps {
                slots.push(pool.insert(s));
            }
            for slot in slots {
                pool.release(slot);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn slotmap_pool(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut pool = unsafe { SlotmapPool::new(n.max(16)) };

        bencher.bench_local(|| unsafe {
            let mut keys = Vec::with_capacity(n);
            for &s in &sexps {
                keys.push(pool.insert(s));
            }
            for key in keys {
                pool.release(key);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }
}

// endregion

// region: Group 3 — Churn (interleaved insert/release)

mod churn {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [1_000, 10_000])]
    fn precious_list(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            let mut live: Vec<SEXP> = Vec::new();
            for (i, &s) in sexps.iter().enumerate() {
                R_PreserveObject(s);
                live.push(s);
                if i % 3 == 0 && !live.is_empty() {
                    R_ReleaseObject(live.remove(0));
                }
            }
            for s in live {
                R_ReleaseObject(s);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn dll_preserve(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            let mut live: Vec<SEXP> = Vec::new();
            for (i, &s) in sexps.iter().enumerate() {
                live.push(preserve::insert_unchecked(s));
                if i % 3 == 0 && !live.is_empty() {
                    preserve::release_unchecked(live.remove(0));
                }
            }
            for cell in live {
                preserve::release_unchecked(cell);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_pool(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut pool = unsafe { VecPool::new(1024) };

        bencher.bench_local(|| unsafe {
            let mut live: Vec<usize> = Vec::new();
            for (i, &s) in sexps.iter().enumerate() {
                live.push(pool.insert(s));
                if i % 3 == 0 && !live.is_empty() {
                    pool.release(live.remove(0));
                }
            }
            for slot in live {
                pool.release(slot);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }
}

// endregion

// region: Group 5 — Random release order

mod random_release {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn dll_preserve(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut order: Vec<usize> = (0..n).collect();
        shuffle(&mut order);

        bencher.bench_local(|| unsafe {
            let mut cells = Vec::with_capacity(n);
            for &s in &sexps {
                cells.push(preserve::insert_unchecked(s));
            }
            for &i in &order {
                preserve::release_unchecked(cells[i]);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn vec_pool(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut pool = unsafe { VecPool::new(n.max(16)) };
        let mut order: Vec<usize> = (0..n).collect();
        shuffle(&mut order);

        bencher.bench_local(|| unsafe {
            let mut slots = Vec::with_capacity(n);
            for &s in &sexps {
                slots.push(pool.insert(s));
            }
            for &i in &order {
                pool.release(slots[i]);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [100, 1_000])]
    fn precious_list(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut order: Vec<usize> = (0..n).collect();
        shuffle(&mut order);

        bencher.bench_local(|| unsafe {
            for &s in &sexps {
                R_PreserveObject(s);
            }
            for &i in &order {
                R_ReleaseObject(sexps[i]);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }
}

// endregion

// region: Group 7 — Replace in loop (pure protection cost, no SEXP allocation)

mod replace_in_loop {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn reprotect_slot(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            let mut idx: std::os::raw::c_int = 0;
            ffi::R_ProtectWithIndex(sexps[0], std::ptr::from_mut(&mut idx));
            for &s in &sexps[1..] {
                ffi::R_Reprotect(s, idx);
            }
            ffi::Rf_unprotect(1);
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn vec_pool_overwrite(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };
        let mut pool = unsafe { VecPool::new(16) };

        bencher.bench_local(|| unsafe {
            let slot = pool.insert(sexps[0]);
            for &s in &sexps[1..] {
                ffi::SET_VECTOR_ELT(pool.backing, slot as ffi::R_xlen_t, s);
            }
            pool.release(slot);
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn dll_reinsert(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            let mut cell = preserve::insert_unchecked(sexps[0]);
            for &s in &sexps[1..] {
                preserve::release_unchecked(cell);
                cell = preserve::insert_unchecked(s);
            }
            preserve::release_unchecked(cell);
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn precious_list_churn(bencher: Bencher, n: usize) {
        let sexps = unsafe { prealloc_sexps(n) };

        bencher.bench_local(|| unsafe {
            R_PreserveObject(sexps[0]);
            for &s in &sexps[1..] {
                R_ReleaseObject(sexps[0]); // release previous
                R_PreserveObject(s);
            }
            R_ReleaseObject(*sexps.last().unwrap());
        });

        unsafe { release_prealloc(&sexps) };
    }
}

// endregion

// region: Group 11 — Vec vs VecDeque free list

mod freelist_strategy {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_churn(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { VecPool::new(1024) };

        bencher.bench_local(|| unsafe {
            for _ in 0..n {
                let slot = pool.insert(sexp);
                pool.release(slot);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn deque_churn(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { DequePool::new(1024) };

        bencher.bench_local(|| unsafe {
            for _ in 0..n {
                let slot = pool.insert(sexp);
                pool.release(slot);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }
}

// endregion

// region: Group 12 — Rf_unprotect_ptr at varying depths

mod unprotect_ptr_depth {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [1, 5, 10, 50, 100, 1_000])]
    fn unprotect_ptr_at_depth(bencher: Bencher, depth: usize) {
        let sexps = unsafe { prealloc_sexps(depth) };

        bencher.bench_local(|| unsafe {
            let target = ffi::Rf_protect(sexps[0]);
            for &s in &sexps[1..] {
                ffi::Rf_protect(s);
            }
            ffi::Rf_unprotect_ptr(target);
            if depth > 1 {
                ffi::Rf_unprotect((depth - 1) as i32);
            }
        });

        unsafe { release_prealloc(&sexps) };
    }

    #[divan::bench(args = [1, 5, 10, 50, 100, 1_000])]
    fn bulk_unprotect_baseline(bencher: Bencher, depth: usize) {
        let sexps = unsafe { prealloc_sexps(depth) };

        bencher.bench_local(|| unsafe {
            for &s in &sexps {
                ffi::Rf_protect(s);
            }
            ffi::Rf_unprotect(depth as i32);
        });

        unsafe { release_prealloc(&sexps) };
    }
}

// endregion

// region: Group 13 — Precious list with background pressure

mod precious_list_scale {
    use super::*;
    use divan::Bencher;

    /// 100 insert+release cycles with N background preserved objects.
    #[divan::bench(args = [0, 100, 1_000, 10_000])]
    fn precious_with_background(bencher: Bencher, background_n: usize) {
        let bg = unsafe { prealloc_sexps(background_n) };
        // Preserve background objects
        for &s in &bg {
            unsafe { R_PreserveObject(s) };
        }
        let test_sexp = unsafe { Rf_ScalarInteger(99) };
        unsafe { R_PreserveObject(test_sexp) };

        bencher.bench_local(|| unsafe {
            for _ in 0..100 {
                R_PreserveObject(test_sexp);
                R_ReleaseObject(test_sexp);
            }
        });

        // Cleanup
        for &s in &bg {
            unsafe { R_ReleaseObject(s) };
        }
        unsafe { R_ReleaseObject(test_sexp) };
        unsafe { release_prealloc(&bg) };
    }
}

// endregion

// region: Group 14 — DLL insert's protect stack interaction

mod dll_stack_interaction {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [0, 100, 1_000, 10_000])]
    fn dll_with_stack_depth(bencher: Bencher, stack_depth: usize) {
        let fill_sexps = unsafe { prealloc_sexps(stack_depth) };
        let test_sexp = unsafe { Rf_ScalarInteger(99) };
        unsafe { R_PreserveObject(test_sexp) };

        bencher.bench_local(|| unsafe {
            for &s in &fill_sexps {
                ffi::Rf_protect(s);
            }
            for _ in 0..100 {
                let cell = preserve::insert_unchecked(test_sexp);
                preserve::release_unchecked(cell);
            }
            if stack_depth > 0 {
                ffi::Rf_unprotect(stack_depth as i32);
            }
        });

        unsafe { R_ReleaseObject(test_sexp) };
        unsafe { release_prealloc(&fill_sexps) };
    }

    #[divan::bench(args = [0, 100, 1_000, 10_000])]
    fn pool_with_stack_depth(bencher: Bencher, stack_depth: usize) {
        let fill_sexps = unsafe { prealloc_sexps(stack_depth) };
        let test_sexp = unsafe { Rf_ScalarInteger(99) };
        unsafe { R_PreserveObject(test_sexp) };
        let mut pool = unsafe { VecPool::new(16) };

        bencher.bench_local(|| unsafe {
            for &s in &fill_sexps {
                ffi::Rf_protect(s);
            }
            for _ in 0..100 {
                let slot = pool.insert(test_sexp);
                pool.release(slot);
            }
            if stack_depth > 0 {
                ffi::Rf_unprotect(stack_depth as i32);
            }
        });

        unsafe { R_ReleaseObject(test_sexp) };
        unsafe { release_prealloc(&fill_sexps) };
    }
}

// endregion

// region: Group 15 — Keyed pools

mod keyed_pools {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn hashmap_churn(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { HashMapPool::new(64) };

        bencher.bench_local(|| unsafe {
            for i in 0..n {
                let key = format!("k_{i}");
                pool.insert(key.clone(), sexp);
                divan::black_box(pool.get(&key));
                pool.release(&key);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn indexmap_churn(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { IndexMapPool::new(64) };

        bencher.bench_local(|| unsafe {
            for i in 0..n {
                let key = format!("k_{i}");
                pool.insert(key.clone(), sexp);
                divan::black_box(pool.get(&key));
                pool.release(&key);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn btreemap_churn(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { BTreeMapPool::new(64) };

        bencher.bench_local(|| unsafe {
            for i in 0..n {
                let key = format!("k_{i}");
                pool.insert(key.clone(), sexp);
                divan::black_box(pool.get(&key));
                pool.release(&key);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn slotmap_baseline(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { SlotmapPool::new(n.max(16)) };

        bencher.bench_local(|| unsafe {
            for _ in 0..n {
                let key = pool.insert(sexp);
                divan::black_box(pool.get(key));
                pool.release(key);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }
}

// endregion

// region: Group 16 — slotmap vs Vec overhead

mod slotmap_overhead {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [1_000, 10_000])]
    fn slotmap_insert_release(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { SlotmapPool::new(n.max(16)) };

        bencher.bench_local(|| unsafe {
            let mut keys = Vec::with_capacity(n);
            for _ in 0..n {
                keys.push(pool.insert(sexp));
            }
            for key in keys {
                pool.release(key);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench(args = [1_000, 10_000])]
    fn vec_insert_release(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };
        let mut pool = unsafe { VecPool::new(n.max(16)) };

        bencher.bench_local(|| unsafe {
            let mut slots = Vec::with_capacity(n);
            for _ in 0..n {
                slots.push(pool.insert(sexp));
            }
            for slot in slots {
                pool.release(slot);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }
}

// endregion

// region: Group 17 — Pool growth cost

mod pool_growth {
    use super::*;
    use divan::Bencher;

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_amortized_from_small(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };

        bencher.bench_local(|| unsafe {
            let mut pool = VecPool::new(16);
            let mut slots = Vec::with_capacity(n);
            for _ in 0..n {
                slots.push(pool.insert(sexp));
            }
            for slot in slots {
                pool.release(slot);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn slotmap_amortized_from_small(bencher: Bencher, n: usize) {
        let sexp = unsafe { Rf_ScalarInteger(42) };
        unsafe { R_PreserveObject(sexp) };

        bencher.bench_local(|| unsafe {
            let mut pool = SlotmapPool::new(16);
            let mut keys = Vec::with_capacity(n);
            for _ in 0..n {
                keys.push(pool.insert(sexp));
            }
            for key in keys {
                pool.release(key);
            }
        });

        unsafe { R_ReleaseObject(sexp) };
    }
}

// endregion
