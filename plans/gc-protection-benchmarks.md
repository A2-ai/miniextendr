# Plan: Comprehensive GC Protection Benchmarks

Background: `analysis/gc-protection-strategies.md`, `plans/unified-gc-protection.md`

## Goal

Benchmark all four protection mechanisms (protect stack, precious list, DLL preserve,
VECSXP pool) head-to-head across realistic workload patterns so we have data — not
theory — for when each wins.

## Existing benchmarks

`miniextendr-bench/benches/gc_protect.rs` already covers:
- ProtectScope vs raw Rf_protect/Rf_unprotect (single and multiple)
- OwnedProtect
- ReprotectSlot vs repeated protect
- List/StrVec construction patterns
- Stack pressure at scale (1k–10k)

`miniextendr-bench/benches/preserve.rs` covers:
- DLL insert+release (checked and unchecked)
- DLL at scale (10k–500k)
- DLL arbitrary release order

`miniextendr-bench/benches/refcount_protect.rs` covers:
- Raw R_PreserveObject/R_ReleaseObject (single and multiple)
- RefCountedArena variants (HashMap, ThreadLocal)

## What's missing

The existing benchmarks don't compare mechanisms against each other in apples-to-apples
scenarios. Each bench file tests its own mechanism in isolation. We need:

1. **Same workload, all four mechanisms** — identical N, identical pattern, only the
   protection backend differs
2. **Scaling curves** — how does each mechanism degrade as N grows (10, 100, 1k, 10k, 100k)?
3. **Workload patterns** that match real usage, not just insert+release cycles
4. **GC pressure measurement** — not just wall time, but allocation count / memory overhead

## Benchmark file: `benches/gc_protection_compare.rs`

All benchmarks use `divan`. Each benchmark group runs the same workload across all
four mechanisms at multiple scales.

### Group 1: Single protect + release (latency)

The atomic operation. How fast is one protect + one release?

```
protect_stack_single          — Rf_protect + Rf_unprotect(1)
precious_list_single          — R_PreserveObject + R_ReleaseObject
dll_preserve_single           — preserve::insert + preserve::release
vecsxp_pool_single            — pool.insert + pool.release
```

This measures **per-operation latency** with no scaling effects. The precious list and
DLL both allocate a CONSXP here; the stack and pool don't. Expect stack ≈ pool < DLL ≈ precious.

### Group 2: Protect N, then release all (batch throughput)

Protect N objects, then release all. Measures throughput at scale.

```
args = [10, 100, 1_000, 10_000, 50_000, 100_000]

batch_protect_stack(n)        — N × Rf_protect, then Rf_unprotect(n)
batch_precious_list(n)        — N × R_PreserveObject, then N × R_ReleaseObject
batch_dll_preserve(n)         — N × preserve::insert, then N × preserve::release
batch_vecsxp_pool(n)          — N × pool.insert, then N × pool.release
```

This is where the precious list's O(n) release compounds: releasing N objects scans
the list N times → O(n²). The DLL and pool stay O(n). The stack is O(1) for the bulk
unprotect. Expect stack >> pool ≈ DLL >> precious at large N.

**Note**: protect stack is capped at 50k (or --max-ppsize). Benchmarks above 50k should
skip the stack and note this as a data point.

### Group 3: Interleaved insert/release (churn)

Protect and release in alternating patterns. Models a cache or allocator where
objects are continuously added and removed.

```
args = [1_000, 10_000, 100_000]

churn_protect_stack(n)        — not possible (LIFO only, can't release middle)
churn_precious_list(n)        — for i in 0..n: preserve; if i%3==0: release oldest
churn_dll_preserve(n)         — same pattern with DLL
churn_vecsxp_pool(n)          — same pattern with pool
```

At any moment, ~2/3 of objects are live. Release is arbitrary order. The precious list
must scan the entire live set for each release. DLL and pool are O(1). The stack can't
participate (LIFO constraint). This is the allocator/cache workload.

### Group 4: Protect N, release in reverse (LIFO)

The protect stack's ideal workload.

```
args = [10, 100, 1_000, 10_000]

lifo_protect_stack(n)         — N × protect, then Rf_unprotect(n)
lifo_precious_list(n)         — N × preserve, then release in reverse
lifo_dll_preserve(n)          — same with DLL
lifo_vecsxp_pool(n)           — same with pool
```

LIFO favors the stack (single Rf_unprotect(n) call). DLL release in reverse order
means each release is at the top of the list → fast. Pool is indifferent to order.
Precious list scans from head → reverse order means each release scans the whole list.

### Group 5: Protect N, release in random order

Worst case for the precious list, neutral for DLL and pool.

```
args = [100, 1_000, 10_000]

random_precious_list(n)       — protect N, shuffle release order
random_dll_preserve(n)        — same with DLL
random_vecsxp_pool(n)         — same with pool
```

Use a fixed seed for reproducibility. The stack can't participate.

### Group 6: Bursty workload (allocate many, release most)

Model: protect 10k objects, release 9,900, keep 100 alive. Then do it again.
Tests memory behavior — DLL and precious list reclaim via GC, pool holds peak VECSXP.

```
args = [3, 10]  (number of burst rounds)

burst_dll_preserve(rounds)
burst_vecsxp_pool(rounds)
burst_precious_list(rounds)
```

Each round: insert 10k, release 9.9k, keep 100. After all rounds, only
100 × rounds objects are live, but pool VECSXP is sized for 10k.

### Group 7: Replace-in-loop (ReprotectSlot vs alternatives)

Model: one slot, replaced N times (accumulator pattern).

```
args = [100, 1_000, 10_000]

replace_reprotect_slot(n)     — R_ProtectWithIndex + N × R_Reprotect
replace_dll_reinsert(n)       — release old cell, insert new (2 ops per iteration)
replace_pool_overwrite(n)     — SET_VECTOR_ELT to overwrite slot (1 op)
replace_precious_list(n)      — R_ReleaseObject + R_PreserveObject (degrades with other preserved)
```

ReprotectSlot is the cheapest (single array write). Pool overwrite is close (single
SET_VECTOR_ELT). DLL must splice out + splice in. Precious list must scan + alloc.

### Group 8: Realistic composite — data.frame construction

Build a data.frame with N columns, each column is a 1000-element integer vector.
This is the actual workload that motivated the analysis.

```
args = [5, 20, 100, 500]

dataframe_protect_scope(ncols)  — ProtectScope for all intermediates
dataframe_dll_preserve(ncols)   — DLL for column protection
dataframe_pool(ncols)           — pool for column protection
```

Each needs: 1 list alloc + N column allocs + 1 names alloc + N CHARSXP allocs.
Total protections: ~2N + 2. Stack should win (zero alloc), pool close behind,
DLL paying CONSXP per column.

### Group 9: GC pressure measurement

Not wall time — count R allocations triggered by protection itself.

```
gc_pressure_dll(n)            — insert N, count GC triggers
gc_pressure_pool(n)           — insert N, count GC triggers
gc_pressure_precious(n)       — insert N, count GC triggers
```

Use `Rf_allocVector(RAWSXP, 0)` in a loop between inserts to probe GC frequency.
Or use `gc()` counts before/after. The DLL and precious list create one CONSXP per
insert; the pool creates zero. This quantifies how much the CONSXP allocation
actually affects GC frequency in practice.

### Group 10: Memory overhead

Measure RSS or R heap size after protecting N objects.

```
args = [1_000, 10_000, 100_000]

memory_dll(n)                 — insert N, measure memory
memory_pool(n)                — insert N, measure memory
memory_precious(n)            — insert N, measure memory
```

DLL: N cons cells (56 bytes each) + N × 8 bytes Rust-side = ~64N bytes
Pool: 1 VECSXP (8N bytes) + slotmap (~12N bytes Rust-side) = ~20N bytes
Precious: N cons cells (56 bytes each) + 0 Rust-side = ~56N bytes

The pool should use ~3x less memory than the DLL at scale.

### Group 11: Pool free-list strategy — Vec vs VecDeque

The pool's internal free list determines which VECSXP slot is reused when a new
object is inserted after a release. Vec (stack/LIFO) reuses the most recently
freed slot; VecDeque (queue/FIFO) reuses the oldest freed slot.

This affects:
- **Cache behavior**: LIFO reuses hot slots (recently written); FIFO spreads access
  across the VECSXP, potentially causing more cache misses
- **GC interaction**: FIFO delays reuse of a slot, giving R's GC more time to process
  the old SEXP before the slot is overwritten. LIFO overwrites immediately.
- **Fragmentation**: FIFO fills holes left-to-right; LIFO clusters at the high end

```
args = [1_000, 10_000, 100_000]

pool_freelist_vec_churn(n)      — pool with Vec<usize> free list, churn pattern
pool_freelist_vecdeque_churn(n) — pool with VecDeque<usize> free list, same pattern
pool_freelist_vec_burst(n)      — Vec free list, burst alloc/release
pool_freelist_vecdeque_burst(n) — VecDeque free list, same burst
```

Churn pattern: insert 1, release 1, repeat N times (maximum free-list traffic).
Burst pattern: insert N, release N/2 (oldest half), insert N/2, release all.

Also measure slot reuse distribution — with Vec, track which slot indices are reused
to see if LIFO clustering is real.

### Group 12: `Rf_unprotect_ptr` — cost of non-LIFO stack removal

The analysis says "avoid" but we should prove it with data. How expensive is
`Rf_unprotect_ptr` compared to `Rf_unprotect(1)` when the item is at various
depths in the stack?

```
args = [1, 5, 10, 50, 100, 1000]

unprotect_ptr_at_depth(depth)  — protect `depth` items, unprotect_ptr the first one
unprotect_lifo_baseline(depth) — protect `depth` items, Rf_unprotect(depth)
```

The first bench protects `depth` items, then calls `Rf_unprotect_ptr` on the one
at the bottom (worst case: scan the entire stack + shift all elements). Compare
against bulk `Rf_unprotect(depth)` to quantify the penalty.

### Group 13: `R_HASH_PRECIOUS` mode

R's precious list has an optional hash table mode (1069 buckets) enabled via the
`R_HASH_PRECIOUS` environment variable. The analysis documents it but the benchmarks
don't test it.

```
args = [100, 1_000, 10_000]

precious_default_release(n)     — R_PreserveObject N, R_ReleaseObject N (singly-linked)
precious_hash_release(n)        — same but with R_HASH_PRECIOUS=1 set before init
```

This requires setting the env var before R initializes, so it may need a separate
bench binary or `env::set_var` before `miniextendr_bench::init()`. The question:
does the hash mode close the gap between precious list and DLL/pool at moderate N?

### Group 14: DLL insert's protect stack interaction

`preserve::insert` temporarily uses 2 protect stack slots (`Rf_protect`/`Rf_unprotect(2)`)
during each insertion. Under high-frequency churn, this creates transient stack pressure.
Benchmark whether this is measurable.

```
args = [10_000, 50_000]

dll_churn_stack_pressure(n)     — alternating insert/release, measure protect stack high-water
pool_churn_stack_pressure(n)    — same with pool (uses zero stack slots)
```

The DLL touches the stack on every insert even though it's a "cross-call" mechanism.
The pool never touches the protect stack. If the stack is near capacity from deep R
call nesting, the DLL's transient usage could overflow it.

### Group 15: Keyed pool collections (HashMap, BTreeMap, IndexMap)

The analysis proposes keyed pools but the other groups only test unkeyed (Vec/slotmap).
Benchmark the overhead of key management.

```
args = [100, 1_000, 10_000]

keyed_hashmap_insert_release(n)   — HashMap<String, usize> keyed pool
keyed_btreemap_insert_release(n)  — BTreeMap<String, usize> keyed pool
keyed_indexmap_insert_release(n)  — IndexMap<String, usize> keyed pool
keyed_slotmap_baseline(n)         — plain slotmap (no key overhead, for comparison)
```

Keys are short strings (`format!("key_{i}")`). Measures the overhead of hashing,
tree insertion, and string allocation on top of the base pool cost.

### Group 16: slotmap generational check overhead

Does the generation check in slotmap measurably slow down `get`/`remove` compared
to raw Vec index access?

```
args = [10_000, 100_000]

slotmap_get_hot(n)          — insert N, then get each 10 times (hot path)
vec_index_get_hot(n)        — same with raw Vec<SEXP> + usize index
slotmap_remove_all(n)       — insert N, remove all
vec_remove_all(n)           — same with Vec free list
```

If the generation check is in the noise (likely — it's a single u32 compare), this
validates slotmap as the default with no performance excuse to use raw indices.

### Group 17: Pool VECSXP growth cost

Measure the actual latency spike when the pool's VECSXP must grow (allocate new,
copy all elements, release old).

```
pool_growth_spike(initial_cap)  — start with small pool, insert until growth triggers
  args = [16, 64, 256, 1024]

pool_growth_amortized(n)        — insert N with small initial cap, measure total/N
  args = [1_000, 10_000, 100_000]
```

The first bench triggers one growth event and measures how long it takes. The second
measures whether amortized cost is smooth in practice or has visible jitter.

## Implementation

1. Add `benches/gc_protection_compare.rs` to miniextendr-bench
2. Implement ProtectPool (needed for pool benchmarks) — this can be a
   minimal in-bench implementation first, then moved to miniextendr-api.
   Implement both Vec and VecDeque variants for Group 11.
3. Add `[[bench]] name = "gc_protection_compare"` to Cargo.toml
4. Run: `cargo bench -p miniextendr-bench --bench gc_protection_compare`
5. Capture results, add to `analysis/gc-protection-benchmarks-results.md`
6. Use results to validate or revise the decision tree in the plan

## What the data will tell us

- **Is the CONSXP allocation from DLL/precious actually measurable?** If not, the
  pool's theoretical advantage may not matter in practice.
- **At what N does precious list O(n²) release become unacceptable?** This determines
  where to draw the "few" vs "many" line in the decision tree.
- **Does pool growth cause measurable latency spikes?** If the amortized cost is
  smooth in practice, the DLL's "no growth spikes" advantage may be theoretical.
- **Is the DLL's memory reclamation (via GC) actually observable?** If R doesn't GC
  frequently enough, the "released cells become garbage" advantage is theoretical.
- **How much does ReprotectSlot actually save over DLL reinsert?** Quantifies whether
  replace-in-loop is worth special-casing.
- **Does Vec vs VecDeque free-list strategy matter?** If the difference is negligible,
  use Vec (simpler). If FIFO helps GC or cache behavior measurably, use VecDeque.
- **How bad is `Rf_unprotect_ptr` really?** If it's fast for small depths (top few items),
  it might be usable in limited contexts. If it's bad even at depth 5, the analysis is right.
- **Does `R_HASH_PRECIOUS` close the gap?** If hash mode makes precious list release O(1)-ish,
  the argument for DLL/pool weakens for moderate N.
- **Does the DLL's protect stack usage matter?** If the transient 2-slot usage during insert
  is invisible, it's fine. If measurable under deep R call nesting, it's a real concern.
- **Is slotmap's generational check free?** If the u32 compare is in the noise, slotmap is
  the right default. If measurable, offer raw Vec as an unsafe fast path.
- **How bad are pool growth spikes?** If a single growth event is <1ms even at 100k elements,
  the DLL's "no growth spikes" advantage is theoretical.
