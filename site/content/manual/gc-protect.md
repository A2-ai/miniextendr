+++
title = "GC Protection Toolkit"
weight = 18
description = "This document covers miniextendr's RAII-based GC protection facilities."
+++

This document covers miniextendr's RAII-based GC protection facilities.

## Overview

R uses a protection stack to prevent garbage collection of objects that are
still in use. miniextendr provides ergonomic Rust wrappers that automatically
balance `PROTECT`/`UNPROTECT` calls.

### Protection Strategies

| Strategy | Scope | Max Size | Release Order | Use Case |
|----------|-------|----------|---------------|----------|
| [PROTECT stack](#protectscope) | Within `.Call` | ~50k (ppsize) | LIFO | Temporary allocations |
| [ProtectPool](#protectpool) | Cross-`.Call` | Unlimited | Any order | Cross-call, many objects (10.1 ns/op) |
| [Preserve list](#preserve-list) | Cross-`.Call` | Unlimited | Any order | Few long-lived R objects |
| [Refcount arenas](#refcount-arenas) | Flexible | Unlimited | Any order | Legacy - see ProtectPool |

### PROTECT Stack Types

| Type | Purpose |
|------|---------|
| [`ProtectScope`](#protectscope) | Batch protection with automatic `UNPROTECT(n)` on drop |
| [`Root<'scope>`](#root) | Lightweight handle tied to a scope's lifetime |
| [`OwnedProtect`](#ownedprotect) | Single-value RAII guard for simple cases |
| [`ReprotectSlot<'scope>`](#reprotectslot) | Protected slot supporting replace-in-place |

## ProtectScope

The primary tool for managing GC protection. Tracks how many values you protect
and calls `UNPROTECT(n)` when dropped.

```rust
unsafe fn my_call(x: SEXP, y: SEXP) -> SEXP {
    let scope = ProtectScope::new();
    let x = scope.protect(x);
    let y = scope.protect(y);

    let result = scope.protect(some_operation(x.get(), y.get()));
    result.get()
} // UNPROTECT(3) called automatically
```

### Allocation Helpers

Combine allocation and protection in one step:

```rust
// Allocate and protect in one call
let vec = scope.alloc_vector(SEXPTYPE::INTSXP, 100);
let mat = scope.alloc_matrix(SEXPTYPE::REALSXP, 10, 20);
let list = scope.alloc_vecsxp(5);
let strvec = scope.alloc_strsxp(10);
```

### Collecting Iterators (`scope.collect`)

Convert Rust iterators directly to typed R vectors:

```rust
// Type is inferred from the iterator's element type
let ints = scope.collect((0..100).map(|i| i as i32));     // → INTSXP
let reals = scope.collect((0..100).map(|i| i as f64));    // → REALSXP
let raw = scope.collect(vec![1u8, 2, 3, 4]);              // → RAWSXP
```

**Type mapping** (via `RNativeType` trait):

| Rust Type | R Vector Type |
|-----------|---------------|
| `i32` | `INTSXP` |
| `f64` | `REALSXP` |
| `u8` | `RAWSXP` |
| `RLogical` | `LGLSXP` |
| `Rcomplex` | `CPLXSXP` |

**For unknown-length iterators** (e.g., `filter`), collect to `Vec` first:

```rust
// filter() doesn't implement ExactSizeIterator
let evens: Vec<i32> = data.iter()
    .filter(|x| *x % 2 == 0)
    .copied()
    .collect();

// Vec implements ExactSizeIterator
let vec = scope.collect(evens);
```

**Why this is efficient**: Typed vectors (INTSXP, REALSXP, etc.) don't need
per-element protection. You allocate once, protect once, then fill by writing
directly to the data pointer. No GC can occur during fills because you're just
doing pointer writes. No R allocations occur.

## Root

A lightweight handle returned by `scope.protect()`. Has no `Drop` implementation
. The scope owns the unprotection responsibility.

```rust
let root: Root<'_> = scope.protect(sexp);
root.get()      // Access the SEXP
root.into_raw() // Consume and return SEXP (still protected until scope drops)
```

## OwnedProtect

Single-object RAII guard. Calls `UNPROTECT(1)` on drop.

```rust
unsafe fn simple_case() -> SEXP {
    let guard = OwnedProtect::new(Rf_allocVector(REALSXP, 10));
    fill_vector(guard.get());
    guard.get() // Safe: unprotect happens after this expression
}
```

**Warning**: Uses `UNPROTECT(1)` which removes the **top** of the stack.
Nested protections from other sources can cause issues. Prefer `ProtectScope`
for complex scenarios.

## ReprotectSlot

A slot created with `R_ProtectWithIndex` that can be updated in-place via
`R_Reprotect`. Essential for accumulator patterns where you repeatedly replace
a protected value.

```rust
unsafe fn accumulate(n: usize) -> SEXP {
    let scope = ProtectScope::new();
    let slot = scope.protect_with_index(Rf_allocVector(INTSXP, 1));

    for i in 0..n {
        // Replace without growing protect stack
        slot.set(Rf_allocVector(INTSXP, i as isize));
    }

    slot.get()
} // Stack usage: always 1, regardless of n
```

### Methods

| Method | Description |
|--------|-------------|
| `get()` | Get the currently protected SEXP |
| `set(x)` | Replace with new value (calls `R_Reprotect`) |
| `set_with(f)` | Safely replace: calls `f()`, temp-protects result, reprotects |
| `take()` | Return current value and clear slot to `R_NilValue` |
| `replace(x)` | Return current value and set slot to `x` |
| `clear()` | Set slot to `R_NilValue` |

### Safe Replacement with `set_with`

The `set_with` method handles the GC gap that exists between allocating a new
value and reprotecting it:

```rust
// Without set_with (manual pattern):
let new_val = Rf_allocVector(INTSXP, n);  // Unprotected!
Rf_protect(new_val);                       // Temp protect
slot.set(new_val);                         // Reprotect
Rf_unprotect(1);                           // Drop temp

// With set_with (handles it for you):
slot.set_with(|| Rf_allocVector(INTSXP, n));
```

### Option-like Semantics

`take()`, `replace()`, and `clear()` provide `Option`-like ergonomics:

```rust
// Take: get value and clear slot
let old = slot.take();  // slot now holds R_NilValue

// Replace: get old value and set new
let old = slot.replace(new_value);

// Clear: just set to R_NilValue
slot.clear();
```

**Important**: Values returned by `take()` and `replace()` are **unprotected**.
If they need to survive further allocations, protect them explicitly.

## ProtectPool

A VECSXP-backed pool that stores protected SEXPs in a single R list, with
slot management and generational key tracking on the Rust side. Designed for
cross-`.Call` protection of many objects with any-order release.

### Architecture

```text
┌─────────────────────────────────────┐
│  R side: VECSXP (GC-traced slots)   │  ← one R_PreserveObject, ever
│  [SEXP][SEXP][NIL][SEXP][NIL][SEXP] │
└──────┬──────────────────────────────┘
       │ slot indices
┌──────┴──────────────────────────────┐
│  Rust side: Vec<u32> generations    │  ← one free list, one generation array
│  + Vec<usize> free_slots            │
└─────────────────────────────────────┘
```

A single `R_PreserveObject` anchors the backing VECSXP. Each insert writes
into a free slot; each release clears the slot and increments its generation
counter. Keys carry both a slot index and a generation, so stale-key
operations are no-ops rather than crashes.

### API

| Method | Description |
|--------|-------------|
| `ProtectPool::new(capacity)` | Create a pool with an initial VECSXP capacity (grows automatically) |
| `pool.insert(sexp)` | Protect a SEXP, returning a `ProtectKey` |
| `pool.get(key)` | Retrieve the SEXP for a key, or `None` if the key is stale |
| `pool.replace(key, sexp)` | Overwrite an existing slot in-place (pool equivalent of `R_Reprotect`) |
| `pool.release(key)` | Release protection; stale keys are silently ignored |
| `pool.len()` | Number of currently protected objects |
| `pool.contains_key(key)` | Check whether a key is currently valid |

`ProtectKey` is 8 bytes (4-byte slot index + 4-byte generation) and is
`Copy`. Dropping a key without calling `release` leaks protection but does
not crash.

`ProtectPool` is `!Send + !Sync` — all operations must occur on the R main
thread.

### Performance

10.1 ns/op for a single insert+release pair. Zero R allocation per insert —
the backing VECSXP is allocated once at pool creation; inserts reuse existing
slots. Contrast with `preserve` (Preserve List), which allocates one CONSXP
per insert (~28.9 ns/op). See `analysis/gc-protection-benchmarks-results.md`
for full benchmark data.

Automatic growth doubles the backing VECSXP when slots are exhausted.
Growth copies existing slot contents, releases the old VECSXP via
`R_ReleaseObject`, and preserves the new one.

### Example

```rust
use miniextendr_api::protect_pool::{ProtectPool, ProtectKey};

unsafe fn build_cross_call_state() -> (ProtectPool, ProtectKey, ProtectKey) {
    let mut pool = ProtectPool::new(16);

    let s1 = SEXP::scalar_integer(42);
    let s2 = SEXP::scalar_real(3.14);
    let k1 = pool.insert(s1);
    let k2 = pool.insert(s2);

    // Both SEXPs survive GC across .Call boundaries
    (pool, k1, k2)
}

unsafe fn use_cross_call_state(pool: &mut ProtectPool, k1: ProtectKey, k2: ProtectKey) {
    // Retrieve — returns None if the key is stale
    let s1 = pool.get(k1).expect("k1 should still be valid");

    // Release when done — any order, not LIFO
    pool.release(k2);
    pool.release(k1);
}
```

### Comparison to ProtectScope and Preserve List

| | `ProtectScope` | `ProtectPool` | Preserve List |
|---|---|---|---|
| Scope | Within one `.Call` | Cross-`.Call` | Cross-`.Call` |
| Release order | LIFO (drop) | Any order | Any order |
| Per-op cost | 7.4 ns | 10.1 ns | 28.9 ns (CONSXP per insert) |
| R allocation per insert | None | None | One CONSXP |
| Max objects | ~50k (ppsize) | Unlimited (grows) | Unlimited |
| Key safety | Lifetime-bound (`Root<'scope>`) | Generational (stale = no-op) | Manual |

Use `ProtectScope` for temporaries that live only within a single `.Call`
invocation. Use `ProtectPool` when protected objects must outlive a `.Call`
boundary, you have many objects or high insert/release churn, and you need
any-order release. Use the Preserve List when you have a small number of
long-lived objects that are rarely released (e.g., cached lookup tables).

## When to Use What

| Scenario | Use |
|----------|-----|
| Multiple values, known at function start | `ProtectScope` |
| Single value, simple case | `OwnedProtect` |
| Accumulator loop, repeated replacement | `ReprotectSlot` |
| Many SEXPs that must outlive a `.Call`, any-order release | `ProtectPool` |
| Building typed vectors from iterators | `scope.collect()` |
| Building lists with unknown length | `ListAccumulator` |
| Building string vectors | `StrVecBuilder` |

## Protection Patterns by R Type

### Typed Vectors (INTSXP, REALSXP, RAWSXP, LGLSXP, CPLXSXP)

**Simple**: Allocate once, protect once, fill directly.

```rust
let vec = scope.alloc_vector(SEXPTYPE::INTSXP, n);
let ptr = INTEGER(vec.get());
for i in 0..n {
    *ptr.add(i) = i as i32;  // No GC possible here
}
```

Or use `scope.collect()` for even simpler code.

### Lists (VECSXP)

**Complex**: Each element might allocate. Use `ListBuilder` or `ListAccumulator`.

```rust
// Known size
let builder = ListBuilder::new(&scope, n);
for i in 0..n {
    builder.set_protected(i, Rf_ScalarInteger(i));
}

// Unknown size (bounded stack usage)
let mut acc = ListAccumulator::new(&scope, 4);
for item in items {
    acc.push(item);
}
```

### String Vectors (STRSXP)

**Complex**: Each `mkChar` allocates. Use `StrVecBuilder`.

```rust
let builder = StrVecBuilder::new(&scope, n);
for i in 0..n {
    builder.set_str(i, "hello");  // Handles CHARSXP protection
}
```

## Bounded vs Unbounded Stack Usage

**Unbounded** (grows with input size):
```rust
for i in 0..n {
    scope.protect(allocate_something());  // Stack grows to n
}
```

**Bounded** (constant regardless of input):
```rust
let slot = scope.protect_with_index(R_NilValue);
for i in 0..n {
    slot.set(allocate_something());  // Stack stays at 1
}
```

R's default `--max-ppsize` is 50000. Unbounded patterns can overflow this limit
with large inputs. Bounded patterns handle any size.

## Preserve List

For objects that must survive across multiple `.Call` invocations:

```rust
use miniextendr_api::preserve;

// Protect an object (any-order release, not limited by ppsize)
let cell = unsafe { preserve::insert(my_sexp) };

// Later, release it (no LIFO constraint)
unsafe { preserve::release(cell); }
```

Uses a circular doubly-linked cons list internally, itself protected via
`R_PreserveObject`. Each SEXP is stored as the TAG of a cell node.

Use `preserve` when you need to keep a small number of R objects alive across
function calls (e.g., cached lookup tables).

## Refcount Arenas

For scenarios involving many SEXPs (hundreds to millions), the PROTECT stack
is too limited (~50k) and `R_PreserveObject`/`R_ReleaseObject` has O(n)
release cost. Refcount arenas provide O(1) protect/unprotect with reference
counting backed by a VECSXP:

```rust
use miniextendr_api::refcount_protect::ThreadLocalArena;

unsafe {
    ThreadLocalArena::init();

    // Protect (O(1) amortized)
    let sexp = ThreadLocalArena::protect(my_sexp);

    // RAII guard alternative
    // let guard = arena.guard(my_sexp);

    // Unprotect (O(1))
    ThreadLocalArena::unprotect(sexp);
}
```

### Arena Variants

| Type | Backing | Thread-Local | Feature |
|------|---------|-------------|---------|
| `RefCountedArena` | BTreeMap + RefCell | No | - |
| `HashMapArena` | HashMap + RefCell | No | - |
| `ThreadLocalArena` | BTreeMap + thread_local | Yes | - |
| `ThreadLocalHashArena` | HashMap + thread_local | Yes | - |
| `FastHashMapArena` | ahash HashMap + RefCell | No | `refcount-fast-hash` |
| `ThreadLocalFastHashArena` | ahash HashMap + thread_local | Yes | `refcount-fast-hash` |

Thread-local variants avoid RefCell borrow overhead and are fastest for hot loops.
HashMap variants are faster than BTreeMap for large collections (O(1) vs O(log n)).

### Choosing a Strategy

```text
Need GC protection?
├─ Within a single .Call?
│  ├─ 1 value → OwnedProtect
│  ├─ Few values (< 100) → ProtectScope
│  └─ Accumulator loop → ReprotectSlot
├─ Across .Call invocations?
│  ├─ Small number (< 10) → preserve::insert
│  └─ Many values (100+) → ThreadLocalArena / ThreadLocalHashArena
└─ Hot loop with many SEXPs?
   └─ ThreadLocalFastHashArena (requires refcount-fast-hash feature)
```
