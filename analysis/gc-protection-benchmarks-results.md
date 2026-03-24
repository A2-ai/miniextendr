# GC Protection Benchmark Results (v6 — corrected)

Machine: Apple Silicon (aarch64), R embedded via miniextendr-engine, divan 0.1.21.
**All benchmarks pre-allocate SEXPs outside the timed region** — numbers reflect
pure protection overhead only, not R allocation cost.

## Single Latency (one protect + release)

| Mechanism | Median |
|-----------|--------|
| protect_stack | **7.4 ns** |
| vec_pool | **9.6 ns** |
| deque_pool | **9.6 ns** |
| slotmap_pool | 11.4 ns |
| precious_list | 13.1 ns |
| dll_preserve | 28.9 ns |

Protect stack is fastest (array write + integer subtract). Vec pool and deque pool
are within 2ns — virtually identical. Slotmap adds 2ns for the generational check.
DLL is **4x slower** than protect stack due to CONSXP allocation + doubly-linked splice.

## Batch Throughput (protect N, release all) — median

| N | stack | vec_pool | slotmap | dll | precious |
|---|-------|----------|---------|-----|----------|
| 10 | **38.8 ns** | 121 ns | 134 ns | 268 ns | 139 ns |
| 100 | **385 ns** | 942 ns | 1.11 µs | 2.12 µs | 6.17 µs |
| 1k | **3.81 µs** | 9.62 µs | 11.7 µs | 27.2 µs | 568 µs |
| 10k | **38.4 µs** | 97.1 µs | 116 µs | 256 µs | — |
| 50k | — | 486 µs | 575 µs | 1.31 ms | — |

Stack is 2.5x faster than vec_pool (bulk Rf_unprotect(n) vs N individual releases).
Vec pool is 2.6x faster than DLL. Precious list is catastrophic at 1k (60x slower
than vec_pool). Per-element cost: stack 3.8ns, vec_pool 9.7ns, slotmap 11.6ns, DLL 25.6ns.

## Churn (interleaved insert/release) — median

| N | vec_pool | dll | precious |
|---|----------|-----|----------|
| 1k | **20.8 µs** | 34.9 µs | 395 µs |
| 10k | **1.16 ms** | 1.24 ms | 75.4 ms |
| 100k | **146 ms** | 148 ms | — |

Vec pool is 1.7x faster than DLL at 1k. At 100k they converge (the Vec.remove(0)
in the churn pattern dominates both). Precious list: 65x slower at 10k.

## Random Release Order — median

| N | vec_pool | dll | precious |
|---|----------|-----|----------|
| 100 | **963 ns** | 2.40 µs | 3.40 µs |
| 1k | **9.87 µs** | 22.7 µs | 281 µs |
| 10k | **111 µs** | 264 µs | — |

Vec pool is **2.4x faster** than DLL on random release at all sizes. Pure protection
cost without allocation noise makes the DLL's CONSXP overhead clearly visible.

## Replace in Loop — median

| N | reprotect_slot | pool_overwrite | dll_reinsert | precious_churn |
|---|----------------|----------------|--------------|----------------|
| 100 | **354 ns** | 463 ns | 2.42 µs | 994 µs |
| 1k | **3.77 µs** | 4.50 µs | 23.9 µs | 119 ms |
| 10k | **37.6 µs** | 45.2 µs | 271 µs | **15.1 s** |

ReprotectSlot: 3.76 ns/op. Pool overwrite: 4.50 ns/op. DLL reinsert: 27.1 ns/op.
Precious list at 10k: **15 seconds** (O(n²) — each iteration scans growing list).

## Vec vs VecDeque Free List — median

| N | vec_churn | deque_churn |
|---|-----------|-------------|
| 1k | 9.79 µs | **9.17 µs** |
| 10k | 97.9 µs | **97.9 µs** |
| 100k | **982 µs** | 982 µs |

Identical. Single-latency also identical (9.6ns both). No measurable difference.

## Rf_unprotect_ptr at Depth — median

| Depth | unprotect_ptr | bulk unprotect | Ratio |
|-------|---------------|----------------|-------|
| 1 | 7.5 ns | 7.4 ns | 1.01x |
| 5 | 26.4 ns | 22.7 ns | 1.16x |
| 10 | 46.6 ns | 41.7 ns | 1.12x |
| 50 | 215 ns | 195 ns | 1.10x |
| 100 | 437 ns | 388 ns | 1.13x |
| 1000 | 4.37 µs | 3.81 µs | 1.15x |

~15% overhead at all depths. The scan+shift cost is modest and scales linearly.

## slotmap vs Vec Overhead — median

| N | slotmap | vec | Ratio |
|---|---------|-----|-------|
| 1k | 11.95 µs | **9.58 µs** | 1.25x |
| 10k | 120 µs | **96.0 µs** | 1.25x |

25% overhead (was 15% when allocation was included). The generational check + slotmap
bookkeeping is a real cost, but the safety (stale-key detection) is worth it.

## DLL Stack Interaction — median (100 DLL ops at various stack depths)

| Stack depth | DLL 100 ops | Pool 100 ops |
|-------------|-------------|--------------|
| 0 | 2.08 µs | 984 ns |
| 100 | 2.52 µs | 1.36 µs |
| 1,000 | 6.00 µs | 4.79 µs |
| 10,000 | 40.8 µs | 39.4 µs |

At depth 10k, DLL and pool are equal — the Rf_protect/Rf_unprotect for filling
the stack dominates. DLL's transient 2-slot usage is not a concern.

## Precious List with Background Pressure — median (100 cycles)

| Background N | 100 cycles | Per-op |
|---|---|---|
| 0 | 992 ns | 9.9 ns |
| 100 | 1.01 µs | 10.1 ns |
| 1,000 | 1.04 µs | 10.4 ns |
| 10,000 | 1.07 µs | 10.7 ns |

**Surprise: no degradation with background pressure.** The previous benchmark was
flawed (it included the cost of R_PreserveObject-ing the background objects in the
timed region). With proper isolation, R_ReleaseObject finds the target quickly because
it was prepended to the head of the list (LIFO behavior of singly-linked list).

This means the precious list is NOT session-global-affected for recently preserved
objects. It's only slow when releasing objects that were preserved long ago (deep in
the list). The O(n) scan starts from the head — recently preserved objects are found fast.

## Keyed Pools — churn (insert + get + release one at a time) — median

| Collection | 100 | 1k | 10k |
|---|---|---|---|
| slotmap (no key) | **1.41 µs** | **14.0 µs** | **140 µs** |
| hashmap | 8.17 µs | 98.6 µs | 1.02 ms |
| indexmap | 8.23 µs | 92.2 µs | 942 µs |
| btreemap | 8.17 µs | 87.7 µs | 906 µs |

Keyed pools are **6-7x slower** than slotmap due to string key allocation + hashing.
BTreeMap is slightly fastest among keyed pools. IndexMap and HashMap are close.

## Pool Growth (amortized from cap=16) — median

| N | vec_pool | slotmap |
|---|----------|---------|
| 1k | **19.2 µs** | 24.6 µs |
| 10k | **236 µs** | 277 µs |
| 100k | **2.06 ms** | 2.26 ms |

Growth adds ~10% overhead for slotmap vs vec. Both are dominated by the
SET_VECTOR_ELT calls, not the growth events.

## Key Decisions Informed by Data

| Decision | Benchmark result | Verdict |
|----------|-----------------|---------|
| Single-op cost | stack 7.4ns, vec_pool 9.6ns, dll 28.9ns | **Pool is 2.2ns slower than stack. DLL is 4x slower.** |
| Batch throughput | vec_pool 2.5x slower than stack, 2.6x faster than DLL | **Stack for temporaries, pool for cross-call** |
| Replace-in-loop | ReprotectSlot 3.8ns/op, pool 4.5ns/op, precious **15 seconds** at 10k | **ReprotectSlot or pool overwrite. Never precious list for loops.** |
| Precious list | Fine for single ops (13ns), catastrophic for loops (O(n²)), background-unaffected for recent objects | **Safe for few long-lived objects. Never for iteration.** |
| slotmap vs Vec | 25% overhead | **slotmap as default (safety), Vec as opt-in fast path** |
| Vec vs VecDeque | Identical | **Vec (simpler)** |
| DLL niche? | 4x slower single-op, 2.6x slower batch, 7x slower replace | **No niche. Pool beats it everywhere.** |
| Rf_unprotect_ptr | 15% overhead at all depths | **Usable when needed** |
| Pool growth | 10% slotmap overhead, both fast | **Not a concern** |
| Keyed pools | 6-7x slower than slotmap | **For named caches only** |
