# Level 1: GC-Frozen Read Windows

~50 lines of C. A `pthread_rwlock_t` around `R_gc_internal` lets threads safely read SEXP
metadata and follow SEXP pointers while GC is frozen.

Source: R source at `background/r-svn/`.

---

## The Mechanism

Add a read-write lock around garbage collection. The main thread takes a write lock when
running GC. Reader threads take shared read locks during parallel regions.

```c
// New in memory.c (~50 lines total):
static pthread_rwlock_t R_GCBarrier = PTHREAD_RWLOCK_INITIALIZER;

// Modified R_gc_internal (src/main/memory.c:3189):
static void R_gc_internal(R_size_t size_needed) {
    pthread_rwlock_wrlock(&R_GCBarrier);   // exclusive — blocks until readers done
    // ... existing GC code (lines 3189-3335) ...
    pthread_rwlock_wrunlock(&R_GCBarrier);
}

// New public API:
void R_AcquireReadLock(void)  { pthread_rwlock_rdlock(&R_GCBarrier); }
void R_ReleaseReadLock(void)  { pthread_rwlock_rdunlock(&R_GCBarrier); }
```

Multiple readers hold the read lock concurrently. GC takes the write lock, waiting until
all readers release. Cost: ~20ns per read window on modern hardware (uncontended
`pthread_rwlock_rdlock`).

---

## Why It Works

The core insight: **if GC can't run, the sxpinfo bitfield is stable**.

The [sxpinfo race](why-no-threaded-sexp-mutation.md#1-the-sexp-header-is-a-non-atomic-bitfield)
exists because GC writes `mark` and `gcgen` to the same 64-bit word that `TYPEOF` and
`LENGTH` read. With the rwlock:

1. Readers hold shared lock → GC blocked → no writes to sxpinfo `mark`/`gcgen` bits
2. GC holds exclusive lock → no readers → free to mark/sweep
3. No overlap → no data race → no undefined behavior

The generation lists (`R_GenHeap[i].Old`, `.New`, `.Free`) are also stable when GC isn't
running — nodes aren't moved between lists outside of GC.

---

## What It Unlocks

With GC frozen, threads can safely perform **read-only traversal** of R's object graph:

| Operation | What it does | Why it's now safe |
|---|---|---|
| `TYPEOF(x)` | Read `sxpinfo.type` (5 bits) | sxpinfo word is stable (GC frozen) |
| `LENGTH(x)` / `XLENGTH(x)` | Read length from sxpinfo or vecsxp | Same |
| `INTEGER(x)`, `REAL(x)`, etc. | Return data pointer (non-ALTREP) | Data region stable, no GC freeing |
| `STRING_ELT(x, i)` | Read CHARSXP from STRSXP data region | Child SEXP won't be freed |
| `VECTOR_ELT(x, i)` | Read SEXP from VECSXP data region | Same |
| `CHAR(x)` | Return `const char*` from CHARSXP | String data stable |
| `CAR(x)`, `CDR(x)` | Follow pairlist pointers | SEXP pointers stable |
| `TAG(x)` | Read pairlist tag | Same |
| `ATTRIB(x)` | Read attribute pairlist | Same |
| `getAttrib(x, sym)` | Traverse attribute chain (read-only) | All pointers stable |

This enables reading **string vectors**, **lists**, **data frames**, and **nested structures**
from worker threads — formally safe rather than the "gray area" at
[Level 0](concurrency-level-0-raw-pointer-handoff.md).

**Note on non-ALTREP strings**: `STRING_ELT` on a regular (non-ALTREP) STRSXP is already
just two pointer reads at Level 0 — the `ALTREP(x)` check reads sxpinfo.alt, and
`STDVEC_DATAPTR(x)[i]` loads the CHARSXP pointer. `CHAR(charsxp)` is pure pointer
arithmetic (no sxpinfo read at all). The GC rwlock makes this formally correct under C11
by eliminating the sxpinfo bitfield race, but in practice the Level 0 gray area already
works on x86-64. See [ALTREP string race demo](../altrep/altrep-string-race-demo.md) for the
full analysis.

---

## Prerequisite: ALTREP Must Be Materialized First

ALTREP dispatch (`ALTVEC_DATAPTR_EX`, `src/main/altrep.c:352-372`) toggles the global
`R_GCEnabled` flag and calls arbitrary C callbacks. This cannot happen from reader threads
even with the GC lock.

**Solution**: Materialize ALTREP vectors on the main thread before entering the read window:

```c
// Main thread — before read window:
for (int i = 0; i < ncol; i++) {
    SEXP col = VECTOR_ELT(df, i);
    if (ALTREP(col)) DATAPTR(col);  // force materialization
}

// Now safe:
R_AcquireReadLock();
// threads can safely read STRING_ELT, VECTOR_ELT, etc.
R_ReleaseReadLock();
```

After materialization, `INTEGER(x)` bypasses ALTREP dispatch and returns the cached data
pointer directly. The `ALTREP(x)` bit in sxpinfo is read-only during the window (stable).

---

## What It Still Can't Do

| Operation | Why |
|---|---|
| `Rf_allocVector` | Touches `R_GenHeap[].Free`, `R_NodesInUse` — allocation is write |
| `Rf_protect` | Writes `R_PPStack[R_PPStackTop++]` — non-atomic increment |
| `mkChar` / `install` | Inserts into `R_StringHash` / `R_SymbolTable` hash chains |
| `Rf_eval` | Modifies `R_GlobalContext`, `R_EvalDepth`, `R_BCNodeStackTop` |
| `SET_*` anything | Fires write barrier (`CHECK_OLD_TO_NEW` → `old_to_new`) |
| `defineVar` / `setVar` | Modifies environment hash chains (`src/main/envir.c:1622-1688`) |
| Refcount changes | `INCREMENT_REFCNT` is non-atomic read-modify-write on sxpinfo |

The read lock only freezes GC. It does NOT make the protect stack, evaluation machinery,
string cache, or write barrier thread-safe. Those require [Level 2](concurrency-level-2-thread-local-allocation-arenas.md)
and beyond.

---

## Cost Analysis

| Component | Cost |
|---|---|
| `pthread_rwlock_rdlock` (uncontended) | ~15-25ns on x86-64 Linux |
| `pthread_rwlock_rdlock` (contended with other readers) | ~20-30ns |
| `pthread_rwlock_wrlock` (GC waiting for readers) | Blocks until all readers done |
| Memory overhead | One `pthread_rwlock_t` (~56 bytes on Linux) |
| Code changes | ~50 lines in `memory.c` |

The main cost is **GC latency**: if a reader holds the lock during a long computation,
GC is delayed. In practice this is fine — the read window should be bounded (like a
parallel map over vector elements), and GC pressure during a read window is unlikely
since no allocation is happening.

---

## Interaction with the R Main Thread

The main thread must **not trigger GC** while reader threads hold the lock. In practice:

1. Main thread reaches a sync point (all R evaluation paused)
2. Main thread materializes ALTREP vectors
3. Main thread signals "read window open"
4. Worker threads take read locks, traverse R objects
5. Workers release read locks
6. Main thread resumes — GC can run again

This is the pattern described in [What would concurrent read-only R take](what-would-concurrent-read-only-r-take.md) —
the GC rwlock is Change 1 of the minimal 3-change set.

---

## What This Would Enable for miniextendr

Currently, miniextendr can only send raw `double*`/`int*` pointers to rayon workers.
With GC-frozen read windows, miniextendr could:

- **Parallel iterate over string vectors**: `STRING_ELT` + `CHAR` to get `&str` slices
- **Parallel iterate over lists**: `VECTOR_ELT` to read nested structures
- **Parallel read data frames**: read column types, dimensions, attributes
- **Parallel traverse R object graphs**: follow pairlist chains, attribute lists

This would be a major step up from Level 0's limitation to numeric arrays only.

---

## Implementation Sketch

```c
// memory.c additions (~50 lines):

#include <pthread.h>

static pthread_rwlock_t R_GCBarrier = PTHREAD_RWLOCK_INITIALIZER;

// Wrap GC with exclusive lock:
static void R_gc_internal(R_size_t size_needed)
{
    R_CHECK_THREAD;
    if (!R_GCEnabled || R_in_gc) {
        // ... existing fast path (lines 3192-3208) ...
        return;
    }

    pthread_rwlock_wrlock(&R_GCBarrier);  // ← NEW: exclusive lock

    BEGIN_SUSPEND_INTERRUPTS {
        R_in_gc = TRUE;
        gc_start_timing();
        gens_collected = RunGenCollect(size_needed);
        gc_end_timing();
        R_in_gc = FALSE;
    } END_SUSPEND_INTERRUPTS;

    pthread_rwlock_wrunlock(&R_GCBarrier);  // ← NEW: release

    // ... existing post-GC code ...
}

// Public API:
void R_AcquireReadLock(void) {
    pthread_rwlock_rdlock(&R_GCBarrier);
}

void R_ReleaseReadLock(void) {
    pthread_rwlock_rdunlock(&R_GCBarrier);
}
```

No ABI break. No per-object cost. No sxpinfo changes. Just a lock around the one function
that moves objects between generation lists and frees memory.

---

## Precedent

This is essentially what Java's ZGC does during its "relocation" phase — mutator threads
(the readers) are paused only during the brief marking phase, not during concurrent
reference processing. The difference is R's GC is much simpler (no concurrent marking),
so a simple rwlock suffices.

Go's GC uses a similar "stop-the-world" phase for stack scanning, with concurrent marking.
The rwlock approach is the simplest possible version of this idea — no concurrent marking,
just mutual exclusion between GC and readers.
