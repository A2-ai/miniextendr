# 2026-05-21 — `column_to_sexp::Character` STRSXP protect bug

## What was attempted

Land #672 (`NamedDataFrameListBuilder`) and #674 (streaming `iter_to_dataframe` / `DataFrameBuilder`). Both ship no-arg gctorture fixtures per the SEXP-storage-across-allocations convention:

- `gc_stress_named_df_list_builder` — two `vec_to_dataframe(&[…])` results pushed through the builder, including an `ErrRow { id: i32, msg: String }` struct.
- `gc_stress_iter_to_dataframe` — iterator over a `StreamRow { id, value, label: String, flag }` struct fed through `iter_to_dataframe`.

## What went wrong

CI's `R tests / Linux` step on both PRs failed at the new gctorture test with:

```
SET_STRING_ELT() can only be applied to a 'character vector', not a 'special'
SET_STRING_ELT() can only be applied to a 'character vector', not a 'weakref'
SET_STRING_ELT() can only be applied to a 'character vector', not a 'environment'
SET_STRING_ELT() can only be applied to a 'character vector', not a 'unknown type #29'
```

Same root cause; the slot type varies with what R recycled the SEXPREC for.

## Root cause

`miniextendr-api/src/serde/columnar.rs::column_to_sexp` (Character arm):

```rust
ColumnBuffer::Character(v) => {
    let sexp = Rf_allocVector(SEXPTYPE::STRSXP, nrow_r);
    for (i, val) in v.iter().enumerate() {
        sexp.set_string_elt(idx, SEXP::charsxp(s));  // ← charsxp allocates
    }
    sexp
}
```

`SEXP::charsxp` is `Rf_mkCharLenCE`, which is an allocating R API call. Under `gctorture(TRUE)` every R allocation triggers a full GC. The freshly-allocated STRSXP `sexp` is not on the protect stack (`Rf_allocVector` returns unprotected; the outer `Rf_protect(col_sexp)` in `assemble_dataframe` happens only *after* `column_to_sexp` returns). So the GC reclaims the STRSXP mid-loop, R recycles the SEXPREC slot for another type, and the next `SET_STRING_ELT` fails type-checking against the recycled slot's `TYPEOF`.

This was a latent bug — pre-existing since `vec_to_dataframe` shipped. None of the prior gctorture fixtures exercised the path: every existing `gc_stress_dataframe_*` uses the `#[derive(DataFrameRow)]` codegen, which goes through `convert.rs::IntoDataFrame`, not the serde `vec_to_dataframe`. The first fixtures to drive `vec_to_dataframe` with a String column were the two added in #672 and #674.

Once the bug was identified, a stripped-down repro (`gc_stress_dataframe_map` in current main, which uses the derive path) confirmed it doesn't hit the bug — locking the diagnosis on the serde path specifically.

## Fix

Six-line PROTECT/UNPROTECT pair around the loop:

```rust
let sexp = Rf_allocVector(SEXPTYPE::STRSXP, nrow_r);
Rf_protect(sexp);
for (i, val) in v.iter().enumerate() { … }
Rf_unprotect(1);
sexp
```

Carried as an amend on both PR #672 (97d20102) and PR #696 (f608befb). After whichever merges first, the other rebases and the duplicate fix collapses.

## Why this took longer than it should have

1. **R-thread panic propagation made bisecting hard.** `with_r_thread`'s panic-via-`resume_unwind` machinery + Rust's test-framework output capture meant `cargo test --nocapture` never printed the panic message for the failing test. Direct invocation of the test binary still suppressed output, even with `RUST_BACKTRACE=1`. Workaround: run the same code path from `Rscript` outside the test framework — the R error message surfaced immediately.

2. **`eprintln!` debug prints went nowhere.** Inside the `#[miniextendr]` function the worker thread's stderr is captured/redirected by R's runtime; debug prints from inside `from_raw_pairs` didn't reach Rscript's stderr. Switched to `assert_eq!` checks that surface as Rust panics — those *do* propagate through the tagged-condition transport back to R, but only when they actually trigger. The bug evaded the assertions because the STRSXP type was correct *at the start* of the loop and only got recycled later, before the second iteration's `SET_STRING_ELT` check.

3. **The error message shape ("not a 'special'", "not a 'weakref'", "not a 'environment'") was misleading.** It suggested the original SEXP was somehow being replaced with a different value, not that the slot was being recycled. Recognising "slot recycle" as the underlying mechanism took a hint from CLAUDE.md's existing PROTECT-discipline note about R-devel's aggressive GC.

## Lessons

- **Every new gctorture fixture is a chance to surface latent GC bugs.** Plan for it. When a new fixture fails, default-suspect "this fixture exercises a path the old fixtures didn't" before "the new fixture is wrong." Bisect by stripping fields, particularly `String` and `Vec<…>` fields that hit allocating R API.
- **The `column_to_sexp` Character path is now the documented landmark** for PROTECT discipline in this file. Future contributors editing this file should look at the new pattern as the reference: alloc → protect → loop with allocating R calls → unprotect → return.
- **Direct `Rscript`-driven repro beats `cargo test` when the panic message gets swallowed.** When the test framework is hiding the error, drop down a level — load the library directly, run the failing call, read the conditionMessage.
- **`eprintln!` inside `#[miniextendr]` functions is unreliable.** Worker thread stderr is captured. For debug printing, use `assert_eq!` with informative messages (they surface as R errors via the tagged-condition transport) or write to a side file with `std::fs`.
