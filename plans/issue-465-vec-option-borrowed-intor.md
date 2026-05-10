+++
title = "issue-465: IntoR for Vec<Option<&str>> / Vec<Option<&[T]>> — borrowed fields in enum DataFrameRow"
description = "Add the missing borrowed Vec<Option<...>> impls so &str and &[T] columns compile in align-mode enum derives."
+++

# issue-465: `IntoR for Vec<Option<&str>>` / `Vec<Option<&[T]>>` — borrowed fields in enum `DataFrameRow`

Tracks: <https://github.com/A2-ai/miniextendr/issues/465>

## 1. Existing `IntoR` landscape for borrowed types

### Borrowed scalar and slice impls — ALREADY WORK (no NA support)

| Impl | File:line | R type |
|------|-----------|--------|
| `impl IntoR for &str` | `miniextendr-api/src/into_r.rs` (via `str_to_charsxp`) | CHARSXP |
| `impl IntoR for Vec<&str>` | `miniextendr-api/src/into_r.rs:1369` | STRSXP, no NAs |
| `impl IntoR for &[&str]` | `miniextendr-api/src/into_r.rs:1264` | STRSXP, no NAs |
| `impl<T: RNativeType> IntoR for &[T]` | `miniextendr-api/src/into_r.rs:323` | typed vector |
| `impl<T: RNativeType> IntoR for Vec<&[T]>` | (via blanket collection impls) | VECSXP list-column |

### `Vec<Option<Cow<'_, str>>>` — ALREADY WORKS (STRSXP + NA)

`impl IntoR for Vec<Option<Cow<'_, str>>>` at `into_r.rs:1318`. Walks the vec, allocates STRSXP, calls `str_to_charsxp(s.as_ref())` for `Some` and `SEXP::na_string()` for `None`. This is the closest existing model for the borrowed-string path.

### Owned `Vec<Option<String>>` — ALREADY WORKS (STRSXP + NA)

`impl IntoR for Vec<Option<String>>` at `into_r.rs:1750`. Same STRSXP+NA pattern, calls `str_to_charsxp(s)` for `Some` and `SEXP::na_string()` for `None`.

### Collection `Vec<Option<C>>` impls from #461 — ALREADY WORK (list-columns)

`vec_option_of_into_r_to_list` helper at `into_r.rs:2084`. Concrete impls:

| Impl | File:line |
|------|-----------|
| `impl<T: RNativeType> IntoR for Vec<Option<Vec<T>>>` | `into_r.rs:2103` |
| `impl IntoR for Vec<Option<Vec<String>>>` | `into_r.rs:2120` |
| `impl<T: RNativeType + Eq + Hash> IntoR for Vec<Option<HashSet<T>>>` | `into_r.rs:2134` |
| `impl IntoR for Vec<Option<HashSet<String>>>` | `into_r.rs:2151` |
| `impl<T: RNativeType + Ord> IntoR for Vec<Option<BTreeSet<T>>>` | `into_r.rs:2165` |
| `impl IntoR for Vec<Option<BTreeSet<String>>>` | `into_r.rs:2182` |
| `impl<V: IntoR> IntoR for Vec<Option<HashMap<String, V>>>` | `into_r.rs:2196` |
| `impl<V: IntoR> IntoR for Vec<Option<BTreeMap<String, V>>>` | `into_r.rs:2210` |

### What is MISSING (the bug)

- `impl<'a> IntoR for Vec<Option<&'a str>>` — does NOT exist
- `impl<'a, T: RNativeType> IntoR for Vec<Option<&'a [T]>>` — does NOT exist
- `impl<'a> IntoR for Vec<Option<&'a [String]>>` — does NOT exist (for symmetry with `Vec<Option<Vec<String>>>`)

## 2. Macro side — derive already threads lifetimes correctly

`miniextendr-macros/src/dataframe_derive/enum_expansion.rs:54`:
```rust
let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
```

The companion struct is generated at `enum_expansion.rs:396`:
```rust
pub struct #df_name #impl_generics #where_clause {
    ...
    pub #name: Vec<Option<#ty>>
    ...
}
```

`#ty` is the raw field type as parsed from the enum variant. For `enum E<'a> { Named { name: &'a str } }`, `#ty` is `&'a str`, producing `Vec<Option<&'a str>>`. The `#impl_generics` carries `<'a>` so the companion struct becomes `pub struct EDataFrame<'a>`. The lifetime PR #462 verified this threading is complete and correct. The macro is fine — only the trait impl is missing.

For `#[dataframe(expand)]` on a `data: &'a [f64]` field, the companion column is `Vec<Option<&'a [f64]>>`. The auto-expand path (`enum_expansion.rs:388–392`) would emit a different shape (multiple pinned scalar columns), so a raw `&'a [T]` with `expand` would go through `Vec<Option<f64>>` scalar expansion, which already works. The `Vec<Option<&'a [T]>>` shape occurs only when `&'a [T]` is used WITHOUT `expand`/`width` — i.e., as an opaque column.

## 3. Design decisions

### `Vec<Option<&str>>` → STRSXP-with-NA (not list-column)

`&str` is a scalar text type — borrowed version of `String`. Users expect identical R-side behavior to `Vec<Option<String>>`, which is a STRSXP column with `NA_character_` for absent rows. A list-column of character scalars (or NULLs) would be surprising and inconsistent.

**Mechanism**: walk the vec, allocate STRSXP of the full length, call `str_to_charsxp(s)` for `Some(s)` and `SEXP::na_string()` for `None`. This is identical to the `Vec<Option<Cow<'_, str>>>` impl at `into_r.rs:1318` — that impl is the direct model. The `str_to_charsxp` and `str_to_charsxp_unchecked` helpers are already `pub(crate)`.

**Coherence**: `Vec<Option<&str>>` is a distinct concrete type from `Vec<Option<String>>`, `Vec<Option<Cow<'_, str>>>`, and all existing `impl_vec_option_into_r!` scalar types (`i32`, `f64`, `bool`). No overlap with existing impls. Rust resolves the specific `impl<'a> IntoR for Vec<Option<&'a str>>` before any hypothetical future blanket. The owned `Vec<Option<String>>` impl at `into_r.rs:1750` is unaffected.

### `Vec<Option<&[T]>>` → list-column (same as `Vec<Option<Vec<T>>>`)

`&[T]` is a borrowed slice — a collection-shaped type. Element-shape Option wraps consistently produce VECSXP list-columns with NULL for None. Deviating here for borrowed slices would create an asymmetry with `Vec<Option<Vec<T>>>`.

**Mechanism**: delegate to `vec_option_of_into_r_to_list(self)`. The helper signature `fn vec_option_of_into_r_to_list<T: IntoR>(items: Vec<Option<T>>)` accepts `T = &'a [U]` because `&'a [U]: IntoR` for `U: RNativeType`. `Vec<Option<&'a [U]>>` is moved into the helper (the Vec is owned; references inside have lifetime `'a` which must outlive the call). This is valid — the Vec's lifetime is consumed by value, and the references within are valid for `'a` which encompasses the call site.

**`Vec<Option<&'a [String]>>`**: add for symmetry with `Vec<Option<Vec<String>>>`. Same delegate-to-helper pattern; `&[String]: IntoR` at `into_r.rs:1229`.

### No `Vec<Option<&'a [&'a str]>>`

The macro does not emit this shape. A field `names: &'a [&'a str]` with no `expand`/`width` would produce `Vec<Option<&'a [&'a str]>>`. This is an unusual pattern in practice and can be tracked as a follow-up if it surfaces. Do NOT add it speculatively.

## 4. Impls to add (3 total)

1. `impl<'a> IntoR for Vec<Option<&'a str>>` — STRSXP+NA path
2. `impl<'a, T: RNativeType> IntoR for Vec<Option<&'a [T]>>` — list-column path
3. `impl<'a> IntoR for Vec<Option<&'a [String]>>` — list-column path

All three use `type Error = std::convert::Infallible`. Impls 2 and 3 use `vec_option_of_into_r_to_list`. Impl 1 uses the STRSXP+NA loop (model: `into_r.rs:1318`).

## 5. Flat work list

1. **`miniextendr-api/src/into_r.rs`** — after `impl IntoR for Vec<Option<Cow<'_, str>>>` (line 1366), add a new `// region: Vec<Option<borrowed string>>` block containing:
   - `impl<'a> IntoR for Vec<Option<&'a str>>` — STRSXP+NA, modeled on the `Cow` impl at line 1318:
     - `into_sexp`: allocate STRSXP via `OwnedProtect`, iterate, `str_to_charsxp(s)` for `Some(s)`, `SEXP::na_string()` for `None`, `set_string_elt`
     - `into_sexp_unchecked`: same with `Rf_allocVector_unchecked` + `str_to_charsxp_unchecked` + `set_string_elt_unchecked`
     - `try_into_sexp` / `try_into_sexp_unchecked` delegate to the checked/unchecked variants

2. **`miniextendr-api/src/into_r.rs`** — inside the existing `// region: Vec<Option<Collection>> conversions` block (after line 2221, before `// endregion`), add:
   - `impl<'a, T: RNativeType> IntoR for Vec<Option<&'a [T]>> where &'a [T]: IntoR` — delegates to `vec_option_of_into_r_to_list(self)`, `try_into_sexp` calls `try_into_sexp()` (same as other collection impls)
   - `impl<'a> IntoR for Vec<Option<&'a [String]>>` — delegates to `vec_option_of_into_r_to_list(self)`

3. **`rpkg/src/rust/dataframe_enum_payload_matrix.rs`** — update the file header comment at line 17: change `"`&[T]` enum payloads are deferred until the lifetime-support PR lands."` to `"`&str` and `&[T]` enum payloads are exercised in the borrowed-string section below."` — add two new regions at the end (before the `#[cfg(test)] mod tests` block):

   **Region 6: `&str` field (STRSXP+NA column)**
   ```
   // region: 6. &str field (borrowed text → STRSXP with NA_character_) ──────────
   ```
   - Define `enum BorrowedStrEvent<'a>` with `#[dataframe(align, tag = "_type")]`:
     - `Named { id: i32, name: &'a str }`
     - `Bare { id: i32 }`
   - Helper `fn borrowed_str_payload<'a>(id: i32, name: &'a str) -> BorrowedStrEvent<'a>`
   - Export `#[miniextendr]` functions (note: MXL112 bars lifetime params on `#[miniextendr]` fns — the fixtures themselves must be owned-typed wrappers that construct `BorrowedStrEvent` internally):
     - `pub fn borrowed_str_split_1v1r() -> List` — single `Named { id: 1, name: "alice" }` row
     - `pub fn borrowed_str_split_1vnr() -> List` — three `Named` rows with distinct names
     - `pub fn borrowed_str_split_nv1r() -> List` — one `Named` + one `Bare` row
     - `pub fn borrowed_str_align_nvnr() -> ToDataFrame<BorrowedStrEventDataFrame<'static>>` — four rows (two Named, two Bare); `name` column should be STRSXP with `NA_character_` for Bare rows
     - `pub fn borrowed_str_split_nvnr() -> List` — four-row NvNr split

   **Region 7: `&[T]` field opaque (list-column with NULL)**
   ```
   // region: 7. &[T] field opaque (borrowed slice → list-column with NULL) ──────
   ```
   - Define `enum BorrowedSliceEvent<'a>` with `#[dataframe(align, tag = "_type")]`:
     - `Buffer { label: String, data: &'a [f64] }` — no `expand`/`width`, opaque list-column
     - `NoBuffer { label: String }`
   - Helper `fn borrowed_slice_payload<'a>(label: &str, data: &'a [f64]) -> BorrowedSliceEvent<'a>`
   - Export `#[miniextendr]` functions (same MXL112 caveat — build `BorrowedSliceEvent` with `'static` data inside the fn body):
     - `pub fn borrowed_slice_split_1v1r() -> List`
     - `pub fn borrowed_slice_split_1vnr() -> List`
     - `pub fn borrowed_slice_split_nv1r() -> List`
     - `pub fn borrowed_slice_align_nvnr() -> ToDataFrame<BorrowedSliceEventDataFrame<'static>>`
     - `pub fn borrowed_slice_split_nvnr() -> List`

   **MXL112 workaround for fixtures**: `#[miniextendr]` rejects explicit lifetime params. The enum variants hold `'static` string/slice literals inside exported fn bodies — no runtime lifetime required. Example:
   ```rust
   #[miniextendr]
   pub fn borrowed_str_split_1v1r() -> List {
       let data: Vec<BorrowedStrEvent<'static>> = vec![
           BorrowedStrEvent::Named { id: 1, name: "alice" },
       ];
       BorrowedStrEvent::to_dataframe_split(data)
   }
   ```

4. **`rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R`** — add two new test sections at the end (before any EOF):

   **Section: `&str` field tests**
   - `"borrowed str — split 1v1r: one Named row, bare has 0 rows"` — verify `name` is a character vector, value is `"alice"`, bare partition has 0 rows
   - `"borrowed str — split 1vNr: multiple Named rows"` — verify character column with distinct values
   - `"borrowed str — split Nv1r: one row each"` — verify bare partition has no `name` column
   - `"borrowed str — align NvNr: present rows are character, absent rows are NA_character_"` — verify `is.character(df$name)`, values for Named rows, `is.na(df$name[bare_idx])` for Bare rows
   - `"borrowed str — split NvNr: name partition has character column, bare partition omits it"` — structure check

   **Section: `&[T]` field opaque tests**
   - Mirror the `vec opaque` tests from lines 21–86 (the `Vec<i32>` opaque section):
   - `"borrowed slice — split 1v1r: one Buffer row, no_buffer has 0 rows"` — verify list-column, `[[1]]` is `c(1, 2, 3)` (REALSXP)
   - `"borrowed slice — split 1vNr: multiple Buffer rows"` — distinct slice values per row
   - `"borrowed slice — split Nv1r: one row each"` — no_buffer omits data column
   - `"borrowed slice — align NvNr: present rows are numeric vectors, absent rows are NULL"` — `expect_type(df$data, "list")`, `expect_null(df$data[[absent_idx]])`
   - `"borrowed slice — split NvNr: buffer partition has list-column, no_buffer partition omits it"`

5. **`rpkg/src/rust/gc_stress_fixtures.rs`** — add a new no-arg `#[miniextendr]` fixture after `gc_stress_vec_option_collection()`:
   ```rust
   /// Exercise `Vec<Option<&str>>` and `Vec<Option<&[T]>>` conversions under GC pressure.
   ///
   /// Allocates STRSXP + list-column SEXPs with interleaved None/Some values to verify
   /// PROTECT discipline across string and slice allocations.
   #[miniextendr]
   pub fn gc_stress_vec_option_borrowed() {
       // Vec<Option<&str>>: STRSXP with NA_character_
       let str_opt: Vec<Option<&str>> = vec![Some("hello"), None, Some("world"), None];
       let _ = str_opt.into_sexp();

       // Vec<Option<&[f64]>>: list-column, NULL for None
       let a: &[f64] = &[1.0, 2.0, 3.0];
       let b: &[f64] = &[4.0];
       let slice_opt: Vec<Option<&[f64]>> = vec![Some(a), None, Some(b), None];
       let _ = slice_opt.into_sexp();

       // Vec<Option<&[String]>>: list-column (character vector per row)
       let sa: Vec<String> = vec!["x".to_string(), "y".to_string()];
       let sb: Vec<String> = vec!["z".to_string()];
       let str_slice_opt: Vec<Option<&[String]>> =
           vec![Some(sa.as_slice()), None, Some(sb.as_slice())];
       let _ = str_slice_opt.into_sexp();
   }
   ```

   **GC discipline note**: `Vec<Option<&str>>::into_sexp` uses `OwnedProtect` on the outer STRSXP (same as `Vec<Option<Cow<'_, str>>>`). `Vec<Option<&[T]>>::into_sexp` delegates to `vec_option_of_into_r_to_list` which uses `OwnedProtect` on the outer VECSXP and calls `(&[T]).into_sexp()` (copies the slice data into a fresh typed vector) for each `Some`. The copy allocates, but the outer list is already protected by `OwnedProtect`. The pattern is safe and consistent with existing collection impls.

6. **`rpkg/R/miniextendr-wrappers.R`** and **`NAMESPACE`** — regenerate after adding `#[miniextendr]` fixtures:
   ```bash
   just configure && just rcmdinstall && just devtools-document
   ```
   Commit generated files in sync with Rust changes (pre-commit hook enforces this).

7. **`CLAUDE.md`** — update the lifetime gotcha note in the "Rust/FFI gotchas" section. The current text reads:
   > "For DataFrameRow enums, the macro also accepts lifetime params, but borrowed fields (e.g., `&'a str`) produce `Vec<Option<&str>>` companion columns that currently lack `IntoR` impls — use `String` for text fields in enum DataFrameRow types."

   Replace with:
   > "For DataFrameRow enums, the macro also accepts lifetime params. Borrowed fields (`&'a str`, `&'a [T]`) produce `Vec<Option<&str>>` / `Vec<Option<&[T]>>` companion columns — both have `IntoR` impls (STRSXP+NA for `&str`, list-column for `&[T]`). Exported `#[miniextendr]` functions cannot carry explicit lifetime params (MXL112); use `'static` literals inside the fn body for fixtures."

8. **Clippy verification** — reproduce both CI jobs before pushing:
   - `clippy_default`: `cargo clippy --workspace --all-targets --locked -- -D warnings`
   - `clippy_all`: same + the full feature list from CLAUDE.md

## 6. Coherence and overlap analysis

- `Vec<Option<&'a str>>` is a concrete type distinct from `Vec<Option<String>>` (lifetime elided = HRTB in practice, but the impl is concrete). Rust resolves the specific impl; no overlap with `impl_vec_option_into_r!` macro (which covers `f64`, `i32`, `bool`, `String` — all owned types).
- `Vec<Option<&'a [T]>>` for `T: RNativeType` is distinct from `Vec<Option<Vec<T>>>`. No coherence conflict.
- `Vec<Option<&'a [String]>>` is distinct from `Vec<Option<Vec<String>>>`. No conflict.
- `Vec<Option<Cow<'_, str>>>` (line 1318) is unaffected — separate concrete type.
- No blanket impl is introduced. All impls follow the existing per-type pattern.

## 7. Files not touched

- `miniextendr-macros/src/dataframe_derive/enum_expansion.rs` — macro already correct
- `miniextendr-macros/src/dataframe_derive.rs` — no changes needed
- `miniextendr-api/src/from_r.rs` — this issue is `IntoR`-only
- `miniextendr-api/src/into_r/collections.rs` — new impls go in `into_r.rs` alongside other `Vec<Option<…>>` impls for locality

## 8. Acceptance criteria (from issue #465)

- `enum E<'a> { Named { id: i32, name: &'a str }, Bare { id: i32 } }` with `#[derive(DataFrameRow)] #[dataframe(align, tag = "_type")]` compiles and `to_dataframe` produces a STRSXP `name` column with `NA_character_` for Bare rows.
- `enum E<'a> { Buffer { label: String, data: &'a [f64] }, NoBuffer { label: String } }` with the same derive compiles; `to_dataframe` produces a list-column `data` with NULL for NoBuffer rows.
- The four-cell cardinality matrix (1v1r / 1vNr / Nv1r / NvNr split + NvNr align) passes for both shapes.
- `Vec<Option<String>>`, `Vec<Option<Cow<'_, str>>>`, and all `Vec<Option<scalar>>` impls are unaffected — existing tests continue to pass.
- GC stress fixture passes under `gctorture(TRUE)`.
