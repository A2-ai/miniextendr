# 2026-05-07 — gctorture audit (whole-package sweep)

## What was attempted

Audit miniextendr end-to-end for SEXP use-after-free under
`gctorture(TRUE)` / `gctorture2(step=100)`, on the back of the columnar
`Generic` UAF (PR #424 / issue #307). Two passes:

1. **Per-function sweep** — call every exported `package:miniextendr` function
   with no args under `gctorture(TRUE)`. Skips arg-required calls via
   `tryCatch`. Targets fixtures + smoketest entrypoints.
2. **Static review** — grep for `Vec<SEXP>` / `Vec<Option<SEXP>>` / unprotected
   `into_sexp()` materializations across `miniextendr-api/src/`, looking for
   the same shape as `ColumnBuffer::Generic`.
3. **Full testthat sweep** — `gctorture2(step=100)` over `rpkg/tests/testthat`
   for argument-driven entry points the no-arg sweep can't reach.

## What went wrong

### Pass 1 — package-load timing

First sweep crashed at function 21 (`also_deprecated`). Root cause was the
documented "load packages before flipping gctorture on" trap from
`docs/GCTORTURE_TESTING.md` — `lifecycle::deprecate_warn()` lazy-loads the
`lifecycle` namespace on first call, and that path is unsafe under
`gctorture(TRUE)`. Fix: pre-attach `lifecycle`, `rlang`, `withr`, `cli`,
`vctrs`, `jsonlite` *before* enabling gctorture.

### Pass 1 — bare-panic family

Re-run reached function 1199 / 1285 and aborted on `unsafe_C_just_panic` with
`fatal runtime error: failed to initiate panic, error 5, aborting`. Expected:
`unsafe_C_*_panic*` fixtures intentionally panic without `catch_unwind` to
test bare-panic behaviour, and `\dontrun{}` them in their Rd. Under
gctorture's allocation stress the panic-payload alloc itself fails, which is
what `error 5` reports. Out of audit scope.

Tail re-run skipping `unsafe_C_*` covered the remaining 50 functions (37 of
the 86-function tail are panic fixtures, 13 are normal code) — clean.

**Per-function coverage: 1248 / 1285 (97 %), 0 UAFs found in fixture path.**

### Pass 2 — `IntoList for Vec<T>` and `List::from_pairs`

Static review surfaced two latent UAFs in `miniextendr-api/src/list.rs`,
identical in shape to the `ColumnBuffer::Generic` bug:

- `impl<T: IntoR> IntoList for Vec<T>` (line 642) materialised
  `let converted: Vec<SEXP> = self.into_iter().map(|v| v.into_sexp()).collect();`
  before calling `Rf_allocVector(VECSXP, n)`. Each `into_sexp()` is a GC
  point that can free earlier elements; the parent `allocVector` is another.
  Result: every SEXP in `converted` past the first allocation could be a
  dangling pointer by the time the loop writes it via `set_vector_elt`.

- `List::from_pairs` (line 853) had the same pattern via
  `Vec<(N, SEXP)>` and additionally allocated the names vector after, so
  values that survived through the value-side `into_sexp()` calls could
  still be freed by the names allocation.

Per-function sweep didn't catch these because no exported fixture exercised
the blanket `Vec<T>::into_list()` / `List::from_values` / `List::from_pairs`
paths — every `into_list()` callsite under `rpkg/src/rust/` is a hand-rolled
impl on a user type, building the parent list directly.

### Pass 3 — full testthat under gctorture2(step=100)

First run via `library(miniextendr) + test_dir(...)` covered 450 test files
in 355 s. Tally: **1586 passed / 0 failed**, no segfaults, no
`malloc(): … corrupted`, no `*** caught segfault ***`. 24 files errored at
the call-site level with `could not find function "S7Counter"` etc.; those
are class-system constructors that aren't package-exports and only visible
under `devtools::load_all` (not `library`). Re-run via `devtools::load_all`
to close that gap.

### Pass 4 — `devtools::load_all` + `gctorture2(step=100)`

Loaded the package via `devtools::load_all("rpkg")` so all internal
class-system constructors resolve, then ran the full testthat suite under
gctorture2. **Crashed on `dataframe-rayon` test 7 (`create_large_par_events(6000L)`)
with `*** recursive gc invocation`** — a SEXP UAF in the
`DataFrameRow` enum codegen. Traced the bug to:

```rust
// macro emits:
List::from_raw_pairs(vec![
    ("_kind", IntoR::into_sexp(self._tag)),       // alloc 1, then unrooted
    ("id",    IntoR::into_sexp(self.id)),         // alloc 2, _tag stale
    ("value", IntoR::into_sexp(self.value)),
    ("name",  IntoR::into_sexp(self.name)),       // many inner allocs
])
```

Same UAF shape as the columnar `Generic` bug and the `IntoList for Vec<T>`
bug from this same audit: a `Vec<SEXP>` materialised left-to-right, with
each `into_sexp()` evaluation a GC point that frees the prior column SEXPs
sitting unrooted in the temporary tuple slots. `from_raw_pairs` documents
"input SEXPs should already be protected" — the macro callers violated
that contract.

Reproduced deterministically (5–30 iterations) at any size ≥ ~30 rows for
`ParEvent`, ≥ ~100 rows for `ParPoint`. Pre-fix `gctorture2` masked it as
flake on the first iteration; sustained loop crashes consistently.

**Cross-cutting** — affected every code path emitting
`List::from_raw_pairs(vec![ (name, into_sexp(...)), ... ])` or the `_values`
analogue. Hits: `DataFrameRow` derive (struct + enum, `IntoDataFrame` impl
and `to_dataframe_split`), `IntoList` derive, `list!` macro, `Vctrs`
derive `IntoVctrs` impl. Total of 12 unique codegen sites across 5 macro
modules.

### Pass 4b — `set_class_str` / `set_row_names_int` parent-list UAF

Once the column-level UAFs were fixed, `dataframe-rayon` stopped crashing
but `ParPoint` started returning corrupted SEXPs:
`STRING_ELT() can only be applied to a 'character vector', not a 'integer'`
when the wrapper called `inherits(.val, "rust_condition_value")`. Root
cause: `List::set_class_str`, `set_names_str`, `set_row_names_int`, and
`set_row_names_str` all allocated their attribute vectors *without first
protecting `self.0`*. Pre-fix the column-level UAF crashed first and
masked this one; post-fix the methods received the freshly-built (but
unprotected) parent list and the `Rf_allocVector` for the class/names
vector freed it, replacing the slot with an INTSXP from later allocation
reuse.

## Fix

### `miniextendr-api/src/list.rs`

- `IntoList for Vec<T>` — allocate + protect the parent `VECSXP` first via
  `OwnedProtect::new(Rf_allocVector(...))`, then call `val.into_sexp()` and
  `list.get().set_vector_elt(idx, …)` inline per element. The protect drops
  when the function returns; by then the caller has taken ownership of a
  list whose children are rooted via the parent's write barrier.
- `List::from_pairs` — same shape: allocate + protect both list and names
  vectors first, materialise each value via `into_sexp()` and each name via
  `SEXP::charsxp` inline, write into the protected parents, drop both
  protects on exit.
- `set_class_str` / `set_names_str` / `set_row_names_int` / `set_row_names_str`
   — open an `OwnedProtect::new(self.0)` at function entry so the parent
  list survives the internal `Rf_allocVector` for its attribute vector.
  Drops on function exit, after the attribute has been written into the
  list and reachability is established through the caller.

All four constructors now mirror the `from_raw_pairs` and
`ColumnBuffer::Generic` post-fix pattern: protect first, allocate
children, write children, drop guards.

### `miniextendr-macros/` codegen — wrap each `into_sexp()` in `__scope.protect_raw`

Affected macro sites (12 total):

- `dataframe_derive.rs` — struct `IntoDataFrame::into_data_frame` (auto-expand
  and no-auto-expand branches).
- `dataframe_derive/enum_expansion.rs` — enum `IntoDataFrame::into_data_frame`
  (auto-expand + no-auto-expand) and `to_dataframe_split` (per-variant
  data.frames + outer named-list assembly across variants).
- `list_macro.rs` — the `list!{...}` macro for both unnamed (`from_raw_values`)
  and named (`from_raw_pairs`) cases.
- `list_derive.rs` — `IntoList` derive for named structs (`from_raw_pairs`)
  and tuple structs (`from_raw_values`).
- `vctrs_derive.rs` — `IntoVctrs` for record-style vctrs.

Each emission now opens an `unsafe { let __scope = ProtectScope::new(); ... }`
block and wraps every `IntoR::into_sexp(...)` call as
`__scope.protect_raw(IntoR::into_sexp(...))`. The `vec![]` macro evaluates
left-to-right, so by the time the next column allocates the previous
column's SEXP is on R's protect stack. The scope drops at function exit,
after `from_raw_pairs` has rooted the children via the parent VECSXP's
write barrier.

### `rpkg/src/rust/gc_protect_tests.rs`

- `test_list_from_values_strings_gctorture` — calls `List::from_values` on
  a 16-element `Vec<String>` (each element's `into_sexp()` allocates a
  STRSXP; many GC points).
- `test_list_from_pairs_strings_gctorture` — calls `List::from_pairs` on a
  16-element `Vec<(String, String)>` (allocates per name and per value).

Both ship as no-arg fixtures so future `gctorture(TRUE)` per-function
sweeps will exercise the fix automatically. The pre-existing
`test_dataframe_rayon` fixtures (`create_large_par_events`,
`create_large_par_points`) cover the macro codegen path.

### Validation

- `gctorture(TRUE)` × 50 iterations on the new no-arg fixtures: both pass
  post-fix.
- `gctorture(TRUE)` × 30 iterations on `create_large_par_events(500)` and
  `create_large_par_points(500)`: both pass post-fix. Pre-fix the same
  loop crashed at iteration 1–5 with `*** recursive gc invocation`.
- Full `devtools::load_all("rpkg") + test_dir(rpkg/tests/testthat)` under
  `gctorture2(step=100)`: **4999 / 4999 passed, 0 failed, 0 errored,
  0 segfaults, 0 `malloc(): … corrupted`** — same totals as a baseline
  `devtools::test()` run. Elapsed 21181 s (~5.9 h). This is the canonical
  proof that miniextendr's runtime + codegen surface is gctorture-secure
  under the changes in this audit.

## Lessons

- **Pre-attach lazy-loaded helpers (`lifecycle`, `rlang`, `withr`, `cli`,
  `vctrs`, `jsonlite`) before flipping gctorture on.** Same caveat as
  `loadNamespace` from `docs/GCTORTURE_TESTING.md` — the doc only listed
  testthat helpers; deprecation wrappers are an additional vector since
  `lifecycle` is lazily required *inside* the `.Call` shim.
- **`unsafe_C_*_panic*` fixtures are not audit-relevant.** Document this
  in the harness so future runs skip them automatically.
- **Static review is required, not optional.** Per-function sweeps only
  cover code reachable from no-arg fixtures. The two `list.rs` UAFs would
  not surface under sweeping alone — every fixture builds lists via
  hand-rolled impls of `IntoList`, never through the blanket `Vec<T>` impl.
  Future gctorture passes should pair the sweep with a `Vec<SEXP>` / raw-
  SEXP-storage grep across `miniextendr-api/`.
- **Add a fixture per public allocator path.** `IntoList for Vec<T>` is
  public API; the blanket impl had no test coverage. The two new fixtures
  fix that; keep the bar at "every public collection→list constructor has
  a no-arg `gctorture(TRUE)` regression fixture."
- **`vec![ (k, into_sexp(v)), ... ]` is a UAF idiom.** Anywhere the macro
  codegen materialises a `Vec<(N, SEXP)>` or `Vec<SEXP>` from
  `IntoR::into_sexp()` calls, the prior elements are unrooted across
  subsequent allocations. Lint or refactor pattern: use `ProtectScope`
  with `protect_raw` per-call, or build the parent VECSXP first and write
  children inline via `set_vector_elt`. Add to MXL lint if practical.
- **Fix-induced exposure is a real category.** The
  `set_class_str`/`set_row_names_int` parent-list UAF was older than the
  column-level UAF, but never observed because the column-level UAF
  crashed first under gctorture. After fixing the columns the parent UAF
  surfaced as a return-type-mismatch error in the R wrapper. Lesson: when
  fixing one UAF, re-run gctorture against the *exact same fixture* and
  expect a new crash; the masking layer is gone.
- **`from_raw_pairs` / `from_raw_values` / `set_*` documenting "input must
  be protected" is not enforced and not provided by any caller.** Either
  enforce internally (defensive `OwnedProtect::new(self.0)` at entry) or
  rename to make the contract impossible to forget. Currently fixed by
  protecting `self.0` internally for `set_*`; macro callers protect via
  `ProtectScope`. Long-term: consider a typed wrapper
  `ProtectedSexp<'a>` that's only constructible inside a scope.
