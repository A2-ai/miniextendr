+++
title = "issue-458: nested enum as enum DataFrameRow variant field"
description = "Flatten a nested DataFrameRow enum as a variant field into prefixed discriminant + payload columns; opt-outs: as_factor (unit-only inner) and as_list (opaque list-column)."
+++

# issue-458: nested enum as enum `DataFrameRow` variant field

Tracks: <https://github.com/A2-ai/miniextendr/issues/458>

Prerequisite: #469 (HashMap/BTreeMap, merged) and #477 (struct fields, merged).
This plan layers on the `EnumResolvedField::Struct` + `scatter_column` + `into_named_columns`
machinery shipped in #477.

---

## Locked design (from issue #458)

A nested enum as a variant field flattens by default.  Inner enum must
`#[derive(DataFrameRow)]`. The expansion emits one discriminant column
(`<field>_variant`) plus prefixed payload columns using the same scatter pattern
as struct-field flattening:

```rust
#[derive(DataFrameRow)]
enum Inner { A, B { val: i32 } }

#[derive(DataFrameRow)]
enum Outer {
    Wrap { kind: Inner },       // → cols: kind_variant, kind_val (NA when A)
    Other,
}
```

Two opt-outs:

```rust
// Unit-only inner: skip DataFrameRow, emit a single factor/character column.
// Requires only Inner: IntoR.
Wrap {
    #[dataframe(as_factor)]
    kind: Inner,               // → col: kind (factor or character vector)
}

// Always-available escape: opaque list-column.
Wrap {
    #[dataframe(as_list)]
    kind: Inner,               // → col: kind (list-col)
}
```

---

## Key insight: zero new macro-level enum/struct distinction needed

`classify_field_type` already falls bare-ident path types (after the
KNOWN_SCALARS guard) to `FieldTypeKind::Struct`.  Enums that carry
`#[derive(DataFrameRow)]` already implement the `DataFrameRow` marker trait and
expose `to_dataframe()`/`into_named_columns()` — exactly the same contract
structs fulfill.  The discriminant column (`<field>_variant`) and prefixed
payload columns are *emitted by the inner enum's own `DataFrameRow` derive*, not
by the outer macro.  The outer macro just calls `inner.into_named_columns()` and
prefixes the result.

So for the **flatten path**, no new `FieldTypeKind` arm is needed.  The
`EnumResolvedField::Struct` arm already handles this correctly — every inner
`DataFrameRow` type (struct or enum) exposes the same trait surface.  The
discriminant column naming (`<field>_variant` vs `<inner_tag>`) is a matter of
what the inner enum's `to_dataframe` produces; if the inner enum uses
`#[dataframe(tag = "_variant")]` the outer prefix produces
`kind__variant`.  See the "inner tag naming" question below.

For **`as_factor`** only: a new `FieldAttrs` field is needed, plus a new
`EnumResolvedField` variant or reuse of `Single` with a new flag.  The `as_list`
path already works for enums via the existing `Single(needs_into_list = true)`
code path — no changes required.

---

## Work items (flat priority order)

### 1. Add `as_factor` to `FieldAttrs` and `parse_field_attrs`

**File:** `miniextendr-macros/src/dataframe_derive.rs`

Extend `FieldAttrs`:

```rust
pub(super) struct FieldAttrs {
    pub(super) skip: bool,
    pub(super) rename: Option<String>,
    pub(super) as_list: bool,       // already present
    pub(super) as_factor: bool,     // NEW
    pub(super) expand: bool,
    pub(super) width: Option<usize>,
}
```

In `parse_field_attrs`, add a new `else if meta.path.is_ident("as_factor")` arm
alongside the existing `as_list` arm:

```rust
} else if meta.path.is_ident("as_factor") {
    attrs.as_factor = true;
    Ok(())
}
```

Update the error string listing known attributes to include `as_factor`.

Add validation: `as_factor` is mutually exclusive with `as_list`, `expand`, and
`width`.  Error messages to add:

- `` `as_factor` and `as_list` are mutually exclusive ``
- `` `as_factor` and `expand`/`unnest` are mutually exclusive ``
- `` `as_factor` and `width` are mutually exclusive ``

**Note:** `as_factor` is semantically valid only on `FieldTypeKind::Struct`
(bare-ident types, i.e. user-defined enums/structs) and makes no sense on
scalars, slices, Vec, or maps.  The error for misuse on a non-struct type is
emitted when the field is resolved in `enum_expansion.rs` (see item 3).

### 2. `classify_field_type` — no changes required

`FieldTypeKind::Struct` already captures bare-ident types that are not KNOWN_SCALARS.
Enums implementing `DataFrameRow` are structurally identical to structs at the
proc-macro level.  The KNOWN_SCALARS guard remains unchanged.

**Explicit verification:** confirm that `i32`, `i64`, `f64`, `String`, `bool`
etc. still classify as `Scalar` (not `Struct`).  The bench
`miniextendr-bench/benches/dataframe.rs` is the canary; if it stops compiling,
the guard regressed.  No change to `classify_field_type` means this cannot
regress.

### 3. `EnumResolvedField` — `AsFactor` flag on `Struct` variant

**File:** `miniextendr-macros/src/dataframe_derive.rs`

Extend `EnumStructFieldData` with an `as_factor: bool` flag instead of
introducing a new enum variant.  The distinction is small enough that a flag is
cleaner and avoids duplicating the registration/codegen match arms for a nearly-
identical case:

```rust
pub(super) struct EnumStructFieldData {
    pub(super) base_name: String,
    pub(super) binding: syn::Ident,
    pub(super) rust_name: syn::Ident,
    pub(super) inner_ty: syn::Type,
    pub(super) as_factor: bool,     // NEW
}
```

**`as_factor = true` codegen contract:**

- Column type: `Vec<Option<InnerTy>>` in companion struct (same as flatten).
- `into_data_frame` / split: emit `Vec<Option<InnerTy>>.into_sexp()` directly —
  i.e. `IntoR for Vec<Option<InnerTy>>` is called, which produces a
  character/factor SEXP.  No `to_dataframe()`, no scatter.
- This requires `InnerTy: IntoR` (not `DataFrameRow`), which is weaker than the
  flatten contract.  For unit-only enums, `IntoR` is already implemented via the
  `#[miniextendr]` / `IntoR for T where T: ToString` chain or an explicit impl.

**Column name:** a single column named `<base_name>` (same as `as_list`).

**Registration:** `as_factor = true` fields register as a single column of type
`InnerTy` (like `Single`).  The companion struct field is `Vec<Option<InnerTy>>`.
In `enum_expansion.rs`, treat an `as_factor` Struct field as a `Single` field
with `needs_into_list = false` and `ty = inner_ty` — i.e. just route it through
the existing `Single` path with no special handling.

**Simplest implementation:** in the resolution loop (both named and tuple variant
branches in `enum_expansion.rs`), when `FieldTypeKind::Struct` is detected and
`fa.as_factor` is true, push an `EnumResolvedField::Single(Box::new(EnumSingleFieldData { ..., needs_into_list: false, ty: inner_ty.clone() }))`.
This avoids touching `EnumStructFieldData` at all for the `as_factor` path and
requires no codegen changes in the Struct arms.

**Compile-time safety:** For `as_factor`, no `DataFrameRow` assertion is emitted.
A compile error will surface naturally if `InnerTy` lacks `IntoR` when `into_sexp`
is called on `Vec<Option<InnerTy>>` — no bespoke assertion needed.

### 4. Resolution loop in `enum_expansion.rs` — handle `as_factor`

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`

In both the named-field and tuple-field resolution branches, the current flow is:

1. If `fa.as_list` → push `Single(needs_into_list = true)`.
2. Else match `classify_field_type(&f.ty)`:
   - `Struct { inner_ty }` → push `Struct(EnumStructFieldData { ..., inner_ty })`.
   - `Scalar` → push `Single(needs_into_list = false)`.
   - etc.

Add a third early-exit branch: if `fa.as_factor && classify_field_type(&f.ty)` is
`Struct { inner_ty }`, push `Single(needs_into_list: false, ty: inner_ty.clone())`.

Emit an error if `fa.as_factor` is true and the field type is NOT
`FieldTypeKind::Struct`:

```rust
if fa.as_factor {
    return Err(syn::Error::new_spanned(
        &f.ty,
        "`as_factor` is only valid on bare-ident enum/struct types, not on \
         scalar, Vec, slice, or map fields",
    ));
}
```

Emit an error if `fa.as_factor` is true and the type has generic arguments
(e.g. `Inner<T>`):

This is automatically handled by the KNOWN_SCALARS guard + bare-ident check
inside `classify_field_type` — a type with angle brackets does not hit the
`Struct` arm.  But for a cleaner error, check `fa.as_factor && FieldTypeKind is
not Struct` and surface:

```rust
"`as_factor` requires a bare-ident type (unit-only enum); use `as_list` for \
 generic or complex types"
```

### 5. Companion struct companion field type for Struct (flatten) — no changes

The companion struct for `EnumResolvedField::Struct(data)` already stores
`Vec<Option<InnerTy>>`.  For enums (Inner = DataFrameRow enum), this is correct.
The `into_data_frame` path calls `Inner::to_dataframe(present_rows)` and then
calls `into_named_columns()` — this returns the inner enum's columns including its
`_variant` / `_tag` discriminant column (if `#[dataframe(tag = "...")]` is set)
plus all payload columns.

**Inner tag naming convention:** if Inner uses `#[dataframe(tag = "_variant")]`,
the outer prefix produces `kind__variant` (double underscore). Document this
caveant and recommend `#[dataframe(tag = "type")]` → `kind_type`, or rely on the
default no-tag mode where the inner enum emits no discriminant column.

For the most ergonomic case:

```rust
#[derive(DataFrameRow)]
#[dataframe(align, tag = "_variant")]
enum Inner { A, B { val: i32 } }
```

Outer `Wrap { kind: Inner }` → columns: `kind__variant` (discriminant), `kind_val`
(payload, NA when A).

**Recommendation in docs:** use `#[dataframe(tag = "variant")]` on Inner (single
underscore) → outer produces `kind_variant`.

### 6. Codegen paths — all three modes, both sequential and split

**Flatten path (`as_factor = false, as_list = false`):**

- Registration: `EnumResolvedField::Struct` with `as_factor: false`.
  Uses existing `struct_cols` collection logic — no changes required (since the
  `EnumResolvedField::Struct(data)` arm is already complete for both sequential
  and split paths).

- `into_data_frame` (sequential + parallel): existing `struct_flatten_pushes` loop
  in `enum_expansion.rs` handles this — collects present rows densely, calls
  `Inner::to_dataframe(present_rows)`, calls `into_named_columns()`, scatters via
  `scatter_column`.  No changes.

- `to_dataframe_split`: existing `EnumResolvedField::Struct` arm in
  `generate_split_method` pushes `binding` into `Vec<Inner>` buffer, calls
  `Inner::to_dataframe(buf)`, calls `into_named_columns()`, prefixes and pushes.
  No changes.

**`as_factor` path:**

- Routed to `EnumResolvedField::Single(needs_into_list: false)` in resolution.
- No changes to `into_data_frame` or split codegen — existing `Single` arm handles it.
- Column is a STRSXP/INTSXP factor depending on `IntoR for Option<InnerTy>`.

**`as_list` path:**

- Already handled by `EnumResolvedField::Single(needs_into_list: true)` when
  `fa.as_list` is true and inner type is `Struct`.
- No changes required.

### 7. Marker trait update for `DataFrameRow` on enums

**File:** `miniextendr-macros/src/dataframe_derive.rs`, function `derive_enum_dataframe`

The existing enum `DataFrameRow` derive emits the `DataFrameRow` marker trait
impl at the end of `derive_enum_dataframe` (search for `marker_impl`).  Verify
this is also emitted for enum DataFrameRow types — it already is (the marker impl
is emitted at the struct-level `derive_struct_dataframe`; confirm
`derive_enum_dataframe` has an equivalent).  If not present, add:

```rust
let marker_impl = quote! {
    impl #impl_generics ::miniextendr_api::markers::DataFrameRow
        for #row_name #ty_generics #where_clause {}
};
```

This is required so that an inner enum used as an outer field satisfies the
compile-time assertion emitted by the outer `DataFrameRow` derive.

**Action:** grep for the marker impl in `derive_enum_dataframe` — if absent, add
it alongside the existing struct marker impl.  This is critical: without it,
nesting `DataFrameRow` enums inside other `DataFrameRow` enums will produce a
compile error about `DataFrameRow` not being implemented.

### 8. Test fixtures — Rust side

**File:** `rpkg/src/rust/dataframe_enum_payload_matrix.rs`

Add a new region "10. Nested enum fields" following the existing region-9 struct
convention.

#### Types to define

```rust
// Unit-only inner enum — only used with as_factor opt-in.
// #[derive(DataFrameRow)] not required; must implement IntoR.
#[derive(Clone, Debug)]
pub enum Direction { North, South, East, West }
// Needs IntoR impl (string conversion). Add in the fixture module.

// Inner enum with payload — used for flatten path.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(tag = "variant")]  // → col name: kind_variant (single underscore)
pub enum Status {
    Active,
    Suspended { reason: String },
}

// Outer: flatten path.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedFlattenEvent {
    Tracked { id: i32, status: Status },
    Other { id: i32 },
}

// Outer: as_factor path.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedFactorEvent {
    Move { id: i32, dir: Direction },
    Stop { id: i32 },
}

// Outer: as_list path (already covered by struct as_list; add a smoke test for enums too).
#[derive(Clone, Debug, DataFrameRow, IntoList)]
pub struct StatusWrapper { pub s: Status }  // NOT needed — as_list on enum directly.

// For as_list: Status must implement IntoList (or IntoR via IntoList).
// Simplest: #[derive(Clone, Debug, DataFrameRow, IntoList)] on Status — but Status has payload,
// so IntoList will serialize it as a list. Alternatively add a simpler unit enum for as_list test.
// Use Direction (unit enum) for as_list smoke test since it has IntoR.
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum NestedListEvent {
    Move {
        id: i32,
        #[dataframe(as_list)]
        dir: Direction,
    },
    Stop { id: i32 },
}
```

**Note on `IntoR` for `Direction`:** Add a simple `IntoR` impl for `Direction`
that maps variants to string values, or derive `IntoR` if supported.  The
`as_factor` path requires `Vec<Option<Direction>>.into_sexp()` to produce a
STRSXP (character vector).  Since `Direction` does not implement `RNativeType`,
the `Vec<Option<Direction>>` blanket impl is not available — a manual `IntoR for
Vec<Option<Direction>>` or an `IntoR for Direction` impl is needed.  Check whether
the existing `#[miniextendr]` machinery can derive this, or write it by hand in
the fixture file.

**Simpler alternative for `as_factor` test:** Use `String`-returning impls or
simply test that the column exists and is character type in R.

#### `#[miniextendr]` fixture functions

All 4 cardinality cells × `to_dataframe` AND `to_dataframe_split` × 3 modes
(flatten / as_factor / as_list) = 24 test entry points minimum.

**Flatten path (8 functions):**

```rust
pub fn nested_flatten_split_1v1r() -> List { ... }    // 1 Tracked row
pub fn nested_flatten_split_1vnr() -> List { ... }    // N Tracked rows
pub fn nested_flatten_split_nv1r() -> List { ... }    // 1 Tracked + 1 Other
pub fn nested_flatten_split_nvnr() -> List { ... }    // N Tracked + N Other
pub fn nested_flatten_align_1v1r() -> ToDataFrame<NestedFlattenEventDataFrame> { ... }
pub fn nested_flatten_align_1vnr() -> ToDataFrame<NestedFlattenEventDataFrame> { ... }
pub fn nested_flatten_align_nv1r() -> ToDataFrame<NestedFlattenEventDataFrame> { ... }
pub fn nested_flatten_align_nvnr() -> ToDataFrame<NestedFlattenEventDataFrame> { ... }
```

The `to_dataframe` align path outputs a single aligned data frame; the
`to_dataframe_split` path outputs a named list of per-variant data frames.
Columns expected from `NestedFlattenEvent`:
- `_type`: character (Tracked / Other)
- `id`: integer (Some / Some)
- `status_variant`: character (Active / Suspended / NA)
- `status_reason`: character (NA / reason string / NA)

**`as_factor` path (8 functions):**

```rust
pub fn nested_factor_split_1v1r() -> List { ... }
pub fn nested_factor_split_1vnr() -> List { ... }
pub fn nested_factor_split_nv1r() -> List { ... }
pub fn nested_factor_split_nvnr() -> List { ... }
pub fn nested_factor_align_1v1r() -> ToDataFrame<NestedFactorEventDataFrame> { ... }
pub fn nested_factor_align_1vnr() -> ToDataFrame<NestedFactorEventDataFrame> { ... }
pub fn nested_factor_align_nv1r() -> ToDataFrame<NestedFactorEventDataFrame> { ... }
pub fn nested_factor_align_nvnr() -> ToDataFrame<NestedFactorEventDataFrame> { ... }
```

Columns expected: `_type`, `id`, `dir` (character/factor, NA for Stop rows).

**`as_list` path (8 functions):**

```rust
pub fn nested_list_split_1v1r() -> List { ... }
pub fn nested_list_split_1vnr() -> List { ... }
pub fn nested_list_split_nv1r() -> List { ... }
pub fn nested_list_split_nvnr() -> List { ... }
pub fn nested_list_align_1v1r() -> ToDataFrame<NestedListEventDataFrame> { ... }
pub fn nested_list_align_1vnr() -> ToDataFrame<NestedListEventDataFrame> { ... }
pub fn nested_list_align_nv1r() -> ToDataFrame<NestedListEventDataFrame> { ... }
pub fn nested_list_align_nvnr() -> ToDataFrame<NestedListEventDataFrame> { ... }
```

Columns expected: `_type`, `id`, `dir` (list-col, NULL for Stop rows).

#### Rust unit tests

Add `#[cfg(test)] mod nested_enum_field_tests` with:

- Flatten: companion struct has `status: Vec<Option<Status>>` field; verify `None`
  for `Other` rows, `Some(Status::Active)` etc. for `Tracked` rows.
- `as_factor`: companion struct has `dir: Vec<Option<Direction>>` field.
- `as_list`: same shape as struct `as_list` test.
- Verify `to_dataframe` absent-variant rows produce `None` in the `status` field.

### 9. Test fixtures — R side

**File:** `rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R`

Add a new section "# 10. Nested enum fields" following the region-9 struct
convention.

#### Flatten path checks (per cardinality cell):

```r
test_that("nested_flatten_split_1v1r: single Tracked row has prefixed columns", {
  res <- nested_flatten_split_1v1r()
  tracked <- res$Tracked
  expect_equal(nrow(tracked), 1L)
  expect_true("status_variant" %in% names(tracked))
  expect_true("status_reason" %in% names(tracked))
  expect_equal(tracked$status_variant, "Active")
  expect_true(is.na(tracked$status_reason))
})
```

Add equivalent tests for 1vNr, Nv1r, NvNr (both split and align):

- Split: verify per-variant partition sizes and column presence.
- Align: verify `NA` fill in `status_variant` and `status_reason` for `Other` rows.
- Verify `status_variant` == `"Suspended"` and `status_reason` is non-NA for
  Suspended rows.
- Verify `status_reason` is `NA` for `Active` rows (unit variant in inner enum).

#### `as_factor` path checks:

- `dir` column is character (or factor) type.
- `dir` values are `"North"` / `"South"` etc. (or factor level labels).
- `dir` is `NA` for `Stop` rows.
- Both split and align, all 4 cardinality cells.

#### `as_list` path checks:

- `dir` column is a list (VECSXP) for align, or list-column for split.
- `NULL` for absent-variant rows in align mode.
- Length-1 character vector per present row in split mode (if `Direction` maps to
  string via `IntoList`).

### 10. GC stress fixture

**File:** `rpkg/src/rust/gc_stress_fixtures.rs`

Add `gc_stress_dataframe_nested_enum()` no-arg function alongside the existing
`gc_stress_dataframe_struct()`.  Pattern: identical to `gc_stress_dataframe_struct`
but using `NestedFlattenEvent`, `NestedFactorEvent`, and `NestedListEvent`:

```rust
/// Exercise nested-enum field DataFrameRow flatten + as_factor + as_list paths
/// under GC pressure.
///
/// Drives both `to_dataframe` (align) and `to_dataframe_split` paths.
/// No arguments — suitable for the fast gctorture sweep.
#[miniextendr]
pub fn gc_stress_dataframe_nested_enum() {
    use miniextendr_api::into_r::IntoR as _;

    // Flatten path
    let flatten_rows = vec![
        NestedFlattenEvent::Tracked { id: 1, status: Status::Active },
        NestedFlattenEvent::Other { id: 2 },
        NestedFlattenEvent::Tracked { id: 3, status: Status::Suspended { reason: "x".into() } },
        NestedFlattenEvent::Other { id: 4 },
    ];
    let _ = NestedFlattenEvent::to_dataframe(flatten_rows.clone()).into_sexp();
    let _ = NestedFlattenEvent::to_dataframe_split(flatten_rows).into_sexp();

    // as_factor path
    let factor_rows = vec![
        NestedFactorEvent::Move { id: 1, dir: Direction::North },
        NestedFactorEvent::Stop { id: 2 },
        NestedFactorEvent::Move { id: 3, dir: Direction::East },
        NestedFactorEvent::Stop { id: 4 },
    ];
    let _ = NestedFactorEvent::to_dataframe(factor_rows.clone()).into_sexp();
    let _ = NestedFactorEvent::to_dataframe_split(factor_rows).into_sexp();

    // as_list path
    let list_rows = vec![
        NestedListEvent::Move { id: 1, dir: Direction::South },
        NestedListEvent::Stop { id: 2 },
        NestedListEvent::Move { id: 3, dir: Direction::West },
        NestedListEvent::Stop { id: 4 },
    ];
    let _ = NestedListEvent::to_dataframe(list_rows.clone()).into_sexp();
    let _ = NestedListEvent::to_dataframe_split(list_rows).into_sexp();
}
```

Add `gc_stress_dataframe_nested_enum` to the rpkg R wrapper via `just configure
&& just rcmdinstall && just devtools-document`.

### 11. UI compile-fail tests

**File:** `miniextendr-macros/tests/ui/`

Add the following test files (`.rs` + `.stderr`):

#### a. `derive_dataframe_enum_nested_enum_no_derive.rs`

Inner enum without `DataFrameRow` derive, used as a non-`as_factor` field (flatten
path).  Expected error: `DataFrameRow` not implemented for inner type + guidance
message from `#[diagnostic::on_unimplemented]`.

```rust
//! Test: nested enum without DataFrameRow derive produces clear error.

use miniextendr_macros::DataFrameRow;

// StatusNoDerived deliberately does NOT derive DataFrameRow.
#[derive(Clone, Debug)]
enum Status { Active, Suspended { reason: String } }

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Tracked { id: i32, status: Status },
    Other { id: i32 },
}

fn main() {}
```

Expected `.stderr`: mirrors `derive_dataframe_enum_struct_field_no_derive.stderr`
pattern — `DataFrameRow` not implemented + `add #[derive(DataFrameRow)] to Status,
or annotate the field with #[dataframe(as_list)]`.

#### b. `derive_dataframe_enum_nested_enum_as_factor_payload.rs`

**Note:** this test verifies the *runtime / compile-time behavior* when a user
annotates a payload-bearing inner enum with `#[dataframe(as_factor)]`.  At
compile time, `as_factor` routes through `Single(needs_into_list: false, ty:
InnerTy)`, which calls `Vec<Option<InnerTy>>.into_sexp()`.  If `InnerTy` does not
implement `IntoR` (which payload enums without an explicit impl do not), the
compile fails with a missing-`IntoR` error.  Write the test to confirm this error
fires:

```rust
//! Test: as_factor on payload-bearing enum (no IntoR impl) → compile error.

use miniextendr_macros::DataFrameRow;

// Inner has a payload variant and no IntoR impl.
#[derive(Clone, Debug)]
enum Status { Active, Suspended { reason: String } }

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Tracked {
        id: i32,
        #[dataframe(as_factor)]
        status: Status,
    },
    Other { id: i32 },
}

fn main() {}
```

Expected `.stderr`: `Vec<Option<Status>>` does not implement `IntoR` (or similar
trait-bound error).  The exact error text will be known after the implementation
compiles; capture it and commit the `.stderr` file.

#### c. `derive_dataframe_enum_nested_enum_as_factor_invalid_type.rs`

`#[dataframe(as_factor)]` on a non-struct/enum field (e.g. `Vec<i32>`).

```rust
//! Test: as_factor on Vec<i32> → compile error.

use miniextendr_macros::DataFrameRow;

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Ev { #[dataframe(as_factor)] vals: Vec<i32> },
    Other,
}

fn main() {}
```

Expected `.stderr`: `` `as_factor` is only valid on bare-ident enum/struct types ``.

#### d. `derive_dataframe_enum_nested_enum_as_factor_as_list_conflict.rs`

Both `as_factor` and `as_list` on the same field.

```rust
//! Test: as_factor + as_list conflict → compile error.

use miniextendr_macros::DataFrameRow;

#[derive(Clone, Debug)]
enum Dir { N, S }

#[derive(DataFrameRow)]
#[dataframe(tag = "_type")]
enum Outer {
    Ev {
        #[dataframe(as_factor, as_list)]
        dir: Dir,
    },
    Other,
}

fn main() {}
```

Expected `.stderr`: `` `as_factor` and `as_list` are mutually exclusive ``.

### 12. `DataFrameRow` marker impl on enums — verify and add if absent

**File:** `miniextendr-macros/src/dataframe_derive.rs`, `derive_enum_dataframe`

Check whether `derive_enum_dataframe` emits the `DataFrameRow` marker trait impl.
If absent, add:

```rust
let enum_marker_impl = quote! {
    impl #impl_generics ::miniextendr_api::markers::DataFrameRow
        for #row_name #ty_generics #where_clause {}
};
```

Include it in the returned `TokenStream` output alongside the companion struct and
`From<Vec<…>>` impl.  Without this, nesting a `DataFrameRow` enum inside another
`DataFrameRow` enum's flatten path will produce a compile error at the
`_assert_inner_is_dataframe_row` site.

### 13. `diagnostic::on_unimplemented` message update for enums

**File:** `miniextendr-api/src/markers.rs`

Update the `#[diagnostic::on_unimplemented]` message on `DataFrameRow` to mention
both structs and enums:

```rust
#[diagnostic::on_unimplemented(
    message = "the trait `DataFrameRow` is not implemented for `{Self}`",
    label = "add `#[derive(DataFrameRow)]` to `{Self}`, or annotate the field \
             with `#[dataframe(as_list)]` to keep it as an opaque list-column, \
             or annotate with `#[dataframe(as_factor)]` for unit-only enums",
    note = "struct- and enum-typed variant fields are flattened by default into \
            prefixed columns; the inner type must implement `DataFrameRow` for \
            this to work"
)]
pub trait DataFrameRow {}
```

### 14. Doc updates

**File:** `docs/DATAFRAME.md`

Update the sentence at line ~203 that says "Nested enums and struct-typed fields
are tracked by issues #458 / #459":
- Remove `#458 /` (struct fields landed in #477).
- Remove `#459` if struct fields are shipped.
- Actually: both #458 and #459 are now being addressed — update to say struct
  fields are shipped in #477 and nested enums are tracked by #458.

Add a new subsection "Nested enum fields" after the "Struct fields" subsection.
Structure:

```markdown
### Nested enum fields — flatten + opt-outs

A nested enum as a variant field flattens by default into prefixed columns.
The inner enum must `#[derive(DataFrameRow)]`.

[code example showing Inner + Outer + columns produced]

#### `as_factor` — unit-only inner enum

[example + note that Inner must have IntoR, not DataFrameRow]
[note about what R type is produced: character vector, NA for absent rows]

#### `as_list` — opaque list-column

[same as struct as_list — Inner must implement IntoList]

#### Inner tag naming

[note about kind_variant vs kind__variant depending on tag = "variant"]
[recommend tag = "variant" on Inner to get clean kind_variant outer col name]
```

**File:** `docs/CONVERSION_MATRIX.md`

Add rows for the new column types produced by nested-enum flatten:

| Nested enum field (flatten) | discriminant col: STRSXP | NA for absent-variant rows |
| Nested enum field (`as_factor`) | STRSXP or INTSXP (factor) | NA for absent-variant rows |

**File:** `CLAUDE.md`

Add a brief gotcha under "Rust/FFI gotchas" or under the DataFrameRow section:

> **Nested `DataFrameRow` enums**: inner enum must emit the `DataFrameRow` marker
> trait (i.e. must `#[derive(DataFrameRow)]` itself — not just `#[derive(IntoR)]`).
> `as_factor` opts out of the `DataFrameRow` requirement — only `IntoR` is needed.
> The inner enum's `#[dataframe(tag = "variant")]` column is exposed as
> `<outer_field>_variant` after prefixing.

### 15. Generated files in sync

After any change that affects R wrapper output:

```bash
just configure && just rcmdinstall && just devtools-document
```

Commit `rpkg/R/miniextendr-wrappers.R`, `rpkg/NAMESPACE`, and `rpkg/man/*.Rd`
in the same commit as the Rust changes that produced them.

### 16. CI reproduction

Before pushing:

```bash
# clippy_default
cargo clippy --workspace --all-targets --locked -- -D warnings

# clippy_all (all non-mutually-exclusive features)
cargo clippy --workspace --all-targets --locked --features rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,vctrs,tinyvec,borsh,connections,nonapi,default-strict,default-coerce,default-r6,default-worker -- -D warnings

just fmt
```

---

## Implementation order (priority-sorted)

1. Add `as_factor` to `FieldAttrs` + `parse_field_attrs` + mutual-exclusion
   validation.
2. Verify (and add if absent) `DataFrameRow` marker impl on enums in
   `derive_enum_dataframe`.
3. Handle `as_factor` in enum resolution loop — route to `Single` path.
4. Emit error for `as_factor` on non-struct types.
5. Write UI compile-fail tests (a, b, c, d) and capture `.stderr` files.
6. Write Rust fixtures in `dataframe_enum_payload_matrix.rs`.
7. Write R tests in `test-dataframe-enum-payload-matrix.R`.
8. Add `gc_stress_dataframe_nested_enum` fixture.
9. Run `just configure && just rcmdinstall && just devtools-document`; commit
   generated files.
10. Update `docs/DATAFRAME.md`, `docs/CONVERSION_MATRIX.md`, `CLAUDE.md`.
11. Reproduce CI clippy (default + all) locally; fix any new lints.

---

## Constraints for implementer

- **gh heredoc backtick rule:** use `<<'EOF'` with raw backticks in `gh pr create`
  body; never pre-escape with `\``.
- **All 24 test cells from the start:** all 4 cardinality cells × `to_dataframe` +
  `to_dataframe_split` × 3 modes (flatten / as_factor / as_list). Do not defer
  cells.
- **Reproduce CI clippy locally before pushing:** both `clippy_default` and
  `clippy_all` feature sets.
- **Generated files committed in sync:** `*-wrappers.R`, `NAMESPACE`, `man/*.Rd`.
- **GC stress fixture is required:** `gc_stress_dataframe_nested_enum()` no-arg fn.
- **UI compile-fail tests are required:** all four listed above. Run with `cargo
  test --test ui` after capturing `.stderr` files.
- **KNOWN_SCALARS guard must not regress:** primitive fields (`i32`, `i64`, `f64`,
  `String`, etc.) must still classify as `Scalar`.  Run `miniextendr-bench/benches/
  dataframe.rs` to confirm.
- **`IntoDataFrame::into_named_columns` migration:** if you need to move
  `IntoDataFrame` between the row type and companion struct, every call site
  (especially the bench) must be updated in the same PR.
- **Inner tag naming:** document the `kind__variant` vs `kind_variant` convention
  clearly. Recommend `#[dataframe(tag = "variant")]` on Inner.
- **No `mod.rs` pattern:** use `foo.rs` + `foo/` directory.
- **Every deferred item → `gh issue create`:** if scope is cut, reference the
  issue in the PR body.

---

## Design decisions (resolved with user 2026-05-10)

**Q1 — RESOLVED: auto-emit `IntoR` for unit-only enums via `DataFrameRow` derive.** When the inner enum has only unit variants, `#[derive(DataFrameRow)]` also emits `impl IntoR for Inner` (variant ident → string via the same convention used by existing enum→string codegen). Skip this auto-emission if any variant has payload. Zero user boilerplate for the common case. The implementer should add this auto-emission in `derive_enum_dataframe` alongside the missing `DataFrameRow` marker impl flagged in section 7.

**Q2 — RESOLVED: `<field>_variant` (single underscore) with compile-time collision detection.** The outer-scope discriminant column is `<field>_variant`. If the inner enum's payload contains a field literally named `variant`, the macro must error at compile time pointing at the collision. The error should suggest renaming the inner field or using `#[dataframe(tag = "...")]` on Inner to pick a different inner tag. Naturally achieved when the inner enum uses `#[dataframe(tag = "variant")]` (the recommended convention).

**Q3 — RESOLVED: real R factor.** `as_factor` emits a column built as `structure(integer_vec, levels = c("Variant1", "Variant2", ...), class = "factor")`. Levels are enumerated at compile time from the enum's variant idents in declaration order. NA rows get `NA_integer_`. Unit-only enums only — payload-bearing enum used with `as_factor` is a compile error (UI test b in this plan). The auto-emitted `IntoR` from Q1 should produce factor SEXP directly (not a character vector).

**Q4 — RESOLVED: keep single `FieldTypeKind::Struct` for both structs and enums.** Treat them identically at the proc-macro level. Codegen dispatches via the runtime `IntoDataFrame::into_named_columns()` trait; `as_factor` is the only differentiator and routes through the existing `Single` path early. No new `FieldTypeKind::NestedEnum` arm.
