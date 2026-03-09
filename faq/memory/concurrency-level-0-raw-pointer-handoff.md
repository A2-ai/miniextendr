# Level 0: Raw Pointer Handoff (Status Quo)

Extract a `double*` / `int*` on the main thread, use it from worker threads. No R API calls
from threads. This is what data.table, RcppParallel, R's own OpenMP code, and miniextendr
do today.

Source: R source at `background/r-svn/`.

---

## How It Works

1. On the R main thread, protect the SEXP and extract a raw C pointer
2. Send the pointer (and length) to worker threads
3. Workers read/write the raw data array — pure C memory, no R API
4. Workers join, main thread continues with the SEXP

```c
// Main thread:
SEXP x = PROTECT(allocVector(REALSXP, n));
double *px = REAL(x);   // raw pointer — just a double[]
int len = LENGTH(x);

// Spawn threads, each works on px[start..end]
// No R calls in threads — just arithmetic on px[]

// Threads join
UNPROTECT(1);
```

---

## What's Safe

**Numeric data regions** for non-ALTREP vectors:

| Accessor | C type | Safe? | Why |
|---|---|---|---|
| `INTEGER(x)` | `int*` | Yes | Contiguous C array, no R metadata |
| `REAL(x)` | `double*` | Yes | Same |
| `LOGICAL(x)` | `int*` | Yes | Same (R logicals are `int`) |
| `RAW(x)` | `Rbyte*` | Yes | Same |
| `COMPLEX(x)` | `Rcomplex*` | Yes | Same (pair of doubles) |

These return pointers into the VECSXP's data region — a plain C array that R allocated
but doesn't touch between API calls. As long as the SEXP is protected (won't be GC'd),
the data pointer is stable.

---

## What's NOT Safe (or: It Depends)

| Operation | Non-ALTREP | ALTREP | Why |
|---|---|---|---|
| `STRING_ELT(x, i)` | Gray area* | **Unsafe** | Non-ALTREP: two pointer reads + sxpinfo `alt` bit check. ALTREP: dispatches through `ALTSTRING_ELT` → `mkChar` → `R_StringHash`. See [ALTREP string race demo](../altrep/altrep-string-race-demo.md). |
| `VECTOR_ELT(x, i)` | Gray area* | **Unsafe** | Same pattern as STRING_ELT |
| `CHAR(charsxp)` | **Safe** | N/A | Pure pointer arithmetic: `(const char*)((SEXPREC_ALIGN*)(x) + 1)`. No sxpinfo read, no globals. |
| `INTEGER_ELT(x, i)` | **Safe** | **Unsafe** | Non-ALTREP: `INTEGER(x)[i]`. ALTREP: dispatch toggles `R_GCEnabled` |
| `TYPEOF(x)` | Gray area* | Gray area* | Reads sxpinfo bitfield — shares 64-bit word with GC `mark` bit |
| `LENGTH(x)` | Gray area* | Gray area* | Same sxpinfo bitfield |
| `DATAPTR(x)` | **Safe** | **Unsafe** | Non-ALTREP: pointer arithmetic. ALTREP: `ALTVEC_DATAPTR_EX` toggles `R_GCEnabled` (`src/main/altrep.c:365-370`) |

*\*Gray area = C11 undefined behavior (sxpinfo bitfield race with GC mark bit), but
harmless on x86-64 where word loads are atomic and the bits don't interfere.*

The key distinction: **raw data pointers** and **pointer arithmetic** (like `CHAR`) are
safe. **ALTREP dispatch** is genuinely unsafe (writes globals, calls `mkChar`, may allocate).
**sxpinfo reads** (like `TYPEOF`, `ALTREP(x)` check) are technically C11 UB but practically
safe on real hardware.

---

## R's Own Use: OpenMP in array.c

R itself uses this pattern for `colSums`/`colMeans` with OpenMP:

```c
// src/main/array.c:1927-1932
if (R_num_math_threads > 0)
    nthreads = R_num_math_threads;
else
    nthreads = 1; /* for now */
#pragma omp parallel for num_threads(nthreads) default(none) \
    firstprivate(x, ans, n, p, type, NaRm, keepNA, R_NaReal, R_NaInt, OP)
```

This works because `colSums` operates on `REAL(x)` — a `double*` extracted before the
parallel region. The OpenMP threads never call any R API function. They just do arithmetic
on the raw array.

The `R_num_math_threads` variable controls thread count. R's `array.c` is one of the very
few places in base R that uses threading at all.

---

## data.table's Approach

data.table's `fread`, `forder`, and grouped operations use OpenMP the same way:

1. Extract `REAL(col)` / `INTEGER(col)` pointers for all relevant columns
2. Parallel-for over rows using raw pointers
3. Write results into pre-allocated output vectors (also raw pointers)
4. Never touch R API from threads

This is documented in data.table's `openmp-utils.c` and is the standard pattern for
high-performance R packages.

---

## RcppParallel's Approach

RcppParallel provides `RVector` and `RMatrix` wrappers that extract `REAL(x)` once and
expose `begin()`/`end()` iterators. The `parallelFor` function distributes index ranges
to TBB workers that only access the raw data region.

---

## miniextendr's Approach

miniextendr uses `r_slice()` / `r_slice_mut()` (in `miniextendr-api/src/from_r.rs`):

```rust
// On the R main thread (inside with_r_thread):
let slice: &[f64] = r_slice(sexp);  // safe wrapper around REAL()

// Send to rayon workers:
slice.par_iter().map(|x| x * 2.0).collect()
```

The `r_slice` helpers handle the edge case where R returns `0x1` (not null) for empty
vectors — Rust 1.93+ validates pointer alignment even for zero-length slices, so raw
`std::slice::from_raw_parts` would SIGABRT on R's misaligned sentinel.

---

## The ALTREP Complication

ALTREP (Alternative Representations, R 3.5+) breaks the "extract pointer, use from threads"
pattern. An ALTREP vector might not have a contiguous data region at all — `INTEGER(x)`
goes through `ALTVEC_DATAPTR_EX`:

```c
// src/main/altrep.c:352-372
static R_INLINE void *ALTVEC_DATAPTR_EX(SEXP x, Rboolean writable)
{
    if (R_in_gc)
        error("cannot get ALTVEC DATAPTR during GC");
    R_CHECK_THREAD;
    int enabled = R_GCEnabled;
    R_GCEnabled = FALSE;          // global flag toggle!
    void *val = ALTVEC_DISPATCH(Dataptr, x, writable);
    R_GCEnabled = enabled;
    return val;
}
```

This dispatches to an arbitrary C function that might allocate, trigger GC, or do anything.
The `R_GCEnabled` toggle is a single global variable with no synchronization.

**Workaround**: Call `DATAPTR(x)` on the main thread to force materialization. After that,
the data pointer is cached and subsequent `INTEGER(x)` returns it directly without dispatch.
miniextendr does this in `r_slice()` — the SEXP access happens on the R main thread via
`with_r_thread`, which materializes ALTREP before returning the raw pointer.

---

## Limitations

This level of concurrency fundamentally cannot:

- **Read ALTREP string vectors lazily** — deferred `STRING_ELT` calls `mkChar` (allocates, touches `R_StringHash`)
- **Allocate R objects** — any allocation can trigger full GC
- **Handle ALTREP lazily** — must materialize on main thread first
- **Read attributes** — `getAttrib` touches R object graph

**Gray area** (technically C11 UB, practically safe on x86-64):

- **Read non-ALTREP string vectors** — `STRING_ELT` on a regular STRSXP is two pointer
  reads, but the `ALTREP(x)` check reads sxpinfo which shares a word with GC's mark bit.
  `CHAR(charsxp)` itself is pure pointer arithmetic (no sxpinfo read, fully safe).
- **Check types** — `TYPEOF` reads sxpinfo, same bitfield race
- **Read list elements** — `VECTOR_ELT` on a non-ALTREP VECSXP is a pointer read + sxpinfo check

**The practical approach**: extract `CHAR` pointers (for strings) or `SEXP` pointers
(for list elements) on the main thread, then send the raw pointers to worker threads.
See [ALTREP string race demo](../altrep/altrep-string-race-demo.md) for worked examples.

For formally correct concurrent SEXP reads, see
[Level 1: GC-frozen read windows](concurrency-level-1-gc-frozen-read-windows.md).

---

## Who Uses This

| Package/System | How |
|---|---|
| R base (`array.c`) | OpenMP `colSums`/`colMeans` with `R_num_math_threads` |
| data.table | OpenMP on `REAL()`/`INTEGER()` pointers for fread, forder, grouping |
| RcppParallel | TBB workers on `RVector`/`RMatrix` wrappers |
| miniextendr | rayon on `r_slice()`/`r_slice_mut()` pointers |
| Stan (rstan) | OpenMP for gradient computation on extracted numeric data |

This pattern is battle-tested and the only form of parallelism that's safe in R today.
