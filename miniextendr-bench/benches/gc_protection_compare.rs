//! Head-to-head GC protection mechanism benchmarks.
//!
//! Compares protect stack, precious list, DLL preserve, and three VECSXP pool
//! variants (Vec, VecDeque, slotmap) under identical workloads.
//! See `plans/gc-protection-benchmarks.md`.

use miniextendr_api::ffi::{self, R_PreserveObject, R_ReleaseObject, Rf_ScalarInteger, SEXP};
use miniextendr_api::preserve;
use miniextendr_bench::pool_prototypes::{DequePool, SlotmapPool, VecPool};

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: helpers

#[inline(always)]
unsafe fn test_sexp(i: usize) -> SEXP {
    unsafe { Rf_ScalarInteger((i % 1000) as i32) }
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

// region: Group 1 — Single protect + release (latency)
// NOTE: Pool benchmarks include pool creation overhead (VECSXP alloc + R_PreserveObject).
// The "steady_state" group below isolates the per-operation cost on an existing pool.

mod single_latency {
    use super::*;

    #[divan::bench]
    fn protect_stack() {
        unsafe {
            let sexp = test_sexp(0);
            ffi::Rf_protect(sexp);
            ffi::Rf_unprotect(1);
            divan::black_box(sexp);
        }
    }

    #[divan::bench]
    fn precious_list() {
        unsafe {
            let sexp = test_sexp(0);
            R_PreserveObject(sexp);
            R_ReleaseObject(sexp);
            divan::black_box(sexp);
        }
    }

    #[divan::bench]
    fn dll_preserve() {
        unsafe {
            let sexp = test_sexp(0);
            let cell = preserve::insert_unchecked(sexp);
            preserve::release_unchecked(cell);
            divan::black_box(cell);
        }
    }

    #[divan::bench]
    fn vec_pool_cold() {
        unsafe {
            let mut pool = VecPool::new(16);
            let sexp = test_sexp(0);
            let slot = pool.insert(sexp);
            pool.release(slot);
            divan::black_box(slot);
        }
    }

    #[divan::bench]
    fn slotmap_pool_cold() {
        unsafe {
            let mut pool = SlotmapPool::new(16);
            let sexp = test_sexp(0);
            let key = pool.insert(sexp);
            pool.release(key);
            divan::black_box(key);
        }
    }
}

// endregion

// region: Group 1b — Steady-state latency (pool already exists)
// Isolates per-operation cost by doing N ops on an existing pool.

mod steady_state_latency {
    use super::*;

    /// 1000 insert+release on an existing vec pool — amortized per-op cost.
    #[divan::bench]
    fn vec_pool_1000_ops() {
        unsafe {
            let mut pool = VecPool::new(16);
            for i in 0..1000 {
                let slot = pool.insert(test_sexp(i));
                pool.release(slot);
            }
        }
    }

    #[divan::bench]
    fn slotmap_pool_1000_ops() {
        unsafe {
            let mut pool = SlotmapPool::new(16);
            for i in 0..1000 {
                let key = pool.insert(test_sexp(i));
                pool.release(key);
            }
        }
    }

    #[divan::bench]
    fn dll_preserve_1000_ops() {
        unsafe {
            for i in 0..1000 {
                let cell = preserve::insert_unchecked(test_sexp(i));
                preserve::release_unchecked(cell);
            }
        }
    }

    #[divan::bench]
    fn protect_stack_1000_ops() {
        unsafe {
            for i in 0..1000 {
                ffi::Rf_protect(test_sexp(i));
                ffi::Rf_unprotect(1);
            }
        }
    }

    #[divan::bench]
    fn precious_list_1000_ops() {
        unsafe {
            for i in 0..1000 {
                let s = test_sexp(i);
                R_PreserveObject(s);
                R_ReleaseObject(s);
            }
        }
    }
}

// endregion

// region: Group 2 — Batch protect N, release all

mod batch_throughput {
    use super::*;

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn protect_stack(n: usize) {
        unsafe {
            for i in 0..n {
                ffi::Rf_protect(test_sexp(i));
            }
            ffi::Rf_unprotect(n as i32);
        }
    }

    // Precious list: cap at 10k (50k is O(n²) ≈ minutes)
    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn precious_list(n: usize) {
        unsafe {
            let mut sexps = Vec::with_capacity(n);
            for i in 0..n {
                let s = test_sexp(i);
                R_PreserveObject(s);
                sexps.push(s);
            }
            for s in sexps {
                R_ReleaseObject(s);
            }
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn dll_preserve(n: usize) {
        unsafe {
            let mut cells = Vec::with_capacity(n);
            for i in 0..n {
                cells.push(preserve::insert_unchecked(test_sexp(i)));
            }
            for cell in cells {
                preserve::release_unchecked(cell);
            }
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn vec_pool(n: usize) {
        unsafe {
            let mut pool = VecPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn slotmap_pool(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(n.max(16));
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            for key in keys {
                pool.release(key);
            }
        }
    }


    // Pre-sized pool (best case) vs small initial (realistic, includes growth)
    #[divan::bench(args = [1_000, 10_000, 50_000])]
    fn vec_pool_small_initial(n: usize) {
        unsafe {
            let mut pool = VecPool::new(16); // starts tiny, must grow
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 50_000])]
    fn slotmap_pool_small_initial(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(16); // starts tiny, must grow
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            for key in keys {
                pool.release(key);
            }
        }
    }
}

// endregion

// region: Group 3 — Interleaved insert/release (churn)

mod churn {
    use super::*;

    #[divan::bench(args = [1_000, 10_000])]
    fn precious_list(n: usize) {
        unsafe {
            let mut live: Vec<SEXP> = Vec::new();
            for i in 0..n {
                let s = test_sexp(i);
                R_PreserveObject(s);
                live.push(s);
                if i % 3 == 0 && !live.is_empty() {
                    R_ReleaseObject(live.remove(0));
                }
            }
            for s in live {
                R_ReleaseObject(s);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn dll_preserve(n: usize) {
        unsafe {
            let mut live: Vec<SEXP> = Vec::new();
            for i in 0..n {
                live.push(preserve::insert_unchecked(test_sexp(i)));
                if i % 3 == 0 && !live.is_empty() {
                    preserve::release_unchecked(live.remove(0));
                }
            }
            for cell in live {
                preserve::release_unchecked(cell);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_pool(n: usize) {
        unsafe {
            let mut pool = VecPool::new(1024);
            let mut live: Vec<usize> = Vec::new();
            for i in 0..n {
                live.push(pool.insert(test_sexp(i)));
                if i % 3 == 0 && !live.is_empty() {
                    pool.release(live.remove(0));
                }
            }
            for slot in live {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn slotmap_pool(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(1024);
            let mut live = Vec::new();
            for i in 0..n {
                live.push(pool.insert(test_sexp(i)));
                if i % 3 == 0 && !live.is_empty() {
                    pool.release(live.remove(0));
                }
            }
            for key in live {
                pool.release(key);
            }
        }
    }
}

// endregion

// region: Group 4 — LIFO release

mod lifo_release {
    use super::*;

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn protect_stack(n: usize) {
        unsafe {
            for i in 0..n {
                ffi::Rf_protect(test_sexp(i));
            }
            ffi::Rf_unprotect(n as i32);
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn dll_preserve(n: usize) {
        unsafe {
            let mut cells = Vec::with_capacity(n);
            for i in 0..n {
                cells.push(preserve::insert_unchecked(test_sexp(i)));
            }
            for cell in cells.into_iter().rev() {
                preserve::release_unchecked(cell);
            }
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn vec_pool(n: usize) {
        unsafe {
            let mut pool = VecPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots.into_iter().rev() {
                pool.release(slot);
            }
        }
    }
}

// endregion

// region: Group 5 — Random release order

mod random_release {
    use super::*;

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn precious_list(n: usize) {
        unsafe {
            let mut sexps = Vec::with_capacity(n);
            for i in 0..n {
                let s = test_sexp(i);
                R_PreserveObject(s);
                sexps.push(s);
            }
            let mut order: Vec<usize> = (0..n).collect();
            shuffle(&mut order);
            for i in order {
                R_ReleaseObject(sexps[i]);
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn dll_preserve(n: usize) {
        unsafe {
            let mut cells = Vec::with_capacity(n);
            for i in 0..n {
                cells.push(preserve::insert_unchecked(test_sexp(i)));
            }
            let mut order: Vec<usize> = (0..n).collect();
            shuffle(&mut order);
            for i in order {
                preserve::release_unchecked(cells[i]);
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn vec_pool(n: usize) {
        unsafe {
            let mut pool = VecPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            let mut order: Vec<usize> = (0..n).collect();
            shuffle(&mut order);
            for i in order {
                pool.release(slots[i]);
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn slotmap_pool(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(n.max(16));
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            let mut order: Vec<usize> = (0..n).collect();
            shuffle(&mut order);
            for i in order {
                pool.release(keys[i]);
            }
        }
    }
}

// endregion

// region: Group 6 — Bursty (alloc many, release most)

mod bursty {
    use super::*;

    const BURST_SIZE: usize = 10_000;
    const KEEP: usize = 100;

    #[divan::bench(args = [3, 10])]
    fn dll_preserve(rounds: usize) {
        unsafe {
            let mut kept: Vec<SEXP> = Vec::new();
            for round in 0..rounds {
                let mut cells = Vec::with_capacity(BURST_SIZE);
                for i in 0..BURST_SIZE {
                    cells.push(preserve::insert_unchecked(test_sexp(round * BURST_SIZE + i)));
                }
                for cell in cells.drain(KEEP..) {
                    preserve::release_unchecked(cell);
                }
                kept.extend(cells);
            }
            for cell in kept {
                preserve::release_unchecked(cell);
            }
        }
    }

    #[divan::bench(args = [3, 10])]
    fn vec_pool(rounds: usize) {
        unsafe {
            let mut pool = VecPool::new(BURST_SIZE);
            let mut kept: Vec<usize> = Vec::new();
            for round in 0..rounds {
                let mut slots = Vec::with_capacity(BURST_SIZE);
                for i in 0..BURST_SIZE {
                    slots.push(pool.insert(test_sexp(round * BURST_SIZE + i)));
                }
                for slot in slots.drain(KEEP..) {
                    pool.release(slot);
                }
                kept.extend(slots);
            }
            for slot in kept {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [3, 10])]
    fn slotmap_pool(rounds: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(BURST_SIZE);
            let mut kept = Vec::new();
            for round in 0..rounds {
                let mut keys = Vec::with_capacity(BURST_SIZE);
                for i in 0..BURST_SIZE {
                    keys.push(pool.insert(test_sexp(round * BURST_SIZE + i)));
                }
                for key in keys.drain(KEEP..) {
                    pool.release(key);
                }
                kept.extend(keys);
            }
            for key in kept {
                pool.release(key);
            }
        }
    }
}

// endregion

// region: Group 7 — Replace in loop

mod replace_in_loop {
    use super::*;

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn reprotect_slot(n: usize) {
        unsafe {
            let mut idx: std::os::raw::c_int = 0;
            ffi::R_ProtectWithIndex(test_sexp(0), std::ptr::from_mut(&mut idx));
            for i in 1..n {
                ffi::R_Reprotect(test_sexp(i), idx);
            }
            ffi::Rf_unprotect(1);
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn dll_reinsert(n: usize) {
        unsafe {
            let mut cell = preserve::insert_unchecked(test_sexp(0));
            for i in 1..n {
                preserve::release_unchecked(cell);
                cell = preserve::insert_unchecked(test_sexp(i));
            }
            preserve::release_unchecked(cell);
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn vec_pool_overwrite(n: usize) {
        unsafe {
            let mut pool = VecPool::new(16);
            let slot = pool.insert(test_sexp(0));
            for i in 1..n {
                ffi::SET_VECTOR_ELT(pool.backing, slot as ffi::R_xlen_t, test_sexp(i));
            }
            pool.release(slot);
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn precious_list_churn(n: usize) {
        unsafe {
            let mut sexp = test_sexp(0);
            R_PreserveObject(sexp);
            for i in 1..n {
                R_ReleaseObject(sexp);
                sexp = test_sexp(i);
                R_PreserveObject(sexp);
            }
            R_ReleaseObject(sexp);
        }
    }
}

// endregion

// region: Group 8 — Data.frame construction

mod dataframe_construction {
    use super::*;
    use miniextendr_api::ffi::{
        R_NamesSymbol, Rf_allocVector, Rf_mkCharLenCE, Rf_setAttrib, SET_STRING_ELT,
        SET_VECTOR_ELT, SEXPTYPE,
    };

    const COL_LEN: usize = 1000;

    #[divan::bench(args = [5, 20, 100])]
    fn protect_scope(ncols: usize) {
        unsafe {
            let list = ffi::Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, ncols as ffi::R_xlen_t));
            let names =
                ffi::Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, ncols as ffi::R_xlen_t));
            for i in 0..ncols {
                let col = ffi::Rf_protect(Rf_allocVector(
                    SEXPTYPE::INTSXP,
                    COL_LEN as ffi::R_xlen_t,
                ));
                let ptr = ffi::INTEGER(col);
                for j in 0..COL_LEN {
                    *ptr.add(j) = (i * COL_LEN + j) as i32;
                }
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, col);
                ffi::Rf_unprotect(1);
                let name = format!("col_{i}");
                let ch = Rf_mkCharLenCE(name.as_ptr().cast(), name.len() as i32, ffi::CE_UTF8);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
            }
            Rf_setAttrib(list, R_NamesSymbol, names);
            ffi::Rf_unprotect(2);
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [5, 20, 100])]
    fn dll_preserve(ncols: usize) {
        unsafe {
            let list = Rf_allocVector(SEXPTYPE::VECSXP, ncols as ffi::R_xlen_t);
            let lc = preserve::insert_unchecked(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, ncols as ffi::R_xlen_t);
            let nc = preserve::insert_unchecked(names);
            for i in 0..ncols {
                let col = Rf_allocVector(SEXPTYPE::INTSXP, COL_LEN as ffi::R_xlen_t);
                let cc = preserve::insert_unchecked(col);
                let ptr = ffi::INTEGER(col);
                for j in 0..COL_LEN {
                    *ptr.add(j) = (i * COL_LEN + j) as i32;
                }
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, col);
                preserve::release_unchecked(cc);
                let name = format!("col_{i}");
                let ch = Rf_mkCharLenCE(name.as_ptr().cast(), name.len() as i32, ffi::CE_UTF8);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
            }
            Rf_setAttrib(list, R_NamesSymbol, names);
            preserve::release_unchecked(nc);
            preserve::release_unchecked(lc);
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [5, 20, 100])]
    fn vec_pool(ncols: usize) {
        unsafe {
            let mut pool = VecPool::new(ncols + 4);
            let list = Rf_allocVector(SEXPTYPE::VECSXP, ncols as ffi::R_xlen_t);
            let ls = pool.insert(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, ncols as ffi::R_xlen_t);
            let ns = pool.insert(names);
            for i in 0..ncols {
                let col = Rf_allocVector(SEXPTYPE::INTSXP, COL_LEN as ffi::R_xlen_t);
                let cs = pool.insert(col);
                let ptr = ffi::INTEGER(col);
                for j in 0..COL_LEN {
                    *ptr.add(j) = (i * COL_LEN + j) as i32;
                }
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, col);
                pool.release(cs);
                let name = format!("col_{i}");
                let ch = Rf_mkCharLenCE(name.as_ptr().cast(), name.len() as i32, ffi::CE_UTF8);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
            }
            Rf_setAttrib(list, R_NamesSymbol, names);
            pool.release(ns);
            pool.release(ls);
            divan::black_box(list);
        }
    }
}

// endregion

// region: Group 9 — JSON-like nested object construction
// Models: allocate key (CHARSXP), allocate value (scalar or nested list),
// set into parent, release temporary. Multiple live temporaries per iteration.

mod json_construction {
    use super::*;
    use miniextendr_api::ffi::{
        Rf_allocVector, Rf_mkCharLenCE, SET_STRING_ELT, SET_VECTOR_ELT, SEXPTYPE,
    };

    /// Build a flat named list with N string keys and integer values.
    /// Each iteration: alloc CHARSXP (for key) + alloc ScalarInteger (for value).
    /// The CHARSXP is consumed by SET_STRING_ELT; the scalar by SET_VECTOR_ELT.
    /// Between those two allocations, the first needs protection.

    #[divan::bench(args = [10, 100, 1_000])]
    fn protect_scope(n: usize) {
        unsafe {
            let list = ffi::Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, n as ffi::R_xlen_t));
            let names = ffi::Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t));
            for i in 0..n {
                // Key: alloc CHARSXP
                let key = format!("key_{i}");
                let ch = Rf_mkCharLenCE(key.as_ptr().cast(), key.len() as i32, ffi::CE_UTF8);
                // Value: alloc scalar — this can trigger GC, so ch needs protection
                ffi::Rf_protect(ch);
                let val = ffi::Rf_ScalarInteger(i as i32);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);
                ffi::Rf_unprotect(1); // ch
            }
            ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);
            ffi::Rf_unprotect(2);
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [10, 100, 1_000])]
    fn vec_pool(n: usize) {
        unsafe {
            let mut pool = VecPool::new(4); // small — only 2-3 live at a time
            let list = Rf_allocVector(SEXPTYPE::VECSXP, n as ffi::R_xlen_t);
            let ls = pool.insert(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
            let ns = pool.insert(names);
            for i in 0..n {
                let key = format!("key_{i}");
                let ch = Rf_mkCharLenCE(key.as_ptr().cast(), key.len() as i32, ffi::CE_UTF8);
                let cs = pool.insert(ch);
                let val = ffi::Rf_ScalarInteger(i as i32);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);
                pool.release(cs); // ch now reachable from names
            }
            ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);
            pool.release(ns);
            pool.release(ls);
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [10, 100, 1_000])]
    fn deque_pool(n: usize) {
        unsafe {
            let mut pool = DequePool::new(4);
            let list = Rf_allocVector(SEXPTYPE::VECSXP, n as ffi::R_xlen_t);
            let ls = pool.insert(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
            let ns = pool.insert(names);
            for i in 0..n {
                let key = format!("key_{i}");
                let ch = Rf_mkCharLenCE(key.as_ptr().cast(), key.len() as i32, ffi::CE_UTF8);
                let cs = pool.insert(ch);
                let val = ffi::Rf_ScalarInteger(i as i32);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);
                pool.release(cs);
            }
            ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);
            pool.release(ns);
            pool.release(ls);
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [10, 100, 1_000])]
    fn dll_preserve(n: usize) {
        unsafe {
            let list = Rf_allocVector(SEXPTYPE::VECSXP, n as ffi::R_xlen_t);
            let lc = preserve::insert_unchecked(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, n as ffi::R_xlen_t);
            let nc = preserve::insert_unchecked(names);
            for i in 0..n {
                let key = format!("key_{i}");
                let ch = Rf_mkCharLenCE(key.as_ptr().cast(), key.len() as i32, ffi::CE_UTF8);
                let cc = preserve::insert_unchecked(ch);
                let val = ffi::Rf_ScalarInteger(i as i32);
                SET_STRING_ELT(names, i as ffi::R_xlen_t, ch);
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, val);
                preserve::release_unchecked(cc);
            }
            ffi::Rf_setAttrib(list, ffi::R_NamesSymbol, names);
            preserve::release_unchecked(nc);
            preserve::release_unchecked(lc);
            divan::black_box(list);
        }
    }
}

// endregion

// region: Group 10 — High-turnover temporaries with multiple live
// Models: 3 live temporaries at any moment, each replaced every iteration.
// Tests the "window" of live protections, not bulk insert/release.

mod high_turnover {
    use super::*;

    #[divan::bench(args = [1_000, 10_000])]
    fn protect_stack_3_live(n: usize) {
        unsafe {
            // 3 ReprotectSlots — constant stack usage
            let mut idx_a: std::os::raw::c_int = 0;
            let mut idx_b: std::os::raw::c_int = 0;
            let mut idx_c: std::os::raw::c_int = 0;
            ffi::R_ProtectWithIndex(test_sexp(0), std::ptr::from_mut(&mut idx_a));
            ffi::R_ProtectWithIndex(test_sexp(1), std::ptr::from_mut(&mut idx_b));
            ffi::R_ProtectWithIndex(test_sexp(2), std::ptr::from_mut(&mut idx_c));
            for i in 0..n {
                ffi::R_Reprotect(test_sexp(i * 3), idx_a);
                ffi::R_Reprotect(test_sexp(i * 3 + 1), idx_b);
                ffi::R_Reprotect(test_sexp(i * 3 + 2), idx_c);
            }
            ffi::Rf_unprotect(3);
        }
    }

    #[divan::bench(args = [1_000, 10_000])]
    fn vec_pool_3_live(n: usize) {
        unsafe {
            let mut pool = VecPool::new(4);
            let a = pool.insert(test_sexp(0));
            let b = pool.insert(test_sexp(1));
            let c = pool.insert(test_sexp(2));
            for i in 0..n {
                // Overwrite in place — no release/reinsert needed
                ffi::SET_VECTOR_ELT(pool.backing, a as ffi::R_xlen_t, test_sexp(i * 3));
                ffi::SET_VECTOR_ELT(pool.backing, b as ffi::R_xlen_t, test_sexp(i * 3 + 1));
                ffi::SET_VECTOR_ELT(pool.backing, c as ffi::R_xlen_t, test_sexp(i * 3 + 2));
            }
            pool.release(a);
            pool.release(b);
            pool.release(c);
        }
    }

    #[divan::bench(args = [1_000, 10_000])]
    fn dll_3_live(n: usize) {
        unsafe {
            let mut a = preserve::insert_unchecked(test_sexp(0));
            let mut b = preserve::insert_unchecked(test_sexp(1));
            let mut c = preserve::insert_unchecked(test_sexp(2));
            for i in 0..n {
                preserve::release_unchecked(a);
                a = preserve::insert_unchecked(test_sexp(i * 3));
                preserve::release_unchecked(b);
                b = preserve::insert_unchecked(test_sexp(i * 3 + 1));
                preserve::release_unchecked(c);
                c = preserve::insert_unchecked(test_sexp(i * 3 + 2));
            }
            preserve::release_unchecked(a);
            preserve::release_unchecked(b);
            preserve::release_unchecked(c);
        }
    }
}

// endregion

// region: Group 11 — Vec vs VecDeque free list

mod freelist_strategy {
    use super::*;

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_churn(n: usize) {
        unsafe {
            let mut pool = VecPool::new(1024);
            for i in 0..n {
                let slot = pool.insert(test_sexp(i));
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn deque_churn(n: usize) {
        unsafe {
            let mut pool = DequePool::new(1024);
            for i in 0..n {
                let slot = pool.insert(test_sexp(i));
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_burst(n: usize) {
        unsafe {
            let mut pool = VecPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            // Release oldest half
            for slot in slots.drain(..n / 2) {
                pool.release(slot);
            }
            // Reinsert half
            for i in 0..n / 2 {
                slots.push(pool.insert(test_sexp(n + i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn deque_burst(n: usize) {
        unsafe {
            let mut pool = DequePool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots.drain(..n / 2) {
                pool.release(slot);
            }
            for i in 0..n / 2 {
                slots.push(pool.insert(test_sexp(n + i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }
}

// endregion

// region: Group 12 — Rf_unprotect_ptr at varying depths

mod unprotect_ptr_depth {
    use super::*;

    #[divan::bench(args = [1, 5, 10, 50, 100, 1_000])]
    fn unprotect_ptr_at_depth(depth: usize) {
        unsafe {
            let target = ffi::Rf_protect(test_sexp(0));
            for i in 1..depth {
                ffi::Rf_protect(test_sexp(i));
            }
            ffi::Rf_unprotect_ptr(target);
            if depth > 1 {
                ffi::Rf_unprotect((depth - 1) as i32);
            }
        }
    }

    #[divan::bench(args = [1, 5, 10, 50, 100, 1_000])]
    fn bulk_unprotect_baseline(depth: usize) {
        unsafe {
            for i in 0..depth {
                ffi::Rf_protect(test_sexp(i));
            }
            ffi::Rf_unprotect(depth as i32);
        }
    }
}

// endregion

// region: Group 16 — slotmap generational check overhead

mod slotmap_overhead {
    use super::*;

    #[divan::bench(args = [1_000, 10_000])]
    fn slotmap_insert_release(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(n.max(16));
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            for key in keys {
                pool.release(key);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000])]
    fn vec_insert_release(n: usize) {
        unsafe {
            let mut pool = VecPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000])]
    fn slotmap_get_hot(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(n.max(16));
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            for _ in 0..10 {
                for &key in &keys {
                    divan::black_box(pool.get(key));
                }
            }
            for key in keys {
                pool.release(key);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000])]
    fn vec_get_hot(n: usize) {
        unsafe {
            let mut pool = VecPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for _ in 0..10 {
                for &slot in &slots {
                    divan::black_box(pool.get(slot));
                }
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }
}

// endregion

// region: Group 17 — Pool growth cost

mod pool_growth {
    use super::*;

    #[divan::bench(args = [16, 64, 256, 1024])]
    fn vec_growth_spike(initial_cap: usize) {
        unsafe {
            let mut pool = VecPool::new(initial_cap);
            let mut slots = Vec::with_capacity(initial_cap + 1);
            for i in 0..=initial_cap {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [16, 64, 256, 1024])]
    fn slotmap_growth_spike(initial_cap: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(initial_cap);
            let mut keys = Vec::with_capacity(initial_cap + 1);
            for i in 0..=initial_cap {
                keys.push(pool.insert(test_sexp(i)));
            }
            for key in keys {
                pool.release(key);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vec_amortized_from_small(n: usize) {
        unsafe {
            let mut pool = VecPool::new(16);
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn slotmap_amortized_from_small(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(16);
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            for key in keys {
                pool.release(key);
            }
        }
    }
}

// endregion

// region: Group 15 — Keyed pool collections (HashMap, BTreeMap, IndexMap)

mod keyed_pools {
    use super::*;
    use miniextendr_bench::pool_prototypes::{BTreeMapPool, HashMapPool, IndexMapPool};

    // Insert N keyed entries, lookup each, release all.

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn hashmap_insert_get_release(n: usize) {
        unsafe {
            let mut pool = HashMapPool::new(n.max(16));
            for i in 0..n {
                pool.insert(format!("key_{i}"), test_sexp(i));
            }
            for i in 0..n {
                divan::black_box(pool.get(&format!("key_{i}")));
            }
            for i in 0..n {
                pool.release(&format!("key_{i}"));
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn btreemap_insert_get_release(n: usize) {
        unsafe {
            let mut pool = BTreeMapPool::new(n.max(16));
            for i in 0..n {
                pool.insert(format!("key_{i}"), test_sexp(i));
            }
            for i in 0..n {
                divan::black_box(pool.get(&format!("key_{i}")));
            }
            for i in 0..n {
                pool.release(&format!("key_{i}"));
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn indexmap_insert_get_release(n: usize) {
        unsafe {
            let mut pool = IndexMapPool::new(n.max(16));
            for i in 0..n {
                pool.insert(format!("key_{i}"), test_sexp(i));
            }
            for i in 0..n {
                divan::black_box(pool.get(&format!("key_{i}")));
            }
            for i in 0..n {
                pool.release(&format!("key_{i}"));
            }
        }
    }

    // Baseline: slotmap (no key overhead, for comparison)
    #[divan::bench(args = [100, 1_000, 10_000])]
    fn slotmap_baseline(n: usize) {
        unsafe {
            let mut pool = SlotmapPool::new(n.max(16));
            let mut keys = Vec::with_capacity(n);
            for i in 0..n {
                keys.push(pool.insert(test_sexp(i)));
            }
            for &key in &keys {
                divan::black_box(pool.get(key));
            }
            for key in keys {
                pool.release(key);
            }
        }
    }

    // Churn: insert, lookup, release one at a time.
    #[divan::bench(args = [100, 1_000, 10_000])]
    fn hashmap_churn(n: usize) {
        unsafe {
            let mut pool = HashMapPool::new(64);
            for i in 0..n {
                let key = format!("k_{i}");
                pool.insert(key.clone(), test_sexp(i));
                divan::black_box(pool.get(&key));
                pool.release(&key);
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn indexmap_churn(n: usize) {
        unsafe {
            let mut pool = IndexMapPool::new(64);
            for i in 0..n {
                let key = format!("k_{i}");
                pool.insert(key.clone(), test_sexp(i));
                divan::black_box(pool.get(&key));
                pool.release(&key);
            }
        }
    }

    #[divan::bench(args = [100, 1_000, 10_000])]
    fn btreemap_churn(n: usize) {
        unsafe {
            let mut pool = BTreeMapPool::new(64);
            for i in 0..n {
                let key = format!("k_{i}");
                pool.insert(key.clone(), test_sexp(i));
                divan::black_box(pool.get(&key));
                pool.release(&key);
            }
        }
    }
}

// endregion
