# DataFrameBuilder pre-declared + growing schema modes

Closes #693 (pre-declared schema, `with_schema`) AND #692 (growing schema, `grow_schema`).
Bundled as one PR because both add a schema-mode knob to the same struct.

Refs #702 (consolidation parent).

## Worktree base

`origin/main` (verified: `git log origin/main..HEAD --oneline` returns empty
at plan-write time). Branch: `worktree-agent-a259680b2fb4cc919`.

## Coordination with in-flight PRs

PR #724 (`map_to_dataframe` / `result_to_dataframe` / `vec_to_dataframe_split`
shape modes / `DataFrameShape` enum) edits `miniextendr-api/src/serde/columnar.rs`.
This PR also edits that file but the surface is largely additive on the
`DataFrameBuilder` struct and a new `TypeSpec` enum. Rebase risk is moderate.
Don't base off pr724.

## Read first

- `miniextendr-api/src/serde/columnar.rs`
  - `DataFrameBuilder<T>` at `columnar.rs:549` (struct), `:565` (impl).
  - `iter_to_dataframe` at `:524`.
  - `Schema` at `:695`, `FieldInfo` at `:690`, `ColumnType` at `:681`,
    `FieldMap` at `:671`, `FieldMapping` at `:659`.
  - `SchemaAccumulator` (`:769`) — already supports both `SingleRow` and
    `Union` modes after the #706 refactor. Growing schema reuses Union mode.
  - `ColumnFiller` at `:1535`, `fill_field` at `:1552` (the `strict` branch
    returns the per-row "field not in schema" error today — we replace this
    with a grow path).
  - `ColumnBuffer` at `:1253`, `push_na` at `:1272`.
- `miniextendr-api/src/serde/error.rs` — `RSerdeError` variants.
- `miniextendr-api/src/serde.rs` — public re-exports (`DataFrameBuilder` is
  already re-exported; `TypeSpec` will need to join).
- `rpkg/src/rust/gc_stress_fixtures.rs:570` — existing
  `gc_stress_iter_to_dataframe` pattern.

## Design

### Part 1 — `TypeSpec` and `DataFrameBuilder::with_schema` (#693)

New public enum mirroring `ColumnType` plus an `Optional` hint:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpec {
    Logical,
    Integer,
    Real,
    Character,
    Generic,
    Optional(Box<TypeSpec>),
}
```

`Optional(T)` is a hint about NA-tolerance only — the column type is still
`T`. It exists so callers can record intent (and the discriminator helps when
a future widening-strict mode arrives). Internally we unwrap to the
underlying `ColumnType`.

```rust
impl TypeSpec {
    fn into_column_type(self) -> ColumnType {
        match self {
            TypeSpec::Logical => ColumnType::Logical,
            TypeSpec::Integer => ColumnType::Integer,
            TypeSpec::Real => ColumnType::Real,
            TypeSpec::Character => ColumnType::Character,
            TypeSpec::Generic => ColumnType::Generic,
            TypeSpec::Optional(inner) => inner.into_column_type(),
        }
    }
}
```

New constructor (consumes `Vec<(String, TypeSpec)>` for ergonomics, since
the schema must outlive the call):

```rust
impl<T: Serialize> DataFrameBuilder<T> {
    pub fn with_schema(
        schema: impl IntoIterator<Item = (impl Into<String>, TypeSpec)>,
        nrow_hint: Option<usize>,
    ) -> Self;
}
```

`with_schema` constructs `Schema { fields, field_map }` directly, allocates
the `ColumnBuffer`s up-front, and seeds `self.filled` so the first push goes
directly through the existing strict `ColumnFiller` — no schema discovery.

**Sub-fields (compound flatten)**: out of scope for the pre-declared variant.
The pre-declared schema is flat. Callers who need nested-struct flattening
either let discovery handle it (default builder) or pre-flatten the names
themselves (`"parent_child"` strings in the slice). The issue's sketch
doesn't ask for compound support and adding it pulls in a `FieldMapping`
serializer the issue doesn't motivate. Document the limitation in the
rustdoc and surface via a follow-up issue if needed.

### Part 2 — `DataFrameBuilder::grow_schema` (#692)

Decision: **builder method** (chainable, opt-in) rather than a separate
constructor. Composes with `with_schema` so callers can start with a
declared partial schema and let the rest grow:

```rust
impl<T: Serialize> DataFrameBuilder<T> {
    pub fn grow_schema(mut self) -> Self {
        self.grow = true;
        self
    }
}
```

`self.grow: bool` is a new field on the struct.

Per push when `grow == true`:

1. Walk the row with a fresh `SchemaAccumulator::new(SchemaMode::Union)`
   and `feed(&row)`.
2. `finalize()` → discovered `Schema`. (Already returns flat schema today,
   handling compound flattening.)
3. For each field in the discovered schema *not present* in
   `self.schema.field_map`:
   - Allocate a fresh `ColumnBuffer::new(col_type, self.nrow_hint.unwrap_or(0))`.
   - Back-fill `self.nrow` NA values via `push_na()`.
   - Push the buffer into `self.columns`.
   - Insert into `self.schema.field_map` and append to `self.schema.fields`.
   - Push a `filled[i] = false` slot.
4. Continue with the existing strict `ColumnFiller` path; new fields are
   now in the schema so `strict=true` won't reject them.

**Compound handling**: if the discovered Compound has the same `key` but
incompatible inner shape, follow today's "first seen wins" rule. New
nested keys land as new flat columns.

**Type widening**: a later row carries a `String` value for a column the
first row mapped as `Integer` (because the first row's value was a plain
int). Today's `ColumnFiller::push_value` silently coerces (matches the
existing behaviour for `Real <- Int`, falls back to NA for true mismatches).
We do not change this — same ambiguity as today's union path. Document it
in the doc comment.

We deliberately **do not** add a new `RSerdeError::SchemaTypeMismatch`
variant: the existing `push_value` flow handles cross-type writes by
NA-coercing or panicking via the extractor on truly compound payloads (the
extractor returns `RSerdeError::Message("compound type in value extractor")`
already). Document this behaviour rather than fork the error enum.

### Behaviour matrix

|                            | default                | `with_schema`         | `grow_schema()`       | `with_schema().grow_schema()` |
|----------------------------|------------------------|-----------------------|-----------------------|-------------------------------|
| first push                 | discovers schema       | uses declared         | discovers schema      | uses declared                 |
| later push: new field      | error                  | error                 | adds + back-fills     | adds + back-fills             |
| later push: missing field  | NA                     | NA                    | NA                    | NA                            |
| later push: type clash     | coerce-or-NA           | coerce-or-NA          | coerce-or-NA          | coerce-or-NA                  |

(`error` cells fire today via `ColumnFiller::fill_field` strict branch.)

### Struct changes

```rust
pub struct DataFrameBuilder<T: Serialize> {
    schema: Option<Schema>,
    columns: Vec<ColumnBuffer>,
    filled: Vec<bool>,
    nrow: usize,
    nrow_hint: Option<usize>,
    scope: crate::ProtectScope,
    grow: bool,                   // NEW — opt-in growing mode
    _marker: core::marker::PhantomData<fn(T)>,
}
```

`with_schema` sets `schema = Some(...)`. `grow_schema()` flips `grow = true`.

### Public surface (re-exports)

`miniextendr-api/src/serde.rs`:
```rust
pub use columnar::{
    ColumnarDataFrame, DataFrameBuilder, NamedDataFrameListBuilder, TypeSpec,
    iter_to_dataframe, vec_to_dataframe, vec_to_dataframe_split,
};
```

## Implementation order

1. `miniextendr-api/src/serde/columnar.rs`:
   - Add `TypeSpec` enum + `into_column_type` helper.
   - Add `grow: bool` field to `DataFrameBuilder`.
   - Initialize `grow = false` in `new`.
   - Add `with_schema` constructor.
   - Add `grow_schema` builder method.
   - In `push`: gate first-push discovery on `schema.is_none()` (unchanged),
     then if `self.grow`, run accumulator + diff + back-fill before the
     strict filler invocation.
   - Adjust `pad_unfilled` / `filled` resizing path for new columns (the
     existing `pad_unfilled` reads `field_map.col_start..col_start +
     total_cols`; with growth we must update `total_cols` in `field_map`).
2. `miniextendr-api/src/serde.rs`: re-export `TypeSpec`.
3. `miniextendr-api/src/serde/columnar.rs` `#[cfg(test)] mod tests`:
   - `with_schema_skips_discovery_on_first_push` — single test verifying
     `Optional(Integer)` first row `None` lands as `NA_INTEGER` not logical.
   - `with_schema_rejects_unknown_field` — strict still fires.
   - `grow_schema_back_fills_na_on_new_field`.
   - `grow_schema_combined_with_with_schema`.
4. `rpkg/src/rust/gc_stress_fixtures.rs`:
   - `gc_stress_builder_with_schema()` — pre-declared schema, push 50 rows
     including some `None`-bearing optional column, exercise the
     Generic-column protect path.
   - `gc_stress_builder_grow_schema()` — drive growth across rows so the
     allocate-and-back-fill path runs under gctorture.
5. `rpkg/tests/testthat/test-iter-to-dataframe.R`:
   - Add `test_that` blocks calling the two new `gc_stress_*` fixtures
     (mirror the existing pattern in the file).
6. `just configure && just rcmdinstall && just force-document` to regenerate
   `R/miniextendr-wrappers.R`, `NAMESPACE`, and `man/*.Rd` for the new
   fixtures.
7. Doc examples (rustdoc) for `TypeSpec`, `with_schema`, `grow_schema` —
   `rust,ignore` runnable to avoid pulling SEXP construction into doctests.

## Verification

- `cargo test -p miniextendr-api --features serde` (sandbox off).
- `just configure && just rcmdinstall && just force-document` (sandbox off).
- `just devtools-test 2>&1 > /tmp/devtools-test.log` and Read the log.
- `just clippy` plus CI's `clippy_all` matrix (per
  `feedback_check_all_features`).
- gctorture sweep over the new no-arg fixtures (per CLAUDE.md convention).
- Revert `rpkg/src/rust/Cargo.lock` to origin/main before commit (per
  CLAUDE.md gotcha confirmed in #710/#714/#724).

## PR

- Title: `feat(serde): DataFrameBuilder pre-declared + growing schema modes (#693 #692)`
- Body uses ai-attribution skill (collapsible `<details>` for AI content).
- Closes #693, #692. Refs #702.

## Open questions for orchestrator

- `TypeSpec` API: I'm taking `IntoIterator<Item = (impl Into<String>,
  TypeSpec)>` rather than the issue's `&[(&str, TypeSpec)]` because the
  builder owns the `Schema` for its full lifetime — slices add a lifetime
  bound that propagates to the struct. Matches `NamedDataFrameListBuilder`
  ergonomics.
- Skipping compound (`FieldMapping::Compound`) in pre-declared schema. Will
  file a follow-up issue if needed; the bulk of pre-declared use cases
  (Arrow batch reader, SQL cursor) are flat.
- `RSerdeError::SchemaTypeMismatch` variant intentionally **not** added —
  the issue body's "Pick first-seen and document" recommendation matches
  today's silent-coerce behaviour, and adding the variant would require
  type-tracking the column's first-seen kind across pushes, which is
  beyond the issue's scope. If the maintainer prefers strict widening, file
  a follow-up.
