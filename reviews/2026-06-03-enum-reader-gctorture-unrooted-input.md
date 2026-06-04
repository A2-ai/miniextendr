# No-arg reader gc_stress fixtures left the input frame unrooted

**Context:** #807 enum `FromDataFrame` reader (PR `feat/from-dataframe-enum-readers`).
While adding the map-column reader I ran the no-arg reader gc_stress fixtures under
`gctorture(TRUE)` and they corrupted the heap.

## What was attempted

```r
library(miniextendr)
gctorture(TRUE)
for (i in 1:10) gc_stress_reader_enum_map()    # and _flatten / _factor
```

## What went wrong

All three reader fixtures (`gc_stress_reader_enum_map`, `_flatten`, `_factor`)
failed under repeated gctorture with type-confusion / out-of-bounds symptoms:

- `column \`id\` could not be converted: expected INTSXP, got REALSXP`
- `map column \`tally\` has 0 keys but 4 values`
- `attempt access index 3/2 in STRING_ELT`

The errors were **non-deterministic across run count**: a *single* call passed, so
the original fixtures (added in the same PR for the flatten/factor shapes) looked
fine in the testthat suite — which calls them **without** `gctorture`. The nightly
no-arg gctorture sweep would have caught them.

## Root cause

The fixtures did:

```rust
let df_sexp = rows.into_dataframe().unwrap().into_sexp(); // unrooted R SEXP
let frame = DataFrame::from_sexp(df_sexp).unwrap();
let back = <Vec<E>>::from_dataframe(&frame).unwrap();
```

A Rust binding (`df_sexp`) does **not** root an R `SEXP`. Under `gctorture(TRUE)`
the next allocation reclaimed the writer-produced frame mid-read (the
flatten/nested-enum reader allocates sub-frames via `select`/`strip_prefix`/
`select_rows`; the factor reader allocates while validating levels). The comment
`// Hold df_sexp live across the read-back` was the misconception — the binding
holds the *pointer* live for Rust, not the *object* live for R's GC.

In real usage the reader's input is an R-rooted call argument (`#[miniextendr] fn
f(df: SEXP)` — R protects `df` for the call), so the reader does **not** root its
own input; that is the caller's job. The no-arg fixture *is* the caller and must
stand in for that root.

## Fix

Root the writer-produced frame for the duration of the read-back:

```rust
let df_sexp = rows.into_dataframe().unwrap().into_sexp();
let _df_guard = unsafe { miniextendr_api::OwnedProtect::new(df_sexp) };
let frame = DataFrame::from_sexp(df_sexp).unwrap();
let back = <Vec<E>>::from_dataframe(&frame).unwrap();
```

After the fix, `map`/`flatten`/`factor` each pass ×10 and interleaved ×5 under
`gctorture(TRUE)`.

## Lesson

Any no-arg gc_stress fixture that builds an R object in Rust and then reads it back
must `OwnedProtect`/`ProtectScope` that object — otherwise the fixture tests a
scenario that can't happen in production (an unrooted input) and reports a false
GC failure (or, worse, passes by luck at low iteration counts and the real bug in
the *reader* — if there were one — stays masked). The reader codegen itself is GC-
correct: it roots every sub-frame it allocates and only assumes its *input* is
already rooted, which is the documented contract.
