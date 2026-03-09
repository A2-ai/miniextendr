# Level 7: Shared-Heap Evaluation

Multiple threads evaluate R code against a shared heap. The "holy grail" — and the
most difficult concurrency level by orders of magnitude.

Source: R source at `background/r-svn/`.

---

## Requirements

This level requires everything below it:

- [Level 6: Concurrent GC](concurrency-level-6-concurrent-gc.md) — GC must work with multiple mutator threads
- [Level 2: Thread-Local Allocation](concurrency-level-2-thread-local-allocation-arenas.md) — per-thread allocation fast path
- [Level 3: Concurrent ALTREP](concurrency-level-3-concurrent-altrep.md) — ALTREP dispatch from any thread

Plus all of the following.

---

## Interpreter State Must Become Per-Thread

Every global that the evaluator touches must become thread-local:

### Evaluation Context (`Defn.h:1552-1566`)

```c
// Currently global:
extern0 SEXP    R_CurrentExpr;           // Defn.h:1552
LibExtern RCNTXT* R_GlobalContext;       // Defn.h:1558
extern Rboolean R_Visible;              // Defn.h:1562
extern0 int     R_EvalDepth INI_as(0);  // Defn.h:1563
```

`R_GlobalContext` is the head of a linked list of `RCNTXT` frames (`Defn.h:1381-1414`):

```c
typedef struct RCNTXT {
    struct RCNTXT *nextcontext;   // chain link
    int callflag;
    JMP_BUF cjmpbuf;             // C stack for longjmp
    int cstacktop;               // protect stack top at entry
    int evaldepth;
    SEXP promargs, callfun, sysparent, call, cloenv, conexit;
    SEXP handlerstack, restartstack;
    struct RPRSTACK *prstack;
    R_bcstack_t *nodestack, *bcprottop;
    R_bcFrame_type *bcframe;
    SEXP srcref;
} RCNTXT;
```

Each thread needs its own context chain. `sys.frame()`, `sys.call()`, `parent.frame()`
walk this chain — they must see only their own thread's frames.

### Bytecode Interpreter (`Defn.h:1534-1538, 1672-1674`)

```c
extern0 int          R_BCIntActive INI_as(0);   // Defn.h:1534
extern0 void*        R_BCpc INI_as(NULL);       // Defn.h:1536
extern0 SEXP         R_BCbody INI_as(NULL);     // Defn.h:1537
extern0 R_bcFrame_type *R_BCFrame INI_as(NULL); // Defn.h:1538
LibExtern R_bcstack_t *R_BCNodeStackTop;        // Defn.h:1672
extern0 R_bcstack_t *R_BCNodeStackBase;         // Defn.h:1673
extern0 R_bcstack_t *R_BCProtTop;               // Defn.h:1674
```

The bytecode interpreter (`bcEval` in `src/main/eval.c`) uses these globals as its
"registers." Two threads in `bcEval` simultaneously would corrupt each other's
instruction pointers, operand stacks, and protection state.

### Protect Stack (`Defn.h:1545-1547`)

```c
LibExtern int    R_PPStackSize  INI_as(R_PPSSIZE);  // 50000
LibExtern int    R_PPStackTop;
LibExtern SEXP*  R_PPStack;
```

Each thread needs its own protect stack. GC must scan all threads' protect stacks
during the root marking phase.

### Error/Warning State (`Defn.h:1625-1636`)

```c
extern0 int     R_CollectWarnings INI_as(0);
extern0 SEXP    R_Warnings;
extern0 SEXP    R_HandlerStack;    // condition handler stack
extern0 SEXP    R_RestartStack;    // available restarts
```

`tryCatch` and `withCallingHandlers` push onto `R_HandlerStack`. Per-thread handler
stacks are needed for proper condition handling.

### Making It Per-Thread

```c
// All of the above become:
_Thread_local SEXP    R_CurrentExpr;
_Thread_local RCNTXT* R_GlobalContext;
_Thread_local Rboolean R_Visible;
_Thread_local int     R_EvalDepth;
_Thread_local int     R_BCIntActive;
_Thread_local void*   R_BCpc;
_Thread_local SEXP    R_BCbody;
_Thread_local R_bcstack_t *R_BCNodeStackTop, *R_BCNodeStackBase, *R_BCProtTop;
_Thread_local int     R_PPStackTop;
_Thread_local SEXP*   R_PPStack;
_Thread_local SEXP    R_HandlerStack;
_Thread_local SEXP    R_RestartStack;
// ... ~30 more globals
```

This is the easy part — mechanical transformation. The hard part is everything below.

---

## Environment Locking

`defineVar` (`src/main/envir.c:1622-1688`) modifies environment hash chains:

```c
// envir.c:1675-1685 — hashed environment path
hashcode = HASHVALUE(c) % HASHSIZE(HASHTAB(rho));
R_HashSet(hashcode, symbol, HASHTAB(rho), value,
          (Rboolean) FRAME_IS_LOCKED(rho));
if (R_HashSizeCheck(HASHTAB(rho)))
    SET_HASHTAB(rho, R_HashResize(HASHTAB(rho)));
```

Two threads calling `defineVar` on the same environment race on the hash chain.
Even `findVarInFrame` (read-only lookup) races with `defineVar` (insert) on the
same chain.

### Options

**Per-environment reader-writer lock:**
```c
// Hypothetical:
typedef struct {
    SEXP frame;
    SEXP hashtab;
    pthread_rwlock_t lock;  // ← new
} R_EnvBody;
```

Reads (`findVarInFrame`) take shared lock. Writes (`defineVar`, `setVar`) take exclusive
lock. Cost: ~20ns per variable access (uncontended rwlock).

**Problem**: R's `<<-` operator searches parent environments. A single `<<-` might
traverse dozens of environments, each needing a lock. Lock ordering must prevent deadlock.

**Per-environment lock-free hash table**: Use a concurrent hash map (like Java's
`ConcurrentHashMap`). More complex but no deadlock risk. ~2000 lines per implementation.

---

## The Binding Semantics Problem

R's scoping rules create deep dependencies between environments:

```r
f <- function() {
    x <- 1          # local environment
    g <- function() {
        x <<- x + 1 # modifies parent's x
    }
    g()
    x               # reads modified value
}
```

`g()` modifies `f()`'s environment via `<<-`. If `g()` runs on a different thread than
`f()`, the write to `x` races with `f()`'s read.

More subtly, `<<-` searches up the parent chain using `findVarInFrame3`
(`envir.c:1525+`). The search itself traverses environment chains that might be
concurrently modified.

### The `ls()` Problem

`ls(envir = e)` iterates all bindings in an environment. A concurrent `defineVar` can
resize the hash table (`R_HashResize` at `envir.c:1685`) during iteration, invalidating
the iterator.

---

## Promise Evaluation

Promises are R's lazy evaluation mechanism. A promise wraps an unevaluated expression
and its environment. It evaluates on first access and caches the result:

```c
// Defn.h:1235-1247
#define PRVALUE0(x) ((x)->u.promsxp.value)
#define PROMISE_IS_EVALUATED(x) (PRVALUE0(x) != R_UnboundValue)
```

### The Race

Two threads forcing the same promise simultaneously:

```
Thread A: if (!PROMISE_IS_EVALUATED(p))    → FALSE (not yet evaluated)
Thread B: if (!PROMISE_IS_EVALUATED(p))    → FALSE (same, not yet evaluated)
Thread A: value = eval(PRCODE(p), PRENV(p))  → evaluating...
Thread B: value = eval(PRCODE(p), PRENV(p))  → also evaluating!
Thread A: SET_PRVALUE(p, value)              → stores result
Thread B: SET_PRVALUE(p, value)              → overwrites with different result?
```

If the promise body has side effects (it often does — printing, assignment, etc.),
double evaluation produces incorrect results.

### Fix: CAS on Promise State

```c
// Hypothetical atomic promise evaluation:
enum { PROMISE_UNEVALUATED, PROMISE_EVALUATING, PROMISE_EVALUATED };

SEXP force_promise(SEXP p) {
    int expected = PROMISE_UNEVALUATED;
    if (atomic_compare_exchange_strong(&p->promise_state, &expected, PROMISE_EVALUATING)) {
        // We won — evaluate
        SEXP val = eval(PRCODE(p), PRENV(p));
        SET_PRVALUE(p, val);
        atomic_store(&p->promise_state, PROMISE_EVALUATED);
        wake_waiters(p);
        return val;
    } else if (expected == PROMISE_EVALUATING) {
        // Another thread is evaluating — wait
        wait_for_evaluation(p);
        return PRVALUE(p);
    } else {
        // Already evaluated
        return PRVALUE(p);
    }
}
```

This needs a wait/wake mechanism (futex or condition variable per promise). Promises
are created in enormous quantities — the overhead must be minimal.

---

## The GIL Alternative

Python's approach: only one thread evaluates R code at a time. Release the GIL for C
extensions that don't need R API access.

```c
// Hypothetical R GIL:
static pthread_mutex_t R_GIL = PTHREAD_MUTEX_INITIALIZER;

SEXP Rf_eval(SEXP e, SEXP rho) {
    // GIL already held by calling thread
    // ... evaluate ...
}

// Release for C computation:
void R_ReleaseGIL(void) {
    pthread_mutex_unlock(&R_GIL);
}
void R_AcquireGIL(void) {
    pthread_mutex_lock(&R_GIL);
}
```

### Pros
- **Simple**: ~500 lines. All existing code works unchanged.
- **No races**: only one thread touches R state at a time.
- **Incremental**: GIL can be released for pure-C/Rust computation.

### Cons
- **Limited parallelism**: only one thread evaluates R code. Others wait.
- **GIL contention**: if threads frequently need R API, they serialize.
- **Python's experience**: the GIL has been Python's biggest scaling limitation for
  25 years. PEP 703 (free-threaded Python) is the result of decades of frustration.

The GIL is probably the **most realistic path** for shared-heap R — it provides threads
with access to the shared heap without requiring any of the hard changes above. The
trade-off is that R evaluation doesn't actually parallelize.

---

## The STM Alternative

Software Transactional Memory: wrap every environment mutation in a transaction.
Retry on conflict.

```c
// Hypothetical:
void defineVar_stm(SEXP symbol, SEXP value, SEXP rho) {
    stm_begin();
    SEXP frame = stm_read(FRAME(rho));
    // ... search and modify ...
    stm_write(FRAME(rho), new_frame);
    stm_commit();  // retries automatically on conflict
}
```

### Pros
- **Elegant**: composable transactions, no deadlocks
- **Optimistic**: no locks on read path, fast when conflicts are rare

### Cons
- **High overhead**: every SEXP read/write goes through the STM runtime (~10x overhead)
- **R is mutation-heavy**: environment operations, refcount updates, write barriers —
  high conflict rate means frequent retries
- **No existing implementation for C**: would need a custom STM runtime

Haskell uses STM successfully, but Haskell's pure functional model means transactions
are small and conflicts are rare. R's imperative style would stress STM heavily.

---

## Estimated Scope

| Component | Lines of C | Complexity |
|---|---|---|
| Per-thread interpreter state | ~2000 | Medium — mechanical |
| Per-thread protect stack + GC root scanning | ~1000 | Medium |
| Environment locking (rwlocks) | ~3000 | High — lock ordering |
| Promise CAS + wait/wake | ~2000 | High — correctness critical |
| Symbol table concurrency | ~1000 | Medium — concurrent hash map |
| String cache concurrency | ~1000 | Medium — concurrent hash map |
| Safepoint/handshake mechanism | ~2000 | High |
| Connection state thread-safety | ~500 | Medium |
| Error handling per-thread | ~500 | Medium |
| C API compatibility layer | ~5000 | Very high — extension compat |
| Testing infrastructure | ~5000+ | — |
| **Total** | **~50,000-100,000+** | **Complete interpreter rewrite** |

Timeline: 5-10 years with a dedicated team. Likely requires ABI break.

---

## No Dynamic Language Has Done This

| Language | Approach | Status |
|---|---|---|
| **Python** | GIL (1992-present), free-threaded experimental (2024) | GIL removal is ongoing, many extensions incompatible |
| **Ruby** | GVL (Global VM Lock), Ractors (2020) | Ractors are isolated (no shared heap) |
| **Lua** | Single-threaded by design, lanes for isolation | No shared-heap threading |
| **JavaScript** | Single-threaded event loop, SharedArrayBuffer (raw data only) | No shared-object threading |
| **Perl** | Interpreter threads (clone), abandoned in favor of fork | Shared heap abandoned as impractical |
| **R** | Single-threaded, fork for parallelism | No threading support |

The only languages with true shared-heap concurrent evaluation are statically typed:
Java, C#, Go, Rust. They achieve it through type systems that prevent data races
(Rust), concurrent GCs with decades of engineering (Java), or goroutine scheduling
with channel-based communication (Go).

Dynamic languages universally rely on either:
- **GIL/GVL**: one thread evaluates at a time (Python, Ruby)
- **Isolation**: separate heaps per thread (Erlang, Ruby Ractors, JS Workers)
- **Fork**: OS-level isolation (R, Perl)

---

## The Realistic Path for R

Given the analysis across all seven levels, the practical concurrency roadmap for R is:

| Level | Effort | Benefit | Likelihood |
|---|---|---|---|
| [Level 0](concurrency-level-0-raw-pointer-handoff.md): Raw pointers | Already done | Numeric parallelism | **Current state** |
| [Level 1](concurrency-level-1-gc-frozen-read-windows.md): GC rwlock | ~50 lines | Read-only SEXP traversal | **Highly feasible** |
| [Level 4](concurrency-level-4-fork-cow-snapshots.md): Fork improvements | ~500-3000 lines | Better mclapply | **Feasible** |
| [Level 3](concurrency-level-3-concurrent-altrep.md): ALTREP annotations | ~500 lines | Lazy parallel reads | **Feasible with Level 1** |
| [Level 2](concurrency-level-2-thread-local-allocation-arenas.md): Thread-local arenas | ~3000 lines | Thread allocation | **Possible, major effort** |
| [Level 5](concurrency-level-5-isolated-sub-interpreters.md): Sub-interpreters | ~15-30K lines | Isolated parallelism | **Possible, multi-year** |
| Level 6: Concurrent GC | ~5-15K lines | GC doesn't block threads | **Research project** |
| Level 7: Shared-heap eval | ~50-100K lines | True threading | **Essentially impossible** |

The biggest bang-for-buck is Levels 0 + 1 + 4: raw pointers for numeric data, GC rwlock
for read-only traversal, and improved fork for full R evaluation. These three cover the
vast majority of practical parallelism needs with minimal R changes.

Shared-heap evaluation (Level 7) is a 5-10 year, 50-100K line project that no dynamic
language has successfully completed. The GIL alternative (~500 lines) provides the same
API with serialized execution — likely the most R would ever implement.
