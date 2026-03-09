# Why You Cannot Mutate R-Managed Objects from Threads

R's entire runtime is a lock-free, single-threaded system by design. There are zero mutexes,
zero atomics, and zero memory barriers anywhere in its SEXP manipulation code. Every R API
call assumes exclusive access to dozens of global variables.

Source: R source at `background/r-svn/` (R-devel trunk).

---

## 1. The SEXP Header Is a Non-Atomic Bitfield

Every R object's header (`sxpinfo_struct`, `src/include/Defn.h:131-149`) packs the GC mark bit,
generation bits, reference count, type tag, and more into a single 64-bit word:

```c
struct sxpinfo_struct {
    SEXPTYPE type :  5;
    unsigned int mark  :  1;     // GC mark bit
    unsigned int gcgen :  1;     // generation number
    unsigned int gccls :  3;     // node class
    unsigned int named : 16;     // reference count
    // ... more bits
};
```

C bitfield reads/writes are **read-modify-write** on the whole word. Two threads writing
different bits (e.g., GC sets `mark`, your code sets `named`) corrupt each other.

---

## 2. The Protect Stack Is a Global Array + Counter

```c
// src/include/Defn.h:1544-1547
LibExtern int    R_PPStackTop;    // plain int, no atomic
LibExtern SEXP*  R_PPStack;      // plain array

// src/include/Rinlinedfuns.h:492-500
INLINE_FUN SEXP protect(SEXP s) {
    R_PPStack[R_PPStackTop++] = s;   // not atomic
    return s;
}
```

Two threads calling `protect()` simultaneously race on `R_PPStackTop` — both read the same
index, both write to the same slot, one protect is lost, GC frees a live object.

---

## 3. Any Allocation Can Trigger Full GC

```c
// src/main/memory.c:2441-2450
SEXP allocSExp(SEXPTYPE t) {
    if (FORCE_GC || NO_FREE_NODES()) {
        R_gc_internal(0);   // FULL GC - walks ALL roots
    }
    GET_FREE_NODE(s);
    ...
}
```

GC (`R_gc_internal`) walks the protect stack, the symbol table, the context chain, the
precious list, every generation's linked list, and the string cache. If your thread triggered
GC while the main thread is mid-operation on any of these structures, you get corrupted
pointers and segfaults.

---

## 4. The Write Barrier Modifies Global Linked Lists

Every `SET_VECTOR_ELT`, `SETCAR`, `SET_STRING_ELT` fires the generational write barrier:

```c
// src/main/memory.c:1271-1278
static void old_to_new(SEXP x, SEXP y) {
    UNSNAP_NODE(x);  // unlink from current doubly-linked list
    SNAP_NODE(x, R_GenHeap[NODE_CLASS(x)].OldToNew[NODE_GENERATION(x)]);
}
```

This splices nodes in/out of `R_GenHeap` doubly-linked lists — the same lists GC traverses.
No locks.

---

## 5. Reference Counting Is Non-Atomic

```c
// src/include/Defn.h:263-267
#define INCREMENT_REFCNT(x) do {                  \
    SEXP irc__x__ = (x);                           \
    if (REFCNT(irc__x__) < REFCNTMAX)              \
        SET_REFCNT(irc__x__, REFCNT(irc__x__) + 1); \
} while (0)
```

Classic TOCTOU race. The `named` field shares the same 64-bit word as the GC `mark` bit.

---

## 6. String Interning and Symbol Tables Are Unsynchronized

`mkChar` searches and inserts into the global `R_StringHash` table (`src/main/envir.c:4295+`).
`install()` does the same to `R_SymbolTable` (`src/main/names.c:1257+`). Both may allocate
(triggering GC), and GC itself sweeps `R_StringHash` to remove dead strings during collection:

```c
// src/main/memory.c:1878+ (inside GC)
while (s != R_NilValue) {
    if (! NODE_IS_MARKED(CXHEAD(s))) {
        CXTAIL(t) = CXTAIL(s);   // modifying the chain GC-side
```

A concurrent `mkChar` traversal sees a chain being spliced apart.

---

## 7. ALTREP Dispatch Toggles Global GC Flag

```c
// src/main/altrep.c:352-372
static R_INLINE void *ALTVEC_DATAPTR_EX(SEXP x, Rboolean writable) {
    int enabled = R_GCEnabled;
    R_GCEnabled = FALSE;          // disable GC globally!
    void *val = ALTVEC_DISPATCH(Dataptr, x, writable);
    R_GCEnabled = enabled;        // re-enable
}
```

Two threads calling `DATAPTR` race on the single global `R_GCEnabled`.

---

## 8. R Knows This — And Considers It Fatal

The thread check exists but is **debug-only** (`src/include/Defn.h:563-571`):

```c
#ifdef THREADCHECK
# define R_CHECK_THREAD R_check_thread(__func__)
#else
# define R_CHECK_THREAD do {} while (0)   // NO-OP in release builds
#endif
```

When enabled, violation calls `R_Suicide()` — immediate process termination, not a
recoverable error. R treats wrong-thread SEXP access as unrecoverable by design.

---

## Complete Hazard Summary

| What you touch | Global state involved | Why it's unsafe |
|---|---|---|
| `Rf_protect` | `R_PPStackTop`, `R_PPStack` | Non-atomic index increment |
| `SET_VECTOR_ELT` | `sxpinfo` bitfield, `R_GenHeap` linked lists | Write barrier splices global lists |
| `mkChar` / `install` | `R_StringHash`, `R_SymbolTable` | Hash chain traversal concurrent with GC sweep |
| Any allocation | `R_GenHeap[].Free`, `R_NodesInUse` + all GC roots | May trigger full GC that walks everything |
| `DATAPTR` (ALTREP) | `R_GCEnabled` | Global flag toggle with no synchronization |
| Refcount changes | `sxpinfo.named` bitfield | Shares 64-bit word with GC mark bit |
| `R_PreserveObject` | `R_PreciousList` | List modification concurrent with GC root scan |

**There is no safe subset of the R API you can call from a background thread.** Even reading
a vector element via `INTEGER_ELT` goes through ALTREP dispatch which touches `R_GCEnabled`.
Even `TYPEOF(x)` reads the bitfield word that GC writes `mark` into. The entire SEXP system
is designed around the invariant that exactly one thread touches it at any time.

---

## What IS Safe from Threads

- Reading raw data pointers (`INTEGER(x)`, `REAL(x)`) obtained on the main thread, for
  **non-ALTREP** vectors whose data won't be GC'd (protected on main thread). The data
  region itself is just a C array — no R metadata touched.
- Pure Rust/C computation on non-R data.
- Sending results back to the main thread for SEXP construction.

This is why miniextendr uses the worker thread pattern: Rust runs on its own thread, and
any SEXP access is marshaled back to the R main thread via `with_r_thread()`.

---

## Key Global Variables (Comprehensive List)

From `src/include/Defn.h:1529-1674` and `src/main/memory.c`:

**Memory management**: `R_NSize`, `R_VSize`, `R_GCEnabled`, `R_in_gc`, `R_NHeap`,
`R_FreeSEXP`, `R_Collected`, `R_GenHeap[NUM_NODE_CLASSES]` (static), `R_NodesInUse` (static),
`R_VStack` (static), `R_PreciousList` (static)

**Protect stack**: `R_PPStackSize`, `R_PPStackTop`, `R_PPStack`

**Evaluation context**: `R_CurrentExpr`, `R_ReturnedValue`, `R_SymbolTable`,
`R_Toplevel`, `R_ToplevelContext`, `R_GlobalContext`, `R_SessionContext`, `R_ExitContext`,
`R_Visible`, `R_EvalDepth`, `R_Expressions`

**Bytecode interpreter**: `R_BCNodeStackTop`, `R_BCNodeStackEnd`, `R_BCNodeStackBase`,
`R_BCProtTop`, `R_BCIntActive`, `R_BCpc`, `R_BCbody`

**Warnings/errors**: `R_CollectWarnings`, `R_Warnings`, `R_HandlerStack`, `R_RestartStack`

**Interrupts**: `R_interrupts_suspended`, `R_interrupts_pending`

**String cache**: `R_StringHash`
