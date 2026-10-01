# R-side preconditions vs the Rust conversion

Date: 2026-09-29
Commit measured: `1b2b4a1e`, "fix(macros,api): no_na on an Either is checked after the conversion, by the arm taken (#1648)".
Decision: [#1661](https://github.com/A2-ai/miniextendr/issues/1661). Bugs found on the way: [#1659](https://github.com/A2-ai/miniextendr/issues/1659) (the Rust `no_na` scan reads ALTREP vectors element by element) and [#1660](https://github.com/A2-ai/miniextendr/issues/1660) (the integer conversions accept a factor's level codes).
Harness: `bench::mark` on the installed rpkg, driven by `rpkg/dev/bench-preconditions/run.R` (see [Reproduce](#reproduce)).

**Question.** Should the generated R-side argument checks move into Rust, or into R's C API at the C wrapper? The checks are `is.integer(x)`, `length(x) == 1L`, `!anyNA(x)`, the whole-number test, `inherits(...)` and the `match_arg` / `choices` helper.

**Answer: move all type checks to Rust now.** The value checks can move too, each after a specific fix (see [Recommendation](#6-recommendation)).

## 0. Environment and method

| | |
|---|---|
| Machine | Apple M3 Max, 14 cores, 36 GB, macOS 26.6.2 |
| R | 4.6.1, `bench` 1.1.4, the rv-managed project library |
| Rust | rustc 1.98.1, rpkg release profile (`codegen-units = 1`) |
| Commit | `1b2b4a1e` (see above) |
| Load (1-min average) | main run 2.03 → 2.22; chunked run 1.43 → 1.45. Other sessions were active. |

**Fixtures.** Everything is in one rpkg module, `rpkg/src/rust/precondition_bench_fixtures.rs`, with every function `noexport`:
- **Pairs:** `pcb_<case>` (default guards) and `pcb_<case>_u` (`no_preconditions`) share one body. Vector bodies return `black_box(len)`, so the copy into a `Vec` is not optimised away.
- **Floors:**
  - `pcb_sexp_floor(x: SEXP)`
  - `pcb_altrep_floor(x: AltrepSexp)`
  - raw `extern "C-unwind"` functions `C_pcb_*` that do no conversion. This is where a check "in the C wrapper, before conversion" would sit.
- **Candidate checks:**
  - NA scans:
    - through `DATAPTR_RO`;
    - through `*_ELT`, the shape of today's `from_r::any_na`;
    - through `*_NO_NA`, then `*_OR_NULL`, then `*_GET_REGION`;
    - the same, with a chunked (vectorised) inner loop.
  - Whole-number scans in the same variants.
  - `Rf_inherits`.

**Measurement.**
- `bench::mark` on the installed, byte-compiled package, with functions bound to local names (no `:::` inside the timed expression).
- Settings:
  - `min_time = 0.4 s`;
  - `filter_gc = FALSE`, so GC caused by an allocating guard counts toward it;
  - each expression runs once before timing (warm-up);
  - all variants of a case share one `mark()` call;
  - 3 reps.
- Tables show the median of the 3 rep medians.
- Deltas use 5%-trimmed means, because the macOS timer's 41.7 ns tick quantises medians.
- Rep spread is within one tick for sub-µs cases and under 2% for ms-scale cases.
- ALTREP inputs are built fresh inside the timed expression, because expansion sticks to the object. Building one costs 67–110 ns.
- **"R heap"** is `mem_alloc`, which counts R vector allocations only. The Rust heap is not counted: the `Vec` copy is 8n or 4n bytes, the same in both variants.

## 1. Existing evidence (reused and re-checked)

- **`docs/FEATURE_DEFAULTS.md`** gives `fast_i32` as 1.4 → 0.7 µs and `fast_sum3` as 3.0 → 0.9 µs. This run reproduces both: 1.35 → 0.68 µs and 2.78 → 0.89 µs.
- **The comment in `miniextendr-macros/src/lib.rs`** says the former `stopifnot()` block cost about 1230 ns per argument and the `isTRUE()` guards cost about half of that. The measured ~690 ns per scalar argument agrees.
- **`miniextendr-bench/benches/c_side_attribution.rs`** cites an R-side floor of about 287 ns. The raw `.Call` floor measures 283–298 ns here.
- **`miniextendr-bench/BENCH_RESULTS_2026-04-20.md`** (divan) bounds the Rust-side checks:
  - `from_r` `scalar_i32` 23.8 ns, `scalar_f64` 21.9 ns;
  - `slice_f64` at 64K 20.9 ns (O(1));
  - `xlength` 3.67 ns, `integer_elt` 7.54 ns.

  So the type and length test inside `TryFromSexp` costs about 20 ns. Dropping the R guard loses no check.
- `Unchecked<T>` did not exist at the measured commit (it arrived with [#1646](https://github.com/A2-ai/miniextendr/pull/1646)), so the comparison uses per-function `no_preconditions`.

## 2. Success path

### 2.0 Where a guard's time goes

These are byte-compiled guard forms, measured as cost over an empty function (`guard_forms.R`):

| form | cost |
|---|---|
| `if (!is.integer(x)) stop()` | 42 ns |
| `if (anyNA(x)) stop()` | 56 ns |
| `if (!is.integer(x) \|\| length(x) != 1L) stop()` | 99 ns |
| `isTRUE(TRUE)` alone | 249 ns |
| `if (!isTRUE(is.integer(x))) stop()` | 281 ns |
| `if (!isTRUE(!anyNA(x))) stop()` | 329 ns |
| two `isTRUE` guards (type + length) | 606 ns |

`isTRUE()` is a base closure, and the byte compiler does not inline it. The same test in C costs about 15 ns: raw `.Call` is 283 ns, and `.Call` plus `TYPEOF`/`XLENGTH` is 298 ns.

### 2.1 Scalars (n = 1)

| param | R guard + Rust | Rust only | saved | share of call |
|---|---|---|---|---|
| `i32` | 1.35 µs | 660 ns | 690 ns | 51% |
| `f64` | 1.37 µs | 693 ns | 676 ns | 49% |
| `bool` | 1.34 µs | 648 ns | 687 ns | 51% |
| `&str` | 1.34 µs | 651 ns | 691 ns | 51% |
| `String` | 1.39 µs | 695 ns | 693 ns | 50% |
| `Option<i32>` given `1L` | 1.40 µs | 658 ns | 746 ns | 53% |
| `Option<i32>` given `NULL` | 1.26 µs | 652 ns | 607 ns | 48% |
| `(i32, f64)` | 2.13 µs | 783 ns | 1.35 µs | 63% |
| `fast_i32` (existing fixture) | 1.35 µs | 677 ns | 669 ns | 50% |
| `fast_sum3` (existing fixture) | 2.78 µs | 886 ns | 1.90 µs | 68% |

R heap is 0 B for every row.

### 2.2 Vectors (one O(1) type guard)

| param | n = 1 | n = 1e3 | n = 1e5 | n = 1e7 |
|---|---|---|---|---|
| `&[f64]` | 986 → 675 ns (32%) | 979 → 678 ns (31%) | 987 → 667 ns (32%) | 961 → 672 ns (30%) |
| `&[i32]` | 981 → 672 ns (31%) | 986 → 670 ns (32%) | 984 → 667 ns (32%) | 980 → 668 ns (32%) |
| `Vec<f64>` | 997 → 686 ns (31%) | 1.15 µs → 845 ns (27%) | 10.8 → 10.7 µs (1%) | 1.23 → 1.23 ms (0%) |
| `Vec<i32>` | 999 → 705 ns (29%) | 1.10 µs → 784 ns (29%) | 6.13 → 5.80 µs (5%) | 585 → 585 µs (0%) |

R heap is 0 B for every row.

### 2.3 `no_na` on a vector

- **R path:** `pcb_nona_slice_*` runs the type guard plus `!anyNA`. The `_u` variant runs `!anyNA` only, because `no_na` survives `no_preconditions`.
- **Rust path:** `#[miniextendr(no_na)] x: Either<&[T], String>` runs the framework's `from_r::any_na` and no R guard.

Added cost of the NA check, non-ALTREP input:

| n | R `anyNA` guard, f64 | Rust `any_na`, f64 | R `anyNA` guard, i32 | Rust `any_na`, i32 |
|---|---|---|---|---|
| 1 | +349 ns | +33 ns | +350 ns | +38 ns |
| 1e3 | +625 ns | +327 ns | +652 ns | +77 ns |
| 1e5 | +27.1 µs | +27.1 µs | +27.9 µs | +4.07 µs |
| 1e7 | +2.71 ms | +2.70 ms | +2.71 ms | +408 µs |

- **The guard pair as the default emits it** (type guard plus NA guard) costs +641 ns at n = 1 and +925 ns at 1e3. From 1e5 up it is 98–100% of the call.
- **Faster scans** (the `chunked` run, Appendix B, at 1e7):
  - A chunked inner loop brings f64 to 1.35 ms, against R's 2.65 ms.
  - For i32, the plain C-API scan is already vectorised and is the best at 389–418 µs. The chunked loop takes 511 µs.

**Classed input** (`structure(runif(n), class = "mx_num")`). For an object with a class attribute, `anyNA()` falls back to `any(is.na(x))`:

| n | R `anyNA` guard | R heap | C-API scan | R heap |
|---|---|---|---|---|
| 1 | +1.07 µs | 0 B | +7 ns | 0 B |
| 1e3 | +3.15 µs | 4.0 KB | +308 ns | 0 B |
| 1e5 | +192 µs | 391 KB | +26.9 µs | 0 B |
| 1e7 | +19.2 ms | 38.1 MB | +2.68 ms | 0 B |

### 2.4 `coerce` `Vec<i32>` given doubles (the whole-number guard)

The R guard is `is.integer(x) || is.logical(x) || is.raw(x) || (is.numeric(x) && all(is.na(x) | x == trunc(x)))`. The Rust conversion already checks every element.

| input | n | R guard + Rust | Rust only | saved | R heap (guard → Rust only) |
|---|---|---|---|---|---|
| whole doubles | 1 | 1.44 µs | 696 ns | 740 ns (52%) | 0 → 0 |
| whole doubles | 1e3 | 6.83 µs | 1.80 µs | 5.03 µs (74%) | 19.7 KB → 0 |
| whole doubles | 1e5 | 556 µs | 109 µs | 447 µs (80%) | 1.9 MB → 0 |
| whole doubles | 1e7 | 52.1 ms | 11.0 ms | 41.1 ms (79%) | 190.7 MB → 0 |
| `as.double(1:n)` | 1e3 | 8.01 µs | 2.16 µs | 5.85 µs (73%) | 27.6 KB → 7.9 KB |
| `as.double(1:n)` | 1e5 | 614 µs | 132 µs | 483 µs (79%) | 2.7 MB → 781 KB |
| `as.double(1:n)` | 1e7 | 60.8 ms | 12.3 ms | 48.5 ms (80%) | 267 MB → 76.3 MB |

- **The R expression alone** costs 37 ms and 190.7 MB at 1e7. It builds four n-length temporaries.
- **The 76.3 MB left on compact input** is the Rust conversion's own expansion through `DATAPTR_RO`. The R expression also expands the sequence, but the cost is paid once. Dropping the guard removes the 190.7 MB of temporaries, not the expansion.
- **Standalone Rust scans at 1e7** allocate nothing:
  - whole doubles: slice 4.04 ms, C API 3.95–4.09 ms, chunked 1.75 ms (21× faster than R's expression);
  - compact input: C API 5.49 ms, chunked 3.14 ms, with no expansion. R's expression takes 49.1 ms and 343 MB.

### 2.5 `inherits` and data frames

| check | R guard | Rust or C API |
|---|---|---|
| `inherits(x, "mx_cls")`, 1 class | +397 ns (37%) | `Rf_inherits` +29 ns |
| `inherits(x, "mx_cls")`, 4 classes, match last | +378 ns | `Rf_inherits` +32 ns |
| `List` param, `inherits` + `is.list` | +713 ns (51%) | Rust `List` conversion |
| `List` param, `inherits` only (`no_preconditions`) | +389 ns (36%) | n/a |

`DataFrame` (5 columns) converts in Rust at 1.04 µs with no R guard. Adding a guard costs:

| guard | 10 rows | 1e3 rows | 1e5 rows |
|---|---|---|---|
| `inherits = "data.frame"` | +394 ns (27%) | +383 ns | +392 ns |
| `no_na` (R dispatches `anyNA.data.frame`) | +3.46 µs (77%) | +5.30 µs (83%) | +147 µs (99%) |

### 2.6 `match_arg` / `choices`

| case | R helper | Rust only | saved |
|---|---|---|---|
| `match_arg` enum, `"slow"` | 2.26 µs | 695 ns | 1.57 µs (69%) |
| `match_arg` enum, `"sl"` (prefix) | 2.30 µs | 761 ns | 1.54 µs (67%) |
| `choices` on `&str`, `"slow"` | 2.25 µs | 662 ns (the `&str` floor; no Rust check exists) | 1.59 µs (71%) |
| argument omitted (formal default) | 1.64–1.67 µs | not possible on the Rust path yet (gap 2) | n/a |

Rust exact matching costs 32 ns over a plain `&str`, and prefix matching 83 ns.

### 2.7 The floor that remains

Hand-made byte-compiled copies of the `pcb_i32_u(1L)` wrapper (`floor_parts.R`):

| variant | time |
|---|---|
| generated wrapper | 673 ns |
| hand copy of it | 661 ns |
| hand copy without the post-call `inherits(.val, "rust_condition_value")` check | 508 ns (that check costs ≈150 ns) |
| hand copy with `.call = NULL` instead of `sys.call()` | 453 ns (`sys.call()` costs ≈210 ns) |
| bare `.Call` | 312 ns |

Once the guards are gone, about half of the remaining cost is these two closure calls. Neither is a precondition.

## 3. Error-path equivalence

The source is `errors.R`: 106 inputs over 20 case labels, each called the way a user would call it. **80 inputs are refused by both paths. In all 80, `class`, `e$kind`, `e$param` and `e$call` are identical.**
- The class is `rust_error / simpleError / error / condition`.
- With two bad parameters, both paths report the first one.

| shape | R guard message | Rust message |
|---|---|---|
| `i32` given `"a"`, `1.5`, `2`, `TRUE`, `NULL`, `list(1L)` | `'x' must be integer` | `'x' must be a single integer: got character` (numeric / logical / NULL / list) |
| `i32` given `1:2`, `integer(0)` | `'x' must have length 1` | `… got length 2` / `got length 0` |
| `i32` given `NA_integer_`; `bool` given `NA` | R guard passes; Rust raises | identical: `… NA is not allowed` |
| `f64`, `bool`, `&str`, `String`, `Option<i32>` | `must be double` / `logical` / `character` / `NULL or integer` / `have length 1` | `must be a single double` / `TRUE or FALSE` / `a single string` / `NULL or a single integer: got …` |
| `&[f64]`, `Vec<f64>` | `must be double` | `must be double: got integer` (…) |
| `&[i32]`, `Vec<i32>` | `must be an integer vector` | `must be integer: got numeric` (…) |
| `no_na` given `c(1, NA)`, `c(1, NaN)`, `c(1L, NA)` | `'x' must not contain NA` | identical; neither path sets `rust_type` |
| `coerce` given `c(1.5, 2)` / `c(1.5, 2.5, 3)` | `must be integer or whole-number numeric` | `…: precision loss (element 1)` / `(elements 1, 2)` |
| `coerce` given `1e10`, `NaN`, `Inf` | R guard passes; Rust raises | identical |
| `DataFrame` given `list(a = 1)`, `1`, `NULL`, `matrix` | `'x' must inherit from 'data.frame'` | `'x' must be a data frame: got list` (…) |
| `match_arg` given `"medium"` / `""` | `should be one of "fast", "slow", "debug"` | `must be one of …: got "medium"` |
| `match_arg` given `1` / `c("fast", "slow")` / `NA` | `must be NULL or a character vector` / `must be of length 1` / `should be one of …` | `… got numeric` / `got length 2` / `NA is not allowed` |

Accepted identically by both paths:
- `NA_real_` for `f64`, and `NA_character_` for `&str`;
- `matrix` and `Sys.Date()` for `&[f64]`;
- `NULL`, `factor("slow")` and the prefix `"sl"` for `match_arg`;
- a ragged data frame (neither path refuses it).

**Always different, and the Rust side is never less informative:** Rust names what it received and adds `e$rust_type`. The tests and docs that pin the R-guard shape would need updating:
- `rpkg/tests/testthat/test-feature-defaults.R` (the `expect_null(...$rust_type)` lines);
- `test-fast-fixtures.R` and `test-class-systems.R`;
- `docs/FEATURE_DEFAULTS.md`, `MINIEXTENDR_ATTRIBUTE.md`, `ERROR_HANDLING.md`, `CONVERSION_MATRIX.md`, `TYPE_CONVERSIONS.md`, `CALL_ATTRIBUTION.md` and `CLASS_SYSTEMS.md`.

Footnote: `no_na &[f64]` given `"a"` is refused by R and accepted by Rust. That comes from the fixture's `Either` having a `String` arm. It is not a gap.

### Gaps, and how to close each

1. **A factor is accepted as an integer** ([#1660](https://github.com/A2-ai/miniextendr/issues/1660)).
   - `i32` given `factor("a")`, and `&[i32]` / `Vec<i32>` given `factor(c("a", "b"))`: R refuses, since `is.integer` is FALSE for a factor. Rust accepts the level codes.
   - *Fix:* refuse a factor (`OBJECT` and `Rf_inherits(x, "factor")`) in the native `INTSXP` conversions; the message would be "got factor".
   - Also audit the other conversions whose R guard is `is.numeric`, which is FALSE for a factor. These were not measured.
2. **`match_arg`, omitted argument / default vector.**
   - Without the helper there is no formal default, so an omitted argument gives base R's "argument "mode" is missing, with no default". Passing the full choice vector gives "got length 3".
   - *Fix:* keep the formal default. In `match_arg_from_sexp`, treat an input equal to `T::CHOICES` as the first choice (an O(k) compare).
3. **`choices` on `&str` has no Rust check** (`"medium"` is accepted). *Fix:* generate a Rust exact-then-prefix matcher over the literal list, and pass the matched choice.
4. **`inherits` has no Rust check, and `Rf_inherits` is not `base::inherits`** (`errors-inherits.csv`).
   - `Rf_inherits` returns FALSE where `base::inherits` returns TRUE for implicit classes (`1L`/`"integer"`, `1`/`"numeric"`, `matrix(1)`/`"matrix"`/`"array"`, a function/`"function"`, `list()`/`"list"`) and for an S4 superclass (a `PcbChild` object and `"PcbBase"`).
   - The two agree on S3 class attributes and on the exact S4 class.
   - *Fix:* use `Rf_inherits` for non-S4 input and a methods-aware check for S4. Refuse implicit-class names at macro time, or map them to `TYPEOF` tests. Raise the error through `arg_check_condition_value`, which already produces the identical condition (see the `no_na` row).
5. **`no_na` on classed objects.**
   - R dispatches `is.na` / `anyNA` (`POSIXlt`, vctrs records, `anyNA.data.frame`). A Rust scan reads storage by `SEXPTYPE`.
   - For `DataFrame`, the R guard refuses NA cells. The Rust `DataFrame::__mx_input_has_na` returns false, so Rust accepted the NA.
   - *Fix:* scan in Rust when the input has no class attribute or its class is known. Otherwise, call R's `anyNA` from C.
6. **The `List` wording leaks internals.** Rust says `'x' must be a list: type mismatch: expected VECSXP, got REALSXP`, where R says `'x' must be a list`. *Fix:* map it to the "got numeric" vocabulary, as the scalar conversions already do. This is the only Rust message found that is worse than R's.

Side findings:
- The `no_na` guard on a `DataFrame` says `'x' must not be NA`. It should say "must not contain NA".
- Neither path refuses a ragged data frame.

## 4. Where a moved check could live

| location | what it can check | cost | effect on ALTREP input |
|---|---|---|---|
| `TryFromSexp` (today) | type, length, scalar NA, element precision/range (coerce), `DataFrame` class, `MatchArg` | ~20 ns for type + length; element checks are fused with the copy | native slice / `Vec` / `SEXP` conversions expand compact vectors |
| C wrapper, before conversion, via the C API | `TYPEOF`/`XLENGTH` (+15 ns), `Rf_inherits` (+30 ns), NA scan via `*_NO_NA` + `*_OR_NULL` + `*_GET_REGION`, whole-number scan by region | see the next table | never expands; O(1) on compact sequences |
| Rust scan after conversion | NA, whole-number | the same inner loops | the conversion has already expanded the input |

Scan costs at n = 1e7, above the floor, non-ALTREP input:

| scan | f64 NA | i32 NA | whole-number | R heap |
|---|---|---|---|---|
| R `anyNA()`, called directly | 2.65 ms | 2.65 ms | n/a | 0 |
| R `all(is.na(x) \| x == trunc(x))` | n/a | n/a | 37 ms | 190.7 MB |
| Rust, `DATAPTR_RO` slice | 2.68 ms | 408 µs | 4.04 ms | 0 |
| Rust, `*_ELT` (today's `any_na`) | 2.71 ms | 415 µs | n/a | 0 |
| C API: `NO_NA` + `OR_NULL` + `GET_REGION` | 2.64–2.69 ms | 389–418 µs | 3.95–4.09 ms | 0 |
| C API with a chunked inner loop | 1.35 ms | 511 µs | 1.75 ms | 0 |

At n = 1e3 for f64: R `anyNA` costs 285 ns, the chunked scan 152 ns.

What the data supports:
- Type, length and class checks belong in or just before `TryFromSexp`.
- Value scans belong in the C wrapper before conversion, through the C API, so compact inputs keep their O(1) answer.
- For `coerce`, keep the element check fused with the conversion, as it is today.

## 5. ALTREP findings

The materialisation probe (`materialise.R`) inspects a fresh `1:1000` or `as.double(1:1000)` after each call:
- **Stay compact:**
  - R `is.integer`, `length`, `anyNA`, `inherits`, `sum`;
  - an `AltrepSexp` parameter;
  - the raw C functions: `TYPEOF`/`XLENGTH`, `Rf_inherits`, the C-API NA and whole-number scans, the `*_ELT` scan, the chunked scans (`chunked_check.R`).
- **Expanded:**
  - R's whole-number guard expression;
  - a `#[miniextendr] x: SEXP` parameter (`TryFromSexp for SEXP` calls `ensure_materialized`);
  - `&[i32]`, `Vec<i32>`, `&[f64]`;
  - `no_na` on `&[i32]` and on an `Either`;
  - `coerce Vec<i32>` in both variants;
  - raw scans through `DATAPTR_RO`.

So a Rust scan does not itself force materialisation, provided it reads through `*_OR_NULL`, `*_GET_REGION` or `*_ELT`. Every native typed conversion materialises anyway.

Compact input, cost above the floor:

| n | R `anyNA` guard | C API (`NO_NA`) | `*_ELT` scan | `DATAPTR_RO` slice scan |
|---|---|---|---|---|
| 1e3 | +376 ns | +18 ns | +5.9 µs | +256 ns, 4 KB |
| 1e5 | +345 ns | +17 ns | +585 µs | +8.3 µs, 391 KB |
| 1e7 | +346 ns | +19 ns | +58.6 ms | +1.01 ms, 38.1 MB |

The table is for `1:n`. `as.double(1:n)` shows the same pattern: +342 ns, +12 ns, +58.6 ms, and +4.0 ms with 76.3 MB.

**The framework's live Rust NA check is a large regression on compact input** ([#1659](https://github.com/A2-ai/miniextendr/issues/1659)).
- `#[miniextendr(no_na)] x: Either<&[i32], String>` given `1:n`, measured against the same `Either` without `no_na`, adds:
  - +5.3 µs at 1e3;
  - +539 µs at 1e5;
  - +53.4 ms at 1e7, so the call goes from 406 µs to 53.8 ms.
- The R `anyNA` guard on the same typed parameter adds +344 ns, +378 ns and +1.46 µs.
- The cause: `from_r::any_na` reads ALTREP vectors element by element through `*_ELT`, and never asks `*_NO_NA`.
- *Fix (needed however the guards question is decided):* check `*_NO_NA` on the input first, then `*_OR_NULL`, then `*_GET_REGION` chunks. That is the shape of `C_pcb_na_capi` / `C_pcb_na_chunked`, which match `anyNA()` on every tested input.
- On the `1:n` input, `&[i32]` under the default guard vs `no_preconditions` differs only by the ~310 ns guard. Both variants expand the input (38.1 MB at 1e7).

## 6. Recommendation

**Move all type checks to Rust. Move the value checks in this order:**

1. **Drop every type and length guard once gaps 1 and 6 are fixed.** This covers the scalar `is.<type>` + `length == 1L` guards, the `Option` forms, the vector `is.double`/`is.integer` guards and `is.list`.
   - Saving: ~690 ns per scalar parameter (~50% of a one-argument call, 63–68% at 2–3 arguments), and ~310 ns per vector parameter (31% at any n).
   - Class, kind, param and call are unchanged. Messages get richer, and `e$rust_type` appears.
2. **Drop the `coerce` whole-number guard now.** It duplicates Rust's per-element check, and Rust's message is better (batched element indices).
   - Saving: 74–80% of the call from n = 1e3 (41 ms of 52 ms at 1e7), plus 190.7 MB of R heap at 1e7.
   - No semantic gap was found (`NA`, `NaN`, `Inf`, `1e10`, `"a"`).
   - Compact input still expands (76.3 MB at 1e7) until the coerce conversion reads by region.
3. **Move `match_arg` to the existing `MatchArg` conversion after closing gap 2.** Saving ≈1.57 µs (69%). For `choices` on `&str`, generate the same matcher (gap 3).
4. **Move `inherits` to `Rf_inherits`,** raised through `arg_check_condition_value`, with an S4 / implicit-class fallback (gap 4). Saving ≈370 ns per class check. Until the fallback exists, keep the R guard for S4 class names.
5. **Move `no_na` only as a new scan before conversion** (`*_NO_NA`, then `*_OR_NULL`, then `*_GET_REGION`, chunked for doubles).
   - It gains ≈350 ns at small n, is 2× faster (f64) and 6–7× faster (i32) at large n, and allocates 0 B instead of 38 MB on classed input at 1e7.
   - Keep R's `anyNA()` for objects that carry a class attribute with their own `is.na` method, and for `DataFrame` parameters (gap 5).
   - Do not build on today's `from_r::any_na`: on `1:1e7` it adds 53.4 ms where R adds under 1.5 µs.
6. **If any guard stays in R, drop its `isTRUE()` wrapper.** None of the predicates can return `NA` except the unsigned `x >= 0` check. Two bare guards cost 99 ns, against 606 ns with `isTRUE`.

Net effect on a one-scalar call: 1.35 µs → 0.66 µs. What remains is mostly `sys.call()` and the post-call `inherits()` check, not argument checking.

## Reproduce

From the repository root (rv activates the project library there):

```bash
just configure && just rcmdinstall
Rscript rpkg/dev/bench-preconditions/run.R target/bench-preconditions               # full run, ~20 min
Rscript rpkg/dev/bench-preconditions/run.R target/bench-preconditions-smoke 1 0.05  # smoke run
```

`run.R <out_dir> [reps] [min_time]` runs each script in its own `Rscript` process and writes every output into `<out_dir>` (a directory under `target/` stays out of git). The scripts also run on their own; each one's header gives its usage:

| script | produces |
|---|---|
| `bench.R <out.csv> [groups...]` | the success-path timings (sections 2 and 4). The default groups are the main run; `chunked` is the second run. `PCB_REPS` / `PCB_MIN_TIME` set reps and `min_time`. |
| `analyze.R <bench.csv>` | the per-variant summary (median, rep range, trimmed mean, R heap) and the derived deltas that sections 2 to 5 quote, as markdown: `summary.md` and `summary-chunked.md`, reproduced as Appendices A and B |
| `errors.R <errors.csv>` | the error-path comparison (section 3): `errors.csv`, `errors-compact.txt`, `errors-inherits.csv` |
| `materialise.R` | which calls expand a compact sequence (section 5) |
| `chunked_check.R` | the chunked scans against `anyNA()` and the R whole-number expression, and their effect on ALTREP input |
| `guard_forms.R` | the guard-form costs (section 2.0) |
| `floor_parts.R` | the remaining floor (section 2.7) |

The fixtures are internal (`noexport`, no man page), so the scripts reach them through `asNamespace("miniextendr")`. None of this runs in `R CMD check` or testthat: `rpkg/dev/` is in `.Rbuildignore`, and testthat only reads `tests/testthat/test*.R`.

### Recheck on the commit that adds these files

The committed fixture and scripts were rerun on main `25f1a315`, which includes [#1646](https://github.com/A2-ai/miniextendr/pull/1646) (`Checked<T>` / `Unchecked<T>`) and [#1653](https://github.com/A2-ai/miniextendr/pull/1653) (the coerce `i32` range and `NA_real_` fix):
- the generated R wrappers of the 47 `pcb_*` functions in the measured run's wrapper snapshot are unchanged, guards included, and the hand copy in `floor_parts.R` still matches the generated `pcb_i32_u`;
- `errors.csv`, `errors-inherits.csv` and the output of `materialise.R` and `chunked_check.R` are identical to the measured run, so section 3 and the ALTREP lists in section 5 hold as written;
- a smoke run (`run.R <out_dir> 1 0.05`, 1-minute load average 2.40) shows the same pattern: `i32` 1.39 µs → 711 ns (49% saved), `coerce Vec<i32>` on whole doubles at 1e7 52.0 → 10.5 ms, `no_na` `Either` on `1:1e7` +54.0 ms.

The timing tables above are the full run at `1b2b4a1e`.

## Appendix A: per-variant summary, main run

The measured run (`summary.md` from `analyze.R`): per-variant medians, rep ranges, trimmed means and R heap, then the derived deltas that sections 2 to 5 quote.

<details>
<summary>Per-variant summary and derived deltas</summary>

### Raw per-variant results (median of 3 rep medians; [min-max of rep medians]; trimmed mean; R-heap bytes)


#### group: altrep

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| &[i32] compact | 1:n | 1000 | guarded | 1.35 µs | 1.35 µs–1.35 µs | 1.35 µs | 4.0 KB |
| &[i32] compact | 1:n | 1000 | unguarded | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 4.0 KB |
| &[i32] compact | 1:n | 100000 | guarded | 5.45 µs | 5.29 µs–5.53 µs | 5.51 µs | 390.7 KB |
| &[i32] compact | 1:n | 100000 | unguarded | 5.13 µs | 5.00 µs–5.21 µs | 5.17 µs | 390.7 KB |
| &[i32] compact | 1:n | 1e+07 | guarded | 402.3 µs | 397.1 µs–403.6 µs | 408.0 µs | 38.1 MB |
| &[i32] compact | 1:n | 1e+07 | unguarded | 402.9 µs | 397.6 µs–403.5 µs | 405.1 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1000 | guard_na | 1.39 µs | 1.39 µs–1.39 µs | 1.40 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | guard_type_na | 1.68 µs | 1.68 µs–1.68 µs | 1.68 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | no_check | 1.03 µs | 1.02 µs–1.07 µs | 1.06 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | rust_any_na | 6.35 µs | 6.27 µs–6.36 µs | 6.34 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 1000 | rust_either_base | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 4.0 KB |
| no_na &[i32] compact | 1:n | 100000 | guard_na | 5.41 µs | 5.33 µs–5.45 µs | 5.42 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | guard_type_na | 5.66 µs | 5.62 µs–5.74 µs | 5.69 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | no_check | 5.04 µs | 5.00 µs–5.08 µs | 5.04 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | rust_any_na | 545.2 µs | 526.6 µs–548.4 µs | 544.6 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 100000 | rust_either_base | 5.12 µs | 5.00 µs–5.21 µs | 5.14 µs | 390.7 KB |
| no_na &[i32] compact | 1:n | 1e+07 | guard_na | 401.3 µs | 396.0 µs–403.4 µs | 405.3 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | guard_type_na | 402.9 µs | 395.4 µs–403.2 µs | 410.3 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | no_check | 402.4 µs | 394.9 µs–403.4 µs | 403.9 µs | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | rust_any_na | 53.75 ms | 52.73 ms–53.76 ms | 53.80 ms | 38.1 MB |
| no_na &[i32] compact | 1:n | 1e+07 | rust_either_base | 402.6 µs | 393.8 µs–403.2 µs | 405.7 µs | 38.1 MB |
| scan compact dbl | as.double(1:n) | 1000 | altrep_floor | 779 ns | 738 ns–779 ns | 777 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | ctor_only | 123 ns | 123 ns–123 ns | 110 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | r_anyNA_guard | 1.11 µs | 1.11 µs–1.15 µs | 1.12 µs | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_capi | 410 ns | 410 ns–410 ns | 420 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_elt | 6.27 µs | 6.15 µs–6.31 µs | 6.31 µs | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_floor | 410 ns | 410 ns–410 ns | 406 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1000 | raw_slice | 1.07 µs | 1.03 µs–1.07 µs | 1.08 µs | 7.9 KB |
| scan compact dbl | as.double(1:n) | 100000 | altrep_floor | 779 ns | 779 ns–779 ns | 776 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | ctor_only | 123 ns | 123 ns–123 ns | 107 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | r_anyNA_guard | 1.11 µs | 1.07 µs–1.15 µs | 1.14 µs | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_capi | 410 ns | 410 ns–410 ns | 426 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_elt | 585.8 µs | 574.5 µs–596.6 µs | 591.8 µs | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_floor | 410 ns | 410 ns–410 ns | 404 ns | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | raw_slice | 38.9 µs | 38.0 µs–39.2 µs | 39.4 µs | 781.3 KB |
| scan compact dbl | as.double(1:n) | 1e+07 | altrep_floor | 779 ns | 738 ns–779 ns | 781 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | ctor_only | 123 ns | 123 ns–123 ns | 109 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | r_anyNA_guard | 1.11 µs | 1.11 µs–1.15 µs | 1.12 µs | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_capi | 410 ns | 410 ns–410 ns | 410 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_elt | 58.51 ms | 57.49 ms–58.52 ms | 58.55 ms | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_floor | 410 ns | 410 ns–410 ns | 397 ns | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | raw_slice | 3.83 ms | 3.78 ms–3.84 ms | 4.00 ms | 76.3 MB |
| scan compact int | 1:n | 1000 | altrep_floor | 697 ns | 697 ns–738 ns | 728 ns | 0 B |
| scan compact int | 1:n | 1000 | ctor_only | 82 ns | 82 ns–82 ns | 67 ns | 0 B |
| scan compact int | 1:n | 1000 | r_anyNA_guard | 1.11 µs | 1.07 µs–1.11 µs | 1.10 µs | 0 B |
| scan compact int | 1:n | 1000 | raw_capi | 369 ns | 369 ns–369 ns | 377 ns | 0 B |
| scan compact int | 1:n | 1000 | raw_elt | 6.23 µs | 6.23 µs–6.40 µs | 6.27 µs | 0 B |
| scan compact int | 1:n | 1000 | raw_floor | 369 ns | 369 ns–369 ns | 359 ns | 0 B |
| scan compact int | 1:n | 1000 | raw_slice | 615 ns | 574 ns–615 ns | 615 ns | 4.0 KB |
| scan compact int | 1:n | 100000 | altrep_floor | 697 ns | 697 ns–738 ns | 721 ns | 0 B |
| scan compact int | 1:n | 100000 | ctor_only | 82 ns | 82 ns–82 ns | 66 ns | 0 B |
| scan compact int | 1:n | 100000 | r_anyNA_guard | 1.07 µs | 1.07 µs–1.07 µs | 1.07 µs | 0 B |
| scan compact int | 1:n | 100000 | raw_capi | 369 ns | 369 ns–369 ns | 375 ns | 0 B |
| scan compact int | 1:n | 100000 | raw_elt | 584.3 µs | 573.5 µs–592.2 µs | 585.6 µs | 0 B |
| scan compact int | 1:n | 100000 | raw_floor | 369 ns | 328 ns–369 ns | 358 ns | 0 B |
| scan compact int | 1:n | 100000 | raw_slice | 8.69 µs | 8.49 µs–8.73 µs | 8.70 µs | 390.7 KB |
| scan compact int | 1:n | 1e+07 | altrep_floor | 697 ns | 697 ns–738 ns | 725 ns | 0 B |
| scan compact int | 1:n | 1e+07 | ctor_only | 82 ns | 82 ns–82 ns | 67 ns | 0 B |
| scan compact int | 1:n | 1e+07 | r_anyNA_guard | 1.07 µs | 1.03 µs–1.07 µs | 1.07 µs | 0 B |
| scan compact int | 1:n | 1e+07 | raw_capi | 369 ns | 369 ns–369 ns | 378 ns | 0 B |
| scan compact int | 1:n | 1e+07 | raw_elt | 58.52 ms | 57.48 ms–58.56 ms | 58.59 ms | 0 B |
| scan compact int | 1:n | 1e+07 | raw_floor | 369 ns | 328 ns–369 ns | 359 ns | 0 B |
| scan compact int | 1:n | 1e+07 | raw_slice | 918.7 µs | 888.9 µs–919.6 µs | 1.01 ms | 38.1 MB |

#### group: choices

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | choices_guard | 2.25 µs | 2.21 µs–2.26 µs | 2.25 µs | 0 B |
| choices &str / match_arg enum | "sl" (prefix) | 1 | match_arg_guard | 2.30 µs | 2.26 µs–2.30 µs | 2.30 µs | 0 B |
| choices &str / match_arg enum | "sl" (prefix) | 1 | rust_match_only | 738 ns | 738 ns–738 ns | 761 ns | 0 B |
| choices &str / match_arg enum | "sl" (prefix) | 1 | str_floor | 656 ns | 656 ns–697 ns | 677 ns | 0 B |
| choices &str / match_arg enum | "slow" | 1 | choices_guard | 2.26 µs | 2.17 µs–2.26 µs | 2.25 µs | 0 B |
| choices &str / match_arg enum | "slow" | 1 | choices_guard_u | 2.21 µs | 2.17 µs–2.21 µs | 2.23 µs | 0 B |
| choices &str / match_arg enum | "slow" | 1 | match_arg_guard | 2.26 µs | 2.21 µs–2.26 µs | 2.26 µs | 0 B |
| choices &str / match_arg enum | "slow" | 1 | rust_match_only | 697 ns | 656 ns–697 ns | 695 ns | 0 B |
| choices &str / match_arg enum | "slow" | 1 | str_floor | 656 ns | 656 ns–656 ns | 662 ns | 0 B |
| choices &str / match_arg enum | omitted (default) | 1 | choices_guard | 1.64 µs | 1.60 µs–1.68 µs | 1.65 µs | 0 B |
| choices &str / match_arg enum | omitted (default) | 1 | match_arg_guard | 1.64 µs | 1.64 µs–1.68 µs | 1.67 µs | 0 B |

#### group: df

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| DataFrame, 5 cols | data.frame | 10 | guard_anyNA | 4.51 µs | 4.43 µs–4.55 µs | 4.50 µs | 0 B |
| DataFrame, 5 cols | data.frame | 10 | guard_inherits | 1.43 µs | 1.39 µs–1.48 µs | 1.44 µs | 0 B |
| DataFrame, 5 cols | data.frame | 10 | no_guard | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | guard_anyNA | 6.35 µs | 6.23 µs–6.40 µs | 6.35 µs | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | guard_inherits | 1.39 µs | 1.39 µs–1.44 µs | 1.43 µs | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | no_guard | 1.03 µs | 1.02 µs–1.03 µs | 1.05 µs | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | guard_anyNA | 147.6 µs | 145.0 µs–147.8 µs | 148.0 µs | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | guard_inherits | 1.44 µs | 1.39 µs–1.44 µs | 1.44 µs | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | no_guard | 1.03 µs | 1.03 µs–1.03 µs | 1.04 µs | 0 B |

#### group: inherits

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| inherits List | list, class mx_cls | 3 | guard_inh | 1.07 µs | 1.07 µs–1.11 µs | 1.08 µs | 0 B |
| inherits List | list, class mx_cls | 3 | guard_inh_type | 1.39 µs | 1.35 µs–1.39 µs | 1.41 µs | 0 B |
| inherits List | list, class mx_cls | 3 | guard_type | 1.03 µs | 984 ns–1.07 µs | 1.05 µs | 0 B |
| inherits List | list, class mx_cls | 3 | no_check | 697 ns | 656 ns–697 ns | 695 ns | 0 B |
| inherits scan | list, 4 classes, match last | 3 | r_inherits_guard | 1.03 µs | 1.03 µs–1.07 µs | 1.05 µs | 0 B |
| inherits scan | list, 4 classes, match last | 3 | raw_floor | 287 ns | 287 ns–287 ns | 294 ns | 0 B |
| inherits scan | list, 4 classes, match last | 3 | raw_rf_inherits | 328 ns | 328 ns–328 ns | 326 ns | 0 B |
| inherits scan | list, 4 classes, match last | 3 | sexp_floor | 656 ns | 656 ns–656 ns | 668 ns | 0 B |
| inherits scan | list, class mx_cls | 3 | r_inherits_guard | 1.07 µs | 1.03 µs–1.07 µs | 1.06 µs | 0 B |
| inherits scan | list, class mx_cls | 3 | raw_floor | 287 ns | 287 ns–328 ns | 293 ns | 0 B |
| inherits scan | list, class mx_cls | 3 | raw_rf_inherits | 328 ns | 328 ns–328 ns | 323 ns | 0 B |
| inherits scan | list, class mx_cls | 3 | sexp_floor | 656 ns | 656 ns–656 ns | 663 ns | 0 B |

#### group: nona

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | guard_na | 1.03 µs | 984 ns–1.03 µs | 1.02 µs | 0 B |
| no_na &[f64] | runif(n) | 1 | guard_type_na | 1.31 µs | 1.31 µs–1.31 µs | 1.31 µs | 0 B |
| no_na &[f64] | runif(n) | 1 | no_check | 656 ns | 656 ns–697 ns | 669 ns | 0 B |
| no_na &[f64] | runif(n) | 1 | rust_any_na | 697 ns | 697 ns–697 ns | 702 ns | 0 B |
| no_na &[f64] | runif(n) | 1 | rust_either_base | 656 ns | 656 ns–697 ns | 670 ns | 0 B |
| no_na &[f64] | runif(n) | 1000 | guard_na | 1.27 µs | 1.27 µs–1.31 µs | 1.30 µs | 0 B |
| no_na &[f64] | runif(n) | 1000 | guard_type_na | 1.60 µs | 1.56 µs–1.60 µs | 1.60 µs | 0 B |
| no_na &[f64] | runif(n) | 1000 | no_check | 656 ns | 656 ns–697 ns | 676 ns | 0 B |
| no_na &[f64] | runif(n) | 1000 | rust_any_na | 984 ns | 984 ns–1.03 µs | 1.01 µs | 0 B |
| no_na &[f64] | runif(n) | 1000 | rust_either_base | 697 ns | 656 ns–697 ns | 687 ns | 0 B |
| no_na &[f64] | runif(n) | 100000 | guard_na | 27.6 µs | 27.6 µs–28.8 µs | 27.8 µs | 0 B |
| no_na &[f64] | runif(n) | 100000 | guard_type_na | 28.0 µs | 28.0 µs–29.2 µs | 28.2 µs | 0 B |
| no_na &[f64] | runif(n) | 100000 | no_check | 656 ns | 656 ns–656 ns | 680 ns | 0 B |
| no_na &[f64] | runif(n) | 100000 | rust_any_na | 27.8 µs | 27.3 µs–27.8 µs | 27.8 µs | 0 B |
| no_na &[f64] | runif(n) | 100000 | rust_either_base | 697 ns | 656 ns–697 ns | 684 ns | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | guard_na | 2.70 ms | 2.69 ms–2.75 ms | 2.71 ms | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | guard_type_na | 2.71 ms | 2.69 ms–2.72 ms | 2.72 ms | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | no_check | 697 ns | 697 ns–697 ns | 690 ns | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | rust_any_na | 2.69 ms | 2.69 ms–2.78 ms | 2.70 ms | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | rust_either_base | 697 ns | 656 ns–697 ns | 688 ns | 0 B |
| no_na &[i32] | int(n) | 1 | guard_na | 1.03 µs | 1.03 µs–1.03 µs | 1.02 µs | 0 B |
| no_na &[i32] | int(n) | 1 | guard_type_na | 1.31 µs | 1.31 µs–1.31 µs | 1.31 µs | 0 B |
| no_na &[i32] | int(n) | 1 | no_check | 656 ns | 656 ns–697 ns | 666 ns | 0 B |
| no_na &[i32] | int(n) | 1 | rust_any_na | 697 ns | 697 ns–738 ns | 707 ns | 0 B |
| no_na &[i32] | int(n) | 1 | rust_either_base | 656 ns | 656 ns–697 ns | 669 ns | 0 B |
| no_na &[i32] | int(n) | 1000 | guard_na | 1.31 µs | 1.31 µs–1.31 µs | 1.33 µs | 0 B |
| no_na &[i32] | int(n) | 1000 | guard_type_na | 1.60 µs | 1.60 µs–1.60 µs | 1.61 µs | 0 B |
| no_na &[i32] | int(n) | 1000 | no_check | 656 ns | 656 ns–656 ns | 679 ns | 0 B |
| no_na &[i32] | int(n) | 1000 | rust_any_na | 738 ns | 738 ns–738 ns | 763 ns | 0 B |
| no_na &[i32] | int(n) | 1000 | rust_either_base | 697 ns | 656 ns–697 ns | 686 ns | 0 B |
| no_na &[i32] | int(n) | 100000 | guard_na | 28.4 µs | 27.9 µs–28.7 µs | 28.6 µs | 0 B |
| no_na &[i32] | int(n) | 100000 | guard_type_na | 28.7 µs | 28.2 µs–28.7 µs | 28.9 µs | 0 B |
| no_na &[i32] | int(n) | 100000 | no_check | 656 ns | 656 ns–656 ns | 683 ns | 0 B |
| no_na &[i32] | int(n) | 100000 | rust_any_na | 4.72 µs | 4.63 µs–4.76 µs | 4.75 µs | 0 B |
| no_na &[i32] | int(n) | 100000 | rust_either_base | 656 ns | 656 ns–697 ns | 678 ns | 0 B |
| no_na &[i32] | int(n) | 1e+07 | guard_na | 2.71 ms | 2.70 ms–2.76 ms | 2.71 ms | 0 B |
| no_na &[i32] | int(n) | 1e+07 | guard_type_na | 2.70 ms | 2.70 ms–2.76 ms | 2.71 ms | 0 B |
| no_na &[i32] | int(n) | 1e+07 | no_check | 697 ns | 656 ns–697 ns | 685 ns | 0 B |
| no_na &[i32] | int(n) | 1e+07 | rust_any_na | 405.5 µs | 398.7 µs–482.3 µs | 408.6 µs | 0 B |
| no_na &[i32] | int(n) | 1e+07 | rust_either_base | 697 ns | 656 ns–697 ns | 685 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | r_anyNA_guard | 1.72 µs | 1.72 µs–1.76 µs | 1.74 µs | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | raw_capi | 287 ns | 287 ns–328 ns | 303 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1 | sexp_floor | 656 ns | 656 ns–656 ns | 665 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | r_anyNA_guard | 3.77 µs | 3.69 µs–3.77 µs | 3.84 µs | 4.0 KB |
| scan classed dbl | structure(runif(n), class) | 1000 | raw_capi | 615 ns | 574 ns–615 ns | 601 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 293 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | sexp_floor | 697 ns | 656 ns–697 ns | 694 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | r_anyNA_guard | 188.8 µs | 188.3 µs–189.1 µs | 192.2 µs | 390.7 KB |
| scan classed dbl | structure(runif(n), class) | 100000 | raw_capi | 27.2 µs | 27.2 µs–27.2 µs | 27.2 µs | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | sexp_floor | 656 ns | 656 ns–697 ns | 684 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | r_anyNA_guard | 19.05 ms | 18.68 ms–19.10 ms | 19.24 ms | 38.1 MB |
| scan classed dbl | structure(runif(n), class) | 1e+07 | raw_capi | 2.68 ms | 2.64 ms–2.70 ms | 2.68 ms | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | sexp_floor | 697 ns | 656 ns–697 ns | 691 ns | 0 B |
| scan dbl | runif(n) | 1 | r_anyNA_guard | 984 ns | 984 ns–1.03 µs | 1.01 µs | 0 B |
| scan dbl | runif(n) | 1 | raw_capi | 328 ns | 328 ns–328 ns | 314 ns | 0 B |
| scan dbl | runif(n) | 1 | raw_elt | 328 ns | 328 ns–328 ns | 319 ns | 0 B |
| scan dbl | runif(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| scan dbl | runif(n) | 1 | raw_slice | 287 ns | 287 ns–328 ns | 309 ns | 0 B |
| scan dbl | runif(n) | 1 | sexp_floor | 656 ns | 615 ns–656 ns | 647 ns | 0 B |
| scan dbl | runif(n) | 1000 | r_anyNA_guard | 1.27 µs | 1.27 µs–1.31 µs | 1.30 µs | 0 B |
| scan dbl | runif(n) | 1000 | raw_capi | 615 ns | 615 ns–615 ns | 601 ns | 0 B |
| scan dbl | runif(n) | 1000 | raw_elt | 615 ns | 574 ns–615 ns | 604 ns | 0 B |
| scan dbl | runif(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan dbl | runif(n) | 1000 | raw_slice | 615 ns | 574 ns–615 ns | 604 ns | 0 B |
| scan dbl | runif(n) | 1000 | sexp_floor | 656 ns | 656 ns–656 ns | 657 ns | 0 B |
| scan dbl | runif(n) | 100000 | r_anyNA_guard | 28.1 µs | 27.6 µs–28.2 µs | 28.1 µs | 0 B |
| scan dbl | runif(n) | 100000 | raw_capi | 27.2 µs | 26.7 µs–27.2 µs | 27.2 µs | 0 B |
| scan dbl | runif(n) | 100000 | raw_elt | 27.2 µs | 26.7 µs–27.2 µs | 27.3 µs | 0 B |
| scan dbl | runif(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| scan dbl | runif(n) | 100000 | raw_slice | 27.2 µs | 26.7 µs–27.2 µs | 27.1 µs | 0 B |
| scan dbl | runif(n) | 100000 | sexp_floor | 656 ns | 656 ns–656 ns | 650 ns | 0 B |
| scan dbl | runif(n) | 1e+07 | r_anyNA_guard | 2.69 ms | 2.69 ms–2.76 ms | 2.69 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_capi | 2.69 ms | 2.68 ms–2.74 ms | 2.69 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_elt | 2.69 ms | 2.69 ms–2.73 ms | 2.71 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan dbl | runif(n) | 1e+07 | raw_slice | 2.68 ms | 2.68 ms–2.68 ms | 2.68 ms | 0 B |
| scan dbl | runif(n) | 1e+07 | sexp_floor | 656 ns | 656 ns–697 ns | 674 ns | 0 B |
| scan int | int(n) | 1 | r_anyNA_guard | 984 ns | 984 ns–1.03 µs | 1.00 µs | 0 B |
| scan int | int(n) | 1 | raw_capi | 328 ns | 328 ns–328 ns | 319 ns | 0 B |
| scan int | int(n) | 1 | raw_elt | 328 ns | 328 ns–328 ns | 321 ns | 0 B |
| scan int | int(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 294 ns | 0 B |
| scan int | int(n) | 1 | raw_slice | 328 ns | 287 ns–328 ns | 313 ns | 0 B |
| scan int | int(n) | 1 | sexp_floor | 656 ns | 656 ns–656 ns | 654 ns | 0 B |
| scan int | int(n) | 1000 | r_anyNA_guard | 1.31 µs | 1.27 µs–1.31 µs | 1.31 µs | 0 B |
| scan int | int(n) | 1000 | raw_capi | 369 ns | 328 ns–369 ns | 357 ns | 0 B |
| scan int | int(n) | 1000 | raw_elt | 369 ns | 369 ns–369 ns | 362 ns | 0 B |
| scan int | int(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 300 ns | 0 B |
| scan int | int(n) | 1000 | raw_slice | 369 ns | 328 ns–369 ns | 355 ns | 0 B |
| scan int | int(n) | 1000 | sexp_floor | 656 ns | 656 ns–656 ns | 660 ns | 0 B |
| scan int | int(n) | 100000 | r_anyNA_guard | 28.3 µs | 27.8 µs–28.3 µs | 28.2 µs | 0 B |
| scan int | int(n) | 100000 | raw_capi | 4.26 µs | 4.22 µs–4.30 µs | 4.28 µs | 0 B |
| scan int | int(n) | 100000 | raw_elt | 4.26 µs | 4.26 µs–4.26 µs | 4.28 µs | 0 B |
| scan int | int(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| scan int | int(n) | 100000 | raw_slice | 4.26 µs | 4.22 µs–4.26 µs | 4.26 µs | 0 B |
| scan int | int(n) | 100000 | sexp_floor | 656 ns | 656 ns–656 ns | 670 ns | 0 B |
| scan int | int(n) | 1e+07 | r_anyNA_guard | 2.72 ms | 2.71 ms–2.77 ms | 2.72 ms | 0 B |
| scan int | int(n) | 1e+07 | raw_capi | 413.5 µs | 407.4 µs–418.0 µs | 417.8 µs | 0 B |
| scan int | int(n) | 1e+07 | raw_elt | 411.0 µs | 410.2 µs–418.9 µs | 415.1 µs | 0 B |
| scan int | int(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| scan int | int(n) | 1e+07 | raw_slice | 405.0 µs | 401.1 µs–421.0 µs | 408.2 µs | 0 B |
| scan int | int(n) | 1e+07 | sexp_floor | 656 ns | 656 ns–656 ns | 666 ns | 0 B |

#### group: scalar

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| (i32, f64) | 1L, 2.5 | 1 | guarded | 2.13 µs | 2.09 µs–2.13 µs | 2.13 µs | 0 B |
| (i32, f64) | 1L, 2.5 | 1 | unguarded | 779 ns | 779 ns–820 ns | 783 ns | 0 B |
| &str | "abc" | 1 | guarded | 1.31 µs | 1.31 µs–1.35 µs | 1.34 µs | 0 B |
| &str | "abc" | 1 | unguarded | 656 ns | 656 ns–656 ns | 651 ns | 0 B |
| bool | TRUE | 1 | guarded | 1.31 µs | 1.31 µs–1.35 µs | 1.34 µs | 0 B |
| bool | TRUE | 1 | unguarded | 656 ns | 656 ns–656 ns | 648 ns | 0 B |
| f64 | 1.5 | 1 | guarded | 1.35 µs | 1.35 µs–1.35 µs | 1.37 µs | 0 B |
| f64 | 1.5 | 1 | unguarded | 697 ns | 656 ns–697 ns | 693 ns | 0 B |
| fast_i32 (existing) | 1L | 1 | guarded | 1.31 µs | 1.31 µs–1.35 µs | 1.35 µs | 0 B |
| fast_i32 (existing) | 1L | 1 | unguarded | 656 ns | 656 ns–656 ns | 677 ns | 0 B |
| fast_sum3 (existing) | 1L,2L,3L | 1 | guarded | 2.79 µs | 2.75 µs–2.87 µs | 2.78 µs | 0 B |
| fast_sum3 (existing) | 1L,2L,3L | 1 | unguarded | 861 ns | 820 ns–861 ns | 886 ns | 0 B |
| floors | 1.5 | 1 | r_guard_only | 574 ns | 574 ns–574 ns | 564 ns | 0 B |
| floors | 1.5 | 1 | raw_call | 287 ns | 287 ns–287 ns | 283 ns | 0 B |
| floors | 1.5 | 1 | raw_type_len | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| i32 | 1L | 1 | guarded | 1.35 µs | 1.31 µs–1.39 µs | 1.35 µs | 0 B |
| i32 | 1L | 1 | unguarded | 656 ns | 656 ns–656 ns | 660 ns | 0 B |
| Option<i32> | 1L | 1 | guarded | 1.39 µs | 1.39 µs–1.39 µs | 1.40 µs | 0 B |
| Option<i32> | 1L | 1 | unguarded | 656 ns | 656 ns–656 ns | 658 ns | 0 B |
| Option<i32> | NULL | 1 | guarded | 1.23 µs | 1.23 µs–1.27 µs | 1.26 µs | 0 B |
| Option<i32> | NULL | 1 | unguarded | 656 ns | 615 ns–656 ns | 652 ns | 0 B |
| String | "abc" | 1 | guarded | 1.35 µs | 1.35 µs–1.39 µs | 1.39 µs | 0 B |
| String | "abc" | 1 | unguarded | 697 ns | 656 ns–697 ns | 695 ns | 0 B |

#### group: vector

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| &[f64] | runif(n) | 1 | guarded | 984 ns | 943 ns–984 ns | 986 ns | 0 B |
| &[f64] | runif(n) | 1 | unguarded | 656 ns | 656 ns–656 ns | 675 ns | 0 B |
| &[f64] | runif(n) | 1000 | guarded | 984 ns | 984 ns–984 ns | 979 ns | 0 B |
| &[f64] | runif(n) | 1000 | unguarded | 656 ns | 656 ns–656 ns | 678 ns | 0 B |
| &[f64] | runif(n) | 100000 | guarded | 984 ns | 984 ns–984 ns | 987 ns | 0 B |
| &[f64] | runif(n) | 100000 | unguarded | 656 ns | 656 ns–697 ns | 667 ns | 0 B |
| &[f64] | runif(n) | 1e+07 | guarded | 943 ns | 943 ns–943 ns | 961 ns | 0 B |
| &[f64] | runif(n) | 1e+07 | unguarded | 656 ns | 656 ns–697 ns | 672 ns | 0 B |
| &[i32] | int(n) | 1 | guarded | 984 ns | 943 ns–984 ns | 981 ns | 0 B |
| &[i32] | int(n) | 1 | unguarded | 656 ns | 656 ns–656 ns | 672 ns | 0 B |
| &[i32] | int(n) | 1000 | guarded | 984 ns | 943 ns–984 ns | 986 ns | 0 B |
| &[i32] | int(n) | 1000 | unguarded | 656 ns | 656 ns–697 ns | 670 ns | 0 B |
| &[i32] | int(n) | 100000 | guarded | 984 ns | 943 ns–984 ns | 984 ns | 0 B |
| &[i32] | int(n) | 100000 | unguarded | 656 ns | 656 ns–697 ns | 667 ns | 0 B |
| &[i32] | int(n) | 1e+07 | guarded | 984 ns | 943 ns–984 ns | 980 ns | 0 B |
| &[i32] | int(n) | 1e+07 | unguarded | 656 ns | 656 ns–697 ns | 668 ns | 0 B |
| Vec<f64> | runif(n) | 1 | guarded | 984 ns | 984 ns–984 ns | 997 ns | 0 B |
| Vec<f64> | runif(n) | 1 | unguarded | 656 ns | 656 ns–697 ns | 686 ns | 0 B |
| Vec<f64> | runif(n) | 1000 | guarded | 1.15 µs | 1.15 µs–1.15 µs | 1.15 µs | 0 B |
| Vec<f64> | runif(n) | 1000 | unguarded | 820 ns | 820 ns–820 ns | 845 ns | 0 B |
| Vec<f64> | runif(n) | 100000 | guarded | 10.7 µs | 10.7 µs–10.9 µs | 10.8 µs | 0 B |
| Vec<f64> | runif(n) | 100000 | unguarded | 10.4 µs | 10.3 µs–10.6 µs | 10.7 µs | 0 B |
| Vec<f64> | runif(n) | 1e+07 | guarded | 1.23 ms | 1.23 ms–1.25 ms | 1.23 ms | 0 B |
| Vec<f64> | runif(n) | 1e+07 | unguarded | 1.23 ms | 1.23 ms–1.28 ms | 1.23 ms | 0 B |
| Vec<i32> | int(n) | 1 | guarded | 984 ns | 984 ns–1.03 µs | 999 ns | 0 B |
| Vec<i32> | int(n) | 1 | unguarded | 697 ns | 697 ns–697 ns | 705 ns | 0 B |
| Vec<i32> | int(n) | 1000 | guarded | 1.07 µs | 1.07 µs–1.11 µs | 1.10 µs | 0 B |
| Vec<i32> | int(n) | 1000 | unguarded | 779 ns | 779 ns–779 ns | 784 ns | 0 B |
| Vec<i32> | int(n) | 100000 | guarded | 6.11 µs | 6.07 µs–6.23 µs | 6.13 µs | 0 B |
| Vec<i32> | int(n) | 100000 | unguarded | 5.74 µs | 5.70 µs–5.86 µs | 5.80 µs | 0 B |
| Vec<i32> | int(n) | 1e+07 | guarded | 583.6 µs | 583.5 µs–605.7 µs | 584.9 µs | 0 B |
| Vec<i32> | int(n) | 1e+07 | unguarded | 582.0 µs | 581.5 µs–595.4 µs | 584.7 µs | 0 B |

#### group: whole

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | guarded | 7.95 µs | 7.83 µs–8.77 µs | 8.01 µs | 27.6 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | raw_capi | 1.03 µs | 984 ns–1.03 µs | 1.03 µs | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | raw_floor | 410 ns | 410 ns–410 ns | 407 ns | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | raw_slice | 1.19 µs | 1.11 µs–1.19 µs | 1.20 µs | 7.9 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | unguarded | 2.13 µs | 2.09 µs–2.13 µs | 2.16 µs | 7.9 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | guarded | 593.9 µs | 570.0 µs–696.0 µs | 614.3 µs | 2.7 MB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | raw_capi | 56.3 µs | 55.3 µs–56.5 µs | 56.5 µs | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | raw_floor | 410 ns | 410 ns–410 ns | 412 ns | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | raw_slice | 52.1 µs | 51.1 µs–53.2 µs | 52.8 µs | 781.3 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | unguarded | 119.4 µs | 116.7 µs–121.8 µs | 131.5 µs | 781.3 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | guarded | 60.30 ms | 57.59 ms–60.58 ms | 60.75 ms | 267.0 MB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | raw_capi | 5.58 ms | 5.49 ms–5.63 ms | 5.56 ms | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | raw_floor | 410 ns | 410 ns–410 ns | 414 ns | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | raw_slice | 5.15 ms | 5.06 ms–5.30 ms | 5.21 ms | 76.3 MB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | unguarded | 11.99 ms | 11.57 ms–12.06 ms | 12.26 ms | 76.3 MB |
| coerce Vec<i32> from doubles | whole doubles | 1 | guarded | 1.44 µs | 1.39 µs–1.44 µs | 1.44 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | r_guard_expr | 738 ns | 697 ns–738 ns | 730 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | raw_capi | 287 ns | 287 ns–287 ns | 308 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | raw_slice | 328 ns | 287 ns–328 ns | 316 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | unguarded | 697 ns | 697 ns–697 ns | 696 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | guarded | 6.56 µs | 6.35 µs–7.26 µs | 6.83 µs | 19.7 KB |
| coerce Vec<i32> from doubles | whole doubles | 1000 | r_guard_expr | 4.76 µs | 4.67 µs–4.76 µs | 4.79 µs | 19.7 KB |
| coerce Vec<i32> from doubles | whole doubles | 1000 | raw_capi | 738 ns | 697 ns–738 ns | 726 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 298 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | raw_slice | 738 ns | 697 ns–738 ns | 736 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | unguarded | 1.80 µs | 1.72 µs–1.80 µs | 1.80 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | guarded | 540.5 µs | 479.5 µs–580.3 µs | 556.4 µs | 1.9 MB |
| coerce Vec<i32> from doubles | whole doubles | 100000 | r_guard_expr | 365.4 µs | 353.4 µs–367.4 µs | 368.0 µs | 1.9 MB |
| coerce Vec<i32> from doubles | whole doubles | 100000 | raw_capi | 40.4 µs | 39.6 µs–40.4 µs | 40.5 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | raw_slice | 40.4 µs | 39.6 µs–40.4 µs | 40.3 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | unguarded | 107.3 µs | 105.2 µs–107.3 µs | 109.3 µs | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | guarded | 48.93 ms | 46.53 ms–56.67 ms | 52.13 ms | 190.7 MB |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | r_guard_expr | 37.87 ms | 37.27 ms–38.17 ms | 41.24 ms | 190.7 MB |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | raw_capi | 4.08 ms | 3.94 ms–4.10 ms | 4.09 ms | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 299 ns | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | raw_slice | 4.03 ms | 3.94 ms–4.09 ms | 4.04 ms | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | unguarded | 10.85 ms | 10.45 ms–19.47 ms | 11.01 ms | 0 B |


### Derived deltas (trimmed means)

#### Scalars: default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| (i32, f64) | 1L, 2.5 | 1 | 2.13 µs | 783 ns | 1.35 µs | 63% | 0 B | 0 B |
| &str | "abc" | 1 | 1.34 µs | 651 ns | 691 ns | 51% | 0 B | 0 B |
| bool | TRUE | 1 | 1.34 µs | 648 ns | 687 ns | 51% | 0 B | 0 B |
| f64 | 1.5 | 1 | 1.37 µs | 693 ns | 676 ns | 49% | 0 B | 0 B |
| fast_i32 (existing) | 1L | 1 | 1.35 µs | 677 ns | 669 ns | 50% | 0 B | 0 B |
| fast_sum3 (existing) | 1L,2L,3L | 1 | 2.78 µs | 886 ns | 1.90 µs | 68% | 0 B | 0 B |
| i32 | 1L | 1 | 1.35 µs | 660 ns | 690 ns | 51% | 0 B | 0 B |
| Option<i32> | 1L | 1 | 1.40 µs | 658 ns | 746 ns | 53% | 0 B | 0 B |
| Option<i32> | NULL | 1 | 1.26 µs | 652 ns | 607 ns | 48% | 0 B | 0 B |
| String | "abc" | 1 | 1.39 µs | 695 ns | 693 ns | 50% | 0 B | 0 B |

#### Vectors: default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| &[f64] | runif(n) | 1 | 986 ns | 675 ns | 311 ns | 32% | 0 B | 0 B |
| &[f64] | runif(n) | 1000 | 979 ns | 678 ns | 301 ns | 31% | 0 B | 0 B |
| &[f64] | runif(n) | 100000 | 987 ns | 667 ns | 320 ns | 32% | 0 B | 0 B |
| &[f64] | runif(n) | 1e+07 | 961 ns | 672 ns | 290 ns | 30% | 0 B | 0 B |
| &[i32] | int(n) | 1 | 981 ns | 672 ns | 309 ns | 31% | 0 B | 0 B |
| &[i32] | int(n) | 1000 | 986 ns | 670 ns | 316 ns | 32% | 0 B | 0 B |
| &[i32] | int(n) | 100000 | 984 ns | 667 ns | 317 ns | 32% | 0 B | 0 B |
| &[i32] | int(n) | 1e+07 | 980 ns | 668 ns | 312 ns | 32% | 0 B | 0 B |
| Vec<f64> | runif(n) | 1 | 997 ns | 686 ns | 310 ns | 31% | 0 B | 0 B |
| Vec<f64> | runif(n) | 1000 | 1.15 µs | 845 ns | 310 ns | 27% | 0 B | 0 B |
| Vec<f64> | runif(n) | 100000 | 10.8 µs | 10.7 µs | 90 ns | 1% | 0 B | 0 B |
| Vec<f64> | runif(n) | 1e+07 | 1.23 ms | 1.23 ms | 4.01 µs | 0% | 0 B | 0 B |
| Vec<i32> | int(n) | 1 | 999 ns | 705 ns | 294 ns | 29% | 0 B | 0 B |
| Vec<i32> | int(n) | 1000 | 1.10 µs | 784 ns | 315 ns | 29% | 0 B | 0 B |
| Vec<i32> | int(n) | 100000 | 6.13 µs | 5.80 µs | 332 ns | 5% | 0 B | 0 B |
| Vec<i32> | int(n) | 1e+07 | 584.9 µs | 584.7 µs | 203 ns | 0% | 0 B | 0 B |

#### no_na typed: R is.double/is.integer + anyNA vs no check

| case | input | n | R type + anyNA | no check | saved | share of R type + anyNA call | R heap R type + anyNA | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | 1.31 µs | 669 ns | 641 ns | 49% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1000 | 1.60 µs | 676 ns | 925 ns | 58% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 100000 | 28.2 µs | 680 ns | 27.5 µs | 98% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | 2.72 ms | 690 ns | 2.72 ms | 100% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1 | 1.31 µs | 666 ns | 647 ns | 49% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1000 | 1.61 µs | 679 ns | 930 ns | 58% | 0 B | 0 B |
| no_na &[i32] | int(n) | 100000 | 28.9 µs | 683 ns | 28.2 µs | 98% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1e+07 | 2.71 ms | 685 ns | 2.71 ms | 100% | 0 B | 0 B |

#### no_na typed: R anyNA only vs no check

| case | input | n | R anyNA | no check | saved | share of R anyNA call | R heap R anyNA | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | 1.02 µs | 669 ns | 349 ns | 34% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1000 | 1.30 µs | 676 ns | 625 ns | 48% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 100000 | 27.8 µs | 680 ns | 27.1 µs | 98% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | 2.71 ms | 690 ns | 2.71 ms | 100% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1 | 1.02 µs | 666 ns | 350 ns | 34% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1000 | 1.33 µs | 679 ns | 652 ns | 49% | 0 B | 0 B |
| no_na &[i32] | int(n) | 100000 | 28.6 µs | 683 ns | 27.9 µs | 98% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1e+07 | 2.71 ms | 685 ns | 2.71 ms | 100% | 0 B | 0 B |

#### no_na typed: Rust any_na (Either) vs Either without check

| case | input | n | Rust any_na | no check | saved | share of Rust any_na call | R heap Rust any_na | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[f64] | runif(n) | 1 | 702 ns | 670 ns | 33 ns | 5% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1000 | 1.01 µs | 687 ns | 327 ns | 32% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 100000 | 27.8 µs | 684 ns | 27.1 µs | 98% | 0 B | 0 B |
| no_na &[f64] | runif(n) | 1e+07 | 2.70 ms | 688 ns | 2.70 ms | 100% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1 | 707 ns | 669 ns | 38 ns | 5% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1000 | 763 ns | 686 ns | 77 ns | 10% | 0 B | 0 B |
| no_na &[i32] | int(n) | 100000 | 4.75 µs | 678 ns | 4.07 µs | 86% | 0 B | 0 B |
| no_na &[i32] | int(n) | 1e+07 | 408.6 µs | 685 ns | 407.9 µs | 100% | 0 B | 0 B |

#### NA scan, R anyNA guard vs bare-SEXP floor

| case | input | n | R anyNA | floor | saved | share of R anyNA call | R heap R anyNA | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan classed dbl | structure(runif(n), class) | 1 | 1.74 µs | 665 ns | 1.07 µs | 62% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | 3.84 µs | 694 ns | 3.15 µs | 82% | 4.0 KB | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | 192.2 µs | 684 ns | 191.5 µs | 100% | 390.7 KB | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | 19.24 ms | 691 ns | 19.23 ms | 100% | 38.1 MB | 0 B |
| scan dbl | runif(n) | 1 | 1.01 µs | 647 ns | 363 ns | 36% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 1.30 µs | 657 ns | 648 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 28.1 µs | 650 ns | 27.5 µs | 98% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.69 ms | 674 ns | 2.69 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 1.00 µs | 654 ns | 351 ns | 35% | 0 B | 0 B |
| scan int | int(n) | 1000 | 1.31 µs | 660 ns | 647 ns | 50% | 0 B | 0 B |
| scan int | int(n) | 100000 | 28.2 µs | 670 ns | 27.6 µs | 98% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 2.72 ms | 666 ns | 2.72 ms | 100% | 0 B | 0 B |

#### NA scan, raw C DATAPTR slice vs raw floor

| case | input | n | slice scan | floor | saved | share of slice scan call | R heap slice scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan dbl | runif(n) | 1 | 309 ns | 298 ns | 11 ns | 3% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 604 ns | 300 ns | 303 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 27.1 µs | 299 ns | 26.8 µs | 99% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.68 ms | 300 ns | 2.68 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 313 ns | 294 ns | 18 ns | 6% | 0 B | 0 B |
| scan int | int(n) | 1000 | 355 ns | 300 ns | 55 ns | 15% | 0 B | 0 B |
| scan int | int(n) | 100000 | 4.26 µs | 298 ns | 3.97 µs | 93% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 408.2 µs | 299 ns | 407.9 µs | 100% | 0 B | 0 B |

#### NA scan, raw C *_ELT (from_r::any_na shape) vs raw floor

| case | input | n | ELT scan | floor | saved | share of ELT scan call | R heap ELT scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan dbl | runif(n) | 1 | 319 ns | 298 ns | 21 ns | 7% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 604 ns | 300 ns | 304 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 27.3 µs | 299 ns | 27.0 µs | 99% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.71 ms | 300 ns | 2.71 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 321 ns | 294 ns | 26 ns | 8% | 0 B | 0 B |
| scan int | int(n) | 1000 | 362 ns | 300 ns | 62 ns | 17% | 0 B | 0 B |
| scan int | int(n) | 100000 | 4.28 µs | 298 ns | 3.98 µs | 93% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 415.1 µs | 299 ns | 414.8 µs | 100% | 0 B | 0 B |

#### NA scan, raw C NO_NA+OR_NULL+GET_REGION vs raw floor

| case | input | n | C-API scan | floor | saved | share of C-API scan call | R heap C-API scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan classed dbl | structure(runif(n), class) | 1 | 303 ns | 296 ns | 7 ns | 2% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 1000 | 601 ns | 293 ns | 308 ns | 51% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 100000 | 27.2 µs | 300 ns | 26.9 µs | 99% | 0 B | 0 B |
| scan classed dbl | structure(runif(n), class) | 1e+07 | 2.68 ms | 299 ns | 2.68 ms | 100% | 0 B | 0 B |
| scan dbl | runif(n) | 1 | 314 ns | 298 ns | 15 ns | 5% | 0 B | 0 B |
| scan dbl | runif(n) | 1000 | 601 ns | 300 ns | 301 ns | 50% | 0 B | 0 B |
| scan dbl | runif(n) | 100000 | 27.2 µs | 299 ns | 26.9 µs | 99% | 0 B | 0 B |
| scan dbl | runif(n) | 1e+07 | 2.69 ms | 300 ns | 2.69 ms | 100% | 0 B | 0 B |
| scan int | int(n) | 1 | 319 ns | 294 ns | 25 ns | 8% | 0 B | 0 B |
| scan int | int(n) | 1000 | 357 ns | 300 ns | 57 ns | 16% | 0 B | 0 B |
| scan int | int(n) | 100000 | 4.28 µs | 298 ns | 3.99 µs | 93% | 0 B | 0 B |
| scan int | int(n) | 1e+07 | 417.8 µs | 299 ns | 417.5 µs | 100% | 0 B | 0 B |

#### ALTREP: R anyNA guard vs AltrepSexp floor

| case | input | n | R anyNA | floor | saved | share of R anyNA call | R heap R anyNA | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 1.12 µs | 777 ns | 345 ns | 31% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 1.14 µs | 776 ns | 368 ns | 32% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 1.12 µs | 781 ns | 342 ns | 30% | 0 B | 0 B |
| scan compact int | 1:n | 1000 | 1.10 µs | 728 ns | 376 ns | 34% | 0 B | 0 B |
| scan compact int | 1:n | 100000 | 1.07 µs | 721 ns | 345 ns | 32% | 0 B | 0 B |
| scan compact int | 1:n | 1e+07 | 1.07 µs | 725 ns | 346 ns | 32% | 0 B | 0 B |

#### ALTREP: raw C-API scan vs raw floor

| case | input | n | C-API scan | floor | saved | share of C-API scan call | R heap C-API scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 420 ns | 406 ns | 15 ns | 3% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 426 ns | 404 ns | 21 ns | 5% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 410 ns | 397 ns | 12 ns | 3% | 0 B | 0 B |
| scan compact int | 1:n | 1000 | 377 ns | 359 ns | 18 ns | 5% | 0 B | 0 B |
| scan compact int | 1:n | 100000 | 375 ns | 358 ns | 17 ns | 5% | 0 B | 0 B |
| scan compact int | 1:n | 1e+07 | 378 ns | 359 ns | 19 ns | 5% | 0 B | 0 B |

#### ALTREP: raw *_ELT scan vs raw floor

| case | input | n | ELT scan | floor | saved | share of ELT scan call | R heap ELT scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 6.31 µs | 406 ns | 5.90 µs | 94% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 591.8 µs | 404 ns | 591.4 µs | 100% | 0 B | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 58.55 ms | 397 ns | 58.55 ms | 100% | 0 B | 0 B |
| scan compact int | 1:n | 1000 | 6.27 µs | 359 ns | 5.91 µs | 94% | 0 B | 0 B |
| scan compact int | 1:n | 100000 | 585.6 µs | 358 ns | 585.3 µs | 100% | 0 B | 0 B |
| scan compact int | 1:n | 1e+07 | 58.59 ms | 359 ns | 58.58 ms | 100% | 0 B | 0 B |

#### ALTREP: raw DATAPTR slice scan vs raw floor

| case | input | n | slice scan | floor | saved | share of slice scan call | R heap slice scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| scan compact dbl | as.double(1:n) | 1000 | 1.08 µs | 406 ns | 674 ns | 62% | 7.9 KB | 0 B |
| scan compact dbl | as.double(1:n) | 100000 | 39.4 µs | 404 ns | 39.0 µs | 99% | 781.3 KB | 0 B |
| scan compact dbl | as.double(1:n) | 1e+07 | 4.00 ms | 397 ns | 4.00 ms | 100% | 76.3 MB | 0 B |
| scan compact int | 1:n | 1000 | 615 ns | 359 ns | 256 ns | 42% | 4.0 KB | 0 B |
| scan compact int | 1:n | 100000 | 8.70 µs | 358 ns | 8.34 µs | 96% | 390.7 KB | 0 B |
| scan compact int | 1:n | 1e+07 | 1.01 ms | 359 ns | 1.01 ms | 100% | 38.1 MB | 0 B |

#### ALTREP: typed R anyNA vs no check

| case | input | n | R anyNA | no check | saved | share of R anyNA call | R heap R anyNA | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[i32] compact | 1:n | 1000 | 1.40 µs | 1.06 µs | 344 ns | 25% | 4.0 KB | 4.0 KB |
| no_na &[i32] compact | 1:n | 100000 | 5.42 µs | 5.04 µs | 378 ns | 7% | 390.7 KB | 390.7 KB |
| no_na &[i32] compact | 1:n | 1e+07 | 405.3 µs | 403.9 µs | 1.46 µs | 0% | 38.1 MB | 38.1 MB |

#### ALTREP: typed Rust any_na vs Either base

| case | input | n | Rust any_na | no check | saved | share of Rust any_na call | R heap Rust any_na | R heap no check |
|---|---|---|---|---|---|---|---|---|
| no_na &[i32] compact | 1:n | 1000 | 6.34 µs | 1.04 µs | 5.30 µs | 84% | 4.0 KB | 4.0 KB |
| no_na &[i32] compact | 1:n | 100000 | 544.6 µs | 5.14 µs | 539.4 µs | 99% | 390.7 KB | 390.7 KB |
| no_na &[i32] compact | 1:n | 1e+07 | 53.80 ms | 405.7 µs | 53.39 ms | 99% | 38.1 MB | 38.1 MB |

#### ALTREP: &[i32] default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| &[i32] compact | 1:n | 1000 | 1.35 µs | 1.04 µs | 311 ns | 23% | 4.0 KB | 4.0 KB |
| &[i32] compact | 1:n | 100000 | 5.51 µs | 5.17 µs | 340 ns | 6% | 390.7 KB | 390.7 KB |
| &[i32] compact | 1:n | 1e+07 | 408.0 µs | 405.1 µs | 2.86 µs | 1% | 38.1 MB | 38.1 MB |

#### Whole-number: coerce default vs no_preconditions

| case | input | n | R guard + Rust | Rust only | saved | share of R guard + Rust call | R heap R guard + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | 8.01 µs | 2.16 µs | 5.85 µs | 73% | 27.6 KB | 7.9 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | 614.3 µs | 131.5 µs | 482.8 µs | 79% | 2.7 MB | 781.3 KB |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | 60.75 ms | 12.26 ms | 48.49 ms | 80% | 267.0 MB | 76.3 MB |
| coerce Vec<i32> from doubles | whole doubles | 1 | 1.44 µs | 696 ns | 740 ns | 52% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | 6.83 µs | 1.80 µs | 5.03 µs | 74% | 19.7 KB | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | 556.4 µs | 109.3 µs | 447.1 µs | 80% | 1.9 MB | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | 52.13 ms | 11.01 ms | 41.13 ms | 79% | 190.7 MB | 0 B |

#### Whole-number: raw C slice scan vs raw floor

| case | input | n | slice scan | floor | saved | share of slice scan call | R heap slice scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | 1.20 µs | 407 ns | 788 ns | 66% | 7.9 KB | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | 52.8 µs | 412 ns | 52.4 µs | 99% | 781.3 KB | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | 5.21 ms | 414 ns | 5.21 ms | 100% | 76.3 MB | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | 316 ns | 298 ns | 18 ns | 6% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | 736 ns | 298 ns | 438 ns | 60% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | 40.3 µs | 299 ns | 40.0 µs | 99% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | 4.04 ms | 299 ns | 4.04 ms | 100% | 0 B | 0 B |

#### Whole-number: raw C OR_NULL/GET_REGION scan vs raw floor

| case | input | n | C-API scan | floor | saved | share of C-API scan call | R heap C-API scan | R heap floor |
|---|---|---|---|---|---|---|---|---|
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1000 | 1.03 µs | 407 ns | 626 ns | 61% | 0 B | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 100000 | 56.5 µs | 412 ns | 56.1 µs | 99% | 0 B | 0 B |
| coerce Vec<i32> from compact dbl | as.double(1:n) | 1e+07 | 5.56 ms | 414 ns | 5.56 ms | 100% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1 | 308 ns | 298 ns | 10 ns | 3% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1000 | 726 ns | 298 ns | 429 ns | 59% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 100000 | 40.5 µs | 299 ns | 40.2 µs | 99% | 0 B | 0 B |
| coerce Vec<i32> from doubles | whole doubles | 1e+07 | 4.09 ms | 299 ns | 4.09 ms | 100% | 0 B | 0 B |

#### inherits List: inherits+is.list vs no check

| case | input | n | R inherits+is.list | no check | saved | share of R inherits+is.list call | R heap R inherits+is.list | R heap no check |
|---|---|---|---|---|---|---|---|---|
| inherits List | list, class mx_cls | 3 | 1.41 µs | 695 ns | 713 ns | 51% | 0 B | 0 B |

#### inherits List: inherits only vs no check

| case | input | n | R inherits | no check | saved | share of R inherits call | R heap R inherits | R heap no check |
|---|---|---|---|---|---|---|---|---|
| inherits List | list, class mx_cls | 3 | 1.08 µs | 695 ns | 389 ns | 36% | 0 B | 0 B |

#### inherits: R guard vs bare-SEXP floor

| case | input | n | R inherits | floor | saved | share of R inherits call | R heap R inherits | R heap floor |
|---|---|---|---|---|---|---|---|---|
| inherits scan | list, 4 classes, match last | 3 | 1.05 µs | 668 ns | 378 ns | 36% | 0 B | 0 B |
| inherits scan | list, class mx_cls | 3 | 1.06 µs | 663 ns | 397 ns | 37% | 0 B | 0 B |

#### inherits: raw Rf_inherits vs raw floor

| case | input | n | Rf_inherits | floor | saved | share of Rf_inherits call | R heap Rf_inherits | R heap floor |
|---|---|---|---|---|---|---|---|---|
| inherits scan | list, 4 classes, match last | 3 | 326 ns | 294 ns | 32 ns | 10% | 0 B | 0 B |
| inherits scan | list, class mx_cls | 3 | 323 ns | 293 ns | 29 ns | 9% | 0 B | 0 B |

#### DataFrame: R inherits guard vs none

| case | input | n | R inherits | none | saved | share of R inherits call | R heap R inherits | R heap none |
|---|---|---|---|---|---|---|---|---|
| DataFrame, 5 cols | data.frame | 10 | 1.44 µs | 1.04 µs | 394 ns | 27% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | 1.43 µs | 1.05 µs | 383 ns | 27% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | 1.44 µs | 1.04 µs | 392 ns | 27% | 0 B | 0 B |

#### DataFrame: R anyNA guard vs none

| case | input | n | R anyNA | none | saved | share of R anyNA call | R heap R anyNA | R heap none |
|---|---|---|---|---|---|---|---|---|
| DataFrame, 5 cols | data.frame | 10 | 4.50 µs | 1.04 µs | 3.46 µs | 77% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 1000 | 6.35 µs | 1.05 µs | 5.30 µs | 83% | 0 B | 0 B |
| DataFrame, 5 cols | data.frame | 100000 | 148.0 µs | 1.04 µs | 146.9 µs | 99% | 0 B | 0 B |

#### choices &str: R helper vs &str floor

| case | input | n | R helper | &str floor | saved | share of R helper call | R heap R helper | R heap &str floor |
|---|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | 2.25 µs | 677 ns | 1.57 µs | 70% | 0 B | 0 B |
| choices &str / match_arg enum | "slow" | 1 | 2.25 µs | 662 ns | 1.59 µs | 71% | 0 B | 0 B |

#### match_arg enum: R helper + Rust vs Rust matching only

| case | input | n | R helper + Rust | Rust only | saved | share of R helper + Rust call | R heap R helper + Rust | R heap Rust only |
|---|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | 2.30 µs | 761 ns | 1.54 µs | 67% | 0 B | 0 B |
| choices &str / match_arg enum | "slow" | 1 | 2.26 µs | 695 ns | 1.57 µs | 69% | 0 B | 0 B |

#### match_arg enum: Rust matching vs &str floor

| case | input | n | Rust match | &str floor | saved | share of Rust match call | R heap Rust match | R heap &str floor |
|---|---|---|---|---|---|---|---|---|
| choices &str / match_arg enum | "sl" (prefix) | 1 | 761 ns | 677 ns | 83 ns | 11% | 0 B | 0 B |
| choices &str / match_arg enum | "slow" | 1 | 695 ns | 662 ns | 32 ns | 5% | 0 B | 0 B |

</details>

## Appendix B: per-variant summary, chunked run

The chunked-scan run (`summary-chunked.md`): R's `anyNA()` and whole-number expression against the C-API scans, plain and chunked.

<details>
<summary>Per-variant summary</summary>

### Raw per-variant results (median of 3 rep medians; [min-max of rep medians]; trimmed mean; R-heap bytes)


#### group: chunked

| case | input | n | variant | median | rep range | trimmed mean | R heap |
|---|---|---|---|---|---|---|---|
| NA scan dbl | runif(n) | 1 | r_anyNA | 1 ns | 1 ns–1 ns | 10 ns | 0 B |
| NA scan dbl | runif(n) | 1 | raw_capi | 287 ns | 287 ns–328 ns | 305 ns | 0 B |
| NA scan dbl | runif(n) | 1 | raw_chunked | 328 ns | 328 ns–328 ns | 311 ns | 0 B |
| NA scan dbl | runif(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 295 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | r_anyNA | 287 ns | 287 ns–287 ns | 285 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | raw_capi | 574 ns | 574 ns–574 ns | 590 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | raw_chunked | 451 ns | 451 ns–451 ns | 454 ns | 0 B |
| NA scan dbl | runif(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 302 ns | 0 B |
| NA scan dbl | runif(n) | 100000 | r_anyNA | 26.3 µs | 26.3 µs–26.3 µs | 26.3 µs | 0 B |
| NA scan dbl | runif(n) | 100000 | raw_capi | 26.7 µs | 26.7 µs–26.7 µs | 26.7 µs | 0 B |
| NA scan dbl | runif(n) | 100000 | raw_chunked | 13.7 µs | 13.7 µs–13.9 µs | 13.7 µs | 0 B |
| NA scan dbl | runif(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 294 ns | 0 B |
| NA scan dbl | runif(n) | 1e+07 | r_anyNA | 2.64 ms | 2.64 ms–2.65 ms | 2.65 ms | 0 B |
| NA scan dbl | runif(n) | 1e+07 | raw_capi | 2.63 ms | 2.63 ms–2.63 ms | 2.64 ms | 0 B |
| NA scan dbl | runif(n) | 1e+07 | raw_chunked | 1.34 ms | 1.34 ms–1.36 ms | 1.35 ms | 0 B |
| NA scan dbl | runif(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 293 ns | 0 B |
| NA scan int | int(n) | 1 | r_anyNA | 1 ns | 1 ns–1 ns | 8 ns | 0 B |
| NA scan int | int(n) | 1 | raw_capi | 328 ns | 287 ns–328 ns | 310 ns | 0 B |
| NA scan int | int(n) | 1 | raw_chunked | 328 ns | 287 ns–328 ns | 310 ns | 0 B |
| NA scan int | int(n) | 1 | raw_floor | 287 ns | 287 ns–287 ns | 297 ns | 0 B |
| NA scan int | int(n) | 1000 | r_anyNA | 287 ns | 287 ns–287 ns | 288 ns | 0 B |
| NA scan int | int(n) | 1000 | raw_capi | 328 ns | 328 ns–328 ns | 346 ns | 0 B |
| NA scan int | int(n) | 1000 | raw_chunked | 369 ns | 369 ns–369 ns | 361 ns | 0 B |
| NA scan int | int(n) | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| NA scan int | int(n) | 100000 | r_anyNA | 26.4 µs | 26.4 µs–26.4 µs | 26.4 µs | 0 B |
| NA scan int | int(n) | 100000 | raw_capi | 4.14 µs | 4.14 µs–4.18 µs | 4.18 µs | 0 B |
| NA scan int | int(n) | 100000 | raw_chunked | 5.45 µs | 5.45 µs–5.45 µs | 5.48 µs | 0 B |
| NA scan int | int(n) | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| NA scan int | int(n) | 1e+07 | r_anyNA | 2.65 ms | 2.65 ms–2.66 ms | 2.65 ms | 0 B |
| NA scan int | int(n) | 1e+07 | raw_capi | 386.3 µs | 385.2 µs–387.1 µs | 389.1 µs | 0 B |
| NA scan int | int(n) | 1e+07 | raw_chunked | 509.7 µs | 509.6 µs–509.7 µs | 511.3 µs | 0 B |
| NA scan int | int(n) | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 295 ns | 0 B |
| whole-number scan | as.double(1:n) | 1000 | r_expr | 5.82 µs | 5.78 µs–5.82 µs | 5.85 µs | 35.4 KB |
| whole-number scan | as.double(1:n) | 1000 | raw_capi | 984 ns | 984 ns–1.03 µs | 1.00 µs | 0 B |
| whole-number scan | as.double(1:n) | 1000 | raw_chunked | 779 ns | 738 ns–779 ns | 765 ns | 0 B |
| whole-number scan | as.double(1:n) | 1000 | raw_floor | 410 ns | 410 ns–410 ns | 400 ns | 0 B |
| whole-number scan | as.double(1:n) | 100000 | r_expr | 474.5 µs | 474.5 µs–475.2 µs | 479.8 µs | 3.4 MB |
| whole-number scan | as.double(1:n) | 100000 | raw_capi | 55.2 µs | 55.2 µs–55.3 µs | 55.3 µs | 0 B |
| whole-number scan | as.double(1:n) | 100000 | raw_chunked | 31.8 µs | 31.8 µs–31.8 µs | 31.9 µs | 0 B |
| whole-number scan | as.double(1:n) | 100000 | raw_floor | 410 ns | 410 ns–410 ns | 401 ns | 0 B |
| whole-number scan | as.double(1:n) | 1e+07 | r_expr | 49.11 ms | 49.07 ms–50.17 ms | 51.89 ms | 343.3 MB |
| whole-number scan | as.double(1:n) | 1e+07 | raw_capi | 5.49 ms | 5.49 ms–5.49 ms | 5.49 ms | 0 B |
| whole-number scan | as.double(1:n) | 1e+07 | raw_chunked | 3.14 ms | 3.14 ms–3.14 ms | 3.14 ms | 0 B |
| whole-number scan | as.double(1:n) | 1e+07 | raw_floor | 410 ns | 410 ns–410 ns | 400 ns | 0 B |
| whole-number scan | whole doubles | 1 | r_expr | 205 ns | 205 ns–205 ns | 220 ns | 0 B |
| whole-number scan | whole doubles | 1 | raw_capi | 287 ns | 287 ns–287 ns | 304 ns | 0 B |
| whole-number scan | whole doubles | 1 | raw_chunked | 287 ns | 287 ns–328 ns | 307 ns | 0 B |
| whole-number scan | whole doubles | 1 | raw_floor | 287 ns | 287 ns–287 ns | 293 ns | 0 B |
| whole-number scan | whole doubles | 1000 | r_expr | 4.10 µs | 4.06 µs–4.55 µs | 4.19 µs | 19.7 KB |
| whole-number scan | whole doubles | 1000 | raw_capi | 697 ns | 697 ns–697 ns | 709 ns | 0 B |
| whole-number scan | whole doubles | 1000 | raw_chunked | 492 ns | 492 ns–492 ns | 482 ns | 0 B |
| whole-number scan | whole doubles | 1000 | raw_floor | 287 ns | 287 ns–287 ns | 295 ns | 0 B |
| whole-number scan | whole doubles | 100000 | r_expr | 352.6 µs | 352.5 µs–352.6 µs | 360.9 µs | 1.9 MB |
| whole-number scan | whole doubles | 100000 | raw_capi | 39.6 µs | 39.6 µs–39.6 µs | 39.7 µs | 0 B |
| whole-number scan | whole doubles | 100000 | raw_chunked | 17.8 µs | 17.8 µs–17.8 µs | 17.8 µs | 0 B |
| whole-number scan | whole doubles | 100000 | raw_floor | 287 ns | 287 ns–287 ns | 296 ns | 0 B |
| whole-number scan | whole doubles | 1e+07 | r_expr | 36.99 ms | 36.69 ms–37.96 ms | 42.30 ms | 190.7 MB |
| whole-number scan | whole doubles | 1e+07 | raw_capi | 3.94 ms | 3.94 ms–3.94 ms | 3.95 ms | 0 B |
| whole-number scan | whole doubles | 1e+07 | raw_chunked | 1.75 ms | 1.75 ms–1.75 ms | 1.75 ms | 0 B |
| whole-number scan | whole doubles | 1e+07 | raw_floor | 287 ns | 287 ns–287 ns | 297 ns | 0 B |

</details>
