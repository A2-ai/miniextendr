//! RAllocator benchmarks.
//!
//! Planned groups:
//! - `alloc_small` / `alloc_large` (bytes -> KB -> MB)
//! - `realloc_grow` / `realloc_shrink`
//! - `dealloc` cost
//! - compare to System allocator (baseline)
//!
//! Parameters:
//! - size classes
//! - alignment classes
