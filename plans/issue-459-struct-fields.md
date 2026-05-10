+++
title = "issue-459: struct as enum DataFrameRow variant field"
description = "Flatten struct-typed variant fields into prefixed columns via DataFrameRow derive; per-field as_list opt-out for opaque list-column."
+++

# issue-459: struct as enum `DataFrameRow` variant field

Tracks: <https://github.com/A2-ai/miniextendr/issues/459>

Locked design (agreed 2026-05-10): a struct-typed field on an enum variant flattens by default into
prefixed columns. The inner type **must** `#[derive(DataFrameRow)]` itself — proc macros can't
reach across crate boundaries to inspect a sibling type's fields, so the contract is explicit.
`#[dataframe(as_list)]` on the field suppresses flattening and keeps the field as a single opaque
list-column.

```rust
#[derive(DataFrameRow)]
struct Point { x: f64, y: f64 }

#[derive(DataFrameRow)]
enum Event {
    Located { id: i32, origin: Point },   // → cols: id, origin_x, origin_y
    Other,
}

// Opt-out: opaque list-column (third-party / arbitrary types)
Located {
    id: i32,
    #[dataframe(as_list)]
    origin: Point,                        // → cols: id, origin (list-col)
}
```

Compile error when inner type lacks `DataFrameRow` — macro surfaces a helpful
"add `#[derive(DataFrameRow)]` to `Point`, or annotate the field with
`#[dataframe(as_list)]`" message.

**Prerequisite**: PR #469 (HashMap/BTreeMap fields) must be merged to `main` before
implementation starts. This plan reads the current worktree state where
`FieldTypeKind::Map` + `EnumResolvedField::Map` are already present; all struct
changes build on that same machinery.

---

## Prior art: `FieldTypeKind::Map` (PR #469)

Every design decision below mirrors the map-field pattern from #469. Key files:

- `miniextendr-macros/src/dataframe_derive.rs` — `FieldTypeKind` enum,
  `classify_field_type`, `parse_field_attrs`, `FieldAttrs`, `EnumResolvedField`,
  `EnumMapFieldData` struct
- `miniextendr-macros/src/dataframe_derive/enum_expansion.rs` — resolution loop
  (named and tuple variants), column registration, sequential / parallel / split
  codegen arms
- `rpkg/src/rust/dataframe_enum_payload_matrix.rs` — region 8 "Map fields" is the
  fixture pattern to copy
- `rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R` — test pattern to copy

---

## 1. `FieldTypeKind::Struct` — detection and classification

**File:** `miniextendr-macros/src/dataframe_derive.rs`, function `classify_field_type` (~line 244)

### Detection strategy

`classify_field_type` uses a fallthrough rule: after all known special-cases
(`[T; N]`, `&[T]`, `Vec<T>`, `Box<[T]>`, `HashMap<K,V>`, `BTreeMap<K,V>`), any
remaining path type that:

1. is a `syn::Type::Path`, AND
2. is NOT one of the explicitly-classified idents above

... becomes `FieldTypeKind::Struct`. The macro cannot know at parse time whether
the path actually implements `DataFrameRow`; it emits a `const _: () = { fn
_assert<T: ::miniextendr_api::DataFrameRow>() {} _assert::<Inner>(); };`
compile-time assertion into the generated output. If `Inner` doesn't implement
`DataFrameRow`, rustc produces a trait-bound error; a `#[rustc_on_unimplemented]`
doc attribute on the `DataFrameRow` trait (if one exists) or a dedicated wrapper
assertion function with a hand-crafted error message provides the guidance.

Rejected alternative: requiring explicit `#[dataframe(flatten)]` — opt-in is less
ergonomic and the locked design specified opt-out via `as_list`.

Add the new variant to `FieldTypeKind`:

```rust
/// A struct-typed field whose type implements `DataFrameRow`.
/// Flattened into `<field>_<inner_col>` columns by default.
/// The inner type is carried so codegen can emit the correct trait constraint.
Struct {
    /// The full field type (used for the compile-time DataFrameRow assertion).
    inner_ty: &'a syn::Type,
},
```

In `classify_field_type`, after the `BTreeMap`/`HashMap` detection block and before
the final `FieldTypeKind::Scalar` fallthrough:

```rust
// Any remaining path type falls through to Struct.
// The compile-time DataFrameRow assertion in codegen surfaces a clear error
// if the inner type doesn't implement the trait.
if let syn::Type::Path(_) = ty {
    return FieldTypeKind::Struct { inner_ty: ty };
}
```

`Scalar` remains the fallthrough for all other type shapes (references, tuples,
function pointers, etc.).

**`as_list` gate**: `as_list` is checked **before** `classify_field_type` in both
`resolve_struct_field` and the enum resolution loop (existing pattern at
`enum_expansion.rs:81`, `200`). A field with `#[dataframe(as_list)]` becomes
`EnumResolvedField::Single` regardless of type, exactly as today. No changes
needed to `parse_field_attrs` or `FieldAttrs` — `as_list` already parses and is
already exclusive with `expand`/`width`.

**`width` and `expand` rejection**: add arms for `FieldTypeKind::Struct` alongside
the existing `FieldTypeKind::Map` rejections in `enum_expansion.rs` (lines ~136-155,
~254-273):

```rust
FieldTypeKind::Struct { .. } => {
    if fa.width.is_some() {
        return Err(syn::Error::new_spanned(
            &f.ty,
            "`width` is not valid on struct fields; use `#[dataframe(as_list)]` \
             to keep as an opaque list-column",
        ));
    }
    if fa.expand {
        return Err(syn::Error::new_spanned(
            &f.ty,
            "`expand`/`unnest` is not valid on struct fields; struct fields flatten \
             by default via their DataFrameRow impl",
        ));
    }
    resolved.push(EnumResolvedField::Struct(Box::new(EnumStructFieldData { .. })));
}
```

Also update `resolve_struct_field` (used for top-level struct `DataFrameRow`
derives): the `FieldTypeKind::Scalar | FieldTypeKind::Map { .. }` arm at
`dataframe_derive.rs:483` becomes
`FieldTypeKind::Scalar | FieldTypeKind::Map { .. } | FieldTypeKind::Struct { .. }`
— struct fields on *struct* `DataFrameRow` types fall through to a single opaque
`Single` column (struct-in-struct flattening is out of scope for this PR; see issue
#459 follow-up note). The error for `width`/`expand` on `Struct` is already
emitted by the shared rejection code in that arm.

---

## 2. `EnumResolvedField::Struct` + `EnumStructFieldData`

**File:** `miniextendr-macros/src/dataframe_derive.rs`, after `EnumMapFieldData` (~line 1684)

Add:

```rust
/// Data for [`EnumResolvedField::Struct`].
///
/// A field whose type implements `DataFrameRow` expands to `<base_name>_<inner_col>`
/// prefixed columns — one output column per column emitted by `Inner::column_names()`.
/// Absent-variant rows produce `None` in every prefixed column.
/// The inner type is carried as a `syn::Type` so codegen can emit the DataFrameRow
/// trait bound assertion and call `Inner::DataFrameColumns::column_names()`.
pub(super) struct EnumStructFieldData {
    /// Base name for column prefixing (field name or `rename` override).
    pub(super) base_name: String,
    /// Binding name in destructure pattern.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Inner struct type.
    pub(super) inner_ty: syn::Type,
}
```

Add `Struct(Box<EnumStructFieldData>)` to `EnumResolvedField` and update its two
`match` impls (`binding()` and `rust_name()`).

---

## 3. DataFrameRow trait contract — how to call into Inner

**The key design question:** what does the macro call on `Inner` to get column
names and push per-row data?

`DataFrameRow` is a derive-only marker — the actual contract is implicit in the
generated companion `{Name}DataFrame` struct. However, the `IntoDataFrame` trait
(in `miniextendr-api`) provides a stable surface:

- `Inner::to_dataframe(rows: Vec<Inner>) -> InnerDataFrame` — batches rows into
  columns (sequential).
- `InnerDataFrame` implements `IntoDataFrame`, which has `into_data_frame(self) -> DataFrameSexp`.

For the **column name** question at registration time: the macro cannot call a
runtime method at macro expansion time. Column names must be derived from the inner
type's `DataFrameRow` derive — but we're in a *different* invocation. The only
compile-time source of truth is the `{Inner}DataFrameColumns` companion type's
field names.

**Practical approach**: the macro emits a compile-time assertion
(`_assert_dataframe_row::<Inner>()`) but **does not try to enumerate Inner's column
names at macro-expansion time**. Instead, codegen uses a runtime helper that
calls `Inner::to_dataframe(vec![row])` per row and merges columns with prefix
injection. This is less efficient than the map approach but correct and simple.

Concrete API to emit in codegen:

```rust
// Sequential path per-row (for the Struct field binding):
let __inner_df = #inner_ty::to_dataframe(vec![#binding]);
// Then for each prefixed column, access __inner_df.<col_name>
```

Wait — `to_dataframe` is an associated function, not a trait method, so the macro
can call `#inner_ty::to_dataframe(...)` directly without any trait bound beyond
"the type implements `DataFrameRow`." The DataFrameRow assertion at the top of the
generated `impl` block enforces this.

**Column name enumeration at macro time**: The macro cannot statically know Inner's
column names. Two strategies:

**Strategy A (chosen)**: Emit a runtime bridge. The generated companion struct
registers *dynamic* extra columns via a parallel sidecar `Vec<(String, Vec<Option<SEXP>>)>`
that is populated row-by-row via `Inner::to_dataframe(vec![row])`, using the
resulting `IntoDataFrame::into_data_frame()` output to extract named columns. This
avoids all compile-time column enumeration but loses static typing on prefixed
columns in the companion struct — they become a dynamic `Vec<(String, SEXP)>` at
`into_data_frame` time.

**Strategy B (preferred, matches existing codegen shape)**: Emit a compile-time
`const COLUMN_NAMES: &[&str]` requirement on `DataFrameRow`. This is a trait
evolution that adds an associated const — feasible but requires changing the trait
and all existing impls, or using a default impl.

**Chosen approach (Strategy A, scoped)**: For this PR, use a **batched-then-prefix**
approach rather than per-row insertion into the companion struct. The companion
struct does **not** gain per-field prefixed columns for struct fields — instead,
the `into_data_frame()` implementation calls `Inner::to_dataframe(inner_rows)` on
the collected `Vec<Option<Inner>>` buffer and merges the resulting data frame into
the parent data frame with column name prefixing at `into_data_frame` time. This
matches R's mental model ("unflatten after the fact") and keeps the companion struct
manageable.

Concretely: the companion struct gains one field per struct variant field:

```rust
pub origin: Vec<Option<Inner>>,  // or Vec<Inner> for required fields
```

And `into_data_frame()` for this column group is:

```rust
// Separate the Some/None rows, call Inner::to_dataframe on the Some subset,
// then scatter columns back with None-fill and prefix.
let (__present_indices, __inner_rows): (Vec<_>, Vec<_>) = self.origin
    .iter()
    .enumerate()
    .filter_map(|(i, opt)| opt.as_ref().map(|v| (i, v.clone())))
    .unzip();
let __inner_df = #inner_ty::to_dataframe(__inner_rows);
// Then for each column in __inner_df, scatter into a Vec<Option<col_elem>> of
// length self.len() with None at absent rows.
```

This requires Inner: Clone + DataFrameRow. Both are reasonable constraints.

**Column registration for the companion struct**: since column names are runtime,
the parent companion struct's `into_data_frame()` emits these as a dynamic block
rather than as static `(name, sexp)` pairs. The outer `into_data_frame` already
builds a `pairs: Vec<(String, SEXP)>` — append the prefixed inner pairs there.

**Implication for parallel path**: `from_rows_par` collects `Vec<Option<Inner>>`
and then calls `Inner::to_dataframe` after joining threads — the inner parallelism
is lost for struct fields (acceptable; #459 is about correctness first).

---

## 4. Column registration in `derive_enum_dataframe`

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`, column
registry block (~line 340)

For `EnumResolvedField::Struct(data)`, register **one** column with the base name
and type `Option<Inner>`:

```rust
EnumResolvedField::Struct(data) => {
    let inner_ty = &data.inner_ty;
    let opt_inner: syn::Type = syn::parse_quote!(Option<#inner_ty>);
    registry.register(&data.base_name, &opt_inner, variant_idx, &vi.name, err_span)?;
}
```

The companion struct therefore has `pub origin: Vec<Option<Inner>>` which is typed
correctly. The `into_data_frame()` method is what performs the flattening.

---

## 5. Sequential codegen — `From<Vec<Enum>>`

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`, sequential
push block (~line 696)

For `EnumResolvedField::Struct(data)`, the push statement for this column is
straightforward — push `Some(binding)` when the variant matches:

```rust
EnumResolvedField::Struct(data) => {
    let col_name = format_ident!("{}", data.base_name);
    return quote! {
        #col_name.push(Some(#binding));
    };
}
```

The companion struct field `origin: Vec<Option<Inner>>` accumulates the inner rows.

The **flattening** happens in the companion struct's `into_data_frame()` override.
This means the standard `IntoDataFrame` impl must be customized for companion
structs that contain struct fields — the macro detects the presence of any
`EnumResolvedField::Struct` and emits a bespoke `into_data_frame()` body instead of
the default per-column `pairs.push((name, into_sexp(col)))` pattern.

The bespoke body for a struct field:

```rust
// For each struct field column (e.g., `origin: Vec<Option<Inner>>`):
{
    let (__present_idx, __inner_rows): (Vec<usize>, Vec<#inner_ty>) =
        self.origin.iter().enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|v| (i, v.clone())))
            .unzip();
    let __n = self.origin.len();
    let __inner_df = #inner_ty::to_dataframe(__inner_rows);
    // into_data_frame gives us the SEXP; we need per-column access.
    // Use the dynamic column extraction helper (to be added to miniextendr-api).
    let __inner_pairs = ::miniextendr_api::convert::dataframe_columns(__inner_df);
    for (col_name, col_sexp) in __inner_pairs {
        // Scatter: build Vec<Option<SEXP>> of len __n, filling None at absent rows.
        // ... (see §8 for the api helper)
        let prefixed = format!("{}_{}", stringify!(origin), col_name);
        __pairs.push((prefixed, __scope.protect_raw(scattered_sexp)));
    }
}
```

**API helper needed** (see §8): `dataframe_columns(df: InnerDataFrame) -> Vec<(String, SEXP)>`
— extracts named column SEXPs from a companion DataFrame. This may require adding
a method to the `IntoDataFrame` trait, or using the existing `into_data_frame()`
and then extracting the R VECSXP's names/elements. The latter avoids a trait change.

---

## 6. Parallel codegen — `from_rows_par`

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`, parallel
write block (~line 940)

For `EnumResolvedField::Struct(data)`:

```rust
EnumResolvedField::Struct(data) => {
    let col_name = format_ident!("{}", data.base_name);
    return quote! {
        #w_name.write(__i, Some(#binding));
    };
}
```

Same as sequential — the inner rows are collected into `Vec<Option<Inner>>` via
the parallel writer; flattening happens at `into_data_frame()` time (not during
parallel iteration), so no rayon constraint is introduced on `Inner`.

---

## 7. Split codegen — `to_dataframe_split`

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`,
`generate_split_method` (~line 1234)

Buffer declaration:

```rust
EnumResolvedField::Struct(data) => {
    let buf = format_ident!("__s_{}_{}", snake, data.base_name);
    let inner_ty = &data.inner_ty;
    buf_decls.push(quote! {
        let mut #buf: Vec<#inner_ty> = Vec::new();
    });
}
```

Push statement (non-optional — split only sees rows of this variant):

```rust
EnumResolvedField::Struct(data) => {
    let buf = format_ident!("__s_{}_{}", snake, data.base_name);
    vec![quote! { #buf.push(#binding); }]
}
```

Length expression:

```rust
EnumResolvedField::Struct(data) => {
    let buf = format_ident!("__s_{}_{}", snake, data.base_name);
    quote! { #buf.len() }
}
```

Pairs output — here we call `Inner::to_dataframe(buf)` and emit prefixed columns:

```rust
EnumResolvedField::Struct(data) => {
    let buf = format_ident!("__s_{}_{}", snake, data.base_name);
    let base = &data.base_name;
    let inner_ty = &data.inner_ty;
    vec![quote! {
        {
            let __inner_df = #inner_ty::to_dataframe(#buf);
            let __inner_pairs = ::miniextendr_api::convert::dataframe_columns(__inner_df);
            for (__col_name, __col_sexp) in __inner_pairs {
                let __prefixed = format!("{}_{}", #base, __col_name);
                #pairs_var.push((__prefixed, __scope.protect_raw(__col_sexp)));
            }
        }
    }]
}
```

---

## 8. API helper: `dataframe_columns`

**File:** `miniextendr-api/src/convert.rs` (or a new submodule)

The macro needs a way to extract `Vec<(String, SEXP)>` from a companion DataFrame
that has already been built. The cleanest approach is to add a free function that
calls `into_data_frame()` (which gives a `DataFrameSexp` / VECSXP), then reads the
R `names()` attribute and the list elements:

```rust
/// Extract named column SEXPs from a companion DataFrame type.
///
/// Called by `DataFrameRow`-derived enum code to flatten struct-typed fields.
/// Returns a `Vec<(String, SEXP)>` — each element is a column name and its SEXP.
/// The returned SEXPs are owned by the data frame SEXP and must be protected by
/// the caller before the data frame SEXP is released.
pub fn dataframe_columns<D: IntoDataFrame>(df: D) -> Vec<(String, SEXP)> {
    // into_data_frame consumes df and returns a DataFrameSexp (a VECSXP).
    let df_sexp = df.into_data_frame().into_sexp();
    // Read names and elements from the VECSXP.
    // ... (R API: Rf_getAttrib + R_NamesSymbol, VECTOR_ELT)
}
```

This function must be `pub` and `#[doc(hidden)]` (used only by macro-generated code).
It must protect `df_sexp` across the name/element extraction loop.

**Scatter helper**: for the sequential/parallel paths where columns must be
scattered (present at some row indices, `None` elsewhere), a second helper:

```rust
/// Scatter column SEXPs from an inner DataFrame into a `Vec<Option<SEXP>>` of
/// length `n`, with `None` at row indices not present in `present_indices`.
pub fn scatter_column(col_sexp: SEXP, present_indices: &[usize], n: usize) -> SEXP {
    // Builds a new VECSXP/type-appropriate SEXP of length n, inserting
    // col_sexp elements at present_indices and NA/NULL elsewhere.
}
```

This helper is called in the sequential/parallel `into_data_frame()` bespoke body.
For the split path, no scattering is needed (every row in a split partition belongs
to the same variant → all rows are "present").

**Alternative (simpler, preferred for initial implementation)**: skip the scatter
helper entirely. In the sequential and parallel `into_data_frame()`, instead of
scattering, call `Inner::to_dataframe` on the full `Vec<Option<Inner>>` by
substituting `Option<Inner>` — but `Inner` is a struct, not an Option. Alternative:
collect present rows into `Vec<Inner>` + present indices, call
`Inner::to_dataframe(present_rows)`, then scatter by building a new SEXP of length
`n` with NULL at absent rows. The scatter can be done column-by-column in R (using
`vector("list", n)` + index assignment) or in Rust with a VECSXP allocation loop.
A Rust loop is preferable (no R evaluation boundary).

---

## 9. `as_list` codegen path

**No new code needed in the classify/resolve loop.** `as_list` is already checked
before `classify_field_type` in both the named-field and tuple-field loops in
`enum_expansion.rs` (lines 81, 200). A field with `#[dataframe(as_list)]` becomes
`EnumResolvedField::Single` with type `Option<Inner>`, and the existing
`Single`-path codegen emits `col.push(Some(binding))` + `Vec<Option<Inner>>::into_sexp()`.

**Requirement for `as_list`**: `Inner: IntoR`. Without `IntoR`, the `Single`-path
`Vec<Option<Inner>>::into_sexp()` call doesn't compile — the user sees a rustc
error "the trait `IntoR` is not implemented for `Inner`." This is the correct
behavior and does not require special-casing.

**Doc note**: `as_list` produces a list-column where each cell is the R list
representation of `Inner` (from its `IntoR` impl, typically from `#[derive(IntoList)]`
or a manual impl). The `DataFrameRow` derive on Inner is **not** required when
using `as_list`.

---

## 10. Compile-time DataFrameRow assertion

In the generated `impl` block for the outer enum's companion struct, emit:

```rust
const _: () = {
    fn _assert_inner_is_dataframe_row<T: ::miniextendr_api::DataFrameRow>() {}
    _assert_inner_is_dataframe_row::<#inner_ty>();
};
```

This produces a rustc error on the inner type when it lacks `DataFrameRow`. The
error message will be:

```
the trait `DataFrameRow` is not implemented for `Point`
```

To supplement this with a helpful hint, add a `#[rustc_on_unimplemented]`
attribute to the `DataFrameRow` trait definition (if possible — requires nightly
or a stable alternative). Stable alternative: a wrapper function with a
`compile_error!` inside a `where` clause — but this requires nightly. For now,
document the error guidance in `docs/DATAFRAME.md` (see §17).

The assertion is emitted unconditionally for every `EnumResolvedField::Struct`
field, once per field (not per variant arm).

---

## 11. Fixture enum definition

**File:** `rpkg/src/rust/dataframe_enum_payload_matrix.rs`, new region after region 8

```rust
// region: 9. Struct fields (DataFrameRow flatten / as_list opt-out) ──────────────
//
// Struct-typed variant fields flatten into prefixed columns by default.
// Inner struct must #[derive(DataFrameRow)]. Per-field as_list keeps the struct
// as an opaque list-column (inner must then implement IntoR/IntoList).

#[derive(Clone, Debug, DataFrameRow)]
pub struct Point { pub x: f64, pub y: f64 }

// Also derive IntoList for the as_list path (needs IntoR):
// (Assumes #[derive(IntoList)] or manual impl is present)
// ...

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StructFlattenEvent {
    Located { id: i32, origin: Point },
    Other { id: i32 },
}

// as_list opt-out variant
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum StructListEvent {
    Located {
        id: i32,
        #[dataframe(as_list)]
        origin: Point,
    },
    Other { id: i32 },
}
```

### Fixture functions — all 4 cardinality cells × 2 paths × 2 modes

**Flatten mode** (`StructFlattenEvent`):

```rust
// 1v1r split
pub fn struct_flatten_split_1v1r() -> List { ... }
// 1vNr split
pub fn struct_flatten_split_1vnr() -> List { ... }
// Nv1r split
pub fn struct_flatten_split_nv1r() -> List { ... }
// NvNr split
pub fn struct_flatten_split_nvnr() -> List { ... }
// NvNr align
pub fn struct_flatten_align_nvnr() -> ToDataFrame<StructFlattenEventDataFrame> { ... }
```

**as_list mode** (`StructListEvent`):

```rust
pub fn struct_list_split_1v1r() -> List { ... }
pub fn struct_list_split_1vnr() -> List { ... }
pub fn struct_list_split_nv1r() -> List { ... }
pub fn struct_list_split_nvnr() -> List { ... }
pub fn struct_list_align_nvnr() -> ToDataFrame<StructListEventDataFrame> { ... }
```

---

## 12. R test file

**File:** `rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R`

New section at the end, following the "Map fields" section:

```r
# region: 9. Struct fields ─────────────────────────────────────────────────────

test_that("struct_flatten_split_1v1r: Located row has prefixed x/y columns", {
  res <- struct_flatten_split_1v1r()
  located <- res$Located
  expect_equal(colnames(located), c("_type", "id", "origin_x", "origin_y"))
  expect_equal(nrow(located), 1L)
  expect_equal(located$id, 1L)
  expect_equal(located$origin_x, 1.0)
  expect_equal(located$origin_y, 2.0)
  other <- res$Other
  expect_equal(nrow(other), 0L)
})

test_that("struct_flatten_split_1vnr: multiple Located rows", {
  res <- struct_flatten_split_1vnr()
  located <- res$Located
  expect_equal(nrow(located), 3L)
  expect_equal(located$origin_x, c(1.0, 3.0, 5.0))
  expect_equal(located$origin_y, c(2.0, 4.0, 6.0))
})

test_that("struct_flatten_split_nv1r: Located + Other each 1 row", {
  res <- struct_flatten_split_nv1r()
  expect_equal(nrow(res$Located), 1L)
  expect_equal(nrow(res$Other), 1L)
})

test_that("struct_flatten_split_nvnr: multiple variants, multiple rows", {
  res <- struct_flatten_split_nvnr()
  expect_true(nrow(res$Located) >= 2L)
  expect_true(nrow(res$Other) >= 2L)
})

test_that("struct_flatten_align_nvnr: aligned NA-fill for absent variant", {
  df <- struct_flatten_align_nvnr()
  expect_true("origin_x" %in% colnames(df))
  expect_true("origin_y" %in% colnames(df))
  # Other rows should have NA in origin_x and origin_y
  other_rows <- df[df$`_type` == "Other", ]
  expect_true(all(is.na(other_rows$origin_x)))
  expect_true(all(is.na(other_rows$origin_y)))
})

test_that("struct_list opt-out: origin column is a list-column", {
  res <- struct_list_split_1v1r()
  located <- res$Located
  expect_true(is.list(located$origin))
  expect_equal(length(located$origin), 1L)
  # Each cell is the R list rep of Point
  pt <- located$origin[[1]]
  expect_equal(pt$x, 1.0)
  expect_equal(pt$y, 2.0)
})

# endregion
```

---

## 13. GC stress fixture

**File:** `rpkg/src/rust/gc_stress_fixtures.rs`

Add a no-arg exported function (following `gc_stress_vec_option_collection` pattern):

```rust
/// Exercise struct-field DataFrameRow flatten + as_list paths under GC pressure.
///
/// Allocates StructFlattenEvent and StructListEvent rows, calls both
/// to_dataframe (align) and to_dataframe_split, and converts to SEXP, verifying
/// that OwnedProtect keeps inner column SEXPs live across scatter allocations.
#[miniextendr]
pub fn gc_stress_dataframe_struct() {
    use crate::dataframe_enum_payload_matrix::{
        Point, StructFlattenEvent, StructListEvent,
    };

    // Flatten path
    let rows = vec![
        StructFlattenEvent::Located { id: 1, origin: Point { x: 1.0, y: 2.0 } },
        StructFlattenEvent::Other { id: 2 },
        StructFlattenEvent::Located { id: 3, origin: Point { x: 3.0, y: 4.0 } },
        StructFlattenEvent::Other { id: 4 },
    ];
    let _ = StructFlattenEvent::to_dataframe(rows.clone()).into_sexp();
    let _ = StructFlattenEvent::to_dataframe_split(rows).into_sexp();

    // as_list path
    let list_rows = vec![
        StructListEvent::Located { id: 1, origin: Point { x: 1.0, y: 2.0 } },
        StructListEvent::Other { id: 2 },
    ];
    let _ = StructListEvent::to_dataframe(list_rows.clone()).into_sexp();
    let _ = StructListEvent::to_dataframe_split(list_rows).into_sexp();
}
```

This fixture is callable with no arguments, enabling the fast gctorture sweep
over rpkg exports (see `docs/GCTORTURE_TESTING.md`).

---

## 14. Compile-fail UI tests

**Directory:** `miniextendr-macros/tests/ui/`

### Test 1: inner type lacks DataFrameRow (and as_list not specified)

**File:** `miniextendr-macros/tests/ui/derive_dataframe_enum_struct_field_no_derive.rs`

```rust
//! Test: struct field without DataFrameRow derive produces clear error.

use miniextendr_macros::DataFrameRow;

// Point deliberately does NOT derive DataFrameRow
struct Point { x: f64, y: f64 }

#[derive(DataFrameRow)]
enum Event {
    Located { id: i32, origin: Point },
}

fn main() {}
```

**File:** `miniextendr-macros/tests/ui/derive_dataframe_enum_struct_field_no_derive.stderr`

```
error[E0277]: the trait `DataFrameRow` is not implemented for `Point`
 --> tests/ui/derive_dataframe_enum_struct_field_no_derive.rs:10:5
  |
  |     origin: Point,
  |     ^^^^^^^^^^^^^^ the trait `DataFrameRow` is not implemented for `Point`
  |
  = help: add `#[derive(DataFrameRow)]` to `Point`, or annotate the field with `#[dataframe(as_list)]` to keep it as an opaque list-column
```

(The exact error message depends on whether rustc emits the help text; the `.stderr`
file must be generated by running `cargo test` with `TRYBUILD=overwrite` after the
feature lands.)

### Test 2: width on struct field rejected

**File:** `miniextendr-macros/tests/ui/derive_dataframe_enum_struct_field_width.rs`

```rust
//! Test: width attribute on struct field is rejected.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
struct Point { x: f64, y: f64 }

#[derive(DataFrameRow)]
enum Event {
    Located {
        #[dataframe(width = 2)]
        origin: Point,
    },
}

fn main() {}
```

**File:** `miniextendr-macros/tests/ui/derive_dataframe_enum_struct_field_width.stderr`

```
error: `width` is not valid on struct fields; use `#[dataframe(as_list)]` to keep as an opaque list-column
```

### Test 3: expand on struct field rejected

**File:** `miniextendr-macros/tests/ui/derive_dataframe_enum_struct_field_expand.rs`

Similar pattern, asserting the `expand` rejection message.

---

## 15. Generated files sync

After any change to `#[miniextendr]` functions in `rpkg/src/rust/`:

```bash
just configure && just rcmdinstall && just devtools-document
```

Commit `rpkg/R/miniextendr-wrappers.R`, `rpkg/NAMESPACE`, and `rpkg/man/*.Rd`
in the same commit as the Rust changes that produced them. The pre-commit hook
at `.githooks/pre-commit` blocks commits where `*-wrappers.R` is staged without
matching `NAMESPACE`.

New exported fixture functions that must appear in `miniextendr-wrappers.R`:

- `gc_stress_dataframe_struct()`
- `struct_flatten_split_1v1r()`, `struct_flatten_split_1vnr()`, etc.
- `struct_list_split_1v1r()`, `struct_list_split_1vnr()`, etc.

---

## 16. CI reproduction — clippy before push

Two jobs must pass `-D warnings`:

```bash
# clippy_default
cargo clippy --workspace --all-targets --locked -- -D warnings

# clippy_all (copy the full feature list from CLAUDE.md)
cargo clippy --workspace --all-targets --locked \
  --features rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,\
num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,\
num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,\
vctrs,tinyvec,borsh,connections,nonapi,default-strict,default-coerce,default-r6,\
default-worker \
  -- -D warnings
```

Also run `cargo fmt --check --all` before push.

---

## 17. Documentation updates

### `docs/DATAFRAME.md`

Add a new subsection after "Map fields — parallel list-column expansion":

```markdown
### Struct fields — recurse and flatten

A struct-typed field on an enum variant flattens into prefixed columns if the
inner type implements `DataFrameRow`:

```rust
#[derive(DataFrameRow)]
struct Point { x: f64, y: f64 }

#[derive(DataFrameRow)]
enum Event {
    Located { id: i32, origin: Point },
    Other { id: i32 },
}
// Columns: _type, id, origin_x, origin_y
// Other rows: origin_x = NA, origin_y = NA
```

The inner type **must** `#[derive(DataFrameRow)]` — the contract is explicit. If
`Point` lacks the derive, rustc emits:

```
the trait `DataFrameRow` is not implemented for `Point`
```

Fix: add `#[derive(DataFrameRow)]` to `Point`, or annotate the field with
`#[dataframe(as_list)]`.

**`as_list` opt-out**: annotate the field to keep it as a single opaque list-column.
The inner type must then implement `IntoR` (typically via `#[derive(IntoList)]`):

```rust
Located {
    id: i32,
    #[dataframe(as_list)]
    origin: Point,  // → single list-column; Point must implement IntoR
}
```

**Detection caveats**:

- **Type aliases**: `type Loc = Point; field: Loc` — the segment is `Loc`, but
  `classify_field_type` treats any unknown path as `Struct`. The compile-time
  assertion still fires if `Loc` (i.e. `Point`) lacks `DataFrameRow`. This is
  correct behavior.
- **Generics on Inner**: `Inner<T>` fields are not supported for struct flattening;
  `DataFrameRow` rejects type parameters on the *outer* enum, and an inner type
  carrying type parameters can't be classified reliably. Use `as_list` for such
  fields.
- **Tuple structs**: `DataFrameRow` on tuple structs generates `_0`, `_1`, ...
  column names. These are valid prefixing targets (`origin_0`, `origin_1`).
- **`Option<Inner>`**: the outer segment is `Option` — `classify_field_type`
  doesn't detect this as `Struct` and treats it as `Scalar`. Use
  `#[dataframe(as_list)]` or unwrap before storing.
```

### `docs/CONVERSION_MATRIX.md`

Add to the DataFrameRow / struct section:

```markdown
| `Inner` (DataFrameRow struct) | Flattened to `<field>_<col>` columns | NA per column |
| `Option<Inner>` (as_list) | VECSXP list-column | NULL |
```

### `CLAUDE.md`

Add to the "Rust/FFI gotchas" section:

```markdown
- **`DataFrameRow` struct-in-enum flattening**: inner struct must `#[derive(DataFrameRow)]`. 
  The macro emits a compile-time `_assert_inner_is_dataframe_row::<Inner>()` check. 
  `as_list` on the field opts out of flattening (inner needs `IntoR` instead). 
  `Option<Inner>` in field position is NOT detected as a struct field — use `as_list`.
```

---

## Work order (flat priority)

1. **Add `FieldTypeKind::Struct`** to `classify_field_type` in
   `miniextendr-macros/src/dataframe_derive.rs` (fallthrough rule after Map).
2. **Add `EnumStructFieldData` struct** and `EnumResolvedField::Struct` variant;
   update `binding()` and `rust_name()` impls.
3. **Resolution loop** in `enum_expansion.rs` — both named-field and tuple-field
   loops: add `FieldTypeKind::Struct` arm with `width`/`expand` rejection and
   `EnumResolvedField::Struct` push.
4. **Column registration** — one `Vec<Option<Inner>>` column per struct field.
5. **Add `dataframe_columns` API helper** in `miniextendr-api/src/convert.rs`
   (extracts `Vec<(String, SEXP)>` from a companion DataFrame SEXP).
6. **Customize `into_data_frame()` emission** for companion structs containing
   struct fields — detect `EnumResolvedField::Struct` presence and emit bespoke
   scatter block instead of generic `pairs.push(...)`.
7. **Sequential push codegen** — `EnumResolvedField::Struct` arm in the sequential
   path: `col.push(Some(binding))`.
8. **Parallel write codegen** — `EnumResolvedField::Struct` arm in the parallel
   path: `w_name.write(__i, Some(binding))`.
9. **Split codegen** — buf declaration, push statement, length expression, pairs
   output with `Inner::to_dataframe(buf)` + prefix loop.
10. **Compile-time assertion** — emit `_assert_inner_is_dataframe_row` in the
    generated impl block for each `EnumResolvedField::Struct` field.
11. **Fixture enum + functions** in
    `rpkg/src/rust/dataframe_enum_payload_matrix.rs` — both `StructFlattenEvent`
    and `StructListEvent`, all 4 cardinality cells × 2 paths × 2 modes.
12. **GC stress fixture** in `rpkg/src/rust/gc_stress_fixtures.rs` —
    `gc_stress_dataframe_struct()` no-arg fn covering both flatten and as_list paths.
13. **R tests** in
    `rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R` — new section 9.
14. **Compile-fail UI tests** (3 fixtures + `.stderr` files):
    - `derive_dataframe_enum_struct_field_no_derive`
    - `derive_dataframe_enum_struct_field_width`
    - `derive_dataframe_enum_struct_field_expand`
15. **Generated files sync** — `just configure && just rcmdinstall &&
    just devtools-document`; commit wrappers + NAMESPACE + man/ in sync.
16. **clippy_default + clippy_all + fmt** — reproduce both CI jobs locally before push.
17. **Documentation** — `docs/DATAFRAME.md` new subsection, `docs/CONVERSION_MATRIX.md`
    table rows, `CLAUDE.md` gotcha entry.
18. **`just site-docs`** — regen `site/content/manual/` if DATAFRAME.md changed.

---

## Implementation constraints

- **`gh` heredoc rule**: raw backticks inside `cat <<'EOF'`; never pre-escape.
- **All 4 cardinality cells from the start**: 1v1r, 1vNr, Nv1r, NvNr for both
  `to_dataframe` (align) and `to_dataframe_split`, both flatten and as_list modes.
- **GC stress no-arg fixture mandatory** for the new SEXP-storage path.
- **Compile-fail UI tests mandatory** for the missing-DataFrameRow error.
- **clippy_default + clippy_all** both before push.
- **Generated files in same commit** as the Rust changes that produced them.
- **No `mod.rs`** — if touching a module, use `foo.rs` + `foo/` directory.
- **Fix all warnings encountered** even if pre-existing and unrelated.
- **Deferred scope cuts** → `gh issue create` referenced in PR body.

---

## Design decisions (resolved with user 2026-05-10)

**Q1 — RESOLVED: trait method on `IntoDataFrame`.** Add `fn into_named_columns(self) -> Vec<(String, SEXP)>` to the `IntoDataFrame` trait, with a default impl via `into_data_frame()`. Macro-generated code calls this on the inner struct's compiled DataFrame to obtain `(prefix + name, column-SEXP)` pairs.

**Q2 — RESOLVED: avoid Clone via presence mask.** The companion struct holds `Vec<Inner>` (densely packed, only the Some rows) plus a parallel `Vec<bool>` (or bitset) presence mask sized to the full row count. Scatter step reads the mask: present rows pull from the dense Inner DataFrame's columns; absent rows fill with the per-column NA default. No `Inner: Clone` requirement.

**Q3 — RESOLVED: use `rustc_on_unimplemented` on the `DataFrameRow` trait.** Stable trait attribute since Rust 1.82. Add `#[rustc_on_unimplemented(message = "..", label = "..", note = "..")]` pointing the user at `#[derive(DataFrameRow)]` on the inner type or `#[dataframe(as_list)]` on the field. Keep the `_assert_inner_is_dataframe_row::<Inner>()` compile-time assertion as belt-and-braces.

**Q4: Struct-in-struct (top-level DataFrameRow struct containing a struct field).**
Currently out of scope — `resolve_struct_field` treats `Struct` kind as `Single`
(opaque). If demand emerges, file a follow-up issue.
