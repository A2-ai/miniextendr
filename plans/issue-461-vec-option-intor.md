+++
title = "issue-461: IntoR for Vec<Option<C>> — opaque container fields in enum DataFrameRow"
description = "Add the missing Vec<Option<C>> impls so opaque container columns compile in align-mode enum derives."
+++

# issue-461: `IntoR for Vec<Option<C>>` — opaque container fields in enum `DataFrameRow`

Tracks: <https://github.com/A2-ai/miniextendr/issues/461>

## 1. Existing `IntoR` landscape

### `Vec<Option<scalar>>` — ALREADY WORK (typed NA vectors)

All of these land as typed R vectors with NA sentinels:

| Impl | File:line | R type |
|------|-----------|--------|
| `impl IntoR for Vec<Option<f64>>` | `miniextendr-api/src/into_r.rs:1492` (`impl_vec_option_into_r!(f64, NA_REAL)`) | REALSXP with NA_real_ |
| `impl IntoR for Vec<Option<i32>>` | `miniextendr-api/src/into_r.rs:1492` (`impl_vec_option_into_r!(i32, NA_INTEGER)`) | INTSXP with NA_integer_ |
| `impl IntoR for Vec<Option<bool>>` | `miniextendr-api/src/into_r.rs:1700` (`impl_vec_option_logical_into_r!`) | LGLSXP with NA_logical_ |
| `impl IntoR for Vec<Option<String>>` | `miniextendr-api/src/into_r.rs:1750` | STRSXP with NA_character_ |
| `impl IntoR for Vec<Option<i64/u64/isize/usize>>` | `miniextendr-api/src/into_r.rs:1530` (smart path) | INTSXP or REALSXP |
| `impl IntoR for Vec<Option<i8/i16/u16/u32/f32>>` | `miniextendr-api/src/into_r.rs:1580` (coerce) | INTSXP or REALSXP |

### Collection types that work as `Vec<C>` (list-columns)

These have `IntoR for Vec<C>` impls (land as VECSXP, one element per row):

| Type `C` | Impl site | R result |
|----------|-----------|----------|
| `Vec<T>` where `T: RNativeType` | `into_r.rs:1390` (blanket) | VECSXP of typed vectors |
| `Vec<String>` | `into_r.rs:1439` | VECSXP of character vectors |
| `HashSet<T>` where `T: RNativeType` | `into_r.rs:2082` | VECSXP of unordered vectors |
| `HashSet<String>` | `into_r.rs:2119` | VECSXP of character vectors |
| `BTreeSet<T>` where `T: RNativeType` | `into_r.rs:2101` | VECSXP of sorted vectors |
| `BTreeSet<String>` | `into_r.rs:2135` | VECSXP of character vectors |
| `HashMap<String, V>` where `V: IntoR` | `into_r/collections.rs:15` (via `impl_map_into_r!`) | VECSXP of named lists |
| `BTreeMap<String, V>` where `V: IntoR` | `into_r/collections.rs:15` (via `impl_map_into_r!`) | VECSXP of named lists |

### What is MISSING (the bug)

`IntoR for Vec<Option<C>>` for ANY container `C`. These do not exist:

- `Vec<Option<Vec<T>>>` — not implemented
- `Vec<Option<HashSet<T>>>` — not implemented
- `Vec<Option<BTreeSet<T>>>` — not implemented
- `Vec<Option<HashMap<String, V>>>` — not implemented (relevant once #457 lands)
- `Vec<Option<BTreeMap<String, V>>>` — not implemented (relevant once #457 lands)

`Option<C>` impls exist at `into_r.rs:968–1167` (all the `Option<Vec<T>>`, `Option<HashSet<T>>`, etc. → NULL for None), but the wrapping in an outer `Vec` is what triggers the missing impl.

## 2. Where the wrap happens in the derive

`miniextendr-macros/src/dataframe_derive/enum_expansion.rs` wraps EVERY schema column in `Vec<Option<T>>` regardless of whether `T` is a scalar or a container. The relevant sites:

- **Companion struct field type** — `enum_expansion.rs:384`:
  ```rust
  quote! { pub #name: Vec<Option<#ty>> }
  ```
  where `#ty` is the raw field type (could be `Vec<i32>`, `HashSet<String>`, etc.).

- **Local accumulator init** — `enum_expansion.rs:572`:
  ```rust
  quote! { let mut #name: Vec<Option<#ty>> = Vec::with_capacity(len); }
  ```

- **NA-fill** — `enum_expansion.rs:651`:
  ```rust
  quote! { #col_name.push(None); }
  ```

- **Present-variant push** — `enum_expansion.rs:622`:
  ```rust
  return quote! { #col_name.push(Some(#binding)); };
  ```

`classify_field_type` (`dataframe_derive.rs:238–287`) maps `Vec<T>` → `VariableVec`, `HashSet` → `Scalar` (because it doesn't match Vec/Box/Array/Slice). So `HashSet<String>` lands in `EnumResolvedField::Single` with `ty = HashSet<String>`, producing `Vec<Option<HashSet<String>>>`.

For `Vec<T>` without `expand`/`width`, the match arm at `enum_expansion.rs:124` falls into `EnumResolvedField::Single` with `ty = Vec<T>`, producing `Vec<Option<Vec<T>>>`.

The same happens in the tuple-variant branch (`enum_expansion.rs:222–231`).

## 3. Fix options

### Option A — Add `IntoR for Vec<Option<C>>` impls for each collection C (RECOMMENDED)

**Mechanism**: `Some(c) → c.into_sexp()` as a VECSXP element; `None → R_NilValue` (NULL). The outer `Vec` becomes a VECSXP list-column where absent rows are NULL and present rows are whatever `C` converts to.

This exactly mirrors the existing `Option<Vec<T>>` → NULL/vector pattern, just wrapped in a `Vec`.

**Coherence note**: A blanket `impl<C: IntoR> IntoR for Vec<Option<C>>` cannot be written. `Vec<Option<i32>>` is already covered by `impl_vec_option_into_r!` (INTSXP with NA). The blanket would overlap. Instead, concrete impls are needed per `C`. This is exactly the pattern already used for `Vec<Vec<T>>`, `Vec<HashSet<T>>`, etc. in `into_r.rs:1389–2148`.

**Code changes — into_r.rs**:

Add a `vec_option_of_into_r_to_list` helper (~15 lines) after `vec_of_into_r_to_list` at line 2078:
```rust
/// Helper: convert Vec<Option<C: IntoR>> to VECSXP, None → R_NilValue.
fn vec_option_of_into_r_to_list<T: IntoR>(items: Vec<Option<T>>) -> crate::ffi::SEXP {
    unsafe {
        let n = items.len();
        let list = OwnedProtect::new(crate::ffi::Rf_allocVector(
            crate::ffi::SEXPTYPE::VECSXP,
            n as crate::ffi::R_xlen_t,
        ));
        for (i, item) in items.into_iter().enumerate() {
            let elt = match item {
                Some(v) => v.into_sexp(),
                None => crate::ffi::SEXP::nil(),
            };
            list.get().set_vector_elt(i as crate::ffi::R_xlen_t, elt);
        }
        *list
    }
}
```

Then add these concrete impls (grouped in a new `// region: Vec<Option<Collection>>` block after `vec_option_of_into_r_to_list`):

1. **`Vec<Option<Vec<T>>>` where `T: RNativeType`** (~20 lines):
   ```rust
   impl<T: crate::ffi::RNativeType> IntoR for Vec<Option<Vec<T>>>
   where Vec<T>: IntoR
   { ... delegates to vec_option_of_into_r_to_list(self) ... }
   ```

2. **`Vec<Option<Vec<String>>>`** (concrete, ~20 lines):
   Same shape.

3. **`Vec<Option<HashSet<T>>>` where `T: RNativeType + Eq + Hash`** (~20 lines):
   Convert each `Option<HashSet<T>>` → `Option<Vec<T>>` before delegating? No: call `vec_option_of_into_r_to_list` directly where each `Some(s)` → `s.into_sexp()`.

4. **`Vec<Option<HashSet<String>>>`** (concrete, ~20 lines).

5. **`Vec<Option<BTreeSet<T>>>` where `T: RNativeType + Ord`** (~20 lines).

6. **`Vec<Option<BTreeSet<String>>>`** (concrete, ~20 lines).

7. **`Vec<Option<HashMap<String, V>>>` where `V: IntoR`** (~20 lines). Needed for #457; add now.

8. **`Vec<Option<BTreeMap<String, V>>>` where `V: IntoR`** (~20 lines). Same.

Total: ~8 impls, ~160 lines in `into_r.rs`. All use `vec_option_of_into_r_to_list` or the same pattern. The unchecked variants just call `try_into_sexp()` (same as many existing collection impls).

**GC discipline**: `vec_option_of_into_r_to_list` must protect the outer list across `item.into_sexp()` calls. `OwnedProtect::new(...)` does this (same pattern as `vec_of_into_r_to_list` at line 2065). `SET_VECTOR_ELT` does not require the inner element to be separately protected when the list itself is protected (R will mark through the list's slots). This is consistent with existing impls.

**No derive changes needed.**

**Scalar `Vec<Option<scalar>>` is unaffected**: all existing concrete impls (`f64`, `i32`, `bool`, `String`, smart-i64, coerce) remain, and Rust resolves them before any new collection impls. The new impls only add `Vec<Option<C>>` for collection Cs that have NO existing `Vec<Option<C>>` impl.

**Trade-offs**:
- Pro: minimum diff, no macro changes, consistent with existing pattern.
- Pro: preserves scalar NA semantics exactly.
- Pro: R-side type is correct: list-column with NULL for absent rows, vector/list for present rows.
- Con: 8 explicit impls instead of 1 blanket. Acceptable — same approach used throughout the file.
- Con: does not cover `Vec<Option<Box<[T]>>>` (not needed by #461, but worth noting).

### Option B — Derive-site special-casing for opaque containers

Instead of adding `IntoR` impls, detect at derive time that a field is an opaque container, and emit a different accumulator type, e.g., `Vec<Option<Vec<T>>>` (reified from `Vec<T>`) that IS already covered.

**Problem**: `HashSet<String>` → `Vec<Option<Vec<String>>>` silently reorders elements, changing the R-side structure without any user opt-in. Also requires pattern-matching on field type AST for every collection type — brittle and must track #457 separately.

**Problem 2**: `Vec<T>` in enum → we'd emit `Vec<Option<Vec<T>>>` from the existing `VariableVec` branch when no `expand`/`width` — but `Vec<Option<Vec<T>>> where T: RNativeType` would also need a new `IntoR` impl (same as Option A, just fewer impls needed since HashSet/BTreeSet would be converted at derive time to Vec).

**Trade-offs**:
- Pro: avoids adding impls for HashSet/BTreeSet variants.
- Con: changes R-side semantics silently for HashSet/BTreeSet (unordered → ordered? lossy? needs documentation).
- Con: requires AST pattern matching for more collection types in the macro; needs updating as new collection types land.
- Con: still needs at minimum `Vec<Option<Vec<T>>>` and `Vec<Option<Vec<String>>>` in `into_r.rs`.
- Overall: more complex, same or more total work, worse user-visible semantics.

### Option C — Blanket `impl<C: IntoR> IntoR for Vec<Option<C>>`

**Problem**: Immediate coherence conflict with all existing `impl IntoR for Vec<Option<scalar>>` impls (i32, f64, bool, String, etc.). Rust E0119. Not achievable without negative impls (unstable).

**Problem 2**: Even if achievable, would land `Vec<Option<i32>>` as a list-column of NULLs instead of an integer vector with NAs — breaking existing users.

**Verdict**: Not viable.

## 4. Recommendation: Option A

Add the `vec_option_of_into_r_to_list` helper and 8 concrete `Vec<Option<C>>` impls to `miniextendr-api/src/into_r.rs`. No macro changes required. Minimal, consistent with existing code style, correct GC discipline. HashMap/BTreeMap impls (#7, #8) can be added now to avoid a follow-up PR when #457 lands.

## 5. Flat work list

1. **`miniextendr-api/src/into_r.rs`** — after `vec_of_into_r_to_list` (line 2078), add new `// region: Vec<Option<Collection>> conversions` block containing:
   - `fn vec_option_of_into_r_to_list<T: IntoR>(items: Vec<Option<T>>) -> SEXP` helper
   - `impl<T: RNativeType> IntoR for Vec<Option<Vec<T>>>` where `Vec<T>: IntoR` — delegates to helper, `Some(v) → v.into_sexp()`, `None → SEXP::nil()`
   - `impl IntoR for Vec<Option<Vec<String>>>` — delegates to helper
   - `impl<T: RNativeType + Eq + Hash> IntoR for Vec<Option<HashSet<T>>>` — delegates to helper via `Some(s) → s.into_sexp()`
   - `impl IntoR for Vec<Option<HashSet<String>>>` — delegates to helper
   - `impl<T: RNativeType + Ord> IntoR for Vec<Option<BTreeSet<T>>>` — delegates to helper
   - `impl IntoR for Vec<Option<BTreeSet<String>>>` — delegates to helper
   - `impl<V: IntoR> IntoR for Vec<Option<HashMap<String, V>>>` — delegates to helper
   - `impl<V: IntoR> IntoR for Vec<Option<BTreeMap<String, V>>>` — delegates to helper

   All impls use `Error = std::convert::Infallible`. All unchecked variants delegate to `try_into_sexp()` (same as other collection impls). The `std::hash::Hash` bound is already in scope at the top of `into_r.rs` (line 27).

2. **`rpkg/src/rust/dataframe_enum_payload_matrix.rs`** — add the three previously-dropped sections with full four-cell cardinality coverage:
   - **Opaque `Vec<i32>` section**: enum with `items: Vec<i32>` field (no `expand`, no `width`). Expose `#[miniextendr]` functions `vec_opaque_split_1v1r()`, `vec_opaque_split_1vnr()`, `vec_opaque_split_nv1r()`, `vec_opaque_align_nvnr()`, and `vec_opaque_split_nvnr()`.
   - **`HashSet<String>` section**: enum with `tags: HashSet<String>` field. Same pattern: `hashset_split_1v1r()`, `hashset_split_1vnr()`, `hashset_split_nv1r()`, `hashset_align_nvnr()`, `hashset_split_nvnr()`.
   - **`BTreeSet<i32>` section**: enum with `cats: BTreeSet<i32>` field. Same pattern: `btreeset_split_1v1r()`, `btreeset_split_1vnr()`, `btreeset_split_nv1r()`, `btreeset_align_nvnr()`, `btreeset_split_nvnr()`.
   Remove the comment at line 19 that says these are blocked on #461.

3. **`rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R`** — add test blocks for all four cardinality cells per new fixture:
   - Opaque `Vec<i32>`: `1v1r`, `1vNr`, `Nv1r` split tests + the existing `NvNr` align and split tests. Verify list-column structure, NULL for absent rows, vector (including empty) for present rows.
   - `HashSet<String>`: `1v1r`, `1vNr`, `Nv1r` split tests + `NvNr` tests. Use `expect_setequal` (unordered) for membership checks.
   - `BTreeSet<i32>`: `1v1r`, `1vNr`, `Nv1r` split tests + `NvNr` tests. Use `expect_equal` to verify sorted order preserved.
   Remove the comment at line 19 mentioning the block on #461.

4. **GC stress fixture** (if the new impls touch SEXP storage across allocations): `vec_option_of_into_r_to_list` calls `item.into_sexp()` inside a loop while the outer VECSXP is `OwnedProtect`-protected. The pattern is safe but should be verified with `gctorture(TRUE)`. Add a no-arg `gc_stress_vec_option_collection()` in `rpkg/src/rust/gc_stress_fixtures.rs` that builds a `Vec<Option<Vec<i32>>>`, `Vec<Option<HashSet<String>>>`, and `Vec<Option<BTreeSet<i32>>>` and calls `into_sexp()` on each.

5. **`rpkg/R/miniextendr-wrappers.R`** and **`NAMESPACE`** — regenerate after adding `#[miniextendr]` fixtures (`just configure && just rcmdinstall && just devtools-document`). Commit generated files in sync with the Rust changes.

6. **Clippy verification** — run both `clippy_default` and `clippy_all` job configurations before pushing (see CLAUDE.md for the exact feature flags).

## 6. Files not touched

- `miniextendr-macros/src/dataframe_derive/enum_expansion.rs` — no changes required.
- `miniextendr-macros/src/dataframe_derive.rs` — no changes required.
- `miniextendr-api/src/into_r/collections.rs` — no changes required (existing `HashSet`/`BTreeSet`/map impls stay there; new `Vec<Option<C>>` impls live in the parent `into_r.rs` alongside the other `Vec<Option<…>>` impls for locality).

## 7. Acceptance criteria (from issue #461)

- Opaque-container variant fields (`Vec<T>`, `HashSet<T>`, `BTreeSet<T>`) compile with `#[derive(DataFrameRow)]` on `align` enums.
- `to_dataframe` NA-fill: absent-variant rows produce `NULL` in the list-column (not crash, not NA, not empty vector).
- `to_dataframe_split` per-variant: each partition's list-column contains the actual collection values.
- `Vec<Option<scalar>>` impls are unchanged — existing tests continue to pass.
