# `RDataFrameBuilder` column-protect window (gctorture segfault)

## What was attempted
Adding `RDataFrameBuilder` to `rayon_bridge.rs` — a parallel heterogeneous
column-fill into an R `data.frame`. Each column is allocated serially on the R
thread, filled in parallel via rayon, then assembled into a `VECSXP`.

## What went wrong
First `gctorture(TRUE)` pass segfaulted immediately:

```
*** caught segfault ***
address 0x68, cause 'invalid permissions'
1: ns$gc_stress_dataframe_rayon()
```

R-release passing but gctorture crashing is the classic PROTECT-window signature.

## Root cause
The first draft collected each built column into a `Vec<Sendable<SEXP>>` while
each column-builder (`with_r_vec` / `build_str_column`) **unprotected its result
on return** (their internal `WorkerUnprotectGuard` / `OwnedProtect` dropped). So
between building column *i* and column *i+1*, column *i* sat **unprotected** —
and column *i+1*'s `Rf_allocVector` can trigger GC, freeing column *i*. The
collected `Sendable<SEXP>` then held a dangling pointer; `SET_VECTOR_ELT` during
assembly dereferenced freed memory.

A second draft re-protected each column but mixed an `OwnedProtect` for the
parent `df` with a manual `Rf_unprotect(ncol)` — the LIFO stack ordering was
`[col0…col_{n-1}, df]`, so `Rf_unprotect(ncol)` popped `df` plus the top `ncol-1`
columns, leaving `df` unprotected. Wrong target.

## Fix
1. Re-PROTECT each column immediately after its builder returns (`Rf_protect` on
   the worker thread), before the next column is built. No allocation happens
   between the builder returning and this protect, so there is no GC gap.
2. In assembly, use explicit manual protect counting (no `OwnedProtect` mixed
   with manual unprotect). Stack on entry is `[col0…col_{n-1}]`; allocate+protect
   `df` → store all columns → `Rf_unprotect(ncol+1)` then `Rf_protect(df)` (no
   alloc in the gap) → allocate/protect/store/unprotect names and row.names
   individually → set cached class → `Rf_unprotect(1)` for `df` and return it
   unprotected (caller's responsibility; no alloc follows).

After the fix the gctorture sweep (25× the no-arg fixture + builder across
nrow ∈ {0,1,5,33,64,100}) passes clean, and the full testthat suite is green
(FAIL 0, PASS 6082).

## Takeaway
Helpers that allocate-and-return an R SEXP unprotected (`with_r_vec`,
`build_str_column`, anything ending in a balanced unprotect) are only safe to
hold if the caller re-protects *before the next allocation*. Collecting several
such results into a `Vec` first and protecting later is a use-after-free under
gctorture. Don't mix RAII `OwnedProtect` with manual bulk `Rf_unprotect(n)` on
the same stack region — the LIFO interleaving bites.

## Follow-up: flattened scheduler (rayon-flatten-granularity branch)
The follow-up that reworks `build()` into a single flat `(column, row-range)`
work-list keeps the same discipline but tightens it: native columns are now
allocated **and** re-protected inside each column's `alloc` step (so the protect
happens before the *next* column's allocation, with no GC gap), and character
columns hold **no** SEXP during the parallel phase (they fill an owned
`Vec<Option<String>>`), so their `STRSXP` is allocated + protected only during
serial assembly. The assembly counts protections explicitly
(`native_protected`, one per column regardless of kind) and balances them with a
single `Rf_unprotect(native_protected + 1)` + immediate `Rf_protect(df)` — no
allocation in the gap, no RAII/manual mix. Re-verified with a gctorture(TRUE)
sweep over the balanced + few-long-columns fixture and the wide/tall/skewed
builders (36/36 ok, 0 fail) and the full testthat suite (FAIL 0, PASS 6134).
