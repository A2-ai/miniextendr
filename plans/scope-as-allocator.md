# Plan: Scope-as-Allocator (Protected-by-Construction)

Builds on `unified-gc-protection.md`. That plan adds `Protector` trait + pool. This plan
changes the **API surface**: instead of allocate-then-protect, the scope IS the allocator.

## Problem

Current pattern has an unprotected gap:

```rust
let sexp = Rf_allocVector(VECSXP, 10);  // unprotected SEXP exists here
let root = scope.protect(sexp);          // now safe
```

Between lines 1 and 2, any R allocation (including `Rf_mkChar` for names) could trigger
GC and collect `sexp`. In practice R doesn't GC between two C-level ops without its
own allocation, but:
- The pattern is error-prone (easy to forget `protect`)
- It's impossible to enforce at compile time
- Every new contributor has to learn the rule

## Insight

R function arguments are auto-protected by `.Call()`. `VECTOR_ELT`, `STRING_ELT`,
`CAR`/`CDR` return references into already-protected parents. The **only** SEXPs that
need protection are ones we **allocate**.

So if the scope provides the allocation functions, `scope.protect(raw_sexp)` becomes
a rare escape hatch, not the primary API. Protection is guaranteed by construction.

## Proposed API

```rust
// Today:
unsafe {
    let scope = ProtectScope::new();
    let vec = scope.protect(Rf_allocVector(INTSXP, 10));
    let names = scope.protect(Rf_allocVector(STRSXP, 10));
    // fill vec and names...
    Rf_setAttrib(*vec, R_NamesSymbol, *names);
    vec.into_raw()
}

// Proposed:
unsafe {
    let scope = ProtectScope::new();
    let vec = scope.alloc_integer(10);
    let names = scope.alloc_character(10);
    // fill vec and names...
    Rf_setAttrib(*vec, R_NamesSymbol, *names);
    vec.into_raw()
}
```

### Core allocation methods on `ProtectScope`

```rust
impl ProtectScope {
    // Vectors
    pub unsafe fn alloc_integer(&self, n: usize) -> Root<'_>;
    pub unsafe fn alloc_real(&self, n: usize) -> Root<'_>;
    pub unsafe fn alloc_logical(&self, n: usize) -> Root<'_>;
    pub unsafe fn alloc_raw(&self, n: usize) -> Root<'_>;
    pub unsafe fn alloc_complex(&self, n: usize) -> Root<'_>;
    pub unsafe fn alloc_character(&self, n: usize) -> Root<'_>;
    pub unsafe fn alloc_list(&self, n: usize) -> Root<'_>;

    // Generic (for computed SEXPTYPE)
    pub unsafe fn alloc_vector(&self, sexptype: SEXPTYPE, n: usize) -> Root<'_>;
    pub unsafe fn alloc_matrix(&self, sexptype: SEXPTYPE, nrow: usize, ncol: usize) -> Root<'_>;

    // Scalars (allocate + set in one step)
    pub unsafe fn scalar_integer(&self, x: i32) -> Root<'_>;
    pub unsafe fn scalar_real(&self, x: f64) -> Root<'_>;
    pub unsafe fn scalar_logical(&self, x: bool) -> Root<'_>;
    pub unsafe fn scalar_string(&self, s: &str) -> Root<'_>;
    pub unsafe fn scalar_complex(&self, x: Rcomplex) -> Root<'_>;
    pub unsafe fn scalar_raw(&self, x: u8) -> Root<'_>;

    // String helpers
    pub unsafe fn mkchar(&self, s: &str) -> Root<'_>;

    // Duplication
    pub unsafe fn duplicate(&self, x: SEXP) -> Root<'_>;
    pub unsafe fn shallow_duplicate(&self, x: SEXP) -> Root<'_>;

    // Coercion
    pub unsafe fn coerce(&self, x: SEXP, target: SEXPTYPE) -> Root<'_>;

    // Environment
    pub unsafe fn new_env(&self, parent: SEXP, hash: bool, size: i32) -> Root<'_>;

    // Escape hatch (for SEXPs from other sources)
    pub unsafe fn protect(&self, x: SEXP) -> Root<'_>;          // existing
    pub unsafe fn protect_raw(&self, x: SEXP) -> SEXP;          // existing
}
```

### Why `usize` not `isize`

R's `R_xlen_t` is `isize`, but user-facing length should be `usize` (Rust convention).
The methods do `isize::try_from(n).expect("length overflow")` internally. Keeps the
API clean while catching impossible lengths at the boundary.

### Typed vector accessors

For convenience, the returned `Root<'_>` could provide typed data access:

```rust
let vec = scope.alloc_integer(10);
let slice: &mut [i32] = vec.as_mut_slice(); // calls INTEGER() + from_raw_parts_mut
slice[0] = 42;
```

This is a separate concern (typed SEXP wrappers) and can be added later. The
allocation API is the priority.

## Implementation

1. Add methods to `ProtectScope` — each is ~5 lines (allocate + protect + return Root)
2. Add same methods to TLS API (`tls::alloc_integer(10)` etc.)
3. Add same methods to `Protector` trait (so generic code works with any backend)
4. Migrate call sites — replace `scope.protect(Rf_allocVector(...))` with `scope.alloc_*()`
5. Lint rule: warn on `Rf_allocVector` / `Rf_ScalarInteger` / etc. outside of gc_protect.rs

### Migration scope

326 R allocation calls across 41 files. Most are `Rf_allocVector` (the generic case)
which maps to `scope.alloc_vector()`. The typed shortcuts (`alloc_integer`, etc.) are
sugar — `alloc_vector(INTSXP, n)` works for everything.

Priority order:
1. `ProtectScope` methods (core implementation)
2. Migrate `into_r.rs` and `list.rs` (highest allocation density)
3. Migrate `serde/`, `optionals/` (medium density)
4. Lint rule to prevent regression

### What NOT to change

- `OwnedProtect::new(Rf_allocVector(...))` — single-value guard, less common, fine as-is
- `IntoR` impls that return a single SEXP immediately — R protects on receipt
- ALTREP callbacks — these run under R's protection regime

## Relationship to `unified-gc-protection.md`

That plan defines the `Protector` trait and pool backend. This plan adds **allocation
methods** to the trait/impls. They compose naturally:

```rust
trait Protector {
    type Handle: Deref<Target = SEXP>;

    // Existing (from unified plan)
    fn protect(&self, sexp: SEXP) -> Self::Handle;

    // New (from this plan)
    unsafe fn alloc_vector(&self, sexptype: SEXPTYPE, n: usize) -> Self::Handle {
        let sexp = Rf_allocVector(sexptype, isize::try_from(n).expect("length overflow"));
        self.protect(sexp)
    }

    unsafe fn alloc_integer(&self, n: usize) -> Self::Handle {
        self.alloc_vector(SEXPTYPE::INTSXP, n)
    }

    // ... etc — default impls on the trait, zero per-backend code
}
```

The allocation methods are **default methods** on `Protector`, not per-impl. One
implementation covers all three backends (ProtectScope, OwnedProtect, ProtectPool).
