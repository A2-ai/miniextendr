# Level 3: Concurrent ALTREP Dispatch

Thread-safe ALTREP methods. Reader threads can call `INTEGER_ELT`, `REAL_ELT`, and
even `DATAPTR` on ALTREP vectors without returning to the main thread.

Source: R source at `background/r-svn/`.

---

## The Problem

ALTREP dispatch (Alternative Representations, R 3.5+) is fundamentally single-threaded.
Every ALTREP access goes through `ALTVEC_DATAPTR_EX`:

```c
// src/main/altrep.c:352-372
static R_INLINE void *ALTVEC_DATAPTR_EX(SEXP x, Rboolean writable)
{
    if (TYPEOF(x) == VECSXP && writable)
        ALTREP_ERROR_IN_CLASS("cannot take a writable DATAPTR of an ALTLIST", x);

    if (R_in_gc)
        error("cannot get ALTVEC DATAPTR during GC");
    R_CHECK_THREAD;
    int enabled = R_GCEnabled;
    R_GCEnabled = FALSE;          // ← global flag, no synchronization
    void *val = ALTVEC_DISPATCH(Dataptr, x, writable);
    R_GCEnabled = enabled;        // ← race if another thread reads R_GCEnabled
    return val;
}
```

Three problems:

1. `R_GCEnabled` is a single global `int` (`src/include/Defn.h:1532`). Two threads
   toggling it race.
2. `ALTVEC_DISPATCH` calls an arbitrary C function pointer. The method can allocate,
   protect, call `mkChar`, trigger GC — anything.
3. `R_CHECK_THREAD` is a no-op in release builds (`src/include/Defn.h:569`). There's
   no runtime enforcement.

Element-wise access (`INTEGER_ELT`, `REAL_ELT`) has the same problem — it dispatches
to the class's `Elt` method:

```c
// altrep.c (simplified):
int ALTINTEGER_ELT(SEXP x, R_xlen_t i) {
    return ALTINTEGER_DISPATCH(Elt, x, i);  // arbitrary C callback
}
```

---

## Approach A: Method Annotations (Surgical)

Add a thread-safety flag to ALTREP class registration. Only dispatch to thread-safe
methods from worker threads; fall back to materialization for unsafe ones.

### New API

```c
// In R_ext/Altrep.h — new flag:
#define R_ALTREP_THREAD_SAFE  0x01

// Registration:
R_altrep_class_t cls = R_make_altinteger_class("compact_intseq", "base", NULL);
R_set_altrep_thread_safety(cls, R_ALTREP_THREAD_SAFE);
```

### Modified Dispatch

```c
static R_INLINE void *ALTVEC_DATAPTR_EX(SEXP x, Rboolean writable)
{
    if (!on_main_thread()) {
        // Check if class is thread-safe
        R_altrep_class_t cls = ALTREP_CLASS(x);
        if (!(cls.flags & R_ALTREP_THREAD_SAFE)) {
            error("ALTREP class not thread-safe; materialize on main thread first");
        }
    }

    // Thread-safe path: skip R_GCEnabled toggle
    if (on_main_thread()) {
        int enabled = R_GCEnabled;
        R_GCEnabled = FALSE;
        void *val = ALTVEC_DISPATCH(Dataptr, x, writable);
        R_GCEnabled = enabled;
        return val;
    } else {
        // Thread-safe methods must not allocate or trigger GC
        return ALTVEC_DISPATCH(Dataptr, x, writable);
    }
}
```

### What Thread-Safe Methods Must Guarantee

A method marked `R_ALTREP_THREAD_SAFE` must:

- **Not allocate** — no `allocVector`, no `PROTECT`, no `mkChar`
- **Not trigger GC** — no `R_gc()`, no allocation that might trigger GC
- **Not call `install()`** — touches `R_SymbolTable`
- **Not call `Rf_eval`** — touches interpreter state
- **Be pure computation** — read from data1/data2, compute, return

### Which Existing Classes Could Qualify

R's built-in ALTREP classes (`src/main/altclasses.c`):

| Class | Thread-safe? | Why |
|---|---|---|
| `compact_intseq` (line 291) | **Yes** | `Elt` is arithmetic: `first + i * incr`. No allocation. |
| `compact_realseq` | **Yes** | Same — pure arithmetic |
| `wrap_integer` / `wrap_real` / etc. | **Maybe** | Delegates to wrapped vector. If wrapped is materialized, yes. |
| `deferred_string` | **No** | `Elt` calls `StringFromInteger`/`StringFromReal` → `mkChar` → allocates → inserts into `R_StringHash`. This is the most common ALTREP string class — created by `as.character()` on int/real vectors (coerce.c:1297-1299). See [ALTREP string race demo](../altrep/altrep-string-race-demo.md). |
| `wrap_string` | **Maybe** | Delegates `Elt` to `STRING_ELT` on wrapped vector. If wrapped is non-ALTREP, it's just a pointer read. |
| `mmap` (external) | **Maybe** | Depends on mmap implementation. Usually just pointer arithmetic. |
| `database-backed` (external) | **No** | I/O, allocation, possibly R callbacks |

The compact sequence classes are the strongest candidates — their `Elt` methods are
pure arithmetic with zero R API calls:

```c
// src/main/altclasses.c (compact_intseq Elt):
static int compact_intseq_Elt(SEXP x, R_xlen_t i)
{
    int n1 = COMPACT_INTSEQ_INFO_FIRST(COMPACT_SEQ_INFO(x));
    int inc = COMPACT_INTSEQ_INFO_INCR(COMPACT_SEQ_INFO(x));
    return (int) (n1 + i * inc);  // pure arithmetic
}
```

---

## Approach B: Thread-Local GC Inhibit (~500 lines)

Replace the global `R_GCEnabled` with a per-thread counter. Each thread can independently
inhibit GC for itself during ALTREP dispatch.

### The Change

```c
// Replace:
// extern0 int R_GCEnabled INI_as(1);   // src/include/Defn.h:1532

// With:
_Thread_local int R_GCInhibitCount = 0;
#define R_GCEnabled (R_GCInhibitCount == 0 && R_GlobalGCEnabled)
static int R_GlobalGCEnabled = 1;
```

### Modified ALTREP Dispatch

```c
static R_INLINE void *ALTVEC_DATAPTR_EX(SEXP x, Rboolean writable)
{
    R_GCInhibitCount++;              // thread-local, no race
    void *val = ALTVEC_DISPATCH(Dataptr, x, writable);
    R_GCInhibitCount--;
    return val;
}
```

### Problem: Methods That Allocate

This only prevents GC, not allocation. If an ALTREP method calls `allocVector`, it touches
`R_GenHeap[c].Free` (a global). With [Level 2 arenas](concurrency-level-2-thread-local-allocation-arenas.md),
allocation would go to the thread-local arena — safe.

Without Level 2, methods that allocate are still unsafe. The thread-local GC inhibit
only solves the `R_GCEnabled` race, not the underlying allocation concurrency problem.

---

## Interaction with Level 1 and Level 2

| Level | What it provides | What ALTREP needs |
|---|---|---|
| [Level 1](concurrency-level-1-gc-frozen-read-windows.md) | GC frozen → sxpinfo stable | Methods can read `data1`/`data2` safely |
| [Level 2](concurrency-level-2-thread-local-allocation-arenas.md) | Thread-local allocation | Methods can allocate temporary objects |
| Level 3 (this) | Thread-safe dispatch | Methods can be called from worker threads |

The practical path:

1. Level 1 (GC rwlock) + Approach A (annotations) — thread-safe pure-computation methods
   work immediately. ~550 lines total.
2. Level 2 (arenas) + Approach B (thread-local inhibit) — methods that allocate also work.
   ~3500 lines total.

---

## Estimated Scope

| Component | Lines of C | Requires |
|---|---|---|
| Approach A: method annotations | ~200 | Level 1 |
| Approach A: modified dispatch | ~100 | Level 1 |
| Approach A: audit existing classes | ~200 (code review) | — |
| Approach B: thread-local GC inhibit | ~100 | Level 2 |
| Approach B: modified allocation path | ~200 | Level 2 |
| **Total (Approach A)** | **~500** | Level 1 |
| **Total (Approach B)** | **~300 additional** | Level 2 |

---

## What This Enables

With concurrent ALTREP dispatch:

- **Lazy parallel iteration**: iterate over ALTREP vectors without materializing first
- **Compact sequence parallelism**: `1:1000000` can be read from N threads simultaneously
  with zero materialization overhead
- **Compute-on-access patterns**: ALTREP classes that compute values on the fly (like
  miniextendr's `#[derive(Altrep)]`) can serve concurrent readers

This is particularly valuable for miniextendr's ALTREP support — Rust-backed ALTREP
vectors with `#[altrep(rust_unwind)]` could potentially serve concurrent readers if
the Rust implementation is `Send + Sync`.

---

## What This Still Can't Do

- **ALTREP methods that call R code** — `Rf_eval` is single-threaded
- **Deferred string conversion** — `mkChar` needs string interning
- **Database-backed vectors** — I/O typically not thread-safe
- **Any method that needs `PROTECT`** — protect stack is per-thread only with Level 2

The next step toward full concurrency is [Level 4: Fork-COW Snapshots](concurrency-level-4-fork-cow-snapshots.md)
(a different approach entirely) or [Level 5: Sub-Interpreters](concurrency-level-5-isolated-sub-interpreters.md)
(same process, isolated heaps).
