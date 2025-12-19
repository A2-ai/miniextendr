//! Comprehensive tests for RAllocator
//!
//! All scenarios are run inside a single test to ensure R API calls stay on the
//! thread that initialized R (required for main-thread-only R APIs).

use miniextendr_api::allocator::RAllocator;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_r() {
    INIT.call_once(|| unsafe {
        let engine = miniextendr_engine::REngine::build()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        // Initialize in same order as rpkg/src/entrypoint.c.in
        miniextendr_api::backtrace::miniextendr_panic_hook();
        miniextendr_api::altrep::miniextendr_altrep_init();
        miniextendr_api::worker::miniextendr_worker_init();
        std::mem::forget(engine);
    });
}

#[test]
fn allocator_suite() {
    initialize_r();

    unsafe {
        test_various_sizes();
        test_various_alignments();
        test_zero_size();
        test_realloc_grow();
        test_realloc_shrink();
        test_realloc_same_size();
        test_realloc_null_ptr();
        test_realloc_to_zero();
        test_multiple_allocations();
        test_stress_realloc();
        test_multiple_threads_sequential();
        test_default_stack_threads_sequential();
    }
}

unsafe fn test_various_sizes() {
    for size_pow in 0..=16 {
        let size = 1usize << size_pow;
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(!ptr.is_null(), "alloc failed for size {}", size);
        assert_eq!(ptr.align_offset(8), 0, "alignment violated for size {}", size);

        for i in 0..size {
            *ptr.add(i) = (i % 256) as u8;
        }
        for i in 0..size {
            assert_eq!(*ptr.add(i), (i % 256) as u8, "data corrupted at {}", i);
        }
        RAllocator.dealloc(ptr, layout);
    }
}

unsafe fn test_various_alignments() {
    for align_pow in 0..=8 {
        let align = 1usize << align_pow;
        let size = align * 4;
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(!ptr.is_null(), "alloc failed for align {}", align);
        assert_eq!(ptr as usize % align, 0, "pointer {} not aligned", ptr as usize);
        std::ptr::write_bytes(ptr, 0xAA, size);
        for i in 0..size {
            assert_eq!(*ptr.add(i), 0xAA, "data mismatch at {}", i);
        }
        RAllocator.dealloc(ptr, layout);
    }
}

unsafe fn test_zero_size() {
    let layout = Layout::from_size_align(0, 1).unwrap();
    let ptr = RAllocator.alloc(layout);
    assert!(ptr.is_null(), "zero-size alloc should return null");
    RAllocator.dealloc(ptr, layout);
}

unsafe fn test_realloc_grow() {
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = RAllocator.alloc(layout);
    assert!(!ptr.is_null());
    for i in 0..64 {
        *ptr.add(i) = i as u8;
    }
    let new_ptr = RAllocator.realloc(ptr, layout, 128);
    assert!(!new_ptr.is_null());
    assert_eq!(new_ptr.align_offset(8), 0);
    for i in 0..64 {
        assert_eq!(*new_ptr.add(i), i as u8);
    }
    let new_layout = Layout::from_size_align(128, 8).unwrap();
    RAllocator.dealloc(new_ptr, new_layout);
}

unsafe fn test_realloc_shrink() {
    let layout = Layout::from_size_align(128, 8).unwrap();
    let ptr = RAllocator.alloc(layout);
    for i in 0..128 {
        *ptr.add(i) = (i % 256) as u8;
    }
    let new_ptr = RAllocator.realloc(ptr, layout, 64);
    assert!(!new_ptr.is_null());
    for i in 0..64 {
        assert_eq!(*new_ptr.add(i), (i % 256) as u8);
    }
    let new_layout = Layout::from_size_align(64, 8).unwrap();
    RAllocator.dealloc(new_ptr, new_layout);
}

unsafe fn test_realloc_same_size() {
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = RAllocator.alloc(layout);
    for i in 0..64 {
        *ptr.add(i) = 0xFF;
    }
    let new_ptr = RAllocator.realloc(ptr, layout, 64);
    assert_eq!(ptr, new_ptr);
    for i in 0..64 {
        assert_eq!(*new_ptr.add(i), 0xFF);
    }
    RAllocator.dealloc(new_ptr, layout);
}

unsafe fn test_realloc_null_ptr() {
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = RAllocator.realloc(std::ptr::null_mut(), layout, 64);
    assert!(!ptr.is_null());
    assert_eq!(ptr.align_offset(8), 0);
    RAllocator.dealloc(ptr, layout);
}

unsafe fn test_realloc_to_zero() {
    let layout = Layout::from_size_align(64, 8).unwrap();
    let ptr = RAllocator.alloc(layout);
    let new_ptr = RAllocator.realloc(ptr, layout, 0);
    assert!(new_ptr.is_null());
}

unsafe fn test_multiple_allocations() {
    let mut ptrs = Vec::new();
    for i in 1..=100 {
        let size = i * 16;
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(!ptr.is_null());
        for j in 0..size {
            *ptr.add(j) = (i % 256) as u8;
        }
        ptrs.push((ptr, layout, i as u8));
    }
    for (ptr, layout, marker) in &ptrs {
        for j in 0..layout.size() {
            assert_eq!(*ptr.add(j), *marker);
        }
    }
    for (ptr, layout, _) in ptrs.into_iter().rev() {
        RAllocator.dealloc(ptr, layout);
    }
}

unsafe fn test_stress_realloc() {
    let mut ptr = RAllocator.alloc(Layout::from_size_align(16, 8).unwrap());
    let mut current_size = 16;
    let mut layout = Layout::from_size_align(16, 8).unwrap();
    for i in 0..16 {
        *ptr.add(i) = i as u8;
    }
    for _ in 0..10 {
        let new_size = current_size * 2;
        ptr = RAllocator.realloc(ptr, layout, new_size);
        assert!(!ptr.is_null());
        for i in 0..current_size {
            assert_eq!(*ptr.add(i), (i % 256) as u8);
        }
        for i in current_size..new_size {
            *ptr.add(i) = (i % 256) as u8;
        }
        current_size = new_size;
        layout = Layout::from_size_align(new_size, 8).unwrap();
    }
    RAllocator.dealloc(ptr, layout);
}

unsafe fn test_multiple_threads_sequential() {
    for thread_id in 0..4 {
        let layout = Layout::from_size_align(256, 16).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(!ptr.is_null(), "alloc failed {}", thread_id);
        assert_eq!(ptr.align_offset(16), 0, "alignment failed {}", thread_id);
        for i in 0..256 {
            *ptr.add(i) = ((thread_id * 100 + i) % 256) as u8;
        }
        for i in 0..256 {
            let expected = ((thread_id * 100 + i) % 256) as u8;
            assert_eq!(*ptr.add(i), expected, "data corrupted {}", thread_id);
        }
        RAllocator.dealloc(ptr, layout);
    }
}

unsafe fn test_default_stack_threads_sequential() {
    for thread_id in 0..4 {
        let layout = Layout::from_size_align(128, 8).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(!ptr.is_null(), "alloc failed {}", thread_id);
        assert_eq!(ptr.align_offset(8), 0, "alignment failed {}", thread_id);
        for i in 0..128 {
            *ptr.add(i) = ((thread_id * 50 + i) % 256) as u8;
        }
        for i in 0..128 {
            let expected = ((thread_id * 50 + i) % 256) as u8;
            assert_eq!(*ptr.add(i), expected, "data corrupted {}", thread_id);
        }
        RAllocator.dealloc(ptr, layout);
    }
}
