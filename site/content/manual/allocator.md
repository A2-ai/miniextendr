+++
title = "R-Backed Global Allocator"
weight = 51
description = "This document covers RAllocator, a Rust GlobalAlloc implementation backed by R's memory manager."
+++

This document covers `RAllocator`, a Rust `GlobalAlloc` implementation backed
by R's memory manager.

## Overview

`RAllocator` routes every Rust heap allocation through R's `Rf_allocVector(RAWSXP, n)`,
so Rust memory participates in R's garbage collection. Each allocation is
GC-protected via the [preserve list](../gc-protect/#preserve-list) and released
on dealloc.

**Source:** `miniextendr-api/src/allocator.rs`

## When to Use

| Scenario | Use RAllocator? |
|----------|-----------------|
| Standalone binary embedding R | Yes |
| Arena-style allocation in `.Call` | Yes |
| `#[global_allocator]` in an R package lib crate | **No** - would be called at compile time when R isn't available |
| Performance-critical hot loops | Probably not - system allocator is faster |

## Memory Layout

Each allocation creates one R `RAWSXP` vector. Inside its data region:

```text
RAWSXP data bytes:
┌──────────────────┬──────────────┬──────────────────────┐
│  alignment pad   │   Header     │   user bytes ...     │
│  (0..align-1)    │  (8 bytes)   │                      │
└──────────────────┴──────────────┴──────────────────────┘
                                  ▲
                                  └── pointer returned to caller
```

The `Header` stores a single `preserve_tag: SEXP`, the cell in the preserve
list that keeps this RAWSXP alive. On dealloc, the allocator reads the header
to recover the tag and releases the preserve cell.

## How It Works

### Allocation

1. Compute total size: alignment padding + `Header` (8 bytes) + requested size
2. `Rf_allocVector(RAWSXP, total)` - allocate an R raw vector
3. `preserve::insert(sexp)` - protect from GC (any-order release, not LIFO)
4. Write the `Header` (preserve tag) immediately before the user pointer
5. Return the aligned user pointer

### Deallocation

1. Read the `Header` just before the pointer → recover `preserve_tag`
2. `preserve::release(tag)` - R's GC can now reclaim the RAWSXP

### Reallocation

1. Recover the original RAWSXP via the header's preserve tag
2. Check if the existing RAWSXP has spare capacity (possible due to alignment
   over-allocation)
3. If it fits → return the same pointer (no copy)
4. Otherwise → allocate new RAWSXP, copy data, release old

### Zero-Sized Types

ZST allocations (`layout.size() == 0`) return null. There's no RAWSXP to
track, and the dangling pointer convention would crash in dealloc when trying
to read the header.

## Thread Safety

All R API calls are routed to the main thread automatically:

| Calling Thread | Behavior |
|----------------|----------|
| R main thread | Executes directly (default path) |
| Worker thread (with `worker-thread` feature, inside `run_on_worker`) | Routes via `with_r_thread` |
| Other threads (Rayon, spawned) | **Panics** |

The panic on arbitrary threads is intentional. R's C API is not thread-safe,
and silently corrupting R's heap would be worse than a loud failure.

## Caveats

### longjmp Risk

`Rf_allocVector` can `longjmp` on allocation failure instead of returning NULL.
If this happens:

- **Inside `with_r_unwind_protect`** (default path): `R_UnwindProtect` catches
  the longjmp, Rust destructors run normally
- **Inside `run_on_worker`** (with `worker-thread` feature): same protection
  via `R_UnwindProtect`
- **Outside protected context**: Rust destructors are **skipped**, causing resource
  leaks (files, locks, etc.)

Best practice: use `RAllocator` inside `with_r_unwind_protect` or the worker
thread pattern, where unwind protection is active.

### Protection Strategy

The allocator uses the **preserve list** (not the PROTECT stack) because:

- Allocations may survive across multiple `.Call` invocations
- Deallocations happen in arbitrary order (not LIFO)
- The preserve list supports O(1) insert and any-order release

See [GC Protection](../gc-protect/) for the full picture of protection strategies.

## Example

```rust
use miniextendr_api::RAllocator;

// In a standalone binary that embeds R:
#[global_allocator]
static ALLOC: RAllocator = RAllocator;

fn main() {
    // All Vec, String, Box, etc. allocations now go through R
    let v = vec![1, 2, 3]; // backed by RAWSXP
}
```

**Do NOT do this in an R package library crate**. The allocator would be
invoked during `cargo build` before R is available.
