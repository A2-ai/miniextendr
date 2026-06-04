# ExternalPtr GC rooting: R_PreserveObject vs ProtectPool (#836)

**Date:** 2026-06-03
**Area:** `miniextendr-api/src/externalptr.rs`, `protect_pool.rs`
**PR:** #841

## What was attempted

Issue #836: `ExternalPtr::new` returned an *unprotected* `EXTPTRSXP`, so a live
Rust handle left its SEXP unrooted — building a `Vec<ExternalPtr<T>>` collected
earlier handles on each element's allocation (40/40 gctorture failures). The
first fix rooted each owning handle with **`R_PreserveObject`** at construction
and **`R_ReleaseObject`** on drop, gated by an `owns: bool` flag.

It was correct (gctorture 40/40), compiled, and was committed + pushed.

## What went wrong

The mechanism choice was wrong on performance. `R_ReleaseObject` scans R's
precious list linearly (O(n)). A `Vec<ExternalPtr>` drops **front-to-back** —
oldest handle first, i.e. the entry *deepest* in R's LIFO precious list — so the
release degrades to **O(n²)** on exactly the workload #827 introduces.

The choice was justified by a doc comment in `protect_pool.rs`
(*"for a few long-lived objects ... like ExternalPtr, use `R_PreserveObject`
directly"*) rather than by the benchmark data already in the repo. That comment's
premise — *"never released in a loop"* — does not hold once `Vec<ExternalPtr>`
is a first-class conversion (#827): a `Vec` drop *is* a release loop.

## Root cause

Trusting a stale prose comment over measured data. The repo already contained a
decisive benchmark suite (`analysis/gc-protection-benchmarks-results.md`,
`miniextendr-bench/benches/gc_protection_compare.rs`) and a strategy analysis
(`analysis/gc-protection-strategies.md`) that explicitly names ExternalPtr as a
**pool** use case and says of the precious list: *"avoid (pool is strictly
better) ... O(n) release."*

## The data (fresh run, 2026-06-03, Apple Silicon — medians)

| Workload | N | `R_PreserveObject` | `ProtectPool` | Ratio |
|---|---|---|---|---|
| batch (protect N, release all) | 1000 | 620 µs | 9.1 µs | **68×** |
| churn (interleaved) | 10000 | 76.7 ms | 1.13 ms | **68×** |
| replace-in-loop | 1000 | 138 ms | — | O(n²) |

Superlinearity is explicit: precious-list batch 100→1000 grows 89× for 10× the
work; churn 1k→10k grows 183×. The pool scales linearly.

## Fix

Root owning handles in a process-wide **`ProtectPool`** (a single GC-traced
VECSXP with Rust-side slot bookkeeping; O(1) any-order release, zero R allocation
per insert). The handle carries an `Option<ProtectKey>` (`Some` = owning,
`None` = borrowed view). The pool lives in a `thread_local!` on R's main thread,
wrapped in `ManuallyDrop` (session-lifetime root table; never `R_ReleaseObject`'d
at teardown). Also corrected the misleading `protect_pool.rs` doc comment.

## Lesson

When a measured benchmark exists for a decision, read it before trusting prose —
especially a comment whose stated precondition a new feature may have invalidated.
"Measure before commit" applies to *mechanism choice*, not just micro-optimisations.

## Follow-up: a bulk builder, not an `unsafe new_unprotected`

The pool makes a held `Vec<ExternalPtr>` GC-safe, but for the common
*build-many-then-hand-to-R* case it still pays a per-element price: a
`ProtectPool` insert **and** release, a `with_r_thread` hop per element, and a
second copy pass to lay the handles into an R list. The tempting "fix" was an
`unsafe fn new_unprotected` that skips rooting for hot paths — but that re-arms
the exact #836 footgun (unrooted handles collected mid-build) and, per the
benches, would only recover the ~19 ns/elt pool cost.

Instead we added a **safe** `ExternalPtr::collect_into_r_list(items)`: it builds
each `EXTPTRSXP` straight into the *protected result list*, so the list roots
every element the instant `SET_VECTOR_ELT` stores it — no unprotected window, no
pool traffic, one `with_r_thread` hop for the whole batch, no copy pass.

Benched (`miniextendr-bench/benches/externalptr.rs`, medians, Apple Silicon)
producing the *same* artifact — a protected `VECSXP` of N external pointers:

| N | `collect_into_r_list` | naive pool-then-list | speedup |
|---|---|---|---|
| 100 | 8.67 µs | 18.74 µs | 2.16× |
| 1000 | 75.8 µs | 190.6 µs | 2.51× |
| 10000 | 833.6 µs | 1.886 ms | 2.26× |

The win (~2.3×) is far larger than the ~10% the pool insert/release alone would
predict, because the destination build also collapses N `with_r_thread` hops to
one and drops both the copy pass and the `Vec<ExternalPtr>` Drop traffic
(`vec_lifecycle` ≈ `vec_pool_then_list` confirms the copy pass itself is nearly
free). GC-safety verified: `gc_stress_externalptr_collect_list()` passes 40/40
under `gctorture(TRUE)`. **Lesson:** when a hot path tempts an `unsafe` opt-out,
check whether a *safe* restructuring (root via the destination) beats it — here
it was both safer and ~2.3× faster.
