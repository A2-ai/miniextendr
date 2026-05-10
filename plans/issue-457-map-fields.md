+++
title = "issue-457: HashMap/BTreeMap as enum DataFrameRow variant fields"
description = "Expand HashMap<K,V> / BTreeMap<K,V> variant fields into parallel _keys / _values list-columns in the DataFrameRow enum derive."
+++

# issue-457: `HashMap`/`BTreeMap` as enum `DataFrameRow` variant fields

Tracks: <https://github.com/A2-ai/miniextendr/issues/457>

Locked design (agreed 2026-05-10): a `HashMap<K,V>` or `BTreeMap<K,V>` field on an enum variant expands
to **two parallel list-columns** named `<field>_keys` and `<field>_values`. Absent-variant rows get
`NULL` in both. No `ToString` constraint on `K`; `BTreeMap` preserves insertion/sorted order,
`HashMap` is non-deterministic — both points must be documented and R tests must account for
`HashMap` ordering.

---

## 1. Add `FieldTypeKind::Map` to `classify_field_type`

**File:** `miniextendr-macros/src/dataframe_derive.rs`, function `classify_field_type` (line ~238)

Currently `HashMap<K,V>` and `BTreeMap<K,V>` fall through to `FieldTypeKind::Scalar` because the
function only special-cases `Vec`, `Box<[T]>`, `&[T]`, and `[T;N]`. Add a new variant:

```rust
pub(super) enum FieldTypeKind<'a> {
    Scalar,
    FixedArray(&'a syn::Type, usize),
    VariableVec(&'a syn::Type),
    BoxedSlice(&'a syn::Type),
    BorrowedSlice(&'a syn::Type),
    /// `HashMap<K, V>` or `BTreeMap<K, V>`. Carries key type, value type, and
    /// which map kind (so codegen can preserve BTreeMap ordering).
    Map { key_ty: &'a syn::Type, val_ty: &'a syn::Type, is_ordered: bool },
}
```

In `classify_field_type`, after the `Vec`/`Box` checks (line ~274), add detection for the two-arg
path case:

```rust
if seg.ident == "HashMap" || seg.ident == "BTreeMap" {
    if let (Some(GenericArgument::Type(key_ty)), Some(GenericArgument::Type(val_ty))) =
        (args.args.first(), args.args.iter().nth(1))
    {
        return FieldTypeKind::Map {
            key_ty,
            val_ty,
            is_ordered: seg.ident == "BTreeMap",
        };
    }
}
```

This function is used by **both** the struct path (`resolve_struct_field`) and the enum path
(`enum_expansion.rs` match on `classify_field_type`). The struct path will encounter `Map` and
fall through to `Single` (correct — a struct map column is already opaque and works as `Vec<MapType>`
via the existing `IntoR` impls). Confirm this by reading `resolve_struct_field` — when `kind` is
`Map`, it should fall through to the same `Scalar` → `Single` path. If `FieldTypeKind::Scalar` is
matched by a wildcard `_` arm, no change is needed in the struct path; if it's explicit, add `|
FieldTypeKind::Map { .. }` to that arm.

**What to confirm before writing code:** check that `resolve_struct_field` (lines ~381–490) has
a wildcard or explicit `Scalar` arm, and confirm there's no `unreachable!()` that would now be
hit.

---

## 2. Add `EnumResolvedField::Map` variant

**File:** `miniextendr-macros/src/dataframe_derive.rs`, around line 1561

Add a new enum variant and its data struct alongside the existing ones:

```rust
pub(super) enum EnumResolvedField {
    Single(Box<EnumSingleFieldData>),
    ExpandedFixed(Box<EnumExpandedFixedData>),
    ExpandedVec(Box<EnumExpandedVecData>),
    AutoExpandVec(Box<EnumAutoExpandVecData>),
    /// `HashMap<K,V>` or `BTreeMap<K,V>` → two parallel list-columns: `<field>_keys`, `<field>_values`.
    Map(Box<EnumMapFieldData>),
}

pub(super) struct EnumMapFieldData {
    /// Base column name (field name or `rename` override).
    pub(super) base_name: String,
    /// Binding name used in destructure pattern.
    pub(super) binding: syn::Ident,
    /// Original Rust field name.
    pub(super) rust_name: syn::Ident,
    /// Key type K.
    pub(super) key_ty: syn::Type,
    /// Value type V.
    pub(super) val_ty: syn::Type,
    /// Whether this is a BTreeMap (preserves key order) vs HashMap (non-deterministic).
    pub(super) is_ordered: bool,
}
```

Update `EnumResolvedField::binding()` and `EnumResolvedField::rust_name()` (lines ~1574–1591) to
handle the new arm:

```rust
Self::Map(data) => &data.binding,
Self::Map(data) => &data.rust_name,
```

---

## 3. Wire `Map` into the enum field resolution loop

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`, inside `derive_enum_dataframe`,
the `Fields::Named` and `Fields::Unnamed` match arms (lines ~67–260).

Currently, the `classify_field_type` result is matched in the `else` branch (after `fa.as_list`
check). Add a `FieldTypeKind::Map { key_ty, val_ty, is_ordered }` arm before the `Scalar` fallthrough:

```rust
FieldTypeKind::Map { key_ty, val_ty, is_ordered } => {
    // `width` / `expand` are not valid on map fields — error early.
    if fa.width.is_some() {
        return Err(syn::Error::new_spanned(
            &f.ty,
            "`width` is not valid on HashMap/BTreeMap fields",
        ));
    }
    if fa.expand {
        return Err(syn::Error::new_spanned(
            &f.ty,
            "`expand`/`unnest` is not valid on HashMap/BTreeMap fields",
        ));
    }
    resolved.push(EnumResolvedField::Map(Box::new(EnumMapFieldData {
        base_name: col_name_str,
        binding: binding.clone(),
        rust_name: rust_name.clone(),
        key_ty: key_ty.clone(),
        val_ty: val_ty.clone(),
        is_ordered,
    })));
}
```

The same block must be added in the `Fields::Unnamed` arm (line ~179 onwards) with `binding`
renamed accordingly. Both named and unnamed variants should produce identical `EnumResolvedField::Map`
payloads — only the destructure pattern differs.

---

## 4. Register map fields in `ColumnRegistry`

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`, lines ~272–304 (the
"Resolve unified schema" region).

Map fields contribute **two** entries to the `ColumnRegistry` per field: one for `<base>_keys` and
one for `<base>_values`. Add a `Map` arm after `AutoExpandVec`:

```rust
EnumResolvedField::Map(data) => {
    let keys_name = format!("{}_keys", data.base_name);
    let vals_name = format!("{}_values", data.base_name);
    // Key column type: Vec<K>
    let key_vec_ty: syn::Type = syn::parse_quote!(Vec<#key_ty>);
    // Value column type: Vec<V>
    let val_vec_ty: syn::Type = syn::parse_quote!(Vec<#val_ty>);
    registry.register(&keys_name, &key_vec_ty, variant_idx, &vi.name, err_span)?;
    registry.register(&vals_name, &val_vec_ty, variant_idx, &vi.name, err_span)?;
}
```

where `key_ty = &data.key_ty`, `val_ty = &data.val_ty`.

**Type-conflict semantics**: if two variants both carry a map field with the same base name but
different `K` or `V`, the existing `ColumnRegistry::register` type-check fires (same as for any
other field type). The user resolves it with `#[dataframe(conflicts = "string")]` which coerces
both key and value columns to `String`. The coerce logic in `registry.register` works on whatever
`syn::Type` is passed in, so passing `Vec<K>` is correct.

---

## 5. Generate companion-struct fields and `IntoDataFrame` / `From<Vec<Enum>>` for map columns

All of this is inside `derive_enum_dataframe` in `enum_expansion.rs`.

### 5a. Companion-struct fields (lines ~379–400)

The `df_fields` builder currently iterates `columns` (which now includes `<base>_keys` and
`<base>_values` entries with types `Vec<K>` and `Vec<V>` respectively). No special handling is
needed here — the existing `col.col_name` / `col.ty` machinery produces:

```rust
pub tally_keys: Vec<Option<Vec<K>>>
pub tally_values: Vec<Option<Vec<V>>>
```

This is the correct shape: each cell is `None` for absent variants, or `Some(vec_of_keys)` /
`Some(vec_of_values)` for matching variants.

### 5b. Column-length checks and `IntoDataFrame` pairs (lines ~429–560)

No special handling — both `_keys` and `_values` are treated as ordinary `Vec<Option<Vec<T>>>`
columns. The `IntoR` impls for `Vec<Option<Vec<T: RNativeType>>>` and `Vec<Option<Vec<String>>>`
(already at `into_r.rs:2244` and `into_r.rs:2261`) cover the common cases.

For less common key/value types (e.g., `Vec<Option<Vec<i64>>>`, `Vec<Option<Vec<bool>>>`), check
if the existing blanket impl `impl<T: RNativeType> IntoR for Vec<Option<Vec<T>>>` covers them. It
should, because `i64`, `bool`, etc. implement `RNativeType` — but confirm at
`miniextendr-api/src/into_r.rs:2244`. The companion struct emits the type directly; Rust will
refuse to compile if `IntoR` is absent, which is the correct failure mode.

### 5c. `From<Vec<Enum>>` match arms — the push logic (lines ~589–722)

In the `col_pushes` generation, for columns named `<base>_keys` and `<base>_values` that have
`present_in.contains(&variant_idx)`, the current `Single` detection branch will fail to find a
matching `EnumResolvedField::Single` with that col_name. This is the only subtle touch-point:
add a `EnumResolvedField::Map` branch in the col-push inner loop:

```rust
EnumResolvedField::Map(data) => {
    let keys_name = format!("{}_keys", data.base_name);
    let vals_name = format!("{}_values", data.base_name);
    let binding = &data.binding;
    if col_name_str == keys_name {
        // Extract keys from the map; collect into Vec<K>
        if data.is_ordered {
            return quote! { #col_name.push(Some(#binding.keys().cloned().collect::<Vec<_>>())); };
        } else {
            return quote! { #col_name.push(Some(#binding.keys().cloned().collect::<Vec<_>>())); };
        }
    }
    if col_name_str == vals_name {
        if data.is_ordered {
            return quote! { #col_name.push(Some(#binding.values().cloned().collect::<Vec<_>>())); };
        } else {
            return quote! { #col_name.push(Some(#binding.values().cloned().collect::<Vec<_>>())); };
        }
    }
}
```

Note: `HashMap` and `BTreeMap` both iterate in their natural order for `keys()` / `values()` —
BTreeMap in sorted key order, HashMap in arbitrary order. **Parallel iteration** (`keys()` then
`values()`) is guaranteed to correspond pairwise only within a single call site, because Rust's
iterator stability guarantee holds for the duration of a single borrow. In the generated code,
the binding `__v_tally` holds the moved map, so `keys()` then `values()` execute on the same
instance in the same sequence — this is safe for both. An alternative and more explicit approach
is to call `.into_iter().unzip()`:

```rust
let (keys, vals): (Vec<_>, Vec<_>) = #binding.into_iter().unzip();
#col_name_keys.push(Some(keys));
#col_name_vals.push(Some(vals));
```

**The `unzip()` approach is strongly preferred** because it is provably correct regardless of
iterator ordering subtleties and does not require calling `keys()` and `values()` separately. Use
it in the generated code for both `HashMap` and `BTreeMap`.

The absent-variant push (`push(None)`) is already generated by the fallthrough code when
`col.present_in` does not contain `variant_idx` — no change needed there.

### 5d. Parallel path (`from_rows_par`) match arms (lines ~816–943)

Same as 5c but for the `ColumnWriter::write` calls. Add the `Map` arm to the col-writes loop,
again using `unzip()`:

```rust
EnumResolvedField::Map(data) => {
    let keys_name = format!("{}_keys", data.base_name);
    let vals_name = format!("{}_values", data.base_name);
    let binding = &data.binding;
    if col_name_str == keys_name {
        return quote! {
            let (__k, _) = #binding.clone().into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
            #w_name.write(__i, Some(__k));
        };
    }
    if col_name_str == vals_name {
        return quote! {
            let (_, __v) = #binding.clone().into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
            #w_name.write(__i, Some(__v));
        };
    }
}
```

**Important**: in the rayon path the binding is moved in the `match` arm; cloning the map for
both `_keys` and `_values` writes means two clones. An alternative is to unzip once into a
`(Vec<K>, Vec<V>)` local before the first `write` call, but this requires a more complex codegen
(intermediate temp variable). The simpler approach is to emit a single unzip per variant arm that
populates both writer slots:

```rust
EnumResolvedField::Map(data) if col_name is the keys col => {
    // emit unzip + write_keys + write_vals all in one block, skip the vals col separately
}
```

The cleanest implementation: iterate `col_writes` in the rayon path but track which map fields
have already been emitted (using a `HashSet<String>` of base_names), and emit a combined block for
both columns when the first one is encountered, and skip the second:

```
if first_col_of_map (keys_col):
    emit: let (__keys, __vals) = binding.into_iter().unzip(); w_keys.write(__i, Some(__keys)); w_vals.write(__i, Some(__vals));
if second_col_of_map (vals_col):
    skip (already handled by keys block)
```

This is slightly complex to implement but avoids the double-clone. Plan this as the preferred
approach and note it in the PR description.

---

## 6. Generate `to_dataframe_split` map handling

**File:** `miniextendr-macros/src/dataframe_derive/enum_expansion.rs`,
`generate_split_method` (lines ~1050–1499).

For split, each variant's data.frame only contains its own fields (non-optional). For a
`Map { base_name, key_ty, val_ty }` field, two buffers are declared:

```rust
let mut __s_{snake}_{base}_keys: Vec<Vec<K>> = Vec::new();
let mut __s_{snake}_{base}_values: Vec<Vec<V>> = Vec::new();
```

The push statement uses `unzip()`:

```rust
let (__keys, __vals) = #binding.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
__s_{snake}_{base}_keys.push(__keys);
__s_{snake}_{base}_values.push(__vals);
```

The df construction pairs (`static path`):

```rust
("{base}_keys",  __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__s_{snake}_{base}_keys))),
("{base}_values", __scope.protect_raw(::miniextendr_api::IntoR::into_sexp(__s_{snake}_{base}_values))),
```

These `Vec<Vec<K>>` / `Vec<Vec<V>>` types need `IntoR` impls — see item 7.

For the len expression (length-ref for the data.frame), `__s_{snake}_{base}_keys.len()` is
sufficient (both buffers grow in lock-step).

In the `match` for `has_auto`, map fields are not auto-expand and do not need the dynamic path;
they follow the static pair construction. If a variant has both auto-expand and map fields,
the `has_auto` check already picks the dynamic path, and static_pair_pushes include the map
column pairs — no special handling needed.

---

## 7. IntoR for `Vec<Vec<K>>` / `Vec<Vec<V>>`

**File:** `miniextendr-api/src/into_r.rs`

For the `to_dataframe_split` path, split buffers are `Vec<Vec<K>>` (no outer Option, because
split only collects rows for the matching variant). The existing blanket impl
`impl<T: RNativeType> IntoR for Vec<Vec<T>>` at line ~1390 (check exact line) covers native
scalar K types. `Vec<Vec<String>>` is at line ~1439. No new impls should be needed for the common
cases.

**What to verify:** whether `Vec<Vec<bool>>` has an explicit `IntoR` or relies on the blanket.
Run `just check` after macro changes; any missing `IntoR` is a compile error pointing exactly to
the gap.

For the align path (`to_dataframe`), columns are `Vec<Option<Vec<K>>>` / `Vec<Option<Vec<V>>>`.
The existing impls at `into_r.rs:2244` (`impl<T: RNativeType> IntoR for Vec<Option<Vec<T>>>`)
and `into_r.rs:2261` (`impl IntoR for Vec<Option<Vec<String>>>`) cover scalar and String key/value
types. No new `into_r.rs` changes are expected — but verify during implementation.

---

## 8. Rust test fixtures (all 4 cardinality cells × both paths × HashMap + BTreeMap)

**File:** `rpkg/src/rust/dataframe_enum_payload_matrix.rs`

Add a new region `// region: Map fields (HashMap<K,V> / BTreeMap<K,V>) ─────────────────`.

### 8a. Enum definitions

```rust
#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum HashMapEvent {
    Tally { label: String, tally: HashMap<String, i32> },
    Empty { label: String },
}

#[derive(Clone, Debug, DataFrameRow)]
#[dataframe(align, tag = "_type")]
pub enum BTreeMapEvent {
    Tally { label: String, tally: BTreeMap<String, i32> },
    Empty { label: String },
}
```

### 8b. Fixtures (one set for each enum, covering all 4 cells)

For **1v1r**: one variant, one row — split returns the matching variant data.frame with 1 row,
the other variant data.frame with 0 rows.

For **1vNr**: one variant, multiple rows.

For **Nv1r**: both variants, one row each.

For **NvNr**: both variants, multiple rows each.

```rust
// HashMap
#[miniextendr] pub fn hashmap_split_1v1r() -> List { ... }
#[miniextendr] pub fn hashmap_split_1vnr() -> List { ... }
#[miniextendr] pub fn hashmap_split_nv1r() -> List { ... }
#[miniextendr] pub fn hashmap_align_nvnr() -> ToDataFrame<HashMapEventDataFrame> { ... }
#[miniextendr] pub fn hashmap_split_nvnr() -> List { ... }

// BTreeMap
#[miniextendr] pub fn btreemap_split_1v1r() -> List { ... }
#[miniextendr] pub fn btreemap_split_1vnr() -> List { ... }
#[miniextendr] pub fn btreemap_split_nv1r() -> List { ... }
#[miniextendr] pub fn btreemap_align_nvnr() -> ToDataFrame<BTreeMapEventDataFrame> { ... }
#[miniextendr] pub fn btreemap_split_nvnr() -> List { ... }
```

Concrete fixture data:
- `tally = HashMap::from([("a".to_string(), 1i32), ("b".to_string(), 2i32)])` (2-entry map)
- `tally = HashMap::from([("x".to_string(), 5i32)])` (1-entry map)
- `tally = HashMap::new()` (empty map — both key/value cells should be `integer(0)` / `character(0)`)

### 8c. `#[cfg(test)]` Rust unit tests

Add a `tests` block in the same file:

```rust
#[test]
fn test_hashmap_event_align() {
    // Verify both tally_keys and tally_values have same length in each row
    let df = HashMapEvent::to_dataframe(vec![
        HashMapEvent::Tally { label: "a".into(), tally: HashMap::from([("x".to_string(), 1i32)]) },
        HashMapEvent::Empty { label: "b".into() },
    ]);
    assert_eq!(df.tally_keys.len(), 2);
    assert_eq!(df.tally_values.len(), 2);
    assert!(df.tally_keys[0].is_some());
    assert!(df.tally_values[0].is_some());
    assert!(df.tally_keys[1].is_none());
    assert!(df.tally_values[1].is_none());
    // lengths within a row must match
    let k = df.tally_keys[0].as_ref().unwrap();
    let v = df.tally_values[0].as_ref().unwrap();
    assert_eq!(k.len(), v.len());
}

#[test]
fn test_btreemap_keys_sorted() {
    let df = BTreeMapEvent::to_dataframe(vec![
        BTreeMapEvent::Tally { label: "a".into(),
            tally: BTreeMap::from([("z".to_string(), 3i32), ("a".to_string(), 1i32)]) },
    ]);
    // BTreeMap preserves sorted order: keys should be ["a", "z"]
    assert_eq!(df.tally_keys[0].as_deref(), Some(vec!["a".to_string(), "z".to_string()].as_slice()));
    assert_eq!(df.tally_values[0].as_deref(), Some(vec![1i32, 3i32].as_slice()));
}

#[test]
fn test_hashmap_empty_map() {
    let df = HashMapEvent::to_dataframe(vec![
        HashMapEvent::Tally { label: "a".into(), tally: HashMap::new() },
    ]);
    assert_eq!(df.tally_keys[0], Some(vec![]));
    assert_eq!(df.tally_values[0], Some(vec![]));
}
```

---

## 9. R test file

**File:** `rpkg/tests/testthat/test-dataframe-enum-payload-matrix.R`

Add a new section `# ── Map fields (HashMap / BTreeMap) ──────────────────────────────────────`.

### 9a. HashMap tests

HashMap key order is non-deterministic — **always** use `setequal` or sort before comparing keys.
Never use `expect_equal(res$tally$tally_keys[[1]], c("a", "b"))` — use
`expect_setequal(res$tally$tally_keys[[1]], c("a", "b"))` and separately verify that
`tally_values[[1]]` contains the correct value for each key by looking up position
(`match(key, tally_keys[[1]])`).

```r
test_that("hashmap — split 1v1r: tally has 1 row with _keys/_values, empty has 0 rows", {
  res <- hashmap_split_1v1r()
  expect_setequal(names(res), c("tally", "empty"))
  expect_s3_class(res$tally, "data.frame")
  expect_equal(nrow(res$tally), 1L)
  expect_true(all(c("label", "tally_keys", "tally_values") %in% names(res$tally)))
  expect_equal(nrow(res$empty), 0L)
  # Parallel structure: same length
  expect_equal(length(res$tally$tally_keys[[1]]), length(res$tally$tally_values[[1]]))
  # HashMap: use setequal for keys, match-based check for values
  expect_setequal(res$tally$tally_keys[[1]], c("a", "b"))
  idx_a <- match("a", res$tally$tally_keys[[1]])
  expect_equal(res$tally$tally_values[[1]][idx_a], 1L)
})

test_that("hashmap — align NvNr: tally rows have _keys/_values list-columns, empty rows have NULL", {
  df <- hashmap_align_nvnr()
  expect_type(df$tally_keys, "list")
  expect_type(df$tally_values, "list")
  # Tally rows: non-NULL, same lengths
  tally_rows <- which(df$`_type` == "Tally")
  for (i in tally_rows) {
    expect_false(is.null(df$tally_keys[[i]]))
    expect_equal(length(df$tally_keys[[i]]), length(df$tally_values[[i]]))
  }
  # Empty rows: NULL
  empty_rows <- which(df$`_type` == "Empty")
  for (i in empty_rows) {
    expect_null(df$tally_keys[[i]])
    expect_null(df$tally_values[[i]])
  }
})
```

Add analogous tests for 1vNr, Nv1r, and split NvNr cells.

### 9b. BTreeMap tests

BTreeMap has deterministic sorted-key order — `expect_equal` is safe:

```r
test_that("btreemap — split 1v1r: keys are sorted", {
  res <- btreemap_split_1v1r()
  expect_equal(res$tally$tally_keys[[1]], sort(res$tally$tally_keys[[1]]))
  # Exact value check (BTreeMap, order is deterministic)
  expect_equal(res$tally$tally_keys[[1]], c("a", "b"))
  expect_equal(res$tally$tally_values[[1]], c(1L, 2L))
})

test_that("btreemap — align NvNr: sorted keys, NULL for empty variant", {
  df <- btreemap_align_nvnr()
  tally_rows <- which(df$`_type` == "Tally")
  for (i in tally_rows) {
    keys <- df$tally_keys[[i]]
    expect_equal(keys, sort(keys))
  }
})
```

### 9c. Empty-map edge case

```r
test_that("hashmap — empty map produces zero-length vectors, not NULL", {
  # An empty HashMap in a Tally row should yield character(0) / integer(0), not NULL
  # (tested via a dedicated fixture that passes an empty map)
  # ... depends on a fixture function exposing the empty-map case
})
```

---

## 10. Validate `width` / `expand` / `as_list` attribute rejection

`width`, `expand`/`unnest`, and `as_list` should all be rejected on map fields with a clear error
message. This is tested at the macro level (UI tests or `compile_fail` doctests). Check if there
are existing UI tests in `miniextendr-macros/tests/` or `miniextendr-lint/tests/`; add
compile-fail test cases for:

- `#[dataframe(width = 2)] tally: HashMap<String, i32>` → error "width is not valid on HashMap/BTreeMap"
- `#[dataframe(expand)] tally: HashMap<String, i32>` → error "expand is not valid on HashMap/BTreeMap"

`as_list` on a map field is deliberately left working (it opts out of the two-column expansion
and keeps the map as a single opaque list-column, which is the existing behavior for `Scalar`
fields). Confirm: since `as_list` is checked before `classify_field_type` in the enum resolution
loop, the `Map` branch will not be reached when `fa.as_list = true`. Document this in the
attribute rejection test as a comment: "`as_list` suppresses expansion; map is kept opaque."

---

## 11. Update `docs/DATAFRAME.md`

**File:** `docs/DATAFRAME.md`

Replace the reference to issue #457 in the "Collection types as fields" section (line ~203):

> `HashMap<K, V>` / `BTreeMap<K, V>` as variant fields, nested enums, and struct-typed fields are
> tracked by issues #457 / #458 / #459.

Change to:

> `HashMap<K, V>` / `BTreeMap<K, V>` variant fields are supported and expand to two parallel
> list-columns. Nested enums and struct-typed fields are tracked by issues #458 / #459.

Add a new subsection after the existing "Collection types" paragraph:

```markdown
### Map fields — parallel list-column expansion

`HashMap<K, V>` and `BTreeMap<K, V>` fields on enum variants expand to two parallel list-columns
named `<field>_keys` and `<field>_values`. Each cell holds a vector of K and a vector of V
respectively, in the same entry order:

```rust
#[derive(Clone, DataFrameRow)]
#[dataframe(align, tag = "_type")]
enum Event {
    Tally { label: String, tally: BTreeMap<String, i32> },
    Empty { label: String },
}
// In R (BTreeMap, sorted key order):
//   _type   label  tally_keys   tally_values
//   Tally   "a"    list("a","b")  list(1L, 2L)
//   Empty   "b"    NULL           NULL
```

Absent-variant rows produce `NULL` in both columns (not NA). An empty map produces
`character(0)` / `integer(0)`, not `NULL`.

**HashMap ordering**: `HashMap` iteration order is non-deterministic. Keys and values are
parallel within a single row, but the key order may differ across rows and across runs. Use
`setequal` or sort-based comparison in R tests, never `expect_equal` on unsorted key vectors.

**BTreeMap ordering**: keys are always in sorted order per the `BTreeMap` contract. `expect_equal`
is safe.

**`as_list` opt-out**: annotate the field with `#[dataframe(as_list)]` to keep it as a single
opaque named-list column (the pre-expansion behavior). Only use this when the named-list
per-row shape is needed directly in R.
```
```

---

## 12. Update `docs/CONVERSION_MATRIX.md`

**File:** `docs/CONVERSION_MATRIX.md`

In the `Vec<Option<C>>` table (around line 219), add two new rows:

| `Vec<Option<Vec<K>>>` (K: RNativeType, keys column) | VECSXP of typed vectors | NULL |
| `Vec<Option<Vec<V>>>` (V: IntoR, values column) | VECSXP | NULL |

Also add a prose note under the table:

> These types appear as the companion-struct column types for `HashMap<K,V>` / `BTreeMap<K,V>`
> enum variant fields (expanded to `<field>_keys` / `<field>_values` columns by `DataFrameRow`
> derive). No new `IntoR` impls are required beyond those already present.

---

## 13. Add CLAUDE.md gotcha for HashMap key-value parallelism

**File:** `CLAUDE.md`, Rust/FFI gotchas section

Add:

> **`HashMap`/`BTreeMap` enum DataFrameRow expansion**: `DataFrameRow` derive on map fields emits
> `unzip()` to produce parallel `_keys` / `_values` vectors in lock-step. Never call `.keys()`
> then `.values()` separately on the same map binding — use `.into_iter().unzip()` to guarantee
> the vectors are pairwise aligned (even though Rust's iterator model guarantees stability within
> a single borrow, `unzip` is explicit and avoids relying on that subtlety).

---

## 14. Sync generated files

After completing Rust + R changes:

```bash
just configure && just rcmdinstall && just devtools-document
```

Commit `rpkg/R/miniextendr-wrappers.R`, `NAMESPACE`, and any new/updated `man/*.Rd` together with
the Rust source changes in the same commit (or a dedicated "regen" commit immediately after).

The pre-commit hook checks `*-wrappers.R` staged ↔ `NAMESPACE` staged; enable once if not already:
`git config core.hooksPath .githooks`.

---

## 15. CI clippy reproduction

Before pushing, run both CI clippy jobs locally:

```bash
# clippy_default
cargo clippy --workspace --all-targets --locked -- -D warnings

# clippy_all (note: --all-features is rejected; use the explicit list)
cargo clippy --workspace --all-targets --locked \
  --features rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,\
num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,\
num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,\
vctrs,tinyvec,borsh,connections,nonapi,default-strict,default-coerce,default-r6,\
default-worker -- -D warnings
```

CI toolchain may flag lints not visible locally (`collapsible_match`, `manual_checked_ops`) — if
any appear on CI but not locally, address them in a fixup commit before the reviewer pass.

---

## 16. GC-discipline check

The new codegen emits `into_sexp()` calls on `Vec<Vec<K>>` and `Vec<Vec<V>>` via the companion
struct `IntoDataFrame` impl. These go through `vec_option_of_into_r_to_list` which already calls
`OwnedProtect::new(Rf_allocVector(...))` and iterates with `set_vector_elt`. Each inner
`Vec<K>.into_sexp()` allocates a fresh SEXP that must not be GC'd before the outer
`set_vector_elt` call.

Confirm the existing `vec_option_of_into_r_to_list` pattern (line ~2225 of `into_r.rs`) is
GC-safe: each inner `into_sexp()` result is passed directly to `set_vector_elt` without being
stored in a local — the SEXP is anchored by the outer VECSXP via R's GC write barrier immediately.
This pattern is already in use for all other `Vec<Option<C>>` columns; no new protection needed.

If unsure after implementation, run a `gctorture(TRUE)` sweep per `docs/GCTORTURE_TESTING.md`.
Ship a no-arg GC stress fixture if the new codegen adds any SEXP-storage path that is not already
covered by existing fixtures.

---

## Constraints summary for the implementer

1. **All 4 cardinality cells** from the start: 1v1r, 1vNr, Nv1r, NvNr — for both `HashMap` and
   `BTreeMap`, both `to_dataframe` (align) and `to_dataframe_split`. The #464 review caught a
   missing cell; do not repeat.

2. **HashMap tests use membership/sort checks** (`setequal`, `sort`, position-based lookup),
   never literal `expect_equal` on unsorted key vectors.

3. **`unzip()` in generated code** — do not use `keys()` + `values()` separately.

4. **Reproduced CI clippy** (`clippy_default` AND `clippy_all`) locally before push.

5. **Generated files committed in sync** (`rpkg/R/miniextendr-wrappers.R`, `NAMESPACE`,
   `man/*.Rd`).

6. **gh heredoc backtick rule**: when composing PR body or issue comments, use single-quoted
   heredoc `cat <<'EOF'` so raw backticks are not escaped. Before any `gh issue` / `gh pr`
   command, scan the body for `\`` and replace with plain `` ` ``.

7. **`as_list` on a map field** is valid and keeps the map opaque. Document this in the PR body
   and in the test's comment.
