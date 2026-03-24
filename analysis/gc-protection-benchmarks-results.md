# GC Protection Benchmark Results

Machine: Apple Silicon (aarch64), R embedded via miniextendr-engine, divan 0.1.21.

## Single Latency (one protect + release)

### Cold start (includes pool creation)

| Mechanism | Median |
|-----------|--------|
| protect_stack | **15.2 ns** |
| precious_list | 17.5 ns |
| dll_preserve | 36.5 ns |
| vec_pool_cold | 57.1 ns |
| slotmap_pool_cold | 93.5 ns |

### Steady state (1000 ops on existing pool, per-op)

| Mechanism | 1000 ops total | Per-op |
|-----------|---------------|--------|
| protect_stack | **14.3 µs** | **14.3 ns** |
| precious_list | 16.5 µs | 16.5 ns |
| vec_pool | 17.5 µs | **17.5 ns** |
| slotmap_pool | 21.1 µs | 21.1 ns |
| dll_preserve | 29.0 µs | 29.0 ns |

**Steady-state is the real comparison.** Pool creation is a one-time cost.
On an existing pool, vec_pool is only 3ns slower than the protect stack.
The DLL is 2x slower than vec_pool in steady state (29ns vs 17.5ns) due to
CONSXP allocation + doubly-linked splice on every insert.

## Batch Throughput (protect N, release all) — median

| N | stack | precious | DLL | vec_pool | slotmap |
|---|-------|----------|-----|----------|---------|
| 10 | **125 ns** | 234 ns | 427 ns | 292 ns | 333 ns |
| 100 | **1.2 µs** | 7.5 µs | 3.8 µs | 1.8 µs | 2.1 µs |
| 1k | **11 µs** | 630 µs | 36 µs | 17 µs | 19 µs |
| 10k | **102 µs** | **114 ms** | 329 µs | 170 µs | 202 µs |
| 50k | — | — | 1.7 ms | 893 µs | 1.0 ms |

**Precious list is catastrophic at scale.** 114ms at 10k — that's O(n²) release.
At 1k it's already 630µs vs 17µs for vec_pool (37x slower).

Stack is king for batch (single `Rf_unprotect(n)` call). Among cross-call mechanisms,
**vec_pool is consistently fastest**, ~2x faster than DLL at all sizes.

## Churn (interleaved insert/release) — median

| N | precious | DLL | vec_pool | slotmap |
|---|----------|-----|----------|---------|
| 1k | 438 µs | 42 µs | **31 µs** | 33 µs |
| 10k | **78 ms** | 1.35 ms | **1.31 ms** | 1.32 ms |
| 100k | — | 155 ms | **152 ms** | 154 ms |

Precious list is 56x slower than pools at 10k. DLL, vec_pool, and slotmap are
**virtually identical** under churn. The CONSXP allocation from the DLL doesn't
measurably hurt — the dominant cost is the R allocation of the test SEXP itself.

## Random Release Order — median

| N | precious | DLL | vec_pool | slotmap |
|---|----------|-----|----------|---------|
| 100 | 4.6 µs | 3.5 µs | **2.1 µs** | 2.5 µs |
| 1k | 324 µs | 34 µs | **20 µs** | 22 µs |
| 10k | **68 ms** | 368 µs | **220 µs** | 282 µs |

Same story. Precious list O(n²) dominates. Pools are ~1.5x faster than DLL.

## Replace in Loop — median

| N | reprotect_slot | pool_overwrite | precious_churn | dll_reinsert |
|---|----------------|----------------|----------------|--------------|
| 100 | **1.08 µs** | 1.22 µs | 1.72 µs | 3.5 µs |
| 1k | **10.6 µs** | 11.6 µs | 16.5 µs | 36.7 µs |
| 10k | **101 µs** | 115 µs | 168 µs | 361 µs |

ReprotectSlot is fastest (single R_Reprotect = array write). Pool overwrite is close
(single SET_VECTOR_ELT). Precious list churn (release+preserve each iteration) is
1.7x. DLL reinsert (release+insert each iteration, allocating a CONSXP) is **3.5x slower**
than ReprotectSlot.

## Data.frame Construction (realistic, N columns × 1000 rows) — median

| N cols | protect_scope | dll_preserve | vec_pool |
|--------|---------------|--------------|----------|
| 5 | 3.0 µs | 3.4 µs | 3.0 µs |
| 20 | **11.8 µs** | 13.2 µs | **9.4 µs** |
| 100 | **59.2 µs** | 62.4 µs | **44.7 µs** |

At 100 columns, vec_pool is 25% faster than protect_scope and 28% faster than DLL.
The pool avoids both CONSXP allocation (DLL) and protect stack pressure (scope).

## Vec vs VecDeque Free List — median

| N | vec_churn | deque_churn | vec_burst | deque_burst |
|---|-----------|-------------|-----------|-------------|
| 1k | **16.5 µs** | 17.2 µs | **28.2 µs** | 27.9 µs |
| 10k | **172 µs** | 168 µs | **280 µs** | 265 µs |
| 100k | **1.69 ms** | 1.74 ms | **3.21 ms** | 3.19 ms |

**Negligible difference.** Vec is marginally faster on churn (LIFO reuse = hot cache),
VecDeque is marginally faster on burst (FIFO spreads access). The difference is <5%.
Use Vec (simpler).

## Rf_unprotect_ptr at Depth — median

| Depth | unprotect_ptr | bulk unprotect |
|-------|---------------|----------------|
| 1 | 14.3 ns | 14.6 ns |
| 5 | 63.2 ns | 59.3 ns |
| 10 | 122 ns | 122 ns |
| 50 | 567 ns | 557 ns |
| 100 | 1.11 µs | 1.06 µs |
| 1000 | 10.8 µs | 10.4 µs |

**Essentially identical.** `Rf_unprotect_ptr` scan + shift is no worse than bulk
`Rf_unprotect` + the cost of the N protect calls. At depth 1000 it's only 4% slower.
The "avoid `Rf_unprotect_ptr`" advice in the analysis was too strong — it's fine for
any depth likely in practice.

## slotmap vs Vec Overhead — median

| N | slotmap insert+release | vec insert+release | slotmap get×10 | vec get×10 |
|---|------------------------|--------------------|----------------|------------|
| 1k | 20.2 µs | **17.5 µs** | 59.4 µs | **55.9 µs** |
| 10k | 203 µs | **173 µs** | 593 µs | **564 µs** |

slotmap is ~15% slower on insert/release and ~5% slower on get. The generational
check is measurable but modest. For the safety it provides (stale-key detection),
15% overhead is acceptable.

## Pool Growth — median

| Initial cap | vec spike | slotmap spike |
|-------------|-----------|---------------|
| 16 | 708 ns | 875 ns |
| 64 | 2.0 µs | 2.4 µs |
| 256 | 7.2 µs | 8.2 µs |
| 1024 | 27.6 µs | 33.2 µs |

Growth spike at 1024 elements: **28 µs**. Amortized from initial cap 16 to 100k:
3.0 ms total (30 ns/element). Growth is not a concern.

## Pre-sized vs Small Initial Pool — batch at 50k, median

| Pool | Pre-sized (cap=N) | Small initial (cap=16) | Growth penalty |
|------|-------------------|------------------------|----------------|
| vec_pool | 899 µs | 1.40 ms | **1.6x** |
| slotmap | 1.01 ms | 1.57 ms | **1.6x** |

Growth adds ~60% overhead at 50k elements. Still faster than DLL (1.77ms pre-sized).
For known sizes, pre-sizing is worth it. For unknown sizes, the growth penalty is
modest and predictable.

## Bursty (10k alloc, 9.9k release, keep 100) — median

| Rounds | DLL | vec_pool | slotmap |
|--------|-----|----------|---------|
| 3 | 996 µs | **607 µs** | 688 µs |
| 10 | 3.44 ms | **1.88 ms** | 2.13 ms |

Vec pool is 1.6x faster than DLL on bursty workloads. The DLL's "memory reclamation"
advantage (released cons cells become GC garbage) doesn't show as a speed advantage.

## JSON-like Named List Construction (N keys + values) — median

| N | protect_scope | vec_pool | deque_pool | dll_preserve |
|---|---------------|----------|------------|--------------|
| 10 | **677 ns** | 828 ns | 833 ns | 911 ns |
| 100 | **6.8 µs** | 7.6 µs | 7.4 µs | 8.5 µs |
| 1k | **68.5 µs** | 75.8 µs | 77.4 µs | 85.5 µs |

Protect scope wins for JSON construction — per-iteration protect/unprotect(1) is
cheaper than pool insert+release. The pool's advantage (any-order release) doesn't
help when each temporary is consumed immediately. DLL is 25% slower from CONSXP churn.

Deque pool shows no advantage over vec pool here — the small pool (cap=4) means
both reuse the same 1-2 slots anyway.

## High-Turnover (3 live temporaries, replaced every iteration) — median

| N iterations | ReprotectSlot | vec_pool overwrite | DLL reinsert |
|---|---|---|---|
| 1k | **29.5 µs** | 34.6 µs | 100.7 µs |
| 10k | **302 µs** | 348 µs | 1.00 ms |

ReprotectSlot is fastest (one R_Reprotect = array write per replacement).
Pool overwrite is close (one SET_VECTOR_ELT per replacement — 15% slower).
DLL must release+reinsert per replacement (allocates a CONSXP each time) — **3.3x slower**.

This pattern (fixed number of live temporaries, replaced frequently) strongly favors
mechanisms that can overwrite in place. The DLL's inability to do this is a real cost.

## Keyed Pools (HashMap, BTreeMap, IndexMap) — Group 15

### Insert + get + release N entries (median)

| Collection | 100 | 1k | 10k |
|---|---|---|---|
| slotmap (no key) | **2.5 µs** | **23.8 µs** | **239 µs** |
| hashmap | 14.7 µs | 148 µs | 1.59 ms |
| indexmap | 16.4 µs | 163 µs | 1.76 ms |
| btreemap | 24.4 µs | 315 µs | 3.37 ms |

### Churn (insert + get + release one at a time, median)

| Collection | 100 | 1k | 10k |
|---|---|---|---|
| btreemap | 8.7 µs | **96 µs** | **1.02 ms** |
| indexmap | 8.5 µs | 101 µs | 1.05 ms |
| hashmap | 9.5 µs | 110 µs | 1.13 ms |

Keyed pools are **6-14x slower** than slotmap due to string key allocation (`format!`)
and hashing/comparison overhead. BTreeMap wins on churn (single-element entry API is
efficient), HashMap wins on bulk operations (amortized hash table access).

The key overhead dominates — the VECSXP pool cost is negligible by comparison. Keyed
pools are appropriate for named caches (string keys are the point), not general protection.

## Key Decisions Informed by Data

| Decision | Benchmark result | Verdict |
|----------|-----------------|---------|
| ExternalPtr → precious list? | Precious list is fine at <100 objects, catastrophic at >1k | **Yes, but only for types with <~100 instances** |
| Allocator stays on DLL? | DLL is 1.5-2x slower than pool at all sizes | **Move to pool if allocator does >1k allocs** |
| Pool needed for churn? | DLL ≈ pool under churn (CONSXP cost invisible) | **Pool wins on batch/random, ties on churn** |
| slotmap vs raw Vec? | 15% overhead for generational safety | **slotmap as default, Vec as unsafe opt-in** |
| Four Protector impls? | Precious list collapses above ~100 objects; DLL ≈ pool on churn | **Three: stack, precious (small N), pool (large N). DLL is optional.** |
| DLL memory reclamation? | Pool is 1.6x faster on bursty, DLL shows no reclamation advantage | **Theoretical — not observable in practice** |
| Pool growth spikes? | 27µs at 1024 elements | **Not a concern** |
| Vec vs VecDeque? | <5% difference | **Vec (simpler)** |
| Rf_unprotect_ptr bad? | Same speed as bulk unprotect at all depths | **Not bad — usable when needed** |
