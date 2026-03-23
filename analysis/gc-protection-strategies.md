# R GC Protection Strategies: Costs, Limits, and Trade-offs

Source: R 4.5.2 `src/main/memory.c`, `src/include/Defn.h`, `src/include/Rinlinedfuns.h`

## The Five Primitives

R provides five GC protection operations, built on two separate mechanisms.

### Mechanism 1: Protect Stack

A **pre-allocated fixed-size array** (`R_PPStack`) with an integer top pointer (`R_PPStackTop`).

#### `Rf_protect(s)`

Push `s` onto the protect stack.

```c
R_PPStack[R_PPStackTop++] = s;
```

- **Cost**: O(1). Single array write + integer increment.
- **Allocates**: No.
- **Limit**: Stack has a fixed capacity (default 50,000; max 500,000 via `--max-ppsize`).
  Overflow is a fatal error that R itself struggles to recover from (the error handler
  needs PROTECT, so R pre-allocates a red zone of 1,000 extra slots just to report the error).

#### `Rf_unprotect(n)`

Pop the top `n` entries.

```c
R_PPStackTop -= n;
```

- **Cost**: O(1). Single integer subtract.
- **Allocates**: No.
- **Constraint**: Strictly LIFO. Pops whatever is on top — if scopes are interleaved,
  this pops the wrong entries. Caller must track the count correctly.

#### `Rf_unprotect_ptr(s)`

Find and remove a specific SEXP from anywhere in the stack.

```c
// scan backwards from top to find s
do {
    if (i == 0) error("pointer not found");
} while (R_PPStack[--i] != s);
// shift everything above down
while (++i < R_PPStackTop) R_PPStack[i - 1] = R_PPStack[i];
R_PPStackTop--;
```

- **Cost**: O(k) where k = distance from top. Scan + array shift.
- **Allocates**: No.
- **R source comment**: *"should be among the top few items"*. Designed for cases where
  the item is near the top but LIFO order was slightly disrupted. NOT designed for
  removing items deep in the stack.
- **Constraint**: Still O(stack_depth) worst case. Still shares the stack size limit.

#### `R_ProtectWithIndex(s, &index)` + `R_Reprotect(s, index)`

Protect and record the stack slot index. Later, overwrite that slot in-place.

```c
// ProtectWithIndex:
R_PPStack[R_PPStackTop++] = s;
*index = R_PPStackTop - 1;

// Reprotect:
R_PPStack[index] = s;
```

- **Cost**: O(1) for both. Array write.
- **Allocates**: No.
- **Use case**: Replacing a protected SEXP without unprotect/re-protect churn. Common in
  loops where each iteration allocates a new SEXP that replaces the previous one.
- **Constraint**: Same stack size limit. Index becomes invalid if stack is unprotected past it.

### Mechanism 2: Precious List

A **global linked list** of cons cells, or optionally a hash table.

#### `R_PreserveObject(s)`

Prepend a cons cell to the global precious list.

```c
// Default: linked list
R_PreciousList = CONS(object, R_PreciousList);

// With R_HASH_PRECIOUS env var: hash table (1069 buckets)
int bin = ((size_t)object >> 3) % 1069;
SET_VECTOR_ELT(R_PreciousList, bin, CONS(object, VECTOR_ELT(R_PreciousList, bin)));
```

- **Cost**: O(1) time, but **allocates a CONSXP cell** (56 bytes on 64-bit).
- **Allocates**: **Yes.** Every call creates GC pressure. In a tight loop protecting
  thousands of objects, this adds thousands of cons cells to the heap.
- **Limit**: None (bounded only by available memory).

#### `R_ReleaseObject(s)`

Scan the list to find and unlink the cons cell.

```c
// Default: linear scan of entire list
for (head = CDR(list); head != R_NilValue; head = CDR(head)) {
    if (CAR(head) == object) { SETCDR(last, CDR(head)); return list; }
}

// With R_HASH_PRECIOUS: scan within bucket
int bin = ((size_t)object >> 3) % 1069;
DeleteFromList(object, VECTOR_ELT(R_PreciousList, bin));
```

- **Cost**: O(n) where n = total preserved objects (default). O(bucket_size) with hash.
- **Allocates**: No.
- **Limit**: None, but cost grows with the number of preserved objects globally.
  If an R session has 10,000 preserved objects, every Release scans up to 10,000 cons cells.

### Mechanism 3: Circular Doubly-Linked Preserve List (cpp11 pattern)

miniextendr's `preserve.rs` implements a third mechanism that sits between the protect
stack and R's precious list. Inspired by cpp11's approach.

**Architecture**: A circular doubly-linked list of R cons cells. The list head is
anchored with a single `R_PreserveObject` call (so it's never GC'd). Individual SEXPs
are stored as TAG of each cell. CAR/CDR form the doubly-linked prev/next pointers.

#### `preserve::insert(x)`

Splice a new cons cell between head and head.next.

```rust
let cell = Rf_cons(head, next);  // allocates CONSXP
SET_TAG(cell, x);                // store the protected SEXP
SETCDR(head, cell);              // link into list
SETCAR(next, cell);
```

- **Cost**: O(1). Allocates one CONSXP (like `R_PreserveObject`).
- **Allocates**: Yes — one cons cell.
- **Limit**: Unlimited (heap-bounded).
- **Uses 2 protect stack slots** temporarily during insertion (`Rf_protect`/`Rf_unprotect(2)`).

#### `preserve::release(cell)`

Splice the cell out of the doubly-linked list.

```rust
let lhs = CAR(cell);   // prev
let rhs = CDR(cell);   // next
SETCDR(lhs, rhs);      // bypass cell
SETCAR(rhs, lhs);
```

- **Cost**: O(1). No scan needed — the caller holds a direct reference to the cell.
- **Allocates**: No.
- **No search**: Unlike `R_ReleaseObject` which must scan a singly-linked list to find the
  cell, the doubly-linked structure allows O(1) removal because the cell knows its neighbors.

#### Why this exists

It combines advantages from both R mechanisms:

- **O(1) release** (like the protect stack, unlike `R_ReleaseObject`'s O(n) scan)
- **Unlimited capacity** (like the precious list, unlike the protect stack's 50k limit)
- **Any-order release** (like the precious list, unlike the protect stack's LIFO requirement)

The cost is one CONSXP allocation per insert (same as `R_PreserveObject`), plus a single
`R_PreserveObject` for the list head (amortized over all insertions).

## Comparison

### R-side costs (what happens inside R's C code)

| Property | Protect Stack | R Precious List | Preserve (DLL) |
|----------|--------------|-----------------|----------------|
| **Protect cost** | O(1), no alloc | O(1), allocs CONSXP | O(1), allocs CONSXP |
| **Release cost** | O(1) or O(k) | O(n) scan | **O(1)** |
| **Capacity** | 50k default, 500k max | Unlimited | Unlimited |
| **Ordering** | LIFO required | Any order | Any order |
| **Scope** | Function-local | Global, indefinite | Global, indefinite |
| **GC pressure** | Zero | 1 CONSXP/protect | 1 CONSXP/protect |
| **Overflow** | Fatal error | Graceful | Graceful |

### Rust-side costs (what each strategy demands of wrapper types)

| Property | Protect Stack | DLL Preserve |
|----------|--------------|--------------|
| **Per-value storage** | 0 bytes | 8 bytes (cell SEXP) |
| **Per-value Drop** | None | `preserve::release(cell)` |
| **Bulk cleanup** | `Rf_unprotect(n)` — one call | n individual `release` calls |
| **Wrapper type** | `Root<'scope>` — just SEXP + PhantomData | must store cell tag |
| **Type complexity** | Zero-cost; no Drop needed | Every type needs Drop impl |

This is the real tradeoff from Rust's perspective. The protect stack costs nothing
per value — `Root<'scope>` is just a SEXP with a zero-sized lifetime marker, and the
scope handles bulk unprotection in one call. The DLL preserve list requires every
Rust wrapper to carry an extra 8-byte cell tag and implement `Drop` to release it.

For types that already have a Drop impl (like `ExternalPtr` which drops the Rust value),
the cell tag is just one more field. For types that wouldn't otherwise need Drop (like a
temporary protected SEXP during data.frame construction), the DLL forces unnecessary
per-value bookkeeping.

The DLL preserve list strictly dominates `R_PreserveObject`/`R_ReleaseObject` — same
insert cost, but O(1) release instead of O(n). The only reason to use R's precious list
directly is for the single anchor of the DLL itself.

## When Each Strategy Wins

### Protect stack: temporary, bounded, LIFO-compatible, zero allocation

Best for:
- Allocating a few R objects within a function, using them, then returning
- Loops with a fixed number of live SEXPs per iteration (use `R_ProtectWithIndex` + `R_Reprotect`)
- Any situation where the protect count is statically known
- **Hot paths where GC pressure matters** — no allocation at all

Danger:
- **Stack overflow at 50,000** (or user-configured max, capped at 500,000). A function that
  protects one SEXP per iteration in a 100k-element loop will crash R.
- LIFO discipline is error-prone in complex control flow (early returns, error paths, multiple scopes).
- Shared with R's own call chain — deep R→R→R→.Call nesting consumes stack from the outside.

### DLL preserve list: cross-call, unbounded, any-order, O(1) release

Best for:
- ExternalPtr stored across R function calls (the object outlives any single `.Call` frame)
- Objects with unpredictable lifetimes (event handlers, caches, global state)
- Cases where LIFO ordering is impossible (async callbacks, shared ownership)
- **The allocator** (`RAllocator`) — backing memory that must survive arbitrary dealloc order

Danger:
- **Allocates a CONSXP on every insert** — not suitable for protecting thousands of
  short-lived temporaries in a tight loop (use the protect stack for that)
- Thread-local — each thread has its own list, which is correct for R's single-threaded
  GC, but means protections can't be shared across threads

### R's precious list: avoid (DLL is strictly better)

`R_PreserveObject` has the same allocation cost as the DLL but O(n) release.
The only use is anchoring the DLL list head (one call per thread, ever).
All other cross-call protection should go through the DLL preserve list.

### `Rf_unprotect_ptr`: avoid

It exists for edge cases where LIFO was almost-but-not-quite maintained. The scan + shift
cost makes it unsuitable as a general-purpose protection mechanism.

## `.Call` Arguments and Return Values Are Protected by R

R protects everything it passes to `.Call` — the arguments are bound in the calling
R frame, which is a GC root. And when `.Call` returns a SEXP, R's evaluator immediately
binds it (to a variable, list slot, function argument, promise, etc.), which is also
a GC root.

This means: **within a `#[miniextendr]` function, inputs and the output are both
protected by R.** Only *intermediate* SEXPs need protection — ones that are live
simultaneously and could be collected when a subsequent allocation triggers GC.

### When protection is actually needed

Protection is only needed when **multiple R allocations coexist** within a single `.Call`
frame. Each R allocation (`Rf_allocVector`, `Rf_mkChar`, `Rf_ScalarReal`, etc.) can
trigger GC, which can collect any unprotected SEXP.

```
Scenario 1: Single allocation — NO protection needed
    let sexp = Rf_allocVector(REALSXP, n);  // allocate
    fill(sexp);                              // fill (no R allocation)
    return sexp;                             // R protects on receipt

Scenario 2: Multiple allocations — protection needed
    let vec = Rf_allocVector(REALSXP, n);    // allocate
    // !! GC can run here, collecting vec !!
    let names = Rf_allocVector(STRSXP, n);   // this allocation may trigger GC
    Rf_setAttrib(vec, R_NamesSymbol, names); // vec may be dangling
```

In Scenario 2, `vec` must be protected before allocating `names`. But `names` itself
does not need protection if nothing allocates after it (the return to R protects it).

### How much protection do typical functions need?

Most `#[miniextendr]` functions follow the pattern: convert inputs (Rust-side, no R alloc),
compute (pure Rust), convert output (one R allocation). This needs **zero protections**.

Functions that need protection:
- **Returning a data.frame**: allocate the list, then allocate each column, then set names,
  then set class. The list must be protected across column allocations. Columns must be
  protected across subsequent column allocations. ~N+2 protections for N columns.
- **Returning a named vector**: allocate the vector, then allocate the names character vector,
  then allocate each CHARSXP. Vector and names need protection. ~2 protections.
- **Building a list of strings**: allocate the STRSXP, then loop calling `Rf_mkChar` for
  each element. The STRSXP needs protection. ~1 protection. (Each `SET_STRING_ELT` makes
  the CHARSXP reachable from the protected STRSXP, so the individual CHARSXPs don't need
  separate protection.)
- **ALTREP materialization**: allocate the target vector, fill from ALTREP source. ~1 protection.

In practice, a `.Call` function rarely needs more than ~5 simultaneous protections.
The 50,000-slot stack is vastly oversized for this use case.

### The real protect stack pressure comes from R, not Rust

The protect stack is shared across the entire call chain. A deeply nested R computation
(R calling R calling R calling `.Call`) accumulates protections at every R-level frame.
The 50k limit exists for this — not for individual `.Call` functions. A single `.Call`
contributing 2-10 protections is negligible.

## What This Means for miniextendr

### Current state

miniextendr already has all three mechanisms implemented:

- **`gc_protect.rs`** — `ProtectScope` + `Root<'scope>` + `ReprotectSlot<'scope>`:
  RAII wrappers around R's protect stack. `ProtectScope` tracks the count and calls
  `Rf_unprotect(n)` on drop. `Root<'scope>` is a lifetime-scoped handle that can't
  outlive the scope. `ReprotectSlot` wraps `R_ProtectWithIndex`/`R_Reprotect` for
  replace-under-protection without growing the stack.

- **`preserve.rs`** — Circular doubly-linked list (cpp11 pattern):
  O(1) insert and O(1) release, unlimited capacity, any-order release. Used by
  `RAllocator` and `ExternalPtr` for cross-call protection. Anchored by a single
  `R_PreserveObject` per thread.

- **`alloc_r_vector`** — Returns unprotected SEXP. Correct when it's the only/last
  allocation before return (R protects on receipt).

### How they compose

```
.Call("my_func", x, y)
│
│  x, y are protected by R (bound in calling frame)
│
├── Pure Rust computation: no protection needed
│
├── Single R allocation → return: no protection needed
│   let sexp = alloc_r_vector(n);
│   fill(sexp);
│   return sexp;  // R protects on receipt
│
├── Multiple R allocations → ProtectScope (protect stack):
│   let scope = ProtectScope::new();
│   let list = scope.protect(Rf_allocVector(VECSXP, n));
│   let names = scope.protect(Rf_allocVector(STRSXP, n));
│   ...
│   return list;  // scope drops → Rf_unprotect(2)
│
├── Loop with reprotection → ReprotectSlot (protect stack):
│   let slot = scope.protect_with_index(initial);
│   for item in items {
│       slot.set(process(slot.get(), item));  // R_Reprotect, no stack growth
│   }
│
└── Cross-call persistence → preserve (DLL):
    let cell = preserve::insert(sexp);   // O(1), allocs CONSXP
    // ... survives across .Call boundaries ...
    preserve::release(cell);              // O(1), no scan
```

### Design principles

1. **Don't protect what doesn't need it.** Single-allocation returns (the common case)
   have zero protection overhead. R protects both inputs and outputs.
2. **Protect stack for temporaries.** Multi-allocation functions use `ProtectScope`.
   Zero R-side cost (no allocation, no GC pressure) AND zero Rust-side cost
   (`Root<'scope>` is just SEXP + PhantomData, no Drop, bulk cleanup in one call).
   Use `ReprotectSlot` for loops that replace a protected value repeatedly.
3. **DLL preserve only when you must cross `.Call` boundaries.** The DLL has real
   Rust-side costs: every wrapper type must carry an 8-byte cell tag and implement
   Drop to release it. This overhead is acceptable for types that already have Drop
   (ExternalPtr drops its Rust value, RAllocator frees its backing memory), but
   should not be imposed on types that don't need cross-call survival.
4. **Never protect per-element.** Building a 1M-element vector = 1 protection for
   the container, then fill. Not 1M protections for the elements.
5. **R's precious list is an implementation detail.** Used once to anchor the DLL
   head. Never used directly for protecting individual objects.
