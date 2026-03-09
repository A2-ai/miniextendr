# What Would It Take to Make R's C-API Read-Only Concurrent Friendly?

The realistic target isn't "reads concurrent with R evaluation" (essentially impossible without
a rewrite). It's: **R main thread reaches a sync point, background threads read SEXP data,
then R resumes.** Like a parallel `lapply` body or a Rust rayon `.par_iter()` over R vector
elements.

Even this "easy" version has five distinct obstacles.

---

## 1. The sxpinfo Bitfield Data Race

Even a pure `TYPEOF(x)` is a data race if GC can write `mark` to the same 64-bit word:

```c
// Both touch the same 64-bit sxpinfo word
// Reader thread:        GC (main thread):
TYPEOF(x)  // read type    x->sxpinfo.mark = 1  // write mark
```

Under C11, this is undefined behavior regardless of whether the values "overlap" — the
compiler and CPU can tear the word.

**Fix options:**

| Approach | Cost | Feasibility |
|---|---|---|
| Split sxpinfo into GC-private + user-visible words | +4-8 bytes per SEXP (billions of objects) | Medium — ABI break, memory increase |
| Atomic load/store on entire sxpinfo word | Every TYPEOF/XLENGTH becomes atomic_load | Low code change, pervasive perf cost |
| **Freeze GC during read window** | Zero per-object cost | Easy if you control the sync point |

Option 3 (freeze GC) is by far the most practical — if the main thread is paused and GC
can't run, the bits are stable.

---

## 2. GC Can Free Objects Out From Under Readers

If GC somehow runs while readers hold raw SEXP pointers, freed objects become dangling pointers.

**Fix:** A read-write barrier around GC:

```c
pthread_rwlock_t R_GCBarrier;

// In R_gc_internal:
pthread_rwlock_wrlock(&R_GCBarrier);  // exclusive — blocks until all readers done
RunGenCollect(size_needed);
pthread_rwlock_wrunlock(&R_GCBarrier);

// In reader threads:
pthread_rwlock_rdlock(&R_GCBarrier);  // shared — multiple readers OK
// ... read SEXP data ...
pthread_rwlock_rdunlock(&R_GCBarrier);
```

Multiple readers hold the read lock concurrently. GC takes the write lock, waits until all
readers release. ~20ns uncontended on modern Linux.

---

## 3. ALTREP Dispatch Is the Showstopper

Even "reading" an ALTREP vector dispatches arbitrary C callbacks:

```c
// INTEGER_ELT dispatches:
int INTEGER_ELT(SEXP x, R_xlen_t i) {
    if (ALTREP(x))
        return ALTINTEGER_ELT(x, i);  // arbitrary C function call
    return INTEGER(x)[i];
}

// DATAPTR toggles a global flag:
R_GCEnabled = FALSE;          // RACE
void *val = ALTVEC_DISPATCH(Dataptr, x, writable);
R_GCEnabled = enabled;        // RACE
```

ALTREP methods can allocate, call `mkChar`, trigger GC, call back into R — anything.

**Option A: Require materialization before the read window (realistic)**

Call `DATAPTR(x)` on the main thread before the parallel region to force materialization.
R already supports this. After materialization, `INTEGER(x)` returns the cached data
pointer with no dispatch.

**Option B: Thread-local R_GCEnabled + thread-safe ALTREP methods (massive API change)**

Replace global `R_GCEnabled` with thread-local counter. Require all ALTREP class methods
to be annotated as thread-safe. Every existing ALTREP class needs auditing.

**Option A is the only realistic path.**

---

## 4. Reference Counting Mutations on "Reads"

R implicitly bumps refcounts when SEXPs flow through the evaluator. For truly read-only
access (just reading data pointers or element values), refcounts aren't touched. But any
R API call that *returns* a SEXP may increment its refcount.

**Fix:** Define a "read-only API surface" that never touches refcounts:

```
Safe for readers (non-ALTREP, GC frozen):
  TYPEOF, XLENGTH, LENGTH           — read sxpinfo (stable when GC frozen)
  INTEGER, REAL, LOGICAL, RAW, COMPLEX — return raw pointer (stable)
  STRING_ELT, VECTOR_ELT            — return child SEXP (no refcount bump)
  CHAR                               — return const char*

NOT safe for readers:
  Rf_allocVector, Rf_protect, mkChar, install — touch global state
  SET_*, SETCAR, SET_STRING_ELT              — fire write barrier
  Rf_eval, Rf_findVar                        — entire evaluation machinery
  DATAPTR on ALTREP                          — arbitrary dispatch
```

---

## 5. String Vectors Have an Indirection Layer

`STRING_ELT(x, i)` returns a CHARSXP (a SEXP). To get the actual `const char*`, you call
`CHAR()`. The CHARSXP is owned by the parent STRSXP, so if the parent is protected, the
child won't be GC'd. But you're reading SEXP pointers from the vector's data region —
requires the same sxpinfo stability guarantees.

**Fix:** Same as #1 — freeze GC during the read window. All SEXP pointers remain valid.

---

## The Minimal Change Set: Three Changes

### Change 1: GC read-write lock (~50 lines)

```c
// New in memory.c:
static pthread_rwlock_t R_GCBarrier = PTHREAD_RWLOCK_INITIALIZER;

// In R_gc_internal:
static void R_gc_internal(R_size_t size_needed) {
    pthread_rwlock_wrlock(&R_GCBarrier);
    // ... existing GC code ...
    pthread_rwlock_wrunlock(&R_GCBarrier);
}

// New public API:
void R_AcquireReadLock(void)  { pthread_rwlock_rdlock(&R_GCBarrier); }
void R_ReleaseReadLock(void)  { pthread_rwlock_rdunlock(&R_GCBarrier); }
```

### Change 2: Materialization requirement (~0 lines of R changes)

Document that SEXPs accessed from reader threads must be non-ALTREP (materialized).
Callers call `DATAPTR(x)` on the main thread before the parallel region.

### Change 3: Read-only API subset (~20 lines of documentation)

Define which C API functions are safe under `R_AcquireReadLock`.

---

## What This Enables

```c
SEXP x = PROTECT(allocVector(REALSXP, 1000000));
double *px = REAL(x);  // materialize / get pointer on main thread

R_AcquireReadLock();
// spawn N threads, each reads px[start..end]
// threads join
R_ReleaseReadLock();

UNPROTECT(1);
```

This is essentially what miniextendr does today with `with_r_thread` + raw pointers, except
the GC lock would make it *formally* safe rather than "works because we happen to not
trigger GC."

---

## What This Does NOT Enable

- Concurrent R evaluation (full interpreter rewrite)
- Concurrent allocation (protect stack, free lists, string cache)
- Concurrent ALTREP dispatch (arbitrary callbacks)
- Concurrent writes to SEXPs (write barrier, refcounts)
- Lock-free reads concurrent with running R code (sxpinfo bitfield race)

The fundamental constraint is R's stop-the-world mark-sweep GC with no concurrent marking.
Making reads concurrent *with GC* would require either a concurrent GC (enormous effort) or
splitting the sxpinfo bitfield (ABI break). The rwlock sidesteps both by preventing overlap.
