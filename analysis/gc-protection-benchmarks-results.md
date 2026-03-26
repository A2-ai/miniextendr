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

## LIFO Release (Group 4) — median

| N | stack | precious | vec_pool | dll |
|---|-------|----------|----------|-----|
| 10 | **42 ns** | 107 ns | 134 ns | 278 ns |
| 100 | **393 ns** | **1.07 µs** | 1.03 µs | 2.46 µs |
| 1k | **3.90 µs** | **10.7 µs** | 9.79 µs | 23.7 µs |
| 10k | **38.9 µs** | — | 101 µs | 247 µs |

**Precious list is competitive with vec_pool in LIFO.** R_ReleaseObject scans from
head — LIFO means the target is always at the head (O(1) per release). This is the
ONE scenario where precious list performs well at moderate N.

## Bursty (Group 6) — median

| Rounds | vec_pool (burst=10k) | dll (burst=10k) | precious (burst=1k) |
|--------|---------------------|-----------------|---------------------|
| 3 | **298 µs** | 735 µs | 93 µs* |

*Precious uses burst=1k (10k would take minutes). Not directly comparable.

## GC Pressure with Work Allocations (Group 9) — median

| N | pool + work | dll + work |
|---|-------------|------------|
| 1k | **21.5 µs** | 32.9 µs |
| 10k | **229 µs** | 343 µs |

Pool is **1.5x faster** when interleaved with real work allocations. The DLL's
extra CONSXP per insert doubles the allocation rate, measurably increasing GC pressure.

## Memory Hold (Group 10) — median

| N | pool | dll | precious |
|---|------|-----|----------|
| 1k | **9.6 µs** | 24.9 µs | 637 µs |
| 10k | **97 µs** | 248 µs | 116 ms |
| 100k | **982 µs** | 2.55 ms | — |

Pool is 2.6x faster than DLL to hold N objects. DLL allocates N CONSXP cells (GC
pressure + cache pollution). Precious list at 10k: O(n²) release dominates.

## Hot-Get Loop (Group 16) — median, 10 reads per key

| N | slotmap | vec |
|---|---------|-----|
| 1k | 38.5 µs | **38.2 µs** |
| 10k | 387 µs | **385 µs** |

**Identical.** The generational check is invisible on reads (single u32 compare,
branch-predicted always-taken). No reason to use raw Vec for get performance.

## Vec vs VecDeque Free List — median

### Churn (insert 1, release 1, repeat N)

| N | vec | deque |
|---|-----|-------|
| 1k | 9.83 µs | **9.79 µs** |
| 10k | **97.9 µs** | 101 µs |
| 100k | **979 µs** | 979 µs |

### Burst (insert N, release half, reinsert half)

| N | vec | deque |
|---|-----|-------|
| 1k | **14.7 µs** | 15.0 µs |
| 10k | **151 µs** | 154 µs |

Identical on both patterns. Single-latency also identical (9.6ns both).

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

## R_HASH_PRECIOUS Mode (Group 13)

Run with `R_HASH_PRECIOUS=1` env var. Enables 1069-bucket hash table for the precious
list instead of singly-linked list. Checked via `getenv()` on first `R_PreserveObject`
call — no R recompilation needed.

### Comparison: default vs hash mode (median)

| Benchmark | Default | R_HASH_PRECIOUS | Speedup |
|-----------|---------|-----------------|---------|
| Single op | 13.1 ns | 17.0 ns | **0.8x (slower)** |
| Batch 100 | 6.17 µs | 1.88 µs | **3.3x** |
| Batch 1k | 568 µs | 19.7 µs | **29x** |
| Random 1k | 281 µs | 18.4 µs | **15x** |
| Churn 10k | 75.4 ms | 1.31 ms | **58x** |
| Replace 10k | 15.1 s | 20.9 ms | **723x** |

Hash mode eliminates the O(n²) catastrophe. Release becomes O(bucket_size) instead of O(n).

### Hash precious vs vec_pool (median)

| Benchmark | R_HASH_PRECIOUS | Vec pool | Pool advantage |
|-----------|-----------------|----------|----------------|
| Single op | 17.0 ns | **9.6 ns** | 1.8x |
| Batch 1k | 19.7 µs | **9.6 µs** | 2.1x |
| Churn 10k | 1.31 ms | **1.16 ms** | 1.1x |
| Replace 10k | 20.9 ms | **45.2 µs** | 462x |

Even with hash mode, the precious list is 1.1-2x slower than vec_pool on most
workloads. Replace-in-loop remains catastrophic (20ms vs 45µs) because each iteration
still allocates a CONSXP + does a hash lookup.

**Verdict**: `R_HASH_PRECIOUS` makes the precious list viable for moderate batch
operations, but the pool is still faster everywhere. And crucially, miniextendr
can't control whether the env var is set — it depends on how R was launched.
The pool's performance is consistent regardless of R session configuration.

## Real ProtectPool vs Prototypes (Group 18) — median

| Pool | Single op | Batch 10k |
|------|-----------|-----------|
| **real ProtectPool** (hand-rolled gen keys) | **10.1 ns** | 112 µs |
| prototype VecPool (no safety) | 10.3 ns | **96.9 µs** |
| prototype SlotmapPool (slotmap crate) | 11.4 ns | 116 µs |

Real pool matches VecPool on single-op (10.1 vs 10.3 ns). Removing the redundant
second free list (slotmap's internal + ours) worked — hand-rolled generational keys
have the same cost as raw `usize` indices with no safety.

Batch is 16% slower than VecPool (generation array `wrapping_add(1)` per release).
Still 2.3x faster than DLL (256 µs).

## Generation Check: Valid vs Stale Keys (Group 19)

| N | Valid keys | Stale keys |
|---|-----------|-----------|
| 1k | 3.94 µs | **463 ns** |
| 10k | 38.7 µs | **4.71 µs** |

Stale keys are **8x faster** — they bail at the u32 generation compare (pure Rust).
Valid keys proceed to `VECTOR_ELT` (R FFI call). The generation check itself is
invisible; the cost is in the FFI call on success.

## replace() vs release()+insert() (Group 20) — median

| Operation | 1k | 10k | Per-op (10k) |
|-----------|-----|------|------|
| **replace** | **4.79 µs** | **47.9 µs** | **4.8 ns** |
| release+insert | 10.8 µs | 108.5 µs | 10.9 ns |

Replace is **2.3x faster** — one SET_VECTOR_ELT vs full release+insert cycle
(SET_VECTOR_ELT(nil) + free list push + pop + SET_VECTOR_ELT(new) + generation bump).
Use `pool.replace(key, new_sexp)` instead of release+insert for in-place updates.

## Real Pool Growth (Group 21) — median

| N | Pre-sized | Small initial (cap=16) | Ratio |
|---|-----------|------------------------|-------|
| 1k | 12.4 µs | 22.1 µs | 1.8x |
| 10k | 121 µs | 251 µs | 2.1x |
| 50k | 578 µs | 1.10 ms | 1.9x |

Growth penalty is ~2x. Pre-sizing is worth it when count is known.
Even with growth, 50k from cap=16 takes only 1.1ms — still fast.

## Key Decisions Informed by Data

| Decision | Benchmark result | Verdict |
|----------|-----------------|---------|
| Single-op cost | stack 7.4ns, real pool 10.1ns, dll 28.9ns | **Real pool is 2.7ns slower than stack. DLL is 4x slower.** |
| Batch throughput | vec_pool 2.5x slower than stack, 2.6x faster than DLL | **Stack for temporaries, pool for cross-call** |
| Replace-in-loop | ReprotectSlot 3.8ns/op, pool 4.5ns/op, precious **15 seconds** at 10k | **ReprotectSlot or pool overwrite. Never precious list for loops.** |
| Precious list | 13ns single, LIFO release = pool speed, non-LIFO = O(n²), background OK for recent | **Safe for LIFO-released long-lived objects. Never for random/loop release.** |
| Hand-rolled vs slotmap | Real pool = VecPool speed (10.1ns), slotmap 11.4ns | **Hand-rolled gen keys: slotmap safety at VecPool speed** |
| Vec vs VecDeque | Identical | **Vec (simpler)** |
| DLL niche? | 4x slower single-op, 2.6x slower batch, 7x slower replace | **No niche. Pool beats it everywhere.** |
| Rf_unprotect_ptr | 15% overhead at all depths | **Usable when needed** |
| Pool growth | 10% slotmap overhead, both fast | **Not a concern** |
| Keyed pools | 6-7x slower than slotmap | **For named caches only** |
