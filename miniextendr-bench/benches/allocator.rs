//! RAllocator benchmarks.
//!
//! Compares R-backed allocation to the system allocator for common sizes
//! and realistic usage patterns.

use miniextendr_api::allocator::RAllocator;
use std::alloc::{GlobalAlloc, Layout, System};

const ALLOC_SIZES: &[usize] = &[8, 64, 1024, 8192, 65536];
const REALLOC_GROW: &[(usize, usize)] = &[(64, 1024), (1024, 65536)];
const REALLOC_SHRINK: &[(usize, usize)] = &[(1024, 64), (65536, 1024)];

// Batch allocation counts
const BATCH_COUNTS: &[usize] = &[10, 100, 1000];

// Vec-like growth: start size -> final size (doubling pattern)
const VEC_GROWTH: &[(usize, usize)] = &[(8, 1024), (64, 8192), (256, 65536)];

fn main() {
    miniextendr_bench::init();
    divan::main();
}

#[divan::bench(args = ALLOC_SIZES)]
fn rallocator_alloc(size: usize) {
    let alloc = RAllocator;
    unsafe {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = alloc.alloc(layout);
        divan::black_box(ptr);
        if !ptr.is_null() {
            alloc.dealloc(ptr, layout);
        }
    }
}

#[divan::bench(args = ALLOC_SIZES)]
fn system_alloc(size: usize) {
    unsafe {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = System.alloc(layout);
        divan::black_box(ptr);
        if !ptr.is_null() {
            System.dealloc(ptr, layout);
        }
    }
}

#[divan::bench(args = REALLOC_GROW)]
fn rallocator_realloc_grow(case: (usize, usize)) {
    let alloc = RAllocator;
    let (from, to) = case;
    unsafe {
        let layout = Layout::from_size_align(from, 8).unwrap();
        let ptr = alloc.alloc(layout);
        if ptr.is_null() {
            return;
        }
        let new_ptr = alloc.realloc(ptr, layout, to);
        divan::black_box(new_ptr);
        if !new_ptr.is_null() {
            let new_layout = Layout::from_size_align(to, 8).unwrap();
            alloc.dealloc(new_ptr, new_layout);
        }
    }
}

#[divan::bench(args = REALLOC_GROW)]
fn system_realloc_grow(case: (usize, usize)) {
    let (from, to) = case;
    unsafe {
        let layout = Layout::from_size_align(from, 8).unwrap();
        let ptr = System.alloc(layout);
        if ptr.is_null() {
            return;
        }
        let new_ptr = System.realloc(ptr, layout, to);
        divan::black_box(new_ptr);
        if !new_ptr.is_null() {
            let new_layout = Layout::from_size_align(to, 8).unwrap();
            System.dealloc(new_ptr, new_layout);
        }
    }
}

#[divan::bench(args = REALLOC_SHRINK)]
fn rallocator_realloc_shrink(case: (usize, usize)) {
    let alloc = RAllocator;
    let (from, to) = case;
    unsafe {
        let layout = Layout::from_size_align(from, 8).unwrap();
        let ptr = alloc.alloc(layout);
        if ptr.is_null() {
            return;
        }
        let new_ptr = alloc.realloc(ptr, layout, to);
        divan::black_box(new_ptr);
        if !new_ptr.is_null() {
            let new_layout = Layout::from_size_align(to, 8).unwrap();
            alloc.dealloc(new_ptr, new_layout);
        }
    }
}

#[divan::bench(args = REALLOC_SHRINK)]
fn system_realloc_shrink(case: (usize, usize)) {
    let (from, to) = case;
    unsafe {
        let layout = Layout::from_size_align(from, 8).unwrap();
        let ptr = System.alloc(layout);
        if ptr.is_null() {
            return;
        }
        let new_ptr = System.realloc(ptr, layout, to);
        divan::black_box(new_ptr);
        if !new_ptr.is_null() {
            let new_layout = Layout::from_size_align(to, 8).unwrap();
            System.dealloc(new_ptr, new_layout);
        }
    }
}

// region: Batch allocations: allocate N objects, then free all (arena-like pattern)

/// Batch allocate 64-byte objects with RAllocator, then free all.
#[divan::bench(args = BATCH_COUNTS)]
fn rallocator_batch_alloc(count: usize) {
    let alloc = RAllocator;
    let layout = Layout::from_size_align(64, 8).unwrap();
    let mut ptrs = Vec::with_capacity(count);

    unsafe {
        // Allocate all
        for _ in 0..count {
            let ptr = alloc.alloc(layout);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
        divan::black_box(&ptrs);

        // Free all
        for ptr in ptrs {
            alloc.dealloc(ptr, layout);
        }
    }
}

/// Batch allocate 64-byte objects with System allocator, then free all.
#[divan::bench(args = BATCH_COUNTS)]
fn system_batch_alloc(count: usize) {
    let layout = Layout::from_size_align(64, 8).unwrap();
    let mut ptrs = Vec::with_capacity(count);

    unsafe {
        // Allocate all
        for _ in 0..count {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                ptrs.push(ptr);
            }
        }
        divan::black_box(&ptrs);

        // Free all
        for ptr in ptrs {
            System.dealloc(ptr, layout);
        }
    }
}
// endregion

// region: Vec-like growth: repeated doubling reallocs (simulates Vec::push pattern)

/// Simulate Vec growth pattern: start small, double until target size.
#[divan::bench(args = VEC_GROWTH)]
fn rallocator_vec_growth(case: (usize, usize)) {
    let alloc = RAllocator;
    let (start, target) = case;

    unsafe {
        let mut size = start;
        let mut layout = Layout::from_size_align(size, 8).unwrap();
        let mut ptr = alloc.alloc(layout);

        if ptr.is_null() {
            return;
        }

        // Double until we reach target
        while size < target {
            let new_size = size * 2;
            let new_ptr = alloc.realloc(ptr, layout, new_size);
            if new_ptr.is_null() {
                alloc.dealloc(ptr, layout);
                return;
            }
            ptr = new_ptr;
            size = new_size;
            layout = Layout::from_size_align(size, 8).unwrap();
        }

        divan::black_box(ptr);
        alloc.dealloc(ptr, layout);
    }
}

/// Simulate Vec growth pattern with System allocator.
#[divan::bench(args = VEC_GROWTH)]
fn system_vec_growth(case: (usize, usize)) {
    let (start, target) = case;

    unsafe {
        let mut size = start;
        let mut layout = Layout::from_size_align(size, 8).unwrap();
        let mut ptr = System.alloc(layout);

        if ptr.is_null() {
            return;
        }

        // Double until we reach target
        while size < target {
            let new_size = size * 2;
            let new_ptr = System.realloc(ptr, layout, new_size);
            if new_ptr.is_null() {
                System.dealloc(ptr, layout);
                return;
            }
            ptr = new_ptr;
            size = new_size;
            layout = Layout::from_size_align(size, 8).unwrap();
        }

        divan::black_box(ptr);
        System.dealloc(ptr, layout);
    }
}
// endregion

// region: Zeroed allocation comparison

/// Zeroed allocation with RAllocator.
#[divan::bench(args = ALLOC_SIZES)]
fn rallocator_alloc_zeroed(size: usize) {
    let alloc = RAllocator;
    unsafe {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = alloc.alloc_zeroed(layout);
        divan::black_box(ptr);
        if !ptr.is_null() {
            alloc.dealloc(ptr, layout);
        }
    }
}

/// Zeroed allocation with System allocator.
#[divan::bench(args = ALLOC_SIZES)]
fn system_alloc_zeroed(size: usize) {
    unsafe {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = System.alloc_zeroed(layout);
        divan::black_box(ptr);
        if !ptr.is_null() {
            System.dealloc(ptr, layout);
        }
    }
}
// endregion

// region: Mixed workload: interleaved alloc/dealloc of varying sizes

/// Mixed workload: interleaved allocations and deallocations.
/// Simulates real-world usage where objects have varying lifetimes.
#[divan::bench]
fn rallocator_mixed_workload() {
    let alloc = RAllocator;
    let sizes = [32, 128, 512, 2048, 64, 256];

    unsafe {
        let mut live: Vec<(*mut u8, Layout)> = Vec::with_capacity(6);

        // Interleaved pattern: alloc, alloc, free oldest, alloc, free oldest, ...
        for (i, &size) in sizes.iter().enumerate() {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = alloc.alloc(layout);
            if !ptr.is_null() {
                live.push((ptr, layout));
            }

            // Free oldest every 2 allocations
            if i % 2 == 1 && !live.is_empty() {
                let (old_ptr, old_layout) = live.remove(0);
                alloc.dealloc(old_ptr, old_layout);
            }
        }

        divan::black_box(&live);

        // Cleanup remaining
        for (ptr, layout) in live {
            alloc.dealloc(ptr, layout);
        }
    }
}

/// Mixed workload with System allocator.
#[divan::bench]
fn system_mixed_workload() {
    let sizes = [32, 128, 512, 2048, 64, 256];

    unsafe {
        let mut live: Vec<(*mut u8, Layout)> = Vec::with_capacity(6);

        // Interleaved pattern: alloc, alloc, free oldest, alloc, free oldest, ...
        for (i, &size) in sizes.iter().enumerate() {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                live.push((ptr, layout));
            }

            // Free oldest every 2 allocations
            if i % 2 == 1 && !live.is_empty() {
                let (old_ptr, old_layout) = live.remove(0);
                System.dealloc(old_ptr, old_layout);
            }
        }

        divan::black_box(&live);

        // Cleanup remaining
        for (ptr, layout) in live {
            System.dealloc(ptr, layout);
        }
    }
}
// endregion
