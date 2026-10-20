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
| [ProtectPool](#protectpool) | Until released or the pool drops | Unlimited | Any order | Values a struct owns, many values, cross-call roots (10.1 ns/op) |
| [Preserve list](#preserve-list) (`Preserved`) | Cross-`.Call` | Unlimited | Any order, O(n) release | One long-lived R object |
| [Refcount arenas](#refcount-arenas) | Flexible | Unlimited | Any order | Legacy - see ProtectPool |

See [When to Use What](#when-to-use-what) for the decision tree.

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

`UNPROTECT(n)` pops the top `n` entries of R's protect stack, whoever pushed
them, so scopes must drop in LIFO order: don't protect through a scope while a
scope or guard made after it is alive, and drop it after every such scope or
guard. `ProtectScope::new` is `unsafe` and its caller promises this. A scope
moved into a struct is easy to drop out of order; give the struct a
[`ProtectPool`](#a-struct-that-owns-its-roots) instead.

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
slot management and generational key tracking on the Rust side. It has two
uses:

- **The per-call root set.** `pool.root(value)` converts a value, roots it
  until the pool drops, and returns the object. A struct can own the pool and
  drop in any order relative to other pools, scopes and guards, and the pool
  takes no protect-stack entry per value.
- **The cross-call pool.** `pool.insert(sexp)` returns a key, and
  `pool.release(key)` frees the slot in O(1), in any order and across `.Call`
  boundaries. Every owning `ExternalPtr` is rooted in one such process-wide
  pool.

Its functions are safe to call on R's main thread, the contract of
`Preserved` and the other owned roots; the type is `!Send + !Sync`.

### A struct that owns its roots

Code that keeps intermediate R objects alive until its `.Call` returns
(attribute vectors, results of R calls, sliced columns) used to move a
`ProtectScope` into a struct:

```rust
// Before: unsafe at every site, and unsound if dropped out of order.
pub(crate) struct Columns {
    scope: ProtectScope,
    cols: Vec<(String, SEXP)>,
}

impl Columns {
    pub(crate) fn keep(&mut self, sexp: SEXP) -> SEXP {
        unsafe { self.scope.protect_raw(sexp) }
    }
}
```

Two such structs dropped in creation order break the stack: the first one's
`UNPROTECT(n)` pops the second one's entries, which are on top, and leaves
its own behind. Own a `ProtectPool` instead:

```rust
// After: safe, and any drop order.
pub(crate) struct Columns {
    roots: ProtectPool,
    cols: Vec<(String, SEXP)>,
}

impl Columns {
    pub(crate) fn new() -> Self {
        Self {
            roots: ProtectPool::new(ProtectPool::DEFAULT_CAPACITY),
            cols: Vec::new(),
        }
    }

    pub(crate) fn keep(&mut self, value: impl IntoR) -> SEXP {
        self.roots.root(value)
    }
}
```

Dropping the pool releases its backing list, and with it every value, whatever
order the structs drop in. `root` (and `insert`) accept a fresh, unprotected
object: when the pool is full, the object stays on the protect stack while the
pool grows, so the allocation that grows it cannot collect it.

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
| `pool.root(value)` | Convert an `IntoR` value, root it until the pool drops, and return the SEXP |
| `pool.insert(sexp)` | Protect a SEXP, returning a `ProtectKey` |
| `pool.get(key)` | Retrieve the SEXP for a key, or `None` if the key is stale |
| `pool.replace(key, sexp)` | Overwrite an existing slot in-place (pool equivalent of `R_Reprotect`) |
| `pool.release(key)` | Release protection; stale keys are silently ignored |
| `pool.len()` | Number of currently protected objects |
| `pool.contains_key(key)` | Check whether a key is currently valid |

`ProtectKey` is 8 bytes (4-byte slot index + 4-byte generation) and is
`Copy`. Dropping a key without calling `release` keeps the value rooted until
the pool drops. A key does not record its pool: use it only with the pool that
returned it.

`ProtectPool` is `!Send + !Sync` — all operations must occur on the R main
thread.

### Performance

10.1 ns/op for a single insert+release pair. Zero R allocation per insert —
the backing VECSXP is allocated once at pool creation; inserts reuse existing
slots. Contrast with R's precious list (`Preserved`), which allocates one
CONSXP per preserve and searches the list on release (13 ns/op for one
preserve+release, O(n²) when many objects are released in insertion order).
See `analysis/gc-protection-benchmarks-results.md` for full benchmark data.

Automatic growth doubles the backing VECSXP when slots are exhausted.
Growth copies existing slot contents, releases the old VECSXP via
`R_ReleaseObject`, and preserves the new one.

### Example

```rust
use miniextendr_api::protect_pool::{ProtectPool, ProtectKey};

fn build_cross_call_state() -> (ProtectPool, ProtectKey, ProtectKey) {
    let mut pool = ProtectPool::new(16);

    let s1 = SEXP::scalar_integer(42);
    let s2 = SEXP::scalar_real(3.14);
    let k1 = pool.insert(s1);
    let k2 = pool.insert(s2);

    // Both SEXPs survive GC across .Call boundaries
    (pool, k1, k2)
}

fn use_cross_call_state(pool: &mut ProtectPool, k1: ProtectKey, k2: ProtectKey) {
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
| Scope | Within one `.Call` | Until released or the pool drops | Cross-`.Call` |
| Release order | LIFO (drop) | Any order | Any order, O(n) search |
| Per-op cost | 7.4 ns | 10.1 ns | 13 ns single; O(n²) in bulk |
| R allocation per insert | None | None (except growth) | One CONSXP |
| Max objects | ~50k (ppsize) | Unlimited (grows) | Unlimited |
| Key safety | Lifetime-bound (`Root<'scope>`) | Generational (stale = no-op) | One handle per root (`Preserved`) |

Use `ProtectScope` for temporaries that live only within a single `.Call`
invocation. Use `ProtectPool` for values a struct owns, for many values, or
for values released in any order, within a call or across calls. Use the
Preserve List for one long-lived object (e.g., a cached lookup table, the data
a Rust value keeps).

## Preserve List

R's own cross-`.Call` rooting primitive: `R_PreserveObject` adds a SEXP to
R's precious list, and `R_ReleaseObject` removes it. `Preserved` is the owned
handle for one such root: it preserves on construction, preserves again on
`Clone` (each handle releases its own entry), and releases on `Drop`. It is
`!Send + !Sync`, dereferences to its `SEXP`, and returns the object to R
through `IntoR`, releasing the root as it does.

```rust
use miniextendr_api::Preserved;

struct Fit {
    data: Preserved, // the data frame the model was fitted on
}

let fit = Fit { data: Preserved::new(df) }; // a fresh, unprotected SEXP is fine
// ... any number of allocations and `.Call`s later, while `fit` lives:
let n = fit.data.len();
```

A struct holding a `Preserved` can live in an `ExternalPtr`; R's finalizer
then drops it and releases the root. `BuiltDataFrame`, `BuiltList`,
`NamedListBuilder`, `REvalError`, `LazyDots` and the `DataFrameShape`
sentinel each hold one.

Cost is the precious-list figure in the [Performance](#performance)
comparison above: each preserve allocates one CONSXP, and `R_ReleaseObject`
searches R's precious list from the head, one step per object preserved after
this one and not yet released. Releasing many objects, or releasing them out of insertion
order, degrades as the list grows. (Starting R with `R_HASH_PRECIOUS` set
splits the list into 1069 hashed buckets.) `ProtectPool` exists precisely to
avoid this: it wraps a *single* `R_PreserveObject` call around a whole VECSXP
and manages slots itself, giving O(1) any-order release regardless of object
count.

Use `Preserved` for a small number of long-lived, rarely released objects (a
cached lookup table, the data a Rust value keeps). For anything with higher
churn or object count, prefer `ProtectPool`.

## When to Use What

```text
Is the SEXP returned straight to R, with no allocation in between?
  → No protection needed.
A temporary within one call, released with the call's other temporaries?
  → ProtectScope (or OwnedProtect for one value; ReprotectSlot to replace
    one value in a loop).
Many values, values dropped in any order, or values a struct owns?
  → ProtectPool: `root(value)` keeps each until the pool drops;
    `insert` / `release` frees them one at a time, across calls too.
One long-lived object, such as a field of a value R owns?
  → Preserved.
```

| Scenario | Use |
|----------|-----|
| Multiple values, known at function start | `ProtectScope` |
| Single value, simple case | `OwnedProtect` |
| Accumulator loop, repeated replacement | `ReprotectSlot` |
| Values a struct owns, or several roots that drop in any order | `ProtectPool::root` |
| Many SEXPs that must outlive a `.Call`, any-order release | `ProtectPool::insert` / `release` |
| One SEXP a Rust value keeps beyond the `.Call` | `Preserved` |
| Building typed vectors from iterators | `scope.collect()` |
| Building lists with unknown length | `ListAccumulator` |
| Building string vectors | `StrVecBuilder` |

### The `Protector` trait

Code that allocates several intermediate SEXPs can take `&mut impl Protector`
and let its caller pick the backend:

```rust
fn ids_and_labels(roots: &mut impl Protector) -> (SEXP, SEXP) {
    let ids = roots.protect(vec![1, 2, 3].into_sexp());
    // `ids` stays rooted while the next conversion allocates.
    let labels = roots.protect(vec!["a", "b", "c"].into_sexp());
    (ids, labels) // both rooted until `roots` drops
}
```

`protect(&mut self, sexp) -> SEXP` is safe. It roots `sexp` until the
protector drops; there is no individual release through the trait. The two
implementations:

- `ProtectScope`. `ProtectScope::new` is `unsafe`: its caller promises the
  main thread and LIFO order, so protecting through a constructed scope is
  safe.
- `ProtectPool`, whose functions are safe on R's main thread.

There is no precious-list `Protector`. `Preserved` is a handle for one object;
preserving values one by one in bulk is the O(n²) release pattern the pool
exists to avoid. `OwnedProtect` is one protect-stack entry, not a precious-list
root.

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

A pushed value is rooted before the list grows, so `acc.push_sexp(fresh)` is
safe for a value nothing else roots yet (#1907).
`ListAccumulator::named(&scope, n)` always sets `names`: `""` for an element
pushed without one, and `character(0)` when nothing was pushed, so an empty
named list can still become a zero-column data frame.

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

## Refcount Arenas

Legacy: new code uses `ProtectPool`, which has the same VECSXP backing without
the counts. Refcount arenas provide O(1) protect/unprotect with reference
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

## Borrowing transient ALTREP elements

An ordinary list or character vector holds its elements, so rooting the vector
keeps every element alive. An ALTREP accessor can instead return a fresh object
that nothing else keeps, and the next accessor call can allocate and collect it.

`Root<'a>` holds a reference to its `ProtectScope`. When a rooted conversion
borrows an element of an ALTREP list or character vector, it retains that
element in the scope before reading the next one. The elements go into a pool
the scope creates on first use, so a large list does not take one PROTECT stack
entry per element. Elements of ordinary vectors skip the pool.

`Vec<&[i32]>`, `Vec<Option<&i32>>`, `Vec<&str>` and the nested `Vec` forms
convert this way with `TryFromRoot::try_from_root(scope.protect(value))`. The
references they return cannot outlive the scope. Generated main-thread wrappers
and trait shims create one scope per borrowed parameter and keep it through the
return conversion. A native list conversion reports every invalid element
together.

A scalar `&str`, `Option<&str>` or `Cow<str>` and the `StrVec` accessors still
read a CHARSXP that only the ALTREP class keeps (#2086). R's deferred strings
and miniextendr's string ALTREP classes cache their elements, so they are safe.

A root keeps values alive; it does not make access exclusive. Mutable
references still need their own borrowing contract, so don't derive them from a
copyable `Root`.
