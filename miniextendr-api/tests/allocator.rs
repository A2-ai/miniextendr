//! Comprehensive tests for RAllocator
//!
//! These integration tests use miniextendr-engine to initialize R

use miniextendr_api::allocator::RAllocator;
use std::alloc::{GlobalAlloc, Layout};
use std::sync::Once;

static INIT: Once = Once::new();

fn initialize_r() {
    INIT.call_once(|| unsafe {
        let engine = miniextendr_engine::REngine::new()
            .with_args(&["R", "--quiet", "--vanilla"])
            .init()
            .expect("Failed to initialize R");
        std::mem::forget(engine);
    });
}

#[test]
fn test_allocator_various_sizes() {
    initialize_r();

    unsafe {
        // Test power-of-2 sizes
        for size_pow in 0..=16 {
            let size = 1usize << size_pow; // 1, 2, 4, 8, ..., 65536
            let layout = Layout::from_size_align(size, 8).unwrap();

            let ptr = RAllocator.alloc(layout);
            assert!(!ptr.is_null(), "alloc failed for size {}", size);
            assert_eq!(
                ptr.align_offset(8),
                0,
                "alignment violated for size {}",
                size
            );

            // Write pattern to verify writable
            for i in 0..size {
                *ptr.add(i) = (i % 256) as u8;
            }

            // Verify pattern
            for i in 0..size {
                assert_eq!(
                    *ptr.add(i),
                    (i % 256) as u8,
                    "data corrupted at offset {}",
                    i
                );
            }

            RAllocator.dealloc(ptr, layout);
        }
    }
}

#[test]
fn test_allocator_various_alignments() {
    initialize_r();
    unsafe {
        // Test alignments: 1, 2, 4, 8, 16, 32, 64, 128, 256
        for align_pow in 0..=8 {
            let align = 1usize << align_pow;
            let size = align * 4; // Allocate 4 units of alignment
            let layout = Layout::from_size_align(size, align).unwrap();

            let ptr = RAllocator.alloc(layout);
            assert!(!ptr.is_null(), "alloc failed for align {}", align);
            assert_eq!(
                ptr as usize % align,
                0,
                "pointer {} not aligned to {}",
                ptr as usize,
                align
            );

            // Write and verify
            std::ptr::write_bytes(ptr, 0xAA, size);
            for i in 0..size {
                assert_eq!(*ptr.add(i), 0xAA, "data mismatch at offset {}", i);
            }

            RAllocator.dealloc(ptr, layout);
        }
    }
}

#[test]
fn test_allocator_zero_size() {
    initialize_r();
    unsafe {
        let layout = Layout::from_size_align(0, 1).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(ptr.is_null(), "zero-size alloc should return null");

        // Dealloc of null should be safe
        RAllocator.dealloc(ptr, layout);
    }
}

#[test]
fn test_allocator_realloc_grow() {
    initialize_r();
    unsafe {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = RAllocator.alloc(layout);
        assert!(!ptr.is_null());

        // Write pattern
        for i in 0..64 {
            *ptr.add(i) = i as u8;
        }

        // Grow to 128 bytes
        let new_ptr = RAllocator.realloc(ptr, layout, 128);
        assert!(!new_ptr.is_null());
        assert_eq!(new_ptr.align_offset(8), 0, "realloc broke alignment");

        // Verify old data preserved
        for i in 0..64 {
            assert_eq!(*new_ptr.add(i), i as u8, "data lost at offset {}", i);
        }

        let new_layout = Layout::from_size_align(128, 8).unwrap();
        RAllocator.dealloc(new_ptr, new_layout);
    }
}

#[test]
fn test_allocator_realloc_shrink() {
    initialize_r();
    unsafe {
        let layout = Layout::from_size_align(128, 8).unwrap();
        let ptr = RAllocator.alloc(layout);

        // Write pattern
        for i in 0..128 {
            *ptr.add(i) = (i % 256) as u8;
        }

        // Shrink to 64 bytes - should reuse same allocation
        let new_ptr = RAllocator.realloc(ptr, layout, 64);
        assert!(!new_ptr.is_null());

        // Likely same pointer (optimization)
        // Verify first 64 bytes preserved
        for i in 0..64 {
            assert_eq!(*new_ptr.add(i), (i % 256) as u8);
        }

        let new_layout = Layout::from_size_align(64, 8).unwrap();
        RAllocator.dealloc(new_ptr, new_layout);
    }
}

#[test]
fn test_allocator_realloc_same_size() {
    initialize_r();
    unsafe {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = RAllocator.alloc(layout);

        for i in 0..64 {
            *ptr.add(i) = 0xFF;
        }

        // Realloc to same size - should return same pointer
        let new_ptr = RAllocator.realloc(ptr, layout, 64);
        assert_eq!(ptr, new_ptr, "realloc same size should reuse pointer");

        // Data should be unchanged
        for i in 0..64 {
            assert_eq!(*new_ptr.add(i), 0xFF);
        }

        RAllocator.dealloc(new_ptr, layout);
    }
}

#[test]
fn test_allocator_realloc_null_ptr() {
    initialize_r();
    unsafe {
        // Realloc of null should behave like alloc
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = RAllocator.realloc(std::ptr::null_mut(), layout, 64);

        assert!(!ptr.is_null());
        assert_eq!(ptr.align_offset(8), 0);

        RAllocator.dealloc(ptr, layout);
    }
}

#[test]
fn test_allocator_realloc_to_zero() {
    initialize_r();
    unsafe {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = RAllocator.alloc(layout);

        // Realloc to zero should dealloc and return null
        let new_ptr = RAllocator.realloc(ptr, layout, 0);
        assert!(new_ptr.is_null(), "realloc to zero should return null");
    }
}

#[test]
fn test_allocator_multiple_allocations() {
    initialize_r();
    unsafe {
        let mut ptrs = Vec::new();

        // Allocate 100 different sizes
        for i in 1..=100 {
            let size = i * 16;
            let layout = Layout::from_size_align(size, 8).unwrap();
            let ptr = RAllocator.alloc(layout);

            assert!(!ptr.is_null());

            // Mark with pattern
            for j in 0..size {
                *ptr.add(j) = (i % 256) as u8;
            }

            ptrs.push((ptr, layout, i as u8));
        }

        // Verify all allocations still valid
        for (ptr, layout, marker) in &ptrs {
            for j in 0..layout.size() {
                assert_eq!(*ptr.add(j), *marker);
            }
        }

        // Deallocate in reverse order
        for (ptr, layout, _) in ptrs.into_iter().rev() {
            RAllocator.dealloc(ptr, layout);
        }
    }
}

#[test]
fn test_allocator_stress_realloc() {
    initialize_r();
    unsafe {
        let mut ptr = RAllocator.alloc(Layout::from_size_align(16, 8).unwrap());
        let mut current_size = 16;
        let mut layout = Layout::from_size_align(16, 8).unwrap();

        // Pattern: start with 16 bytes
        for i in 0..16 {
            *ptr.add(i) = i as u8;
        }

        // Grow exponentially, verify data preserved
        for _ in 0..10 {
            let new_size = current_size * 2;
            ptr = RAllocator.realloc(ptr, layout, new_size);
            assert!(!ptr.is_null());

            // Verify old data still there
            for i in 0..current_size {
                assert_eq!(*ptr.add(i), (i % 256) as u8);
            }

            // Fill new space
            for i in current_size..new_size {
                *ptr.add(i) = (i % 256) as u8;
            }

            current_size = new_size;
            layout = Layout::from_size_align(new_size, 8).unwrap();
        }

        RAllocator.dealloc(ptr, layout);
    }
}

#[test]
fn test_allocator_multiple_threads() {
    initialize_r();

    use miniextendr_api::thread::RThreadBuilder;

    // Spawn 4 threads, each allocates and verifies data
    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            RThreadBuilder::new()
                .stack_size(8 * 1024 * 1024)
                .name(format!("alloc-test-{}", thread_id))
                .spawn(move || unsafe {
                    let layout = Layout::from_size_align(256, 16).unwrap();
                    let ptr = RAllocator.alloc(layout);

                    assert!(!ptr.is_null(), "Thread {} alloc failed", thread_id);
                    assert_eq!(
                        ptr.align_offset(16),
                        0,
                        "Thread {} alignment failed",
                        thread_id
                    );

                    // Write thread-specific pattern
                    for i in 0..256 {
                        *ptr.add(i) = ((thread_id * 100 + i) % 256) as u8;
                    }

                    // Verify pattern
                    for i in 0..256 {
                        let expected = ((thread_id * 100 + i) % 256) as u8;
                        assert_eq!(
                            *ptr.add(i),
                            expected,
                            "Thread {} data corrupted at {}",
                            thread_id,
                            i
                        );
                    }

                    RAllocator.dealloc(ptr, layout);
                    thread_id
                })
                .expect("Failed to spawn thread")
        })
        .collect();

    // Join all threads
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.join().expect("Thread panicked");
        assert_eq!(result, i, "Thread {} returned wrong value", i);
    }
}
