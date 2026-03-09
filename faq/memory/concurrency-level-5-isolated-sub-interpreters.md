# Level 5: Isolated Sub-Interpreters

Multiple R runtimes in one process. Like Python 3.12 subinterpreters — each has its own
heap, GC, protect stack, evaluation state. Communication via explicit message passing.

Source: R source at `background/r-svn/`.

---

## The Problem: One R Instance Per Process

R has ~200+ global variables. Every piece of R's runtime — GC, evaluation, protect stack,
string cache, symbol table, bytecode interpreter — is stored in global variables declared
in `src/include/Defn.h:1529-1674`.

There is **one** R instance per process. Period. You can't create a second one.

---

## The Global Variable Inventory

### Memory Management (`Defn.h:1529-1542` + `memory.c` statics)

```c
extern0 int     R_GCEnabled INI_as(1);        // Defn.h:1532
extern0 int     R_in_gc INI_as(0);            // Defn.h:1533
```

Plus statics in `memory.c`:
- `R_GenHeap[NUM_NODE_CLASSES]` — 8 generation heaps with free lists (memory.c:621-630)
- `R_NodesInUse` — node counter (memory.c:616)
- `R_PreciousList` — permanent protect list
- `R_FreeSEXP` — free SEXP pointer
- `R_VStack` — vector stack

### Protect Stack (`Defn.h:1545-1547`)

```c
LibExtern int    R_PPStackSize  INI_as(R_PPSSIZE);  // 50000 by default
LibExtern int    R_PPStackTop;
LibExtern SEXP*  R_PPStack;
```

### Evaluation Context (`Defn.h:1552-1566`)

```c
extern0 SEXP    R_CurrentExpr;                // Defn.h:1552
LibExtern RCNTXT* R_GlobalContext;            // Defn.h:1558
extern Rboolean R_Visible;                    // Defn.h:1562
extern0 int     R_EvalDepth INI_as(0);       // Defn.h:1563
extern0 int     R_Expressions INI_as(5000);  // Defn.h:1566
```

The context chain (`RCNTXT`, `Defn.h:1381-1414`) is a linked list via `nextcontext`:

```c
typedef struct RCNTXT {
    struct RCNTXT *nextcontext;   // the chain
    int callflag;
    JMP_BUF cjmpbuf;             // C stack for longjmp
    int cstacktop;               // protect stack top at entry
    int evaldepth;
    SEXP promargs, callfun, sysparent, call, cloenv, conexit;
    SEXP handlerstack, restartstack;
    R_bcstack_t *nodestack, *bcprottop;
    // ...
} RCNTXT;
```

### Bytecode Interpreter (`Defn.h:1534-1538, 1672-1674`)

```c
extern0 int      R_BCIntActive INI_as(0);     // Defn.h:1534
extern0 void*    R_BCpc INI_as(NULL);         // Defn.h:1536
extern0 SEXP     R_BCbody INI_as(NULL);       // Defn.h:1537
extern0 R_bcFrame_type *R_BCFrame INI_as(NULL); // Defn.h:1538
LibExtern R_bcstack_t *R_BCNodeStackTop;      // Defn.h:1672
LibExtern R_bcstack_t *R_BCNodeStackEnd;      // Defn.h:1672
extern0 R_bcstack_t *R_BCNodeStackBase;       // Defn.h:1673
extern0 R_bcstack_t *R_BCProtTop;             // Defn.h:1674
```

### String Cache and Symbol Table

```c
// R_StringHash — envir.c:4169 (static)
// R_SymbolTable — names.c:1206 (SEXP array, HSIZE buckets)
```

Both are global hash tables. `R_StringHash` is swept by GC (`memory.c:1878+`).
`R_SymbolTable` is a root for GC marking.

### Error/Warning State (`Defn.h:1625-1636`)

```c
extern0 int     R_CollectWarnings INI_as(0);
extern0 SEXP    R_Warnings;
extern0 SEXP    R_HandlerStack;    // Defn.h:1629
extern0 SEXP    R_RestartStack;    // Defn.h:1630
```

### Environment Singletons

```c
// Rinternals.h / Defn.h:
LibExtern SEXP R_GlobalEnv;
LibExtern SEXP R_BaseEnv;
LibExtern SEXP R_EmptyEnv;
LibExtern SEXP R_NilValue;
LibExtern SEXP R_MissingArg;
LibExtern SEXP R_UnboundValue;
```

These are shared across all R code. `R_GlobalEnv` is where top-level assignments go.

### Connection State

Connections (`src/include/R_ext/Connections.h:50-85`) use global tables and file descriptors.

### Interrupts

```c
extern0 Rboolean R_interrupts_suspended INI_as(FALSE);
extern0 Rboolean R_interrupts_pending INI_as(FALSE);
```

---

## What Sub-Interpreters Would Require

### Step 1: `R_Runtime` Struct (~5000 lines)

Bundle all global state into a runtime struct:

```c
typedef struct R_Runtime {
    // Memory management
    int GCEnabled;
    int in_gc;
    struct { SEXP Old[2], New, Free; /* ... */ } GenHeap[8];
    R_size_t NodesInUse;
    SEXP PreciousList;

    // Protect stack
    int PPStackSize, PPStackTop;
    SEXP *PPStack;

    // Evaluation
    SEXP CurrentExpr;
    RCNTXT *GlobalContext;
    Rboolean Visible;
    int EvalDepth;
    int Expressions;

    // Bytecode
    int BCIntActive;
    void *BCpc;
    SEXP BCbody;
    R_bcstack_t *BCNodeStackTop, *BCNodeStackEnd, *BCNodeStackBase, *BCProtTop;

    // String cache + symbol table
    SEXP StringHash;
    SEXP *SymbolTable;

    // Environments
    SEXP GlobalEnv, BaseEnv, EmptyEnv;
    SEXP NilValue, MissingArg, UnboundValue;

    // Error/warning
    int CollectWarnings;
    SEXP Warnings, HandlerStack, RestartStack;

    // ... 150+ more fields ...
} R_Runtime;
```

Every function that currently reads a global (which is essentially every function in R)
must be changed to read from `R_Runtime *rt` instead.

### Step 2: Thread-Local Runtime Pointer (~500 lines)

```c
_Thread_local R_Runtime *R_CurrentRuntime = NULL;

// Every function's prologue:
#define R_RT (R_CurrentRuntime)
#define R_PPStackTop (R_RT->PPStackTop)
#define R_PPStack (R_RT->PPStack)
// ...
```

This macro approach minimizes code changes — existing code like `R_PPStack[R_PPStackTop++]`
would work through the macros.

### Step 3: Runtime Initialization (~2000 lines)

```c
R_Runtime *R_CreateRuntime(void) {
    R_Runtime *rt = calloc(1, sizeof(R_Runtime));
    // Initialize GC heaps
    // Initialize protect stack
    // Initialize base environment
    // Load base packages
    // Set up singletons (R_NilValue, R_MissingArg, etc.)
    return rt;
}
```

The tricky part: `R_NilValue`, `R_MissingArg`, and other singletons currently exist
once per process. Each sub-interpreter needs its own copies. But code that does
`if (x == R_NilValue)` expects pointer equality — each runtime's `R_NilValue` must be
a distinct object.

### Step 4: Shared vs. Private State (~3000 lines)

Some state could potentially be shared (immutable):

- **Base package bytecode** — compiled once, read-only
- **Base environment bindings** — mostly immutable after startup
- **Character encoding tables** — pure data

Everything else must be private:

- **GC heaps** — each runtime collects independently
- **Protect stacks** — per-runtime
- **Global environment** — per-runtime (user bindings differ)
- **Symbol table** — could be shared if read-only, but `install()` inserts

---

## Data Sharing Model

### Option A: Explicit Serialization (Like Fork)

Sub-interpreters communicate by serializing R objects to bytes and deserializing in the
other runtime. Same as `mclapply` but without the fork overhead.

Pros: Simple, no shared-heap complexity
Cons: Serialization cost for large objects

### Option B: Shared Immutable Objects

Objects can be shared between runtimes if they are immutable (refcount frozen,
no pending write barrier). Requires a cross-runtime reference counting scheme.

Pros: Zero-copy for read-only data
Cons: Complex reference counting, must prevent GC in one runtime from freeing objects
      used by another

### Option C: Shared Memory Regions

Designated `mmap`'d regions visible to all runtimes. Objects in shared regions are
never GC'd — explicitly managed.

Pros: Zero-copy, simple ownership
Cons: Manual memory management, fragmentation

---

## Python's Experience

Python's sub-interpreter journey (PEP 554 → PEP 684):

| Year | Milestone |
|---|---|
| 2017 | PEP 554 proposed sub-interpreters |
| 2019 | Per-interpreter GIL prototype |
| 2022 | PEP 684: per-interpreter GIL accepted |
| 2023 | Python 3.12: `_interpreters` module (experimental) |
| 2024 | Python 3.13: free-threaded mode (no GIL, experimental) |
| 2025 | Still restrictions: no sharing of most objects |

**Key lessons from Python:**

1. **Took ~5 years** from proposal to basic functionality
2. **Extension modules** are the hardest part — C extensions assume one interpreter
3. **Object sharing** is severely restricted — most objects can't cross interpreters
4. **Global state in C code** is the bottleneck — every `static` variable in every
   extension is a potential hazard

R would face the same challenges, amplified by:
- R's C API is far larger than Python's
- R packages with compiled code (Rcpp, data.table, etc.) assume global state
- R's GC is more tightly coupled to global variables than Python's

---

## Estimated Scope

| Component | Lines of C | Effort |
|---|---|---|
| `R_Runtime` struct definition | ~500 | Medium — catalogue all globals |
| Macro/accessor migration | ~5000 | Tedious — touch every file |
| Runtime initialization | ~2000 | Hard — replicate startup sequence |
| Cross-runtime serialization | ~1000 | Medium — extend existing serialize |
| Shared immutable objects | ~2000 | Hard — cross-runtime refcounting |
| Base package sharing | ~1500 | Hard — separate mutable from immutable |
| Extension compatibility layer | ~3000 | Very hard — C API compatibility |
| **Total** | **~15,000-30,000** | **Multi-year project, likely ABI break** |

---

## What This Enables

- **True per-task isolation** without fork overhead (~10μs vs ~1ms)
- **Windows support** for isolated parallelism (no `fork()` needed)
- **Fine-grained parallelism** — thousands of sub-interpreters feasible
- **Resource limits** — per-interpreter memory caps, timeout enforcement
- **Security isolation** — sandboxed evaluation environments

---

## What This Doesn't Enable

- **Shared mutable state** — each interpreter has its own heap
- **Fine-grained data sharing** — objects can't be freely passed between interpreters
- **Concurrent evaluation of the same code** — each interpreter evaluates independently
- **Reduced memory footprint** — each interpreter needs its own copy of loaded packages

For shared-heap concurrency, you need [Level 6: Concurrent GC](concurrency-level-6-concurrent-gc.md)
and [Level 7: Shared Heap Evaluation](concurrency-level-7-shared-heap-evaluation.md).

Sub-interpreters trade off memory for simplicity — each interpreter is a fully independent
R runtime that doesn't need to coordinate with others. This is often the right trade-off.
