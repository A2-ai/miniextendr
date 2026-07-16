# Efficient `get_region` Implementations

`get_region` is the bulk-read callback of the ALTREP protocol: R hands you a
half-open block `[start, start + len)` and a destination buffer, you fill the
buffer and return how many elements you wrote. For integer, real, logical, raw,
and complex ALTREP classes it is the difference between one FFI dispatch per
*element* and one per *block* when R scans your vector.

This guide covers when R actually calls `get_region`, what an implementor
controls (and cannot control), the bounds contract, and cache-friendly filling
patterns for lazy vectors. The core example is compile-tested as the doctest on
[`AltIntegerData::get_region`] in `miniextendr-api/src/altrep_data/traits.rs`.

String (`STRSXP`) and list (`VECSXP`) classes have no region path — R's ALTREP
API defines `Get_region` only for the five fixed-width families.

## When R calls `get_region`

R is strictly **pull-based**. Your class exposes callbacks; R invokes them when
its own C code needs data, synchronously, on the main R thread. The dispatch
chain for integers (real/logical/raw/complex are identical) lives in R's
`src/main/altrep.c`:

```c
R_xlen_t INTEGER_GET_REGION(SEXP sx, R_xlen_t i, R_xlen_t n, int *buf)
{
    const int *x = INTEGER_OR_NULL(sx);   /* Dataptr_or_null dispatch */
    if (x != NULL) {
        /* contiguous memory available: direct copy,
           the class Get_region method is NOT called */
    }
    else
        return ALTINTEGER_DISPATCH(Get_region, sx, i, n, buf);
}
```

Two consequences:

1. **`dataptr_or_null` short-circuits `get_region`.** Once your class reports a
   contiguous pointer (e.g. after miniextendr materializes into the `data2`
   cache), R copies from memory directly and your `get_region` is never called
   again for that object.
2. Only classes that stay unmaterialized (return `None` from
   `AltrepDataptr::dataptr_or_null`, the derive default) see region traffic.

### Who issues region requests

Most callers go through `ITERATE_BY_REGION` (R's `R_ext/Itermacros.h`), which
walks the vector in chunks through a **512-element stack buffer**
(`GET_REGION_BUFSIZE`). In R 4.5 sources the users include:

| R operation | Source file |
|---|---|
| `sum()`, `mean()`, `prod()`, `min()`, `max()`, `range()` | `src/main/summary.c` |
| `any()`, `all()` (logical) | `src/main/summary.c` |
| `anyNA()` | `src/main/coerce.c` |
| `duplicated()`, `unique()` | `src/main/unique.c` (reverse order for `fromLast = TRUE`) |
| `is.unsorted()` / sortedness scans before sorting | `src/main/sort.c` |
| printing and `format()` | `src/main/printvector.c`, `src/main/format.c` |

For the summary-group members, R only iterates when your class does not answer
the metadata methods first: an `AltIntegerData::sum()` override that returns
`Some(...)` means `sum(x)` never touches `get_region` at all.

Not every request is ≤ 512 elements. R also issues **whole-vector one-shot**
calls — its own compact-sequence `Duplicate` method does
`INTEGER_GET_REGION(x, 0, n, ...)` with `n` equal to the full length. Never
bake a chunk-size assumption into an implementation.

### What you control — and what you don't

You control exactly one thing: **how a requested block is filled**, and the
count you return. You do not control:

- *when* (or whether) `get_region` is called;
- the block size or alignment R picks;
- the traversal order (forward for most scans, backward for
  `duplicated(fromLast = TRUE)`);
- threading — calls are synchronous on the R main thread, like every ALTREP
  callback.

**There is no class-side prefetch.** An ALTREP class cannot schedule
`get_region` invocations, hint future ranges, or ask R to read ahead — the
protocol has no such hook, so "region prefetch" as a framework feature is a
non-idea (it was removed from the feature backlog for this reason; see
[#1345](https://github.com/A2-ai/miniextendr/issues/1345)). What you *can* do
is keep state inside your own type: when a callback does run, you may fill a
private cache beyond the requested range, complete your own outstanding I/O, or
reuse previously decoded chunks (see the streaming types below). That overlaps
*your* work — it never changes when R asks.

## What the framework already does

Every `#[derive(AltrepInteger)]` / `AltrepReal` / `AltrepLogical` / `AltrepRaw`
/ `AltrepComplex` class — field-based or `#[altrep(manual)]` — registers a
`Get_region` method unconditionally (`HAS_GET_REGION = true` in the generated
low-level impl). The bridge trampoline (`altrep_bridge.rs`) wraps R's raw
`(SEXP, i, n, *mut T)` into a `&mut [T]` of exactly `n` elements, applies the
class's guard mode **once per block**, and forwards to your data trait:

```text
R: ITERATE_BY_REGION            miniextendr
  Get_region(x, i, 512, buf) ──► t_int_get_region (guard, slice) ──► AltIntegerData::get_region(&self, start, len, buf)
```

The default `Alt*Data::get_region` is already a Rust-side `elt` loop (the
`fill_region` helper): one FFI crossing and one guard per block, with your
`elt(&self, i)` inlined inside. Compare that with a class that registers **no**
`Get_region` method at all — R's fallback loops `INTEGER_ELT(sx, k + i)`, a
full ALTREP dispatch *per element*. So miniextendr's floor is already the
"batched elt" pattern; overriding `get_region` is about making the fill itself
cheaper than `len` independent `elt` calls.

Override when the block has structure a per-element accessor recomputes:

- a per-block invariant you can hoist (base offset of an arithmetic sequence,
  file offset of a chunk);
- a contiguous sub-range you can `copy_from_slice`;
- compressed / chunked / remote storage where decoding once per block beats
  decoding once per element.

## The bounds contract

R guarantees `buf` holds at least `n` elements, and the trampoline hands your
data trait a `&mut [T]` of exactly that length. Your job:

1. Clamp the request: fill
   `n_filled = len.min(buf.len()).min(self.len().saturating_sub(start))`
   elements.
2. Return `n_filled` — the number of elements actually written. A `start` at or
   past the end returns `0`.
3. Never write past `buf[n_filled]`.

The generated low-level layer already rejects `start < 0 || len <= 0` before
your data trait runs, and the default implementation performs exactly the clamp
above — so if you override, replicate it. `fill_region` in
`miniextendr-api/src/altrep_data/core.rs` is the reference.

## A lazy type with a cache-friendly `get_region`

The canonical shape — hoist the per-block work, then fill sequentially. This
example is the doctest on [`AltIntegerData::get_region`], so it compiles and
runs against the current traits on every `cargo test`:

```rust
use miniextendr_api::altrep_data::{AltIntegerData, AltrepLen};

/// Lazy arithmetic sequence: element i is `start + i * step`.
struct LazySeq { start: i32, step: i32, len: usize }

impl AltrepLen for LazySeq {
    fn len(&self) -> usize { self.len }
}

impl AltIntegerData for LazySeq {
    fn elt(&self, i: usize) -> i32 {
        self.start + (i as i32) * self.step
    }

    /// Bulk fill: clamp once, hoist the block base out of the loop,
    /// then write `buf` sequentially (cache-friendly, no per-element
    /// bounds or dispatch overhead).
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        let n = len.min(buf.len()).min(self.len.saturating_sub(start));
        let base = self.start + (start as i32) * self.step;
        for (k, slot) in buf[..n].iter_mut().enumerate() {
            *slot = base + (k as i32) * self.step;
        }
        n
    }
}

let seq = LazySeq { start: 10, step: 2, len: 100 };
let mut buf = [0i32; 8];

// Interior block: fully filled.
assert_eq!(seq.get_region(5, 8, &mut buf), 8);
assert_eq!(buf, [20, 22, 24, 26, 28, 30, 32, 34]);

// Tail block: clamped to the vector length.
assert_eq!(seq.get_region(96, 8, &mut buf), 4);
assert_eq!(&buf[..4], &[202, 204, 206, 208]);

// Past the end: nothing written.
assert_eq!(seq.get_region(100, 8, &mut buf), 0);
```

For contiguous backing, skip the loop entirely — the built-in `[T; N]` impls
show the pattern:

```rust
fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
    let end = (start + len).min(N);
    let actual_len = end.saturating_sub(start);
    if actual_len > 0 {
        buf[..actual_len].copy_from_slice(&self[start..end]);
    }
    actual_len
}
```

## Chunk-cached streaming types — forward `get_region` in wrappers

`StreamingIntData` / `StreamingRealData`
(`miniextendr_api::altrep_data`) are lazy adaptors over a reader closure.
Their two access paths differ deliberately:

- `elt(i)` loads the containing chunk into a `BTreeMap` cache and reads from
  it — good for scattered single-element access;
- `get_region(start, len, buf)` **bypasses the cache** and drives the reader
  straight into R's buffer — no intermediate copy, no cache churn during a
  full scan.

That split only survives if newtype wrappers forward it. A wrapper that
implements only `elt` silently reverts `get_region` to the default elt-loop,
which re-walks the chunk cache once per element:

```rust
impl AltIntegerData for MyStreamingWrapper {
    fn elt(&self, i: usize) -> i32 {
        self.inner.elt(i)
    }

    // Without this forward, sum()/anyNA()/printing degrade to
    // per-element chunk-cache lookups.
    fn get_region(&self, start: usize, len: usize, buf: &mut [i32]) -> usize {
        self.inner.get_region(start, len, buf)
    }
}
```

The rpkg fixtures (`rpkg/src/rust/streaming_altrep_tests.rs`, and the sparse
iterator wrappers in `rpkg/src/rust/lib.rs`) forward this way. The same rule
applies to any delegation wrapper: forward every optional method whose inner
implementation is better than the default, or the derive quietly discards the
optimization.

Sparse iterators (`SparseIterIntData` / `SparseIterRealData`) inherit a
`get_region` tuned for their skip-ahead storage — see
[SPARSE_ITERATOR_ALTREP.md](SPARSE_ITERATOR_ALTREP.md).

## `elt` vs `get_region` vs `dataptr`

| | `elt` | `get_region` | `dataptr` |
|---|---|---|---|
| Granularity | one element | one block (≤ 512 via `ITERATE_BY_REGION`, up to full length) | whole vector |
| Cost per call | guard + trampoline + extract, per element | guard + trampoline + extract, amortized over the block | first call materializes (alloc + full fill), then raw pointer |
| Memory | none | caller's buffer only | full copy lives in `data2` until GC |
| Typical consumers | `x[i]`, scalar subscripting | `sum`/`mean`/`min`/`max`/`range`/`prod`, `any`/`all`, `anyNA`, `duplicated`/`unique`, sortedness scans, printing | arithmetic (`x + y`), `sort()`, `.C`/`.Fortran`, most vectorized math |

Fallback behavior around the table:

- **No contiguous pointer yet** (`dataptr_or_null` → `None`, the derive
  default): scans go through `get_region`; single-element access goes through
  `elt`.
- **Contiguous pointer available** (your `AltrepDataptr` returned `Some`, or a
  prior `dataptr` call materialized into `data2`): both paths short-circuit to
  direct memory; neither `elt` nor `get_region` is called.
- **Materialization is elt-driven.** When R demands `DATAPTR` and your type
  cannot provide a pointer, miniextendr materializes into `data2` by calling
  R's `*_ELT` per element — a fast `get_region` does not accelerate that path.
  If dense repeated access is the expected pattern, implement
  `AltrepDataptr` (see [ALTREP.md § Materialization and
  DATAPTR](ALTREP.md#materialization-and-dataptr)) instead of leaning on
  regions.
- **Metadata beats iteration.** `sum`/`min`/`max`/`no_na`/`is_sorted`
  overrides answer the consumer *without* touching element data; provide them
  when O(1) answers exist, and `get_region` handles the consumers that must
  actually see the values.

## Checklist

- [ ] Custom `get_region` clamps to `len.min(buf.len()).min(self.len() - start)`
      and returns the count written.
- [ ] No chunk-size assumptions (requests can be full-length).
- [ ] Per-block invariants hoisted out of the fill loop;
      `copy_from_slice` used for contiguous sub-ranges.
- [ ] Delegation wrappers forward `get_region` (and other non-default
      optional methods) to the inner type.
- [ ] O(1) metadata (`sum`, `min`, `max`, `no_na`, `is_sorted`) provided where
      possible so R skips iteration entirely.
- [ ] No prefetch fantasies: internal caching is fine, but R decides when your
      callbacks run.

## See also

- [ALTREP.md](ALTREP.md) — complete guide (optional methods, materialization)
- [ALTREP_QUICKREF.md](ALTREP_QUICKREF.md) — quick reference
- [SPARSE_ITERATOR_ALTREP.md](SPARSE_ITERATOR_ALTREP.md) — skip-ahead iterator storage
- [ALTREP_GUARDS.md](ALTREP_GUARDS.md) — guard modes applied per callback
- R sources: `src/main/altrep.c` (dispatch + defaults),
  `src/include/R_ext/Itermacros.h` (`ITERATE_BY_REGION`),
  `src/main/summary.c` (the heaviest consumer)

[`AltIntegerData::get_region`]: https://github.com/A2-ai/miniextendr/blob/main/miniextendr-api/src/altrep_data/traits.rs
