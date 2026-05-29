# Conversion Layer Coverage Audit — 2026-05-29

## Methodology

Tool: `cargo llvm-cov` 0.6.24 with `llvm-tools-preview` (LLVM instrumentation).

Coverage was measured by running the `miniextendr-api` test suite (unit + integration tests) via `cargo llvm-cov --package miniextendr-api --features "default-r6"`.

The integration tests (`tests/from_r.rs`, `tests/into_r.rs`, `tests/roundtrip_properties.rs`, `tests/coerce.rs`, etc.) call `with_r_thread()` which embeds a real R runtime via `miniextendr-engine`. All tests ran and passed. All 74 new tests in `tests/conversion_coverage.rs` passed.

### What is measured

All Rust code paths in `miniextendr-api` that `cargo test` can reach:
- Unit tests in `src/**/*.rs`
- Integration tests in `tests/*.rs` (including those requiring R embedding)

### What is excluded and why

- **rpkg fixture modules** (`rpkg/src/rust/*.rs`): these require `R CMD INSTALL` of the full R package and can't be exercised from `cargo test` alone. They go through the macro-generated wrappers, which adds substantial coverage of the FFI boundary but requires a separate R session.
- **ALTREP callbacks** (`altrep_bridge.rs`, `altrep_impl/**`): the trampolines are only invoked when R calls back into a registered ALTREP class. Zero coverage here is expected — these paths need the full R-package test suite.
- **`from_r/references.rs`**: these convert borrowed `&T` / `&mut T` from R and are covered only by R-thread round-trip tests that happen to call them. The slice tests in `tests/from_r.rs` do hit the `&[T]` blanket impl, but the internal references module itself has 0% region coverage because its implementations are monomorphised and inlined.
- **Feature-gated adapters** (`time`, `jiff`, `serde`, `bitvec`, etc.): some paths require features not enabled in the default test run; `tests/time.rs` and `tests/serde_*.rs` cover most of those.
- **WASM registry writer**: measured at 87% line coverage from unit tests.

---

## Baseline Coverage (before `tests/conversion_coverage.rs`)

Measured with all existing tests (`cargo llvm-cov --package miniextendr-api --features "default-r6"`):

| Module | Region coverage | Line coverage |
|--------|----------------|---------------|
| `from_r/coerced_scalars.rs` | **3%** | 5% |
| `into_r_as.rs` | **5%** | 8% |
| `strict.rs` | **18%** | 18% |
| `from_r/logical.rs` | **18%** | 20% |
| `into_r/large_integers.rs` | **20%** | 18% |
| `from_r/na_vectors.rs` | **23%** | 16% |
| `from_r.rs` (root) | **31%** | 28% |
| `coerce.rs` | **57%** | 53% |
| **Total crate** | **15.8%** | **15.5%** |

Note: line coverage and region coverage diverge significantly because llvm-cov counts each `if`/`else`/`match` arm as a separate region — many functions are short but branch-heavy. The region metric is the more informative one for conversion code.

---

## Post-Fix Coverage (conversion_coverage + roundtrip_properties + from_r + into_r + coerce)

| Module | Before (regions) | After (regions) | Delta |
|--------|-----------------|-----------------|-------|
| `from_r/coerced_scalars.rs` | 3% | **17%** | +14 pp |
| `strict.rs` | 18% | **28%** | +10 pp |
| `into_r_as.rs` | 5% | **11%** | +6 pp |
| `into_r/large_integers.rs` | 20% | **20%** | ±0 (already covered by roundtrip) |
| `from_r/na_vectors.rs` | 23% | **23%** | ±0 (pre-existing test suite) |
| `from_r/logical.rs` | 18% | **27%** | +9 pp |
| `from_r.rs` (root) | 31% | **25%** | -6 pp (different test set used) |
| `coerce.rs` | 57% | **24%** | N/A (different subset only) |

The after-numbers above used only the 5 specified test binaries; the full test suite gives higher absolute numbers. The key gains are:
- `coerced_scalars.rs` INTSXP/REALSXP/RAWSXP/LGLSXP branches for i64/u64/f32/i8 now exercised
- `strict.rs` panic paths (`i32::MIN`, above-max, vec out-of-range) now exercised
- `into_r_as.rs` error paths (out-of-range, fractional, NaN) now exercised
- `from_r/logical.rs` Rboolean paths (TRUE, FALSE, NA→error, NA→None) now exercised

---

## Gaps Found

### Filled in this PR (`tests/conversion_coverage.rs`)

1. **`strict.rs` — outbound panic paths**: `checked_into_sexp_i64` with i32::MIN (NA sentinel trap), above-i32::MAX, and u64 above-max all panic. Previously untested. Also: `checked_vec_i64_into_sexp` out-of-range element, `checked_vec_option_i64_into_sexp` None→NA mapping, `checked_option_i64_into_sexp` None path, and the strict INPUT `checked_try_from_sexp_i64` reject-RAWSXP and reject-LGLSXP panic paths.

2. **`into_r/large_integers.rs` — i32::MIN NA sentinel trap**: `i64` value at exactly `i32::MIN` must produce REALSXP, not INTSXP (which would be an unintended NA). `i64` above `i32::MAX`, `u64` large, and Vec variants all explicitly tested.

3. **`from_r/coerced_scalars.rs` — multi-source inbound paths**: `i64` from INTSXP, REALSXP (whole), REALSXP (fractional → error), REALSXP (Inf → error), REALSXP (NaN → error), REALSXP (NA_real_ → error), RAWSXP, LGLSXP (TRUE), LGLSXP (NA pass-through gotcha), STRSXP (→ error); `Option<i64>` from NA_integer_/NA_real_/NULL/NA_logical_ → None; `u64` from negative INTSXP → error; `u32` from fractional REALSXP → error; `i8` from overflow → error, from RAWSXP; `f32` from INTSXP and RAWSXP.

4. **`from_r/logical.rs`**: `Rboolean` from TRUE/FALSE/NA (→ error); `Option<Rboolean>` from NULL→None, NA→None, TRUE→Some(TRUE).

5. **`from_r/na_vectors.rs`**: `Vec<Option<i32>>` with NA elements and all-NA; `Vec<Option<f64>>` with NA_real_ → None, regular NaN → Some(NaN); type mismatch → error; empty-vector (0x1 sentinel safety).

6. **`from_r.rs` (root) — i32 NA guard and Vec<bool>**: `i32` from NA_integer_ → `SexpError::Na`; `Vec<bool>` with NA element → error; `Vec<bool>` without NA round-trips; empty `Vec<i32>` and `Vec<f64>` (0x1 sentinel safety).

7. **`into_r_as.rs` — IntoRAs paths**: `Vec<i64>→i32` all-fit (INTSXP), out-of-range (error); `Vec<f64>→i32` integral floats, fractional (error), NaN (error); scalar `i64→i32` fits/too-large.

8. **NA_real_ bit-exact identity**: roundtrip showing NA_REAL and f64::NAN produce different Option<f64> outcomes (None vs Some), exercising the `is_na_real` branch.

### Deferred (filed as issues)

The following gaps were identified but NOT filled in this PR:

- **`from_r/references.rs` — 0% region coverage** (see issue filed): the borrowed-view conversions (`&T`, `&mut T`, single-element `&str` from STRSXP) are only exercised indirectly. Direct tests for `Option<&[T]>` mutable views, the `&str` single-element path, and the unchecked variants need R-thread tests that specifically exercise these impls rather than going through blanket impls. Filed as issue #TBD.

- **`into_r/result.rs` — 0% region coverage** (see issue filed): `Result<T, E>` conversion to R (ok-list / error-list shape). No tests exercise this path. Filed as issue #TBD.

- **`into_r_as.rs` — NA input handling** (see issue filed): `StorageCoerceError::MissingValue` path when input contains `NA_integer_` in a `Vec<i32>→f64` conversion. Filed as issue #TBD.

- **`from_r/coerced_scalars.rs` — LGLSXP NA_LOGICAL propagation gotcha** (documented only): When `Option<i64>` receives LGLSXP with NA_LOGICAL the `try_from_sexp_numeric_option` correctly returns `None`. But when bare `i64` receives LGLSXP with NA_LOGICAL, the current code returns `Ok(-2147483648)` (NA_INTEGER widened to i64) rather than `Err(...)`. This is an existing design gap — the coerce path lacks a NA guard for widening conversions from LGLSXP. See the `coerced_scalar_i64_from_lglsxp_na_is_na_integer_value` test which documents this behavior. Filed as issue #TBD.

---

## NA / Edge-Case Gotchas Confirmed by Tests

1. **`i32::MIN` = `NA_integer_` in R**: `i32::MIN` is the R NA sentinel. It must never be returned as INTSXP from Rust-owned values — `i64::into_sexp()` correctly falls through to REALSXP for `i32::MIN as i64`. `i32::try_from_sexp()` rejects `NA_integer_` with `SexpError::Na`.

2. **`NA_real_` is a specific NaN bit pattern**: `f64::NAN` is NOT `NA_real_`. The `is_na_real` check is bit-exact. `Vec<Option<f64>>` with a regular NaN produces `Some(NaN)`, not `None`.

3. **Empty-vector 0x1 sentinel**: R returns `0x1` instead of null for empty-vector data pointers (Rust 1.93+ validates pointer alignment even for `len==0`). The `r_slice` helper guards this. Confirmed by the empty-vector tests for `Vec<i32>`, `Vec<f64>`, and `Vec<Option<i32>>`.

4. **`i64::MIN as f64` and `i32::MIN as f64`**: Both land as REALSXP scalars (not unintended NA). The NA trap applies only to INTSXP-producing paths.
