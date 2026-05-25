# 2026-05-25 — `dataframe_to_vec` nested struct un-flattening (#688)

## Goal

Close #688: let `dataframe_to_vec::<T>` reconstruct nested struct fields when
the underlying data.frame has flattened columns with `parent_child` naming
(matching the columnar serializer at
`miniextendr-api/src/serde/columnar.rs:975` —
`field.name = format!("{key}_{}", field.name)`).

## Problem

```rust
#[derive(Deserialize)] struct Address { city: String, zip: String }
#[derive(Deserialize)] struct Person  { name: String, address: Address }

// data.frame columns: name, address_city, address_zip
let people: Vec<Person> = dataframe_to_vec(sexp)?;   // fails today
```

Today the row deserialiser iterates flat column names and looks up
`address` as a literal column.

## Approach

Type-driven dispatch: when the visitor calls `next_value_seed`, hand it a
`MaybeNestedDeserializer` that *can* serve both shapes — `deserialize_str`/
`deserialize_i32`/etc. route to a `CellDeserializer` for the bare column,
while `deserialize_struct`/`deserialize_map` route to a nested
`RowMapAccess` that views the columns under the `<key>_` prefix as
top-level columns.

The visitor itself decides which path via the serde-derived
`deserialize_*` call — no schema introspection on our side.

### Sketch

```rust
struct RowMapAccess<'sexp> {
    view: &'sexp DataFrameView,
    row: usize,
    /// Top-level field-or-prefix-group names visible at this nesting level.
    /// At top level: column names. At nested level: stripped names with the
    /// next underscore-segment treated as the "field name".
    keys: Vec<String>,
    /// Prefix applied to keys to find real column names ("" at top level,
    /// "address_" one level in).
    prefix: String,
    idx: usize,
}

enum MaybeNested<'sexp, 'n> {
    /// Key matched a real column at the current prefix.
    Cell(CellDeserializer<'sexp, 'n>),
    /// Key is a group name — the visitor will recurse via deserialize_struct
    /// / deserialize_map.
    Group { view: &'sexp DataFrameView, row: usize, prefix: String },
}
```

Key derivation at a nesting level with `prefix`:
1. Iterate column names that start with `prefix`.
2. For each, strip the prefix → `rest`.
3. Split `rest` at the first `_`. The head is the visible key for this level.
4. Deduplicate, preserve column order.
5. When a column name equals the prefix concatenated with the head (no `_`),
   the key is a "bare" / "flat" field — the visitor can ask for a scalar.
6. When other columns share the same head as a prefix-group, the key is a
   group — the visitor can ask for a nested struct.

The `MaybeNested` deserialiser handed to `next_value_seed` carries enough
information to serve either path; the visitor's chosen `deserialize_*` call
picks the shape.

### Disambiguation tie-breaker

When both shapes are reachable for the same head — e.g., column `last`
exists AND columns `last_*` exist — the visitor will tell us which it
wants via its `deserialize_*` call. So we just need to make `MaybeNested`
work for both. No tie-breaker policy needed except for *enumeration order*
in `next_key_seed`:

- Each distinct head is emitted exactly once.
- Order matches first occurrence in `view.names()`.

For a field name with `_` in it (e.g., `last_name`) where no separate
`last` column exists but the visitor's struct *does* have a field named
`last_name` — that's the ambiguous case. With our greedy "split at first
`_`" rule, `last_name` would be reported as a key `last` with a
`MaybeNested` whose only "child" is `name`. The visitor would call
`deserialize_str` expecting field `last` (mismatch — the visitor
actually wants `last_name`). serde would then complain "missing field
`last_name`", not "unknown field `last`", because serde doesn't tell us
the target field names up-front; it just consumes whatever keys
`next_key_seed` yields.

**This is a real ambiguity.** Documented resolution: column names that
contain `_` are interpreted as nested-struct paths. To deserialise a
flat field with an underscore in its name, either:

1. Rename the column on the R side before calling `dataframe_to_vec`.
2. Use `#[serde(rename = "last_name")]` on a field named `last` whose
   sibling structure happens to align. (Not generally useful — this is
   a workaround.)
3. (Out of scope; future) Pass a target-type hint up-front so we can
   detect the longest matching prefix. Filed as follow-up issue.

Concretely the v1 policy is "greedy split at first `_`". Tests cover
both the success path (no underscore conflict) and the failure path
(visitor expecting `last_name` against a column literally named `last_name`).

### `#[serde(flatten)]`

Out of scope for v1. Flatten goes through `deserialize_map` and asks for
arbitrary keys. Our `RowMapAccess` would have to yield *all* descendant
flat keys for the flatten path. Filed as follow-up issue.

### Edge cases

- **Empty data.frame**: `is_empty_dataframe` already returns `vec![]`.
- **Missing nested column** (e.g., struct expects `address_zip` but
  data.frame only has `address_city`): the inner `CellDeserializer`
  errors via `MissingField`. Error message names the nested column
  (`address_zip`).
- **Tuple structs / tuple variants**: out of scope (data.frames don't
  carry positional payload).
- **Cycles**: serde rejects at compile time (`Box<Self>`).

## Implementation steps

1. **`dataframe_de.rs`** — rework `RowMapAccess`:
   - Add `prefix: String` field.
   - `next_key_seed` derives heads per the algorithm above; iterates with
     `idx`.
   - `next_value_seed` returns a `MaybeNested` deserialiser.
2. **`MaybeNested` deserialiser**:
   - `deserialize_struct`/`deserialize_map` → `visitor.visit_map(RowMapAccess::with_prefix(view, row, prefix + head + "_"))`.
   - All cell deserialise paths (`bool`/`i*`/`u*`/`f*`/`char`/`str`/`string`/`option`/`identifier`/`any`) delegate to a `CellDeserializer` for the bare column at `prefix + head`. If no bare column → return a `MissingField` error.
   - `deserialize_option` peeks: if the bare column exists, behave like
     `CellDeserializer::deserialize_option`. If not, eagerly assume `Some`
     and recurse to map (an `Option<NestedStruct>` with all-NA inner cells
     becomes `Some(NestedStruct { … None })`, since we can't distinguish
     row-wise). This is the same behaviour `vec_to_dataframe`'s round-trip
     produces.
3. **Tests** — unit tests in `dataframe_de` and rpkg/testthat:
   - One-level nesting (Person/Address).
   - Two-level nesting (Order/Customer/Address).
   - Underscore-in-flat-name happy case (no nested siblings).
   - Disambiguation failure path (documented limitation): field named
     `last_name` when columns are `last_name` only — works because we
     greedy-split and visitor asks for `last`, which goes through a
     `MaybeNested` whose only path forward is `name` (nested). If the
     visitor truly wants flat `last_name`, that's the documented failure.
     Add a test asserting the failure case, plus a test asserting the
     happy nested case with `last_name` columns.
   - Missing nested column: error message contains `address_zip`.
4. **gctorture fixture** — `gc_stress_dataframe_to_vec_nested()` in
   `rpkg/src/rust/gc_stress_fixtures.rs`. Round-trips a `Vec<Person>` with
   `Address` nested.
5. **Docs** — update the "Flat structs only" limitations note in
   `dataframe_de.rs`. Bump to "Nested structs are supported via the
   `parent_child` naming convention from the columnar serializer".

## Verification

- `cargo test -p miniextendr-api --features serde` clean.
- `just configure && just rcmdinstall && just force-document`
  (sandbox disabled).
- `just devtools-test` clean.
- `just clippy` + CI's `clippy_all` reproduced.
- gctorture sweep over the new fixture.
- Revert `rpkg/src/rust/Cargo.lock` to origin/main before commit.

## PR

Title: `feat(serde): dataframe_to_vec nested struct un-flattening (#688)`.
Body via ai-attribution skill. Closes #688.
