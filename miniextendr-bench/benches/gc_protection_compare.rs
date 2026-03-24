//! Head-to-head GC protection mechanism benchmarks.
//!
//! Compares all four mechanisms (protect stack, precious list, DLL preserve,
//! VECSXP pool) under identical workloads. See `plans/gc-protection-benchmarks.md`.

use miniextendr_api::ffi::{
    self, R_NilValue, R_PreserveObject, R_ReleaseObject, Rf_ScalarInteger, Rf_protect,
    Rf_unprotect, Rf_unprotect_ptr, SEXP,
};
use miniextendr_api::preserve;

fn main() {
    miniextendr_bench::init();
    divan::main();
}

// region: helpers

/// Allocate a test SEXP (cheap scalar integer).
#[inline(always)]
unsafe fn test_sexp(i: usize) -> SEXP {
    unsafe { Rf_ScalarInteger((i % 1000) as i32) }
}

// A minimal VECSXP pool for benchmarking (no slotmap dependency needed yet).
// Uses Vec<usize> free list — measures the raw pool overhead without generational checks.
struct VecsxpPool {
    backing: SEXP,
    capacity: usize,
    len: usize,
    free_list: Vec<usize>,
}

impl VecsxpPool {
    unsafe fn new(capacity: usize) -> Self {
        unsafe {
            let backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, capacity as ffi::R_xlen_t);
            R_PreserveObject(backing);
            Self {
                backing,
                capacity,
                len: 0,
                free_list: Vec::with_capacity(capacity),
            }
        }
    }

    #[inline]
    unsafe fn insert(&mut self, sexp: SEXP) -> usize {
        let slot = if let Some(s) = self.free_list.pop() {
            s
        } else {
            if self.len >= self.capacity {
                unsafe { self.grow() };
            }
            let s = self.len;
            self.len += 1;
            s
        };
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, sexp) };
        slot
    }

    #[inline]
    unsafe fn release(&mut self, slot: usize) {
        unsafe { ffi::SET_VECTOR_ELT(self.backing, slot as ffi::R_xlen_t, R_NilValue) };
        self.free_list.push(slot);
    }

    unsafe fn grow(&mut self) {
        let new_cap = self.capacity * 2;
        unsafe {
            let new_backing =
                ffi::Rf_allocVector(ffi::SEXPTYPE::VECSXP, new_cap as ffi::R_xlen_t);
            Rf_protect(new_backing);
            R_PreserveObject(new_backing);
            for i in 0..self.capacity {
                let elt = ffi::VECTOR_ELT(self.backing, i as ffi::R_xlen_t);
                ffi::SET_VECTOR_ELT(new_backing, i as ffi::R_xlen_t, elt);
            }
            R_ReleaseObject(self.backing);
            Rf_unprotect(1);
            self.backing = new_backing;
            self.capacity = new_cap;
        }
    }
}

impl Drop for VecsxpPool {
    fn drop(&mut self) {
        unsafe { R_ReleaseObject(self.backing) };
    }
}

// endregion

// region: Group 1 — Single protect + release (latency)

mod single_latency {
    use super::*;

    #[divan::bench]
    fn protect_stack() {
        unsafe {
            let sexp = test_sexp(0);
            Rf_protect(sexp);
            Rf_unprotect(1);
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
    fn vecsxp_pool() {
        unsafe {
            // Pool created once, reused across iterations via black_box trick
            // Actually divan runs this many times, so create a pool per call for fairness
            let mut pool = VecsxpPool::new(16);
            let sexp = test_sexp(0);
            let slot = pool.insert(sexp);
            pool.release(slot);
            divan::black_box(slot);
        }
    }
}

// endregion

// region: Group 2 — Protect N, then release all (batch throughput)

mod batch_throughput {
    use super::*;

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
    fn protect_stack(n: usize) {
        unsafe {
            for i in 0..n {
                Rf_protect(test_sexp(i));
            }
            Rf_unprotect(n as i32);
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000, 50_000])]
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
    fn vecsxp_pool(n: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(n.max(16));
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }
}

// endregion

// region: Group 3 — Interleaved insert/release (churn)

mod churn {
    use super::*;

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn precious_list(n: usize) {
        unsafe {
            let mut live: Vec<SEXP> = Vec::new();
            for i in 0..n {
                let s = test_sexp(i);
                R_PreserveObject(s);
                live.push(s);
                if i % 3 == 0 && !live.is_empty() {
                    let oldest = live.remove(0);
                    R_ReleaseObject(oldest);
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
                    let oldest = live.remove(0);
                    preserve::release_unchecked(oldest);
                }
            }
            for cell in live {
                preserve::release_unchecked(cell);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn vecsxp_pool(n: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(1024);
            let mut live: Vec<usize> = Vec::new();
            for i in 0..n {
                live.push(pool.insert(test_sexp(i)));
                if i % 3 == 0 && !live.is_empty() {
                    let oldest = live.remove(0);
                    pool.release(oldest);
                }
            }
            for slot in live {
                pool.release(slot);
            }
        }
    }
}

// endregion

// region: Group 4 — LIFO release (protect stack's ideal)

mod lifo_release {
    use super::*;

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn protect_stack(n: usize) {
        unsafe {
            for i in 0..n {
                Rf_protect(test_sexp(i));
            }
            Rf_unprotect(n as i32);
        }
    }

    #[divan::bench(args = [10, 100, 1_000, 10_000])]
    fn precious_list(n: usize) {
        unsafe {
            let mut sexps = Vec::with_capacity(n);
            for i in 0..n {
                let s = test_sexp(i);
                R_PreserveObject(s);
                sexps.push(s);
            }
            // Release in reverse (LIFO)
            for s in sexps.into_iter().rev() {
                R_ReleaseObject(s);
            }
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
    fn vecsxp_pool(n: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(n.max(16));
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

    /// Simple deterministic shuffle (Fisher-Yates with fixed seed).
    fn shuffle(v: &mut [usize]) {
        let mut state: u64 = 0xDEAD_BEEF_CAFE_1234;
        for i in (1..v.len()).rev() {
            // xorshift64
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let j = (state as usize) % (i + 1);
            v.swap(i, j);
        }
    }

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
    fn vecsxp_pool(n: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(n.max(16));
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
}

// endregion

// region: Group 6 — Bursty workload (allocate many, release most)

mod bursty {
    use super::*;

    const BURST_SIZE: usize = 10_000;
    const KEEP_PER_BURST: usize = 100;

    #[divan::bench(args = [3, 10])]
    fn dll_preserve(rounds: usize) {
        unsafe {
            let mut kept: Vec<SEXP> = Vec::new();
            for round in 0..rounds {
                let mut cells = Vec::with_capacity(BURST_SIZE);
                for i in 0..BURST_SIZE {
                    cells.push(preserve::insert_unchecked(test_sexp(round * BURST_SIZE + i)));
                }
                // Release all but KEEP_PER_BURST
                for cell in cells.drain(KEEP_PER_BURST..) {
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
    fn vecsxp_pool(rounds: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(BURST_SIZE);
            let mut kept: Vec<usize> = Vec::new();
            for round in 0..rounds {
                let mut slots = Vec::with_capacity(BURST_SIZE);
                for i in 0..BURST_SIZE {
                    slots.push(pool.insert(test_sexp(round * BURST_SIZE + i)));
                }
                for slot in slots.drain(KEEP_PER_BURST..) {
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
    fn precious_list(rounds: usize) {
        unsafe {
            let mut kept: Vec<SEXP> = Vec::new();
            for round in 0..rounds {
                let mut sexps = Vec::with_capacity(BURST_SIZE);
                for i in 0..BURST_SIZE {
                    let s = test_sexp(round * BURST_SIZE + i);
                    R_PreserveObject(s);
                    sexps.push(s);
                }
                for s in sexps.drain(KEEP_PER_BURST..) {
                    R_ReleaseObject(s);
                }
                kept.extend(sexps);
            }
            for s in kept {
                R_ReleaseObject(s);
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
            Rf_unprotect(1);
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
    fn pool_overwrite(n: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(16);
            let slot = pool.insert(test_sexp(0));
            for i in 1..n {
                // Overwrite in place — no release/reinsert needed
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

// region: Group 12 — Rf_unprotect_ptr at various depths

mod unprotect_ptr_depth {
    use super::*;

    #[divan::bench(args = [1, 5, 10, 50, 100, 1_000])]
    fn unprotect_ptr_at_depth(depth: usize) {
        unsafe {
            // The target is the first thing protected (deepest in stack)
            let target = Rf_protect(test_sexp(0));
            for i in 1..depth {
                Rf_protect(test_sexp(i));
            }
            // Remove the deepest item — scans entire stack
            Rf_unprotect_ptr(target);
            // Clean up the rest
            if depth > 1 {
                Rf_unprotect((depth - 1) as i32);
            }
        }
    }

    #[divan::bench(args = [1, 5, 10, 50, 100, 1_000])]
    fn unprotect_bulk_baseline(depth: usize) {
        unsafe {
            for i in 0..depth {
                Rf_protect(test_sexp(i));
            }
            Rf_unprotect(depth as i32);
        }
    }
}

// endregion

// region: Group 8 — Data.frame construction (realistic composite)

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
            // Protect list
            let list = Rf_protect(Rf_allocVector(SEXPTYPE::VECSXP, ncols as ffi::R_xlen_t));
            // Protect names
            let names = Rf_protect(Rf_allocVector(SEXPTYPE::STRSXP, ncols as ffi::R_xlen_t));

            for i in 0..ncols {
                // Each column allocation — protect until set into list
                let col = Rf_protect(Rf_allocVector(
                    SEXPTYPE::INTSXP,
                    COL_LEN as ffi::R_xlen_t,
                ));
                let ptr = ffi::INTEGER(col);
                for j in 0..COL_LEN {
                    *ptr.add(j) = (i * COL_LEN + j) as i32;
                }
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, col);
                Rf_unprotect(1); // col now reachable from list

                let name = format!("col_{i}");
                let charsxp = Rf_mkCharLenCE(
                    name.as_ptr().cast(),
                    name.len() as i32,
                    ffi::CE_UTF8,
                );
                SET_STRING_ELT(names, i as ffi::R_xlen_t, charsxp);
            }

            Rf_setAttrib(list, R_NamesSymbol, names);
            Rf_unprotect(2); // list + names
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [5, 20, 100])]
    fn dll_preserve(ncols: usize) {
        unsafe {
            let list = Rf_allocVector(SEXPTYPE::VECSXP, ncols as ffi::R_xlen_t);
            let list_cell = preserve::insert_unchecked(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, ncols as ffi::R_xlen_t);
            let names_cell = preserve::insert_unchecked(names);

            for i in 0..ncols {
                let col = Rf_allocVector(SEXPTYPE::INTSXP, COL_LEN as ffi::R_xlen_t);
                let col_cell = preserve::insert_unchecked(col);
                let ptr = ffi::INTEGER(col);
                for j in 0..COL_LEN {
                    *ptr.add(j) = (i * COL_LEN + j) as i32;
                }
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, col);
                preserve::release_unchecked(col_cell);

                let name = format!("col_{i}");
                let charsxp = Rf_mkCharLenCE(
                    name.as_ptr().cast(),
                    name.len() as i32,
                    ffi::CE_UTF8,
                );
                SET_STRING_ELT(names, i as ffi::R_xlen_t, charsxp);
            }

            Rf_setAttrib(list, R_NamesSymbol, names);
            preserve::release_unchecked(names_cell);
            preserve::release_unchecked(list_cell);
            divan::black_box(list);
        }
    }

    #[divan::bench(args = [5, 20, 100])]
    fn vecsxp_pool(ncols: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(ncols + 4);
            let list = Rf_allocVector(SEXPTYPE::VECSXP, ncols as ffi::R_xlen_t);
            let list_slot = pool.insert(list);
            let names = Rf_allocVector(SEXPTYPE::STRSXP, ncols as ffi::R_xlen_t);
            let names_slot = pool.insert(names);

            for i in 0..ncols {
                let col = Rf_allocVector(SEXPTYPE::INTSXP, COL_LEN as ffi::R_xlen_t);
                let col_slot = pool.insert(col);
                let ptr = ffi::INTEGER(col);
                for j in 0..COL_LEN {
                    *ptr.add(j) = (i * COL_LEN + j) as i32;
                }
                SET_VECTOR_ELT(list, i as ffi::R_xlen_t, col);
                pool.release(col_slot);

                let name = format!("col_{i}");
                let charsxp = Rf_mkCharLenCE(
                    name.as_ptr().cast(),
                    name.len() as i32,
                    ffi::CE_UTF8,
                );
                SET_STRING_ELT(names, i as ffi::R_xlen_t, charsxp);
            }

            Rf_setAttrib(list, R_NamesSymbol, names);
            pool.release(names_slot);
            pool.release(list_slot);
            divan::black_box(list);
        }
    }
}

// endregion

// region: Group 17 — Pool growth cost

mod pool_growth {
    use super::*;

    #[divan::bench(args = [16, 64, 256, 1024])]
    fn growth_spike(initial_cap: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(initial_cap);
            // Fill to capacity, then one more to trigger growth
            let mut slots = Vec::with_capacity(initial_cap + 1);
            for i in 0..=initial_cap {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }

    #[divan::bench(args = [1_000, 10_000, 100_000])]
    fn amortized_from_small(n: usize) {
        unsafe {
            let mut pool = VecsxpPool::new(16); // small initial, many growths
            let mut slots = Vec::with_capacity(n);
            for i in 0..n {
                slots.push(pool.insert(test_sexp(i)));
            }
            for slot in slots {
                pool.release(slot);
            }
        }
    }
}

// endregion
