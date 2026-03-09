# ALTREP String Race: A Concrete Demonstration

A worked example showing exactly when concurrent string reads are safe, unsafe, or
in a gray area. Written in Rust with R's C API.

Source: R source at `background/r-svn/`.

---

## Background: Two Paths Through STRING_ELT

`STRING_ELT` (Rinlinedfuns.h:475-482) has two paths:

```c
INLINE_FUN SEXP STRING_ELT(SEXP x, R_xlen_t i) {
    if (ALTREP(x))                       // read sxpinfo.alt (bit 7)
        return ALTSTRING_ELT(x, i);      // ALTREP path — heavy
    else {
        SEXP *ps = STDVEC_DATAPTR(x);    // pointer arithmetic: (SEXPREC_ALIGN*)(x) + 1
        return ps[i];                     // load SEXP pointer from array
    }
}
```

And `CHAR` (Defn.h:423) is pure pointer arithmetic — no function call, no globals:

```c
#define STDVEC_DATAPTR(x)  ((void *) (((SEXPREC_ALIGN *) (x)) + 1))
#define CHAR(x)            ((const char *) STDVEC_DATAPTR(x))
```

The string bytes live immediately after the CHARSXP header in memory. `CHAR` just adds
`sizeof(SEXPREC_ALIGN)` to the pointer.

Note: the public API function `R_CHAR` (memory.c:4100-4105) adds a `TYPEOF(x)` validation
check (reads sxpinfo), but the internal `CHAR` macro does not.

---

## When Are String Vectors ALTREP?

**Only when created by `as.character()` on integer or real vectors.** This is the deferred
string coercion optimization (coerce.c:1294-1302):

```c
case STRSXP:
    if (ATTRIB(v) == R_NilValue)
        switch(TYPEOF(v)) {
        case INTSXP:
        case REALSXP:
            ans = R_deferred_coerceToString(v, NULL);  // ← creates ALTREP
            return ans;
        }
    ans = coerceToString(v);  // ← regular STRSXP
```

| How the string vector was created | ALTREP? |
|---|---|
| `as.character(1:1000000)` | **Yes** — deferred conversion |
| `as.character(c(1.5, 2.7))` | **Yes** — deferred conversion |
| `c("hello", "world")` | No |
| `paste0("x", 1:10)` | No — paste materializes |
| `character(100)` | No |
| `readLines("file.txt")` | No |
| Returned from `.Call()` | No |
| `names(some_vector)` | Maybe — `wrap_string` ALTREP |

The `deferred_string` class (altclasses.c:845-869) lazily converts each element via
`StringFromInteger`/`StringFromReal` → `mkChar` on first access.

---

## The Broken Case: Deferred ALTREP String from a Thread

```rust
unsafe fn broken_concurrent_string_read(int_vec: SEXP, n: usize) {
    // Main thread: coerce integer → character.
    // Returns a DEFERRED ALTREP string — no strings materialized yet.
    let strsxp = Rf_coerceVector(int_vec, STRSXP);
    Rf_protect(strsxp);

    // This is ALTREP (coerce.c:1297-1299)
    assert!(ALTREP(strsxp) != 0);

    // Spawn a thread that reads elements
    let handle = thread::spawn(move || {
        for i in 0..n {
            // STRING_ELT dispatches through ALTSTRING_ELT (altrep.c:518-530):
            //
            //   int enabled = R_GCEnabled;
            //   R_GCEnabled = FALSE;          // ← WRITE TO GLOBAL
            //   val = ALTSTRING_DISPATCH(Elt, x, i);
            //   R_GCEnabled = enabled;        // ← WRITE TO GLOBAL
            //
            // ALTSTRING_DISPATCH calls deferred_string_Elt (altclasses.c:776-788):
            //   → StringFromInteger(INTEGER_ELT(data, i))
            //   → mkChar(buf)
            //   mkChar searches/inserts R_StringHash (global hash table)
            //   mkChar may call allocVector() → may trigger R_gc_internal
            //
            let charsxp = STRING_ELT(strsxp, i as R_xlen_t);
            let _s = CHAR(charsxp);
        }
    });

    // Meanwhile, main thread does normal R work.
    // Any allocation can trigger R_gc_internal, which:
    //   1. Reads R_GCEnabled (worker is writing it)
    //   2. Walks R_StringHash (worker's mkChar is modifying it)
    //   3. Writes mark bits to sxpinfo (worker reads sxpinfo via ALTREP check)
    let another_vec = Rf_allocVector(REALSXP, 1000);
    Rf_protect(another_vec);

    handle.join().unwrap();
    Rf_unprotect(2);
}
```

**What goes wrong:**

| Race | Global | Consequence |
|---|---|---|
| Worker writes `R_GCEnabled = FALSE`, main reads it in `R_gc_internal` | `R_GCEnabled` | GC skipped → memory leak; or GC proceeds → double corruption |
| Worker's `mkChar` traverses `R_StringHash`, GC sweeps dead strings from it | `R_StringHash` chain | Hash chain corruption → segfault on next `mkChar` |
| Worker's `mkChar` calls `allocVector` for new CHARSXP, races with main's allocation | `R_GenHeap[c].Free` | Two threads pop same free node → use-after-free |

---

## Making Races Visible with `gctorture`

R's GC torture modes force GC to run much more frequently than normal, dramatically
increasing the probability that a GC cycle overlaps with concurrent access.

### How `gctorture` Works Internally

`gctorture` sets two global variables (memory.c:233-234):

```c
static int gc_force_wait = 0;   // countdown until next forced GC
static int gc_force_gap = 0;    // reset value after each forced GC
```

Every allocation path in R checks (memory.c:2447, 2466, 2485, etc.):

```c
if (FORCE_GC || NO_FREE_NODES()) {
    R_gc_internal(size_needed);
}
```

Where `FORCE_GC` (memory.c:236) decrements the wait counter and triggers GC when it
reaches zero:

```c
#define FORCE_GC (gc_pending || \
    (gc_force_wait > 0 ? (--gc_force_wait > 0 ? 0 : \
        (gc_force_wait = gc_force_gap, 1)) : 0))
```

With `gctorture(TRUE)` (gap=1), GC runs on **every single allocation**. With
`gctorture2(step=10)`, GC runs every 10 allocations.

### Why `gctorture` Makes the Broken Case Catastrophic

```rust
unsafe fn broken_with_gctorture(int_vec: SEXP, n: usize) {
    // Enable GC torture: force GC on every allocation.
    // gctorture(TRUE) sets gc_force_gap = gc_force_wait = 1
    //
    // IMPORTANT: gc_force_wait and gc_force_gap are themselves unprotected
    // globals — they have no synchronization either. But since only the main
    // thread should be calling R API functions, R doesn't care. The problem
    // is when a worker thread also triggers allocations...
    Rf_eval(
        Rf_lang2(Rf_install("gctorture"), Rf_ScalarLogical(1)),
        R_GlobalEnv,
    );

    let strsxp = Rf_coerceVector(int_vec, STRSXP);
    Rf_protect(strsxp);

    let handle = thread::spawn(move || {
        for i in 0..n {
            // Each STRING_ELT on the deferred ALTREP string calls:
            //   deferred_string_Elt → StringFromInteger → mkChar → allocVector
            //
            // With gctorture(TRUE), that allocVector checks FORCE_GC:
            //   --gc_force_wait == 0 → gc_force_wait = gc_force_gap → triggers GC
            //
            // But FORCE_GC decrements gc_force_wait (a global!) from the worker.
            // The main thread's allocations also decrement gc_force_wait.
            // Two threads doing --gc_force_wait is a data race on the counter.
            //
            // Then R_gc_internal runs on the worker thread:
            //   - Walks ALL generation lists (R_GenHeap[0..7].Old, .New)
            //   - Writes mark bits to every reachable SEXP's sxpinfo
            //   - Frees unmarked nodes back to free lists
            //   - Rehashes R_StringHash if needed
            //
            // While the main thread's GC might ALSO be running.
            // Two concurrent mark-sweep passes = catastrophic corruption.
            let charsxp = STRING_ELT(strsxp, i as R_xlen_t);
            let _s = CHAR(charsxp);
        }
    });

    // Main thread: every allocation forces GC too.
    // With n=10000, we get ~10000 worker GCs + main thread GCs all racing.
    for _ in 0..100 {
        let tmp = Rf_allocVector(REALSXP, 10);
        Rf_protect(tmp);
        Rf_unprotect(1);
    }

    handle.join().unwrap();

    // Disable torture
    Rf_eval(
        Rf_lang2(Rf_install("gctorture"), Rf_ScalarLogical(0)),
        R_GlobalEnv,
    );
    Rf_unprotect(1);
}
```

Without `gctorture`, the broken case might appear to work if the main thread doesn't
happen to trigger GC during the window. With `gctorture(TRUE)`, GC fires on **every
allocation on both threads**, turning a probabilistic race into a near-certain crash.

### Additional Races Exposed by `gctorture`

| Race | Without torture | With torture |
|---|---|---|
| `R_GCEnabled` toggle | Worker toggles, main might read | Both threads trigger GC → both read/write `R_GCEnabled` |
| `R_StringHash` corruption | Worker inserts, GC might sweep | Worker inserts, GC sweeps every allocation → guaranteed overlap |
| `R_GenHeap[c].Free` list | Worker pops, main might pop | Both threads pop free nodes every allocation → immediate corruption |
| `gc_force_wait` counter | N/A (main thread only) | Both threads decrement → data race on the torture counter itself |
| `R_PPStackTop` protect stack | Worker calls `Rf_protect` in `mkChar` | Worker's protect interleaves with main's → stack corruption |

### Using `gctorture2` for Controlled Stress

`gctorture2(step, wait, inhibit_release)` is more configurable:

- **`step`**: GC every `step` allocations (default: 0 = off)
- **`wait`**: Skip first `wait` allocations before starting torture
- **`inhibit_release`**: If TRUE, don't release freed memory back to OS (builds with
  `PROTECTCHECK` only) — catches use-after-free bugs where freed SEXPs are still referenced

```rust
// gctorture2(step=1, wait=0, inhibit_release=FALSE)
// Same as gctorture(TRUE) but returns previous step value
unsafe fn setup_gctorture2() {
    Rf_eval(
        Rf_lang4(
            Rf_install("gctorture2"),
            Rf_ScalarInteger(1),    // step: GC every allocation
            Rf_ScalarInteger(0),    // wait: start immediately
            Rf_ScalarLogical(0),    // inhibit_release: FALSE
        ),
        R_GlobalEnv,
    );
}
```

For the broken case, even `gctorture2(step=100)` is enough to reliably trigger crashes —
each `mkChar` in `deferred_string_Elt` does at least one allocation, so with 10000 elements
you get ~100 forced GCs overlapping with the worker thread.

### The Gray Area Under `gctorture`

Interestingly, `gctorture` does **not** break the gray area case (non-ALTREP `STRING_ELT`).
The reader thread makes zero allocations — it just reads pointers. Even with GC running on
every main-thread allocation:

- The sxpinfo bitfield race is the same with or without torture (GC writes mark bits either way)
- The STRSXP is protected, so GC won't free it or its CHARSXP children
- No worker-thread allocations means no worker-triggered GC
- The gray area remains "technically UB, practically safe" regardless of torture settings

This makes `gctorture` a useful **diagnostic**: if your concurrent code crashes under
`gctorture`, you have a real bug (not just theoretical UB). If it survives, you're likely
in the gray area at worst.

---

## The Gray Area: Regular STRSXP from a Thread

```rust
unsafe fn gray_area(strsxp: SEXP, n: usize) {
    Rf_protect(strsxp);
    assert!(ALTREP(strsxp) == 0); // must be regular STRSXP

    thread::scope(|s| {
        s.spawn(|| {
            for i in 0..n {
                // Non-ALTREP path (Rinlinedfuns.h:478-480):
                //   SEXP *ps = STDVEC_DATAPTR(x);  // pointer arithmetic
                //   return ps[i];                    // load SEXP from array
                //
                // But first: ALTREP(x) reads x->sxpinfo.alt (bit 7).
                // GC might be writing x->sxpinfo.mark (bit 24) concurrently.
                // Same 64-bit word → C11 undefined behavior.
                // x86-64: word loads are atomic, bits don't interfere → works fine.
                let charsxp = STRING_ELT(strsxp, i as R_xlen_t);

                // CHAR: pure pointer arithmetic. No sxpinfo. No globals.
                let _s = CHAR(charsxp);
            }
        });

        // Main thread allocates → may trigger GC → GC writes mark bits
        let _ = Rf_allocVector(INTSXP, 100);
    });

    Rf_unprotect(1);
}
```

**What could theoretically go wrong:**

The `ALTREP(x)` check reads `sxpinfo.alt` (bit 7). If GC is simultaneously marking
the same object (writing `sxpinfo.mark`, bit 24), that's two threads accessing the same
64-bit word — undefined behavior under C11.

**What actually goes wrong on real hardware:** Nothing. On x86-64:
- 64-bit aligned loads/stores are atomic
- The `alt` bit (set at creation, never written again) and `mark` bit (written by GC)
  are in different bit positions
- No compiler would merge or reorder a plain struct field read into something dangerous

This is "technically UB, practically safe on every real platform."

---

## The Safe Case: Pre-Extract on Main Thread

```rust
unsafe fn safe_concurrent_string_read(strsxp: SEXP, n: usize) {
    Rf_protect(strsxp);

    // If ALTREP, force materialization on main thread first
    if ALTREP(strsxp) != 0 {
        DATAPTR(strsxp); // triggers expand_deferred_string (altclasses.c:747-761)
    }

    // Extract all CHAR pointers on main thread — no R API needed from threads
    let mut ptrs: Vec<*const c_char> = Vec::with_capacity(n);
    for i in 0..n {
        let charsxp = STRING_ELT(strsxp, i as R_xlen_t);
        ptrs.push(CHAR(charsxp)); // pure pointer arithmetic
    }

    // Threads read raw C strings — zero R interaction
    let ptrs_ref = &ptrs;
    thread::scope(|s| {
        s.spawn(|| {
            for &ptr in ptrs_ref.iter() {
                let _s = CStr::from_ptr(ptr);
                // Just reading bytes from a stable memory region.
                // The CHARSXP is protected (reachable from protected STRSXP).
                // R's GC is non-moving — address is stable.
                // CHAR data is immutable (interned strings never change).
            }
        });
    });

    Rf_unprotect(1);
}
```

**Why this is safe:**
- `CHAR` pointers are stable: R's GC is non-moving, and the STRSXP is protected
- String content is immutable: CHARSXPs are interned and shared, never modified
- No R API calls from threads: just `CStr::from_ptr` on pre-extracted `const char*`

---

## Decision Tree

```
Is the STRSXP ALTREP?
├─ Yes → DO NOT read from threads
│        Force materialization on main thread first: DATAPTR(x)
│        Then re-check: is it still ALTREP after materialization?
│        └─ The expanded data is cached, but the SEXP itself stays ALTREP.
│           Safest: extract CHAR pointers on main thread (safe case above).
│
└─ No (regular STRSXP)
   ├─ Option A: STRING_ELT + CHAR from thread (gray area)
   │  Technically C11 UB (sxpinfo bitfield race with GC mark bit).
   │  Practically safe on x86-64. Not guaranteed by the standard.
   │
   └─ Option B: Extract CHAR pointers on main thread (safe case)
      Zero R API from threads. Formally and practically safe.
      This is what miniextendr should do.
```
