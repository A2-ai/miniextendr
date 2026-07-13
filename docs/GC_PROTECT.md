# GC Protection Toolkit

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

#### Full Wrapper Reference

| Method | Wraps | Description |
|--------|-------|-------------|
| `alloc_vector(ty, n)` | `Rf_allocVector` | Generic vector of type `ty` and length `n` |
| `alloc_matrix(ty, nrow, ncol)` | `Rf_allocMatrix` | 2-D matrix |
| `alloc_list(n)` | `Rf_allocList` | Pairlist (LISTSXP) of length `n` |
| `alloc_vecsxp(n)` | `Rf_allocVector(VECSXP)` | Generic list (VECSXP) |
| `alloc_strsxp(n)` | `Rf_allocVector(STRSXP)` | Character vector |
| `alloc_integer(n)` | `Rf_allocVector(INTSXP)` | Integer vector |
| `alloc_real(n)` | `Rf_allocVector(REALSXP)` | Real vector |
| `alloc_logical(n)` | `Rf_allocVector(LGLSXP)` | Logical vector |
| `alloc_raw(n)` | `Rf_allocVector(RAWSXP)` | Raw vector |
| `alloc_complex(n)` | `Rf_allocVector(CPLXSXP)` | Complex vector |
| `alloc_character(n)` | `Rf_allocVector(STRSXP)` | Character vector (alias) |
| `alloc_array(ty, dims)` | `Rf_allocArray` | N-D array; dims INTSXP protected inside scope |
| `alloc_3d_array(ty, nrow, ncol, nface)` | `Rf_alloc3DArray` | 3-D array |
| `alloc_lang(n)` | `Rf_allocLang` | Language object (LANGSXP) of length `n` |
| `alloc_s4_object()` | `Rf_allocS4Object` | S4 object (S4SXP) |
| `mkchar(s)` | `Rf_mkCharLenCE(..., CE_UTF8)` | CHARSXP from `&str` (UTF-8) |
| `mkchar_ce(s, enc)` | `Rf_mkCharLenCE` | CHARSXP from `&str` with specified encoding |
| `mkchar_len_ce(bytes, enc)` | `Rf_mkCharLenCE` | CHARSXP from `&[u8]` with specified encoding |
| `cons(car, cdr)` | `Rf_cons` | Pairlist cons cell |
| `lcons(car, cdr)` | `Rf_lcons` | Language cons cell |
| `lengthgets(x, n)` | `Rf_lengthgets` | Resize vector, returns new protected copy |
| `xlengthgets(x, n)` | `Rf_xlengthgets` | Resize vector (long-vector form) |
| `duplicate(x)` | `Rf_duplicate` | Deep copy |
| `shallow_duplicate(x)` | `Rf_shallow_duplicate` | Shallow copy |
| `coerce(x, ty)` | `Rf_coerce` | Type coercion |
| `new_env(parent, hash, size)` | `R_NewEnv` | New environment |
| `make_external_ptr(p, tag, prot)` | `R_MakeExternalPtr` | Raw external pointer (escape hatch; prefer `ExternalPtr<T>`) |
| `scalar_integer(x)` | `Rf_ScalarInteger` | Scalar integer |
| `scalar_real(x)` | `Rf_ScalarReal` | Scalar real |
| `scalar_logical(x)` | `Rf_ScalarLogical` | Scalar logical |
| `scalar_complex(x)` | `Rf_ScalarComplex` | Scalar complex |
| `scalar_raw(x)` | `Rf_ScalarRaw` | Scalar raw |
| `scalar_string(s)` | `Rf_ScalarString(Rf_mkCharLenCE(...))` | Scalar character |
| `collect(iter)` | _(typed fill)_ | Allocate + fill from exact-size iterator |

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

## Preserve List

R's own cross-`.Call` rooting primitive: call `R_PreserveObject` directly to
add a SEXP to R's internal preserved list, and `R_ReleaseObject` to remove it.
There is no `ProtectScope`/`ProtectPool`/`OwnedProtect`-style Rust wrapper for
this — miniextendr code that needs a single long-lived root calls the two FFI
functions directly and pairs them with a `Drop` impl. `BuiltDataFrame`
(`miniextendr-api/src/dataframe.rs`) is the canonical example: it calls
`R_PreserveObject` in its constructor and `R_ReleaseObject` in `Drop`, rooting
exactly one frame for the handle's lifetime. `TxtProgressBar` and the
`ExternalPtr` type-tag table (`mx_abi.rs`) follow the same pattern.

```rust
unsafe fn root_for_later(sexp: SEXP) {
    R_PreserveObject(sexp); // adds one CONSXP to R's preserved list
    // ... sexp survives GC across `.Call` boundaries until released ...
    R_ReleaseObject(sexp); // O(n) scan of the preserved list
}
```

Cost is the ~28.9 ns/op figure in the [Performance](#performance) comparison
above: each insert allocates one CONSXP, and `R_ReleaseObject` scans R's
preserved list linearly, so releasing many objects — or releasing out of
insertion order — degrades as the list grows. `ProtectPool` exists precisely
to avoid this: it wraps a *single* `R_PreserveObject` call around a whole
VECSXP and manages slots itself, giving O(1) any-order release regardless of
object count.

Reach for the Preserve List directly only for a small number of long-lived,
rarely-released objects (a singleton cache, or one owned handle like
`BuiltDataFrame`). For anything with higher churn or object count, prefer
`ProtectPool`.

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

## ProtectPool

For objects that must survive across multiple `.Call` invocations:

```rust
use miniextendr_api::protect_pool::ProtectPool;

// Pool backed by a VECSXP with generational keys (O(1) insert/release).
let mut pool = unsafe { ProtectPool::new(16) };
let key = unsafe { pool.insert(my_sexp) };

// Later, release it (no LIFO constraint)
unsafe { pool.release(key); }
```

`ProtectPool` keeps protected SEXPs in a VECSXP backing slot, indexed by
generational keys (slotmap-style). Single VECSXP protect via
`R_PreserveObject` on construction; per-insert is a slot write with no R
allocation.

Use `ProtectPool` (or `R_PreserveObject` directly) when you need to keep R
objects alive across function calls (e.g., cached lookup tables).

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

| Type | Backing | Thread-Local |
|------|---------|-------------|
| `RefCountedArena` | BTreeMap + RefCell | No |
| `ThreadLocalArena` | BTreeMap + thread_local | Yes |

Thread-local variants avoid RefCell borrow overhead and are fastest for hot loops.
These are the only two flavors instantiated anywhere in the tree; a HashMap-
and ahash-backed variant family existed previously but was removed for having
zero production or test consumers (see `refcount_protect.rs`'s module docs for
the tracking issue if you need to re-add one).

### Choosing a Strategy

```text
Need GC protection?
├─ Within a single .Call?
│  ├─ 1 value → OwnedProtect
│  ├─ Few values (< 100) → ProtectScope
│  └─ Accumulator loop → ReprotectSlot
├─ Across .Call invocations?
│  └─ Small number (< 10) → ProtectPool or R_PreserveObject
└─ Many values (100+) / hot loop with many SEXPs?
   └─ ThreadLocalArena (or RefCountedArena for a non-thread-local instance)
```
