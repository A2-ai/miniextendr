# Level 2: Thread-Local Allocation Arenas

Per-thread nurseries. Worker threads can allocate temporary SEXPs in thread-local arenas
that never enter the shared GC heap. Results "promoted" to the shared heap on the main
thread when returned.

Source: R source at `background/r-svn/`.

---

## The Idea

Each worker thread gets its own:

- **Free list** — like `R_GenHeap[c].Free` (`src/main/memory.c:841-853`) but thread-local
- **Page allocator** — private pages, no contention on `R_GenHeap[c].pages`
- **Mini protect stack** — like `R_PPStack`/`R_PPStackTop` (`src/include/Defn.h:1545-1547`) but thread-local
- **Arena boundary** — all objects allocated by this thread are tagged as arena-local

Arena-local objects never enter the shared `R_GenHeap`. No write barrier
(`CHECK_OLD_TO_NEW` at `src/main/memory.c:1313-1314`), no generation tracking, no
interaction with the main GC.

When the worker is done, results are **promoted**: bulk-copied from the arena to the
shared heap on the main thread, then the arena is freed wholesale.

---

## What R's Allocator Looks Like Today

```c
// src/main/memory.c:841-853
#define CLASS_GET_FREE_NODE(c,s) do { \
  SEXP __n__ = R_GenHeap[c].Free; \
  if (__n__ == R_GenHeap[c].New) { \
    GetNewPage(c); \
    __n__ = R_GenHeap[c].Free; \
  } \
  R_GenHeap[c].Free = NEXT_NODE(__n__); \
  R_NodesInUse++; \
  (s) = __n__; \
} while (0)
```

This touches three globals: `R_GenHeap[c].Free`, `R_GenHeap[c].New`, `R_NodesInUse`.
All three are plain variables with no synchronization. The `NEXT_NODE` / `PREV_NODE`
macros (`memory.c:634-641`) traverse circular doubly-linked lists:

```c
#define NEXT_NODE(s) (s)->gengc_next_node
#define PREV_NODE(s) (s)->gengc_prev_node
```

These are the Baker-style free lists (`memory.c:640-659`):

```c
#define UNSNAP_NODE(s) do { \
  SEXP next = NEXT_NODE(s); \
  SEXP prev = PREV_NODE(s); \
  SET_NEXT_NODE(prev, next); \
  SET_PREV_NODE(next, prev); \
} while(0)

#define SNAP_NODE(s,t) do { \
  SEXP next = (t); \
  SEXP prev = PREV_NODE(next); \
  SET_NEXT_NODE(prev, s); \
  SET_PREV_NODE(s, prev); \
  SET_NEXT_NODE(s, next); \
  SET_PREV_NODE(next, s); \
} while(0)
```

Making these thread-safe requires either per-thread copies or lock-free linked lists.
Per-thread copies are far simpler.

---

## Thread-Local Arena Design

### Per-Thread State

```c
// Hypothetical:
_Thread_local struct {
    struct {
        SEXP Free;
        SEXPREC Peg;          // sentinel for circular list
        PAGE_HEADER *pages;
    } ArenaHeap[NUM_NODE_CLASSES];  // 8 classes, same as R_GenHeap

    SEXP *PPStack;            // mini protect stack
    int PPStackTop;
    int PPStackSize;          // e.g. 1000 (much smaller than main's 50000)

    int NodesInUse;
} R_ThreadArena;
```

### Allocation

Arena allocation looks identical to `CLASS_GET_FREE_NODE`, but operates on
`R_ThreadArena.ArenaHeap` instead of `R_GenHeap`:

```c
#define ARENA_GET_FREE_NODE(c,s) do { \
  SEXP __n__ = R_ThreadArena.ArenaHeap[c].Free; \
  if (__n__ == &R_ThreadArena.ArenaHeap[c].Peg) { \
    ArenaGetNewPage(c); \
    __n__ = R_ThreadArena.ArenaHeap[c].Free; \
  } \
  R_ThreadArena.ArenaHeap[c].Free = NEXT_NODE(__n__); \
  R_ThreadArena.NodesInUse++; \
  (s) = __n__; \
} while (0)
```

No synchronization needed — each thread has its own arena. No write barrier because
arena objects only point to other arena objects (or to immutable shared-heap objects).

### Protection

Arena objects use the thread-local mini protect stack:

```c
SEXP arena_protect(SEXP s) {
    R_ThreadArena.PPStack[R_ThreadArena.PPStackTop++] = s;
    return s;
}
```

When the arena is freed, all protected objects in it are discarded — no `unprotect` needed.

### Promotion

When a worker thread returns a result to the main thread:

```c
SEXP arena_promote(SEXP s) {
    // Deep copy from arena to shared heap
    // Recursively copies all reachable arena objects
    // Returns new shared-heap SEXP
}
```

This is a bulk operation on the main thread. After promotion, the entire arena is freed
in one shot (free all pages, reset all lists).

---

## The String Interning Problem

`mkChar` (`src/main/envir.c:4169+`) inserts into the global `R_StringHash` table:

```c
// envir.c:4158-4167 — hash function
static unsigned int char_hash(const char *s, int len) {
    unsigned int h = 5381;
    for (p = (char *) s, i = 0; i < len; p++, i++)
        h = ((h << 5) + h) + (*p);
    return h;
}
```

The hash table uses pairlist chains. GC sweeps dead strings from it
(`src/main/memory.c:1878+`). Two problems:

1. **Concurrent lookup**: multiple threads traversing the same hash chain
2. **Concurrent insert**: multiple threads prepending to the same chain

### Options

**Option A: Thread-local string tables** — each arena has its own string hash table.
At promotion time, merge arena strings into `R_StringHash`. Duplicates detected during
merge. Cost: O(n_strings) merge per worker.

**Option B: Concurrent hash map** — replace `R_StringHash` with a lock-striped or
lock-free hash table. Much more complex, touches GC sweep path.

**Option C: Read-only access to `R_StringHash`** — workers can look up existing strings
(with GC frozen per [Level 1](concurrency-level-1-gc-frozen-read-windows.md)) but not
insert. New strings go to thread-local table, merged at promotion.

Option C is the most practical — most strings in a parallel computation already exist.

---

## The Symbol Table Problem

`install()` (`src/main/names.c:1257-1278`) does the same to `R_SymbolTable`:

```c
// names.c:1266-1267
for (sym = R_SymbolTable[i]; sym != R_NilValue; sym = CDR(sym))
    if (strcmp(name, CHAR(PRINTNAME(CAR(sym)))) == 0) return (CAR(sym));
```

And inserts:

```c
// names.c:1277
R_SymbolTable[i] = CONS(sym, R_SymbolTable[i]);
```

Same options as string interning. In practice, symbols are almost always pre-existing
(they're function names, parameter names, etc.), so read-only access with GC frozen
covers the common case.

---

## The Write Barrier Question

R's write barrier (`CHECK_OLD_TO_NEW`, `src/main/memory.c:1313-1314`) fires when a
young object is stored into an old object:

```c
#define CHECK_OLD_TO_NEW(x,y) do { \
  if (NODE_IS_OLDER(CHK(x), CHK(y))) old_to_new(x,y); } while (0)
```

Arena objects bypass this entirely:

- Arena → arena assignments: no generations, no barrier needed
- Arena → shared heap: impossible (arena objects can't be stored into shared heap objects)
- Shared heap → arena: impossible (arena objects aren't visible to shared heap)

The only cross-boundary operation is promotion, which deep-copies into the shared heap
on the main thread (where the normal write barrier applies).

---

## Estimated Scope

| Component | Lines of C | Complexity |
|---|---|---|
| Thread-local arena allocator | ~500 | Medium — mirrors existing `R_GenHeap` |
| Thread-local protect stack | ~100 | Low — just arrays |
| Thread-local string table | ~300 | Medium — hash table + merge |
| Promotion (deep copy) | ~400 | Medium — recursive SEXP copy |
| Arena lifecycle management | ~200 | Low — init/teardown |
| API surface (`R_ArenaAlloc`, etc.) | ~200 | Low — wrappers |
| Integration with `R_gc_internal` | ~100 | Low — skip arena objects during GC |
| Testing | ~200+ | — |
| **Total** | **~2000-3000** | **Major refactor of memory.c** |

---

## Precedent

| System | Mechanism | Similarity |
|---|---|---|
| Java (HotSpot) | Thread-Local Allocation Buffers (TLABs) | Per-thread bump pointer into shared heap |
| Go | P-local `mcache` + `mspan` | Per-P freelists, bulk allocation from central heap |
| .NET | Per-thread Gen0 nursery | Young generation is thread-local, promoted on GC |
| OCaml 5.0 | Per-domain minor heaps | Each domain (thread) has its own nursery |

The OCaml 5.0 approach is closest to what R would need — OCaml also has a GC-managed
heap with no concurrent marking, and added per-domain minor heaps for multicore support.

---

## What This Enables

With thread-local arenas, worker threads can:

- **Allocate vectors** — `allocVector` uses arena, not shared heap
- **Create strings** — `mkChar` uses arena string table
- **Build R objects** — lists, data frames, environments (arena-local)
- **Protect objects** — arena-local protect stack

But they still can't:

- **Evaluate R code** — `Rf_eval` touches too many globals ([Level 7](concurrency-level-7-shared-heap-evaluation.md))
- **Modify shared objects** — write barrier only works on main thread
- **Call arbitrary R functions** — would need the entire interpreter to be thread-safe
- **Dispatch ALTREP lazily** — still need [Level 3](concurrency-level-3-concurrent-altrep.md)

Thread-local arenas are the prerequisite for [Level 3: Concurrent ALTREP](concurrency-level-3-concurrent-altrep.md) —
ALTREP methods that allocate need somewhere thread-safe to put the results.
