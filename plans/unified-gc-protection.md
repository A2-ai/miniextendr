# Plan: Unified GC Protection API

Background: `analysis/gc-protection-strategies.md`

## Goal

Replace the fragmented protection landscape (manual `Rf_protect`/`Rf_unprotect` calls,
`preserve.rs` DLL, `gc_protect.rs` scope) with a unified `Protector` trait. Every FFI
function that returns an unprotected SEXP gets a safe wrapper that returns a protected
handle. Users never call `Rf_protect` directly.

## Which R primitives are used, and when

Two R mechanisms, plus one Rust-side mechanism:

1. **Protect stack** (`Rf_protect` / `Rf_unprotect`) — for temporaries within a `.Call`.
   7.4 ns/op, LIFO, 50k limit. Exposed via `ProtectScope`. Also provides
   `R_ProtectWithIndex`/`R_Reprotect` for replace-in-loop (3.8 ns/op, no stack growth).

2. **R's precious list** (`R_PreserveObject` / `R_ReleaseObject`) — for a small number
   of long-lived cross-call objects that are NEVER released in a loop. 13 ns/op for
   single protect+release. SEXP is its own handle — zero Rust-side bookkeeping.
   O(n) release but LIFO-fast (recently preserved objects found first). Ideal for
   ExternalPtr (you already hold the SEXP, there are few of them, they live a long time).
   **NEVER use in a loop** — 15 seconds at 10k iterations due to O(n²) list growth.

3. **VECSXP pool** (new, replaces DLL preserve list) — for all other cross-call usage.
   9.6 ns/op single, 2.5x faster than DLL at batch, 7x faster at replace-in-loop.
   One GC-traced VECSXP with slot management via `slotmap` (11.4 ns/op with safety)
   or `Vec` (9.6 ns/op raw). Zero R allocation per insert/release. Generational keys
   prevent stale-handle bugs.

The **DLL preserve list** (`preserve.rs`) is retired. Benchmarks show it's 3-7x slower
than the VECSXP pool across all workloads (28.9 ns vs 9.6 ns single-op, 271 µs vs 45 µs
replace-in-loop at 10k). Its only theoretical advantage (R-object handle, memory shrinkage)
didn't produce measurable benefits.

### Decision tree

```
Is the SEXP returned directly to R (single allocation, no intermediate)?
  → No protection needed. R protects on receipt.

Is the SEXP temporary within a .Call (used, then discarded before return)?
  → ProtectScope (protect stack). 7.4 ns/op, bulk Rf_unprotect(n) on drop.

Does the SEXP replace another in a loop (same slot, many iterations)?
  → ReprotectSlot (R_ProtectWithIndex + R_Reprotect). 3.8 ns/op, no stack growth.
  → Or pool overwrite (SET_VECTOR_ELT in place). 4.5 ns/op.

Must the SEXP survive across .Call boundaries?
  ├── Few long-lived objects, never released in a loop?
  │   → R_PreserveObject / R_ReleaseObject (precious list). 13 ns/op.
  │     SEXP is its own handle. Zero Rust bookkeeping.
  │
  └── Anything else (many objects, churn, loops, allocator backing)?
      → ProtectPool (VECSXP + slotmap). 11.4 ns/op with safety,
        9.6 ns/op raw Vec. Zero R alloc per insert.
```

## The `Protector` trait

```rust
trait Protector {
    type Handle: Deref<Target = SEXP>;
    fn protect(&self, sexp: SEXP) -> Self::Handle;
}
```

Three implementations:

- `ProtectScope` (existing) — stack-backed, `Handle = Root<'scope>`, no Drop, bulk cleanup
- `OwnedProtect` (existing) — precious-list-backed, `Handle = OwnedProtect`,
  has Drop (`R_ReleaseObject`), SEXP is the handle (zero extra state)
- `ProtectPool` (new) — VECSXP + slotmap-backed, `Handle = PoolHandle`, has Drop, any-order

Functions that allocate intermediate SEXPs take `&impl Protector` and are generic over
the backend. The caller decides which backend by passing a scope, an owned guard, or a pool.

`ReprotectSlot` is not part of the `Protector` trait — it's a special-purpose tool on
`ProtectScope` for the replace-in-loop pattern. Pool slots are naturally replaceable
(just `SET_VECTOR_ELT` to overwrite), so the pool doesn't need a separate reprotect API.

## `ProtectPool`: VECSXP + hand-rolled generational keys

**IMPLEMENTED** in `miniextendr-api/src/protect_pool.rs`. No external dependencies
(slotmap was evaluated but hand-rolled keys matched VecPool speed at 10.1 ns/op
while slotmap was 11.4 ns/op due to redundant second free list).

```rust
pub struct ProtectKey {
    slot: u32,
    generation: u32,
}

pub struct ProtectPool {
    backing: SEXP,           // VECSXP, anchored by one R_PreserveObject
    generations: Vec<u32>,   // generation counter per slot
    free_slots: Vec<usize>,  // recycled slot indices
    next_slot: usize,
    len: usize,
}
```

- Insert: O(1), zero R allocation (just SET_VECTOR_ELT)
- Release: O(1), generational-key safe (stale keys are no-ops)
- Replace: O(1), in-place overwrite (pool equivalent of R_Reprotect)
- Growth: allocate larger VECSXP, copy, swap — amortized O(1)

## FFI wrappers that need protected variants

Every R FFI function that allocates a new SEXP must have a variant that returns a
protected handle instead of a raw SEXP. These are the functions where the returned
SEXP is vulnerable to GC if another R allocation follows.

### Vector/matrix allocation

These are the core allocation functions. Every call site currently either manually
protects or hopes nothing allocates before the SEXP is used.

- `Rf_allocVector(type, n)` → `scope.alloc_vector(type, n)` (already in gc_protect.rs)
- `Rf_allocMatrix(type, nrow, ncol)` → `scope.alloc_matrix(type, nrow, ncol)` (already exists)
- `Rf_allocArray(type, dims)` → needs wrapper
- `Rf_alloc3DArray(type, x, y, z)` → needs wrapper
- `Rf_allocList(n)` → `scope.alloc_list(n)` (already exists)
- `Rf_allocLang(n)` → needs wrapper
- `Rf_allocS4Object()` → needs wrapper
- `Rf_allocSExp(type)` → needs wrapper

### Scalar constructors

Return a single-element R vector. Vulnerable if you construct multiple scalars.

- `Rf_ScalarInteger(x)` → needs wrapper
- `Rf_ScalarReal(x)` → needs wrapper
- `Rf_ScalarLogical(x)` → needs wrapper
- `Rf_ScalarString(x)` → needs wrapper
- `Rf_ScalarComplex(x)` → needs wrapper
- `Rf_ScalarRaw(x)` → needs wrapper

### String/character creation

`Rf_mkChar`/`Rf_mkCharLenCE` return CHARSXP. Usually immediately passed to
`SET_STRING_ELT` (which makes it reachable from the parent STRSXP), but vulnerable
in the gap between creation and insertion.

- `Rf_mkChar(s)` → needs wrapper
- `Rf_mkCharLen(s, n)` → needs wrapper
- `Rf_mkCharLenCE(s, n, enc)` → needs wrapper
- `Rf_mkCharCE(s, enc)` → needs wrapper

### External pointers

`R_MakeExternalPtr` allocates an EXTPTRSXP. Currently `externalptr.rs` manually
protects with `preserve::insert`. Should use the protector trait.

- `R_MakeExternalPtr(p, tag, prot)` → needs wrapper
- `R_MakeExternalPtrFn(p, tag, prot)` → needs wrapper

### Cons cell / pairlist construction

`Rf_cons` and `Rf_lcons` allocate cons cells. Used in `preserve.rs` itself and
in pairlist construction.

- `Rf_cons(car, cdr)` → needs wrapper
- `Rf_lcons(car, cdr)` → needs wrapper

### Duplication

`Rf_duplicate` / `Rf_shallow_duplicate` allocate new copies.

- `Rf_duplicate(x)` → needs wrapper
- `Rf_shallow_duplicate(x)` → needs wrapper

### Coercion

`Rf_coerceVector` may allocate a new vector.

- `Rf_coerceVector(x, type)` → needs wrapper

### Length change

`Rf_lengthgets` / `Rf_xlengthgets` allocate a resized copy.

- `Rf_lengthgets(x, n)` → needs wrapper
- `Rf_xlengthgets(x, n)` → needs wrapper

### Environment creation

`R_NewEnv` allocates a new environment.

- `R_NewEnv(parent, hash, size)` → needs wrapper

## What does NOT need wrapping

Functions that return already-protected or non-SEXP values:

- `Rf_protect` / `Rf_unprotect` — these ARE the protection mechanism
- `Rf_install` / `Rf_installChar` — symbols are never GC'd once created
- Data pointer accessors (`INTEGER`, `REAL`, etc.) — return raw C pointers
- Predicates (`Rf_isNull`, `Rf_isInteger`, etc.) — return int
- Getters (`CAR`, `CDR`, `TAG`, `ATTRIB`, `STRING_ELT`, `VECTOR_ELT`) — return
  SEXPs that are already protected by their parent
- `R_NilValue`, `R_NaString`, etc. — global constants, always protected

## Implementation approach

1. Add `slotmap` dependency to miniextendr-api (always-on, it's 0 deps and no-std)
2. Implement `ProtectPool` in a new `protect_pool.rs`
   - VECSXP anchored by one `R_PreserveObject`
   - `SlotMap<ProtectKey, ()>` for generational slot management (11.4 ns/op)
   - `PoolHandle` with Drop that calls release
   - Growth: allocate larger VECSXP, copy elements, reprotect, release old
   - Reuse infrastructure from `refcount_protect.rs` (ArenaState, growth logic)
3. Add `Protector` trait to `gc_protect.rs`, implement for all three:
   - `ProtectScope` (stack) — existing, just add trait impl
   - `OwnedProtect` (precious list) — existing, add trait impl
   - `ProtectPool` (VECSXP + slotmap) — new
4. Add protected wrappers for all FFI functions listed above
   (many already exist on `ProtectScope` — fill in the gaps)
5. Migrate `externalptr.rs` from `preserve::insert`/`release` to `R_PreserveObject`
   directly (few long-lived objects, SEXP is its own handle, zero bookkeeping)
6. Migrate `allocator.rs` from `preserve::insert`/`release` to `ProtectPool`
   (DLL is 3x slower than pool; allocator needs O(1) release for arbitrary dealloc)
7. Migrate manual `Rf_protect`/`Rf_unprotect` call sites to `ProtectScope`
8. Delete `preserve.rs` DLL (replaced by pool)
9. Strip refcounting from `refcount_protect.rs`, keep VECSXP pool infrastructure
10. Lint rule: warn on direct `Rf_protect`/`Rf_unprotect` outside gc_protect.rs

## Relationship to `refcount_protect.rs`

`refcount_protect.rs` implements a VECSXP-backed pool with HashMap/BTreeMap and
reference counting. However, the refcounting is unnecessary: R's API is single-threaded,
so all protection calls must happen on the main thread. Multiple threads can't
independently protect/release the same SEXP — they must route through the main thread,
which serializes access. Shared ownership of protection doesn't exist in this model.

**Refactor `refcount_protect.rs`, don't delete it.** The module has good infrastructure
that `ProtectPool` should build on:

- **`MapStorage` trait** — generic over BTreeMap/HashMap/ahash. Reusable as-is.
- **`ArenaState`** — VECSXP pool with free list, growth (doubling + copy), capacity
  management. This IS the proposed `ProtectPool`, already implemented.
- **`Arena<M>`** — RefCell wrapper with `ArenaGuard` RAII (Drop-based unprotect).
- **`define_thread_local_arena!`** — zero-overhead thread-local pool via UnsafeCell.
- **Growth logic** — allocate new VECSXP, copy elements, R_PreserveObject/R_ReleaseObject swap.

**What to remove:** the refcounting layer (`Entry.count`, `decrement_and_maybe_remove`).
R's API is single-threaded — multiple threads can't independently protect/release the
same SEXP without routing through the main thread, which serializes access. Shared
ownership of protection doesn't exist in this model. Replace refcounting with either
slotmap generational keys or simple unique ownership (one protector per SEXP).

**What to add:** slotmap backend as an alternative to HashMap/BTreeMap via `MapStorage`,
or replace `MapStorage` entirely with slotmap if benchmarks show it's faster.

## Benchmark findings (resolved)

See `analysis/gc-protection-benchmarks-results.md` for full data.

| Question | Answer | Data |
|----------|--------|------|
| ExternalPtr → precious list? | **Yes** — 13 ns/op, no background degradation for recent objects | Group 13 |
| Allocator → pool? | **Yes** — pool is 3x faster than DLL (9.6 vs 28.9 ns/op) | Group 1 |
| DLL has a niche? | **No** — pool beats it 2.6-7x across all workloads | Groups 1,2,5,7 |
| slotmap overhead? | **25%** (11.4 vs 9.6 ns/op) — acceptable for safety | Group 16 |
| Vec vs VecDeque? | **Identical** (9.6 ns both) — use Vec | Group 11 |
| Pool growth? | **Not a concern** — 10% overhead from cap=16 to 100k | Group 17 |
| Precious list in loops? | **NEVER** — 15 seconds at 10k iterations (O(n²)) | Group 7 |
| Precious list background? | **No degradation** for recent objects (LIFO prepend) | Group 13 |
| Rf_unprotect_ptr? | **Fine** — 15% overhead at all depths | Group 12 |

## Migration priority

High — these have multiple unprotected allocations in sequence:
- `into_r.rs` — data.frame construction, named vector construction, STRSXP building
- `serde/columnar.rs` — deserializes multi-column data.frames
- `list.rs` — list building with names
- `externalptr.rs` — MakeExternalPtr + register finalizer
- `altrep_impl.rs` — materialization allocates target vector

Medium — occasional multi-allocation:
- `optionals/*.rs` — ndarray, nalgebra, arrow, time conversions
- `factor.rs` — levels + integer vector
- `expression.rs` — RCall/RSymbol construction
- `vctrs.rs` — class construction

Low — single allocation (already safe, R protects on receipt):
- Simple `IntoR` impls (Vec<f64> → REALSXP, etc.)
- Scalar constructors returning immediately
