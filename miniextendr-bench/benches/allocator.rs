//! RAllocator benchmarks.
//!
//! Compares R-backed allocation to the system allocator for common sizes.

use miniextendr_api::allocator::RAllocator;
use std::alloc::{GlobalAlloc, Layout, System};

const ALLOC_SIZES: &[usize] = &[8, 64, 1024, 8192, 65536];
const REALLOC_GROW: &[(usize, usize)] = &[(64, 1024), (1024, 65536)];
const REALLOC_SHRINK: &[(usize, usize)] = &[(1024, 64), (65536, 1024)];

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
