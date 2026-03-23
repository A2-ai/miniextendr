# Plan: Unified GC Protection API

Background: `analysis/gc-protection-strategies.md`

## Goal

Replace the fragmented protection landscape (manual `Rf_protect`/`Rf_unprotect` calls,
`preserve.rs` DLL, `gc_protect.rs` scope) with a unified `Protector` trait. Every FFI
function that returns an unprotected SEXP gets a safe wrapper that returns a protected
handle. Users never call `Rf_protect` directly.

## Which R primitives are used, and when

Three R mechanisms, each filling a specific role:

1. **Protect stack** (`Rf_protect` / `Rf_unprotect`) — for temporaries within a `.Call`.
   Zero cost, LIFO, 50k limit. Exposed via `ProtectScope`. Also provides
   `R_ProtectWithIndex`/`R_Reprotect` for replace-in-loop without stack growth
   (exposed via `ReprotectSlot`).

2. **R's precious list** (`R_PreserveObject` / `R_ReleaseObject`) — for a small number
   of long-lived cross-call objects. The SEXP is its own handle — zero Rust-side
   bookkeeping. O(n) release is acceptable when n is small. Ideal for ExternalPtr
   (you already hold the SEXP, there are few of them, they live a long time).

3. **DLL preserve list** (`preserve.rs`, existing) — for moderate cross-call usage
   where the protection token needs to be an R object (storable in TAG/PROT slots
   of other SEXPs). O(1) insert (allocates CONSXP), O(1) release (splice out).
   Memory naturally shrinks — released cells become GC garbage. R-side introspectable.
   Ideal for `RAllocator` (stores cell in Header for dealloc lookup) and bursty
   workloads where memory should reclaim after mass release.

4. **VECSXP pool** (new) — for high-frequency insert/release where CONSXP allocation
   per insert is measurable GC pressure. One GC-traced VECSXP with slot management
   via `slotmap`. Zero R allocation per insert/release. Generational keys prevent
   stale-handle bugs. Exposed via `ProtectPool`. Ideal for bulk caches with many
   simultaneous protections.

### When each cross-call mechanism wins

| | Precious list | DLL preserve | VECSXP pool |
|---|---|---|---|
| **Insert R cost** | CONSXP | CONSXP | zero (amortized growth) |
| **Release R cost** | O(n) scan | O(1) | O(1) |
| **Rust per-value** | 0 bytes | 8 bytes (cell SEXP) | 8 bytes (slotmap key) |
| **Handle is R object** | yes (SEXP itself) | yes (cell SEXP) | no (Rust key) |
| **Memory after release** | GC reclaims cons | GC reclaims cons | slot stays allocated |
| **Growth spikes** | none | none | copy on VECSXP resize |
| **Introspectable from R** | yes | yes | no |

- **Precious list** wins: few objects, long-lived, zero bookkeeping wanted
- **DLL** wins: handle must be an R object (storable in SEXP slots), bursty workloads
  where memory should shrink after mass release, no growth spikes wanted
- **Pool** wins: high-frequency churn where per-insert CONSXP allocation matters,
  many simultaneous protections, stale-key safety wanted

### Decision tree

```
Is the SEXP returned directly to R (single allocation, no intermediate)?
  → No protection needed. R protects on receipt.

Is the SEXP temporary within a .Call (used, then discarded before return)?
  → ProtectScope (protect stack). Zero cost, bulk Rf_unprotect(n) on drop.

Does the SEXP replace another in a loop (same slot, many iterations)?
  → ReprotectSlot (R_ProtectWithIndex + R_Reprotect). No stack growth.

Must the SEXP survive across .Call boundaries?
  ├── Few long-lived objects, zero bookkeeping wanted?
  │   → R_PreserveObject / R_ReleaseObject (precious list).
  │     SEXP is its own handle. O(n) release is fine when n is small.
  │
  ├── Handle must be an R object, or bursty alloc/release pattern?
  │   → DLL preserve list (preserve.rs). Cell is storable in TAG/PROT
  │     slots. GC reclaims released cells naturally.
  │
  └── High-frequency churn, many simultaneous protections?
      → ProtectPool (VECSXP + slotmap). Zero R alloc per insert,
        O(1) release, generational stale-key safety.
```

## The `Protector` trait

```rust
trait Protector {
    type Handle: Deref<Target = SEXP>;
    fn protect(&self, sexp: SEXP) -> Self::Handle;
}
```

Four implementations:

- `ProtectScope` (existing) — stack-backed, `Handle = Root<'scope>`, no Drop, bulk cleanup
- `OwnedProtect` (existing) — precious-list-backed, `Handle = OwnedProtect`,
  has Drop (`R_ReleaseObject`), SEXP is the handle (zero extra state)
- `DllPreserve` (existing `preserve.rs`) — DLL-backed, `Handle = PreserveCell`,
  has Drop (`preserve::release`), cell is an R object (storable in SEXP slots)
- `ProtectPool` (new) — VECSXP + slotmap-backed, `Handle = PoolHandle`, has Drop, any-order

Functions that allocate intermediate SEXPs take `&impl Protector` and are generic over
the backend. The caller decides which backend by passing a scope, an owned guard, or a pool.

`ReprotectSlot` is not part of the `Protector` trait — it's a special-purpose tool on
`ProtectScope` for the replace-in-loop pattern. Pool slots are naturally replaceable
(just `SET_VECTOR_ELT` to overwrite), so the pool doesn't need a separate reprotect API.

## `ProtectPool`: VECSXP + slotmap

Add `slotmap` as optional dependency (feature-gated? or always available — it's small,
no-std compatible, zero deps).

```rust
use slotmap::{SlotMap, new_key_type};

new_key_type! { struct ProtectKey; }

struct ProtectPool {
    pool: SEXP,                     // VECSXP, anchored by one R_PreserveObject
    slots: SlotMap<ProtectKey, ()>,
}

struct PoolHandle {
    key: ProtectKey,
    pool: *const ProtectPool,       // or Rc<RefCell<ProtectPool>> for safety
}

impl Drop for PoolHandle {
    fn drop(&mut self) { self.pool.release(self.key); }
}
```

- Insert: O(1), zero R allocation (just SET_VECTOR_ELT)
- Release: O(1), generational-key safe
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
   - `SlotMap<ProtectKey, ()>` for generational slot management
   - `PoolHandle` with Drop that calls release
   - Growth: allocate larger VECSXP, copy elements, reprotect, release old
3. Add `Protector` trait to `gc_protect.rs`, implement for all four:
   - `ProtectScope` (stack) — existing, just add trait impl
   - `OwnedProtect` (precious list) — existing, add trait impl
   - `DllPreserve` (DLL) — wrap existing `preserve.rs`, add trait impl
   - `ProtectPool` (VECSXP + slotmap) — new
4. Add protected wrappers for all FFI functions listed above
   (many already exist on `ProtectScope` — fill in the gaps)
5. Migrate `externalptr.rs` from `preserve::insert`/`release` to `R_PreserveObject`
   directly (few long-lived objects, SEXP is its own handle, zero bookkeeping)
6. Keep `allocator.rs` on `preserve.rs` DLL — the cell is stored in a C-layout Header
   and needs to be an R object. DLL is the right fit. (Evaluate pool later if GC
   pressure from CONSXP allocation is measured as a problem.)
7. Migrate manual `Rf_protect`/`Rf_unprotect` call sites to `ProtectScope`
8. Lint rule: warn on direct `Rf_protect`/`Rf_unprotect` outside gc_protect.rs

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

## If benchmarks show negligible differences

The most likely result at typical miniextendr scales (tens to low hundreds of protected
objects) is that all mechanisms perform similarly. If so:

- **Keep only two**: protect stack (`ProtectScope`) + precious list (`OwnedProtect`)
- **Don't build the pool** — the complexity isn't justified
- **Retire the DLL** — precious list is simpler for the same use case at small N
- **Keep `refcount_protect` only if refcounting is actually used** somewhere; otherwise retire it too
- The `Protector` trait still has value (abstracts over stack vs precious list) but with
  only two implementations, not four

The simplest correct codebase wins when performance is equal.

## Decisions pending benchmark results

See `plans/gc-protection-benchmarks.md` for the benchmark plan. The following decisions
in this plan are assumptions that benchmark data could overturn:

| Decision | Assumption | Benchmark that tests it | What changes if wrong |
|----------|-----------|------------------------|----------------------|
| ExternalPtr → precious list (step 5) | "few objects" means O(n) release is fine | Group 2 (batch), Group 13 (R_HASH_PRECIOUS) | Keep ExternalPtr on DLL if precious list degrades at moderate N (>50) |
| Allocator stays on DLL (step 6) | CONSXP allocation is acceptable | Group 9 (GC pressure), Group 3 (churn) | Move allocator to pool if CONSXP allocation measurably increases GC frequency |
| Pool needed for churn | DLL's CONSXP cost matters at high frequency | Group 3 (churn), Group 9 (GC pressure) | Drop the pool entirely if DLL handles 100k churn with negligible overhead |
| slotmap as pool backend | Generational check is free | Group 16 (slotmap overhead) | Offer raw Vec pool as unsafe fast path if check is measurable |
| Four Protector impls needed | Each mechanism has a distinct winning niche | Groups 2, 3, 6, 13 | Reduce to two (stack + one cross-call) if niches collapse |
| DLL memory reclamation matters | GC reclaims cons cells between bursts | Group 6 (bursty) | Remove "bursty workload" from DLL's advantages if GC doesn't reclaim |
| Pool growth spikes are a concern | Reallocation has visible latency | Group 17 (growth cost) | Remove "no growth spikes" from DLL's advantages if spikes are <100μs |
| Vec vs VecDeque for pool free list | Unknown which is better | Group 11 (free-list strategy) | Use whichever wins; Vec if negligible difference |

**Run benchmarks before implementing steps 5-6.** The trait and FFI wrappers (steps 1-4)
are independent of these decisions.

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
