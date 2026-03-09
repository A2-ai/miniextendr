# Level 6: Concurrent Garbage Collection

GC runs while mutator threads continue. The prerequisite for any real shared-heap
concurrency — without this, threads must stop whenever GC runs.

Source: R source at `background/r-svn/`.

---

## R's Current GC: Stop-the-World Mark-Sweep

R uses a 3-generation stop-the-world mark-sweep collector (`src/main/memory.c`).

### Structure

```c
// memory.c:526-538 — node classes
#define NUM_NODE_CLASSES 8
static int NodeClassSize[NUM_SMALL_NODE_CLASSES] = { 0, 1, 2, 4, 8, 16 };

// memory.c:544-554 — generations
#define NUM_OLD_GENERATIONS 2

// memory.c:621-630 — per-class, per-generation heap structure
static struct {
    SEXP Old[NUM_OLD_GENERATIONS], New, Free;
    SEXPREC OldPeg[NUM_OLD_GENERATIONS], NewPeg;
    SEXP OldToNew[NUM_OLD_GENERATIONS];
    SEXPREC OldToNewPeg[NUM_OLD_GENERATIONS];
    int OldCount[NUM_OLD_GENERATIONS], AllocCount, PageCount;
    PAGE_HEADER *pages;
} R_GenHeap[NUM_NODE_CLASSES];
```

8 node classes × 3 generations (New + 2 Old) = 24 linked lists of SEXP nodes.

### Allocation Fast Path

```c
// memory.c:841-853
#define GET_FREE_NODE(s) CLASS_GET_FREE_NODE(0,s)
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

~10-15 instructions on the fast path: check free list, pop node, increment counter.
Baker-style circular doubly-linked free lists (`memory.c:640-659`).

### Collection

```c
// memory.c:3189-3241 — GC entry point
static void R_gc_internal(R_size_t size_needed)
{
    // ...
    BEGIN_SUSPEND_INTERRUPTS {
        R_in_gc = TRUE;
        gens_collected = RunGenCollect(size_needed);
        R_in_gc = FALSE;
    } END_SUSPEND_INTERRUPTS;
}
```

`RunGenCollect` (`memory.c:1681+`) has four phases:

1. **Eliminate old-to-new references** — scan `OldToNew` lists, age children (lines 1710-1720)
2. **Unmark and merge generations** — unmark nodes, promote survivors (lines 1731-1744)
3. **Mark roots** — forward `R_NilValue`, `R_GlobalEnv`, `R_HandlerStack`, protect stack,
   precious list, symbol table, context chain (lines 1758-1775)
4. **Mark reachable objects** — depth-first traversal via `FORWARD_CHILDREN` (lines 1776+)

Everything is stop-the-world: the mutator (R evaluation) is paused during the entire
mark-sweep cycle.

---

## Option A: Concurrent Mark-Sweep

### Tri-Color Marking

The standard approach for concurrent GC: objects are white (unmarked), gray (marked but
children not yet scanned), or black (marked and children scanned).

```
White → not yet reached by GC
Gray  → reached, but children not yet traced
Black → reached, all children traced
```

Mutator threads run concurrently with the marking phase. A write barrier logs mutations
that might violate the tri-color invariant.

### R Already Has a Write Barrier

```c
// memory.c:1313-1314
#define CHECK_OLD_TO_NEW(x,y) do { \
  if (NODE_IS_OLDER(CHK(x), CHK(y))) old_to_new(x,y); } while (0)

// memory.c:1271-1279
static void old_to_new(SEXP x, SEXP y)
{
    UNSNAP_NODE(x);
    SNAP_NODE(x, R_GenHeap[NODE_CLASS(x)].OldToNew[NODE_GENERATION(x)]);
}
```

This is a generational write barrier — it tracks when old objects point to new objects.
For concurrent marking, extend it to also log gray objects:

```c
// Hypothetical concurrent write barrier:
#define CONCURRENT_WRITE_BARRIER(x, y) do { \
    CHECK_OLD_TO_NEW(x, y); \
    if (gc_marking_active && IS_BLACK(x)) { \
        /* x was already marked black, but now points to white y */ \
        /* Re-gray x so the marker will re-scan its children */ \
        MARK_GRAY(x); \
        log_to_mark_stack(x); \
    } \
} while (0)
```

### Phases

1. **Initial mark (STW)** — brief pause to mark roots. Same as current phase 3, but only
   roots (~1ms for large heaps).
2. **Concurrent mark** — marker thread traces object graph while mutators run. Write barrier
   logs new references. Multiple passes may be needed (re-mark gray objects created by
   mutator writes).
3. **Remark (STW)** — brief pause to drain the write barrier log and finish marking.
   Should be very short if concurrent marking did most of the work.
4. **Concurrent sweep** — sweep thread returns white nodes to free lists while mutators run.
   Safe because white nodes are unreachable by definition.

### Cost

- Write barrier overhead: ~5-10 instructions per pointer store (check + conditional log)
- Marking thread: runs concurrently, no mutator pause
- STW pauses: two brief pauses (initial mark + remark), ~100μs-1ms each
- Overall: much better latency than current full STW, slightly worse throughput

---

## Option B: Concurrent Copying/Compacting

Move objects during collection. Requires read barriers.

### The Problem

R's object model is pointer-heavy. Every SEXP reference is a raw pointer. Moving an
object means updating every pointer to it.

A read barrier intercepts every SEXP dereference:

```c
// Every access to an object field:
#define READ_BARRIER(x) ((x)->forwarding_ptr ? (x)->forwarding_ptr : (x))

// CAR becomes:
#define CAR(x) (READ_BARRIER((READ_BARRIER(x))->u.listsxp.carval))
```

### Cost

Every SEXP access goes through an indirection. For R's pointer-heavy code (pairlists,
environments, closures), this is ~20-50% overhead on pointer-chasing workloads.

This is likely **too expensive for R**. Java's ZGC and Shenandoah use colored pointers
(metadata in unused pointer bits) to make read barriers cheaper, but R's SEXPs don't
have spare pointer bits.

---

## The sxpinfo Problem

R's mark bit shares a 64-bit word with the type, refcount, and other metadata:

```c
// Defn.h:131-149
struct sxpinfo_struct {
    SEXPTYPE type      :  5;   // bits 0-4
    unsigned int scalar:  1;   // bit 5
    unsigned int obj   :  1;   // bit 6
    unsigned int alt   :  1;   // bit 7
    unsigned int gp    : 16;   // bits 8-23
    unsigned int mark  :  1;   // bit 24 ← GC mark
    unsigned int debug :  1;   // bit 25
    unsigned int trace :  1;   // bit 26
    unsigned int spare :  1;   // bit 27
    unsigned int gcgen :  1;   // bit 28 ← generation
    unsigned int gccls :  3;   // bits 29-31 ← node class
    unsigned int named : NAMED_BITS; // refcount
    unsigned int extra : 32 - NAMED_BITS;
};
```

For concurrent marking, the GC marker writes `mark` while mutators read `type`, `named`,
etc. — from the same 64-bit word. Under C11, this is a data race (UB) even if the bits
don't overlap, because C bitfield access is whole-word read-modify-write.

### Fix Options

| Approach | Cost | ABI impact |
|---|---|---|
| Atomic CAS on entire sxpinfo word | Every `TYPEOF`/`XLENGTH` becomes `atomic_load` | No ABI break, pervasive perf cost |
| Split sxpinfo: GC word + user word | +8 bytes per SEXP (billions of objects) | ABI break |
| Use `mark` as a separate `_Atomic int` | +4 bytes per SEXP | ABI break |

The atomic CAS approach is the most practical for a first implementation:

```c
// Marking:
uint64_t old_info = atomic_load(&x->sxpinfo_word);
uint64_t new_info = old_info | MARK_BIT;
atomic_compare_exchange_strong(&x->sxpinfo_word, &old_info, new_info);

// Reading type (mutator):
uint64_t info = atomic_load(&x->sxpinfo_word);
SEXPTYPE type = info & 0x1F;
```

Cost: ~2-5ns per access on x86-64 (vs ~0.5ns for plain load). This is measurable but
probably acceptable for a concurrent GC.

---

## Reference Counting Interaction

R uses reference counting (`named` / refcount in sxpinfo) for copy-on-modify semantics:

```c
// Defn.h:263-267
#define INCREMENT_REFCNT(x) do {                  \
    SEXP irc__x__ = (x);                           \
    if (REFCNT(irc__x__) < REFCNTMAX)              \
        SET_REFCNT(irc__x__, REFCNT(irc__x__) + 1); \
} while (0)
```

This is a non-atomic read-modify-write on the sxpinfo word. Options:

1. **Atomic refcounts** — `atomic_fetch_add` on a separate refcount field. +4 bytes per SEXP.
2. **Tracing-only collector** — drop refcounting entirely, use only tracing GC. Major
   semantic change — `copy-on-modify` would need a different mechanism.
3. **Deferred refcounting** — mutator increments thread-local refcount buffers, periodically
   merged to the object. Complex but avoids atomic overhead on every reference.

---

## Free List Concurrency

`GET_FREE_NODE` pops from a global linked list (`R_GenHeap[c].Free`). For concurrent
allocation:

```c
// Option 1: Per-thread free lists (like Level 2 arenas)
_Thread_local SEXP thread_free_list[NUM_NODE_CLASSES];
// Threads pop from local list; refill from global list under lock

// Option 2: Lock-free LIFO stack (Treiber stack)
// CAS-based push/pop on R_GenHeap[c].Free
// ABA problem requires hazard pointers or epoch reclamation
```

Per-thread free lists are simpler and faster (no CAS contention).

---

## Estimated Scope

| Component | Lines of C | Complexity |
|---|---|---|
| Concurrent mark phase | ~2000 | High — tri-color + write barrier |
| Atomic sxpinfo access | ~500 | Medium — pervasive macro change |
| Concurrent sweep | ~1000 | Medium — parallel free list return |
| Per-thread free lists | ~500 | Medium — thread-local allocation |
| Write barrier extension | ~500 | Medium — add gray logging |
| STW pause coordination | ~500 | High — safepoint mechanism |
| Atomic/deferred refcounting | ~1000 | High — semantic change |
| **Total** | **~5,000-15,000** | **Fundamental memory.c rewrite** |

---

## Precedent

| System | GC type | Lines | Pause times |
|---|---|---|---|
| Java ZGC | Concurrent compacting | ~100K | <1ms (target) |
| Java Shenandoah | Concurrent compacting | ~80K | <10ms |
| Go | Concurrent mark-sweep | ~10K | <500μs |
| OCaml 5.0 | Stop-the-world major + concurrent minor | ~5K delta | ~10ms |
| Lua | Incremental mark-sweep | ~2K | ~1ms |

R's GC is closest to Lua's in simplicity. An incremental/concurrent version would likely
be ~5K-15K lines, between Lua and Go in complexity.

---

## What This Enables

With concurrent GC, the `R_GCBarrier` rwlock from [Level 1](concurrency-level-1-gc-frozen-read-windows.md)
becomes unnecessary — GC and mutators can overlap. This unlocks:

- **No STW pauses** for reader threads (GC doesn't block them)
- **Concurrent allocation** from multiple threads (with per-thread free lists)
- **Foundation for Level 7** — shared-heap evaluation needs concurrent GC

But concurrent GC alone doesn't make R's interpreter thread-safe — that requires
[Level 7: Shared Heap Evaluation](concurrency-level-7-shared-heap-evaluation.md).
