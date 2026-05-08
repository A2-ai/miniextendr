# 2026-05-08 — UAF audit under `gctorture`

## The bug

Across the framework, every code path that built a `Vec<SEXP>` or
`Vec<(N, SEXP)>` via repeated `IntoR::into_sexp()` calls left earlier
SEXPs unrooted while later ones allocated. The pattern is the same shape
as the columnar `Generic` buffer bug fixed in #307 — just spread across
more sites.

```rust
// the bad pattern (canonical):
List::from_raw_pairs(vec![
    ("k1", IntoR::into_sexp(self.a)),   // alloc 1, sits in vec! temp, unrooted
    ("k2", IntoR::into_sexp(self.b)),   // alloc 2 — alloc 1's SEXP can be GC'd
    ("k3", IntoR::into_sexp(self.c)),   // alloc 3 — allocs 1+2 stale
])
```

R's GC doesn't scan Rust stack/heap. When `gctorture(TRUE)` fires GC on
every `Rf_allocVector`, every prior SEXP sitting in a Rust temporary is
swept. Reads via `set_vector_elt(idx, freed_sexp)` are UAF.

Reproducer (pre-fix):

```r
library(miniextendr)
gctorture(TRUE)
for (i in 1:30) create_large_par_events(500L)
# *** recursive gc invocation
```

## The fix

Two complementary moves:

1. **API methods** (`miniextendr-api/src/list.rs`) — protect inputs and
   `self` internally, instead of relying on caller protection. Builds the
   parent first, writes children inline.

   ```rust
   // post-fix: List::from_pairs
   let list = OwnedProtect::new(Rf_allocVector(VECSXP, n));
   let names = OwnedProtect::new(Rf_allocVector(STRSXP, n));
   for (i, (name, val)) in pairs.into_iter().enumerate() {
       list.get().set_vector_elt(idx, val.into_sexp());
       names.get().set_string_elt(idx, SEXP::charsxp(name.as_ref()));
   }
   ```

   Same shape for `set_class_str`, `set_names_str`, `set_row_names_int`,
   `set_row_names_str`: open `OwnedProtect::new(self.0)` at entry so the
   parent list survives the internal `Rf_allocVector` for its attribute
   vector. (These were broken pre-fix too, but the column-level UAFs
   crashed first and masked them.)

2. **Codegen** (`miniextendr-macros/`) — wrap every emitted
   `into_sexp()` call in `__scope.protect_raw(...)` so previous SEXPs
   stay on R's protect stack across the next allocation.

   ```rust
   // post-fix: dataframe_derive emits this
   unsafe {
       let __scope = ProtectScope::new();
       List::from_raw_pairs(vec![
           ("k1", __scope.protect_raw(IntoR::into_sexp(self.a))),
           ("k2", __scope.protect_raw(IntoR::into_sexp(self.b))),
           ("k3", __scope.protect_raw(IntoR::into_sexp(self.c))),
       ])
       .set_class_str(&["data.frame"])
       .set_row_names_int(_n_rows)
   }
   ```

   `vec![]` evaluates left to right, so by the time the next
   `into_sexp()` allocates, the previous SEXP is on R's protect stack.
   The scope drops at function exit — by then `from_raw_pairs` has
   rooted the children via the parent VECSXP's write barrier.

Affected codegen sites (12):
`dataframe_derive` (struct), `dataframe_derive/enum_expansion`
(`IntoDataFrame` + `to_dataframe_split` + outer named-list assembly),
`list_macro` (named + unnamed), `list_derive` (named struct + tuple
struct), `vctrs_derive` (record `IntoVctrs`).

## Validation

| Pass | Coverage | Result |
|------|----------|--------|
| Per-function sweep (`gctorture(TRUE)`, no-arg fixtures) | 1248 / 1285 exports | 0 crashes |
| Targeted repro (`gctorture(TRUE)` × 30) on `create_large_par_events(500)`, `create_large_par_points(500)`, two new `from_values_strings` / `from_pairs_strings` fixtures | 4 fixtures × 30 iters = 120 runs | 0 crashes post-fix |
| Full testthat (`gctorture2(step=100)`, `devtools::load_all`) | 4999 tests / 1591 files / 5.9 h | 4999 passed, 0 failed, 0 errored, 0 segfaults |

The 37 untested exports in the per-function sweep are the documented
bare-panic `unsafe_C_*` family (intentionally `panic!()` without
`catch_unwind`; abort under gctorture by design — out of audit scope).

## Lessons

- **Sweep + static review must be paired.** No fixture exercised the
  blanket `IntoList for Vec<T>` impl — sweeping alone wouldn't have
  found it. A `Vec<SEXP>` grep across `miniextendr-api/` is now part of
  the audit recipe.
- **Pre-attach lazy-loaded helpers (`lifecycle`, `rlang`, `withr`,
  `cli`, `vctrs`, `jsonlite`) before flipping gctorture on.** Otherwise
  the first wrapper that calls `lifecycle::deprecate_warn()` triggers
  `loadNamespace` on a torture-amplified path. Add to
  `docs/GCTORTURE_TESTING.md`.
- **Fix-induced exposure.** The `set_class_str` / `set_row_names_int`
  parent-list UAF was older than the column-level UAFs but never
  observed because the column UAF crashed first. After fixing one UAF,
  re-run gctorture against the *same* fixture; expect a new crash.
- **`from_raw_pairs` / `from_raw_values` / `set_*` documenting "input
  must be protected" is a contract no caller honoured.** Defensive
  protect at function entry is cheap (one PROTECT per call) and avoids
  the trap entirely. Long-term: a typed wrapper `ProtectedSexp<'a>` only
  constructible inside a scope would make the contract impossible to
  forget.
- **`vec![ (k, into_sexp(v)), ... ]` is a UAF idiom.** Worth a lint
  (`MXL???`) so the pattern can't recur silently in new codegen.

## Follow-ups (not in this PR)

- Lint or refactor: catch `vec![ ... into_sexp(...) ... ]` at compile
  time.
- Make more rpkg fixtures exported so the per-function `gctorture(TRUE)`
  sweep covers them automatically next time. Currently 36 % of exports
  fall through as "argument required" because they aren't no-arg test
  fixtures.
- A nightly CI job invoking `gctorture2(step=100) + test_dir(...)` would
  catch this class of bug pre-merge. Tracked as TODO in
  `docs/GCTORTURE_TESTING.md`.

Full audit details, harness pitfalls, and per-pass logs live in
`reviews/2026-05-07-gctorture-audit.md`.
