# FAQ

Reference documents grounded in R source code (`background/r-svn/`).

## ALTREP

- [Which R operations produce ALTREP?](altrep/altrep-under-the-hood.md) — Every R operation that silently returns ALTREP: `1:n` → `compact_intseq`, `as.character(int)` → `deferred_string`, `sort()` → `wrap_*`, attribute modification → `wrap_*`, plus `compact_realseq` and `mmap_*`.
- [ALTREP string race demo](altrep/altrep-string-race-demo.md) — Concrete Rust code showing when concurrent string reads are safe (pre-extracted `CHAR` pointers), gray area (non-ALTREP `STRING_ELT` — C11 UB, x86-64 safe), and broken (deferred ALTREP `STRING_ELT` — races on `R_GCEnabled` and `R_StringHash`).

## Memory & Concurrency

R's single-threaded runtime and what it would take to change it, from trivial to impossible.

- [Why no threaded SEXP mutation](memory/why-no-threaded-sexp-mutation.md) — R has zero locks/atomics; every R API call touches unprotected globals.
- [What would concurrent read-only R take](memory/what-would-concurrent-read-only-r-take.md) — 3 changes: GC rwlock, ALTREP materialization, read-only API subset.

### Concurrency Levels Series

Eight distinct levels of concurrency that could theoretically be built into R, from the status quo through full shared-heap multi-threaded evaluation.

| Level | File | Effort | Summary |
|---|---|---|---|
| 0 | [Raw pointer handoff](memory/concurrency-level-0-raw-pointer-handoff.md) | Already done | Extract `double*` on main thread, use from workers |
| 1 | [GC-frozen read windows](memory/concurrency-level-1-gc-frozen-read-windows.md) | ~50 lines | `pthread_rwlock_t` around GC; full read-only SEXP traversal |
| 2 | [Thread-local allocation arenas](memory/concurrency-level-2-thread-local-allocation-arenas.md) | ~2-3K lines | Per-thread nurseries; string interning is the hard part |
| 3 | [Concurrent ALTREP](memory/concurrency-level-3-concurrent-altrep.md) | ~500 lines | Thread-safe ALTREP dispatch via method annotations |
| 4 | [Fork-COW snapshots](memory/concurrency-level-4-fork-cow-snapshots.md) | ~500-3K lines | Existing `mclapply` analyzed; GC-COW problem and fixes |
| 5 | [Isolated sub-interpreters](memory/concurrency-level-5-isolated-sub-interpreters.md) | ~15-30K lines | Multiple R runtimes in one process; multi-year project |
| 6 | [Concurrent GC](memory/concurrency-level-6-concurrent-gc.md) | ~5-15K lines | GC runs while mutators continue; fundamental rewrite |
| 7 | [Shared-heap evaluation](memory/concurrency-level-7-shared-heap-evaluation.md) | ~50-100K lines | True multi-threaded R eval; no dynamic language has done this |
