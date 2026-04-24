+++
title = "Fix: ColumnarDataFrame all-None columns produce list instead of typed NA vector"
description = "Teach SchemaDiscoverer to carry a runtime type hint so Option<T> fields with all-None rows emit typed NA vectors instead of list(NULL)."
weight = 50
+++

# Fix: ColumnarDataFrame all-None columns produce list instead of typed NA vector

## Problem summary

`ColumnarDataFrame::from_rows` serializes `Vec<T: Serialize>` into a columnar R
data.frame. When a struct field is `Option<T>` and every row holds `None`, the
resulting column is a `list(NULL, NULL, ...)` instead of an appropriately typed NA
vector (`NA_integer_`, `NA_real_`, `NA_logical_`, or `NA_character_`). This causes
downstream R code that assumes an atomic column type (e.g., `is.numeric`, `sum`,
dplyr verbs) to fail or behave incorrectly.

## Root cause

`miniextendr-api/src/serde/columnar.rs`, line 608-609 in `SchemaDiscoverer::process_field`:

```rust
let mut type_probe = TypeProbe {
    col_type: ColumnType::Generic,   // ← starts Generic
};
let _ = value.serialize(&mut type_probe);  // ← None: TypeProbe::serialize_none is a no-op
```

`TypeProbe::serialize_none` (line 780) does nothing — it intentionally preserves
whatever type was already recorded, so that a sequence probe that starts with `None`
upgrades when it later sees a `Some`. But when ALL rows are `None`, the type never
upgrades and stays `ColumnType::Generic`, causing `ColumnBuffer::new` to allocate a
`Generic(Vec<Option<SEXP>>)`. The fill pass then calls `RSerializer` on each `None`
value (line 943-945), which emits `SEXP::nil()` (NULL).

The fundamental issue is that `Serialize` gives the schema discoverer no static type
information from `Option<T>` — it only observes the runtime value, and `None` carries
no inner-type information through serde.

## Chosen design: runtime type-hint map on `ColumnarDataFrame`

**Rejected alternative A — `ColumnKind` trait with a new `from_rows` bound**: requires
either a derive macro change (proc-macro work in `miniextendr-macros`) or a
second public entrypoint with a different bound. The trait cannot be automatically
satisfied without a blanket impl that conflicts with the `Generic` fallback, and the
`from_rows<T: Serialize>` call site has no static access to field types.

**Rejected alternative B — attribute `#[miniextendr(column_type = "numeric")]`**: needs
proc-macro changes, adds attribute surface area, and is harder to use with external
types (e.g., types from other crates where the user can't add attributes).

**Chosen approach — builder with a runtime hint map**: Add a `ColumnarBuilder` that
collects `(field_name, ColumnType)` hints before running schema discovery. Hints are
consulted in `discover_schema_union` after probing: if the probed type is still
`Generic` (meaning all rows were `None`), the hint overrides it. The existing
`ColumnarDataFrame::from_rows` static method is kept unchanged as a zero-hint
convenience; `ColumnarDataFrame::builder()` is the new entry point.

This approach:
- Requires no macro changes
- Requires no new trait bounds or derives
- Is forward-compatible — a future `ColumnKind`-derive path can generate builder calls
- Works for external types the user cannot annotate
- Is self-describing at the call site:
  ```rust
  ColumnarDataFrame::builder()
      .hint("score", ColumnType::Real)
      .hint("flags", ColumnType::Logical)
      .from_rows(&rows)
  ```

`ColumnType` is made `pub` (it is currently crate-private).

## Implementation work items

### 1. Make `ColumnType` public and add `ColumnarBuilder`

**File**: `miniextendr-api/src/serde/columnar.rs`

- Change `enum ColumnType` to `pub enum ColumnType`. Add `#[non_exhaustive]` so
  we can add variants later without a breaking change.
- Add `pub struct ColumnarBuilder` with fields:
  ```rust
  pub struct ColumnarBuilder {
      hints: HashMap<String, ColumnType>,
  }
  ```
- Add `impl ColumnarDataFrame { pub fn builder() -> ColumnarBuilder }`.
- Add `impl ColumnarBuilder`:
  - `pub fn hint(mut self, field: impl Into<String>, col_type: ColumnType) -> Self`
  - `pub fn from_rows<T: Serialize>(self, rows: &[T]) -> Result<ColumnarDataFrame, RSerdeError>`
    — same body as the current `from_rows`, but passes `self.hints` into
    `discover_schema_union`.
- Update `ColumnarDataFrame::from_rows` to call
  `discover_schema_union_with_hints(rows, &HashMap::new())` internally, or refactor
  it to delegate to `ColumnarBuilder::default().from_rows(rows)`.

### 2. Thread hints through schema discovery

**File**: `miniextendr-api/src/serde/columnar.rs`

- Rename `discover_schema_union` to `discover_schema_union_with_hints` and add a
  `hints: &HashMap<String, ColumnType>` parameter.
- After the `TypeProbe` step in `SchemaDiscoverer::process_field` (line 608), if
  `type_probe.col_type == ColumnType::Generic`, check the hints map for the fully
  qualified field name. If a hint is found, use it instead of `Generic`.
- The field name available inside `process_field` is the flat serde field key (e.g.,
  `"score"`). For nested fields, the key passed to `process_field` is the struct
  field name at that nesting level, not the flattened name. Hints should match the
  serde field key passed to `process_field` at the top level only (nested fields are
  individually probed by `try_discover_nested`, not via process_field directly). This
  means hints for top-level fields work out of the box; nested fields can be
  separately hinted by their flattened name via a post-discovery override, or by
  hinting the top-level field key if the nested struct is fully `None`. Add a note in
  the public docs about this.
- The `hints` map must be threaded into `SchemaDiscoverer`. Add a `hints` field:
  ```rust
  struct SchemaDiscoverer<'h> {
      hints: &'h HashMap<String, ColumnType>,
      // ... existing fields
  }
  ```
  Update `SchemaDiscoverer::new` to accept `hints`. Update all construction sites.
- Keep `discover_schema_union` as a private wrapper calling
  `discover_schema_union_with_hints(rows, &HashMap::new())` so nothing else breaks.

### 3. Re-export `ColumnType` from the serde module

**File**: `miniextendr-api/src/serde.rs`

- Add `ColumnType` to the re-export line:
  ```rust
  pub use columnar::{ColumnType, ColumnarBuilder, ColumnarDataFrame, vec_to_dataframe};
  ```

### 4. Update module-level doc comment for `Option<T>` → `None` behavior

**File**: `miniextendr-api/src/serde.rs` (module doc) and `miniextendr-api/src/serde/columnar.rs` (top-level doc and `from_rows` rustdoc)

- In the `from_rows` rustdoc: expand the `Option<T>` row in the type table to note
  the all-`None` limitation and point to `ColumnarDataFrame::builder().hint(...)`.
- Add an example block showing the builder with `.hint("field", ColumnType::Real)`.
- In the columnar module top-level comment: add a "Caveats" section noting that schema
  discovery is runtime-only and cannot infer column type when all rows have `None`.

### 5. Rust fixture functions for the all-None test cases

**File**: `rpkg/src/rust/columnar_flatten_tests.rs`

Add new fixture structs and `#[miniextendr]` functions:

```rust
// All-None: Option<f64>, Option<i32>, Option<bool>, Option<String>
#[derive(Serialize)]
struct WithAllNoneReal { x: f64, score: Option<f64> }

#[derive(Serialize)]
struct WithAllNoneInt  { x: f64, count: Option<i32> }

#[derive(Serialize)]
struct WithAllNoneBool { x: f64, flag:  Option<bool> }

#[derive(Serialize)]
struct WithAllNoneStr  { x: f64, label: Option<String> }

// Mixed: first few rows None, later rows Some (regression guard — already worked)
#[derive(Serialize)]
struct WithLeadingNone { x: f64, value: Option<f64> }
```

Add `#[miniextendr]` functions:

- `test_columnar_all_none_real()` — two rows with `score = None`, builder hints Real
- `test_columnar_all_none_int()` — two rows with `count = None`, builder hints Integer
- `test_columnar_all_none_bool()` — two rows with `flag = None`, builder hints Logical
- `test_columnar_all_none_str()` — two rows with `label = None`, builder hints Character
- `test_columnar_all_none_no_hint()` — two rows with `score = None`, NO hint (documents
  the Generic fallback so the test suite confirms the fallback still produces a list)
- `test_columnar_leading_none()` — three rows: `None, None, Some(42.0)` with no hint
  (regression: type upgrades to Real when Some is seen, already works)

### 6. R testthat coverage

**File**: `rpkg/tests/testthat/test-columnar-flatten.R`

Add a new block after the existing skip-serializing-if tests:

```r
test_that("all-None Option<f64> with hint produces NA_real_ column", {
  df <- test_columnar_all_none_real()
  expect_type(df$score, "double")
  expect_true(all(is.na(df$score)))
})

test_that("all-None Option<i32> with hint produces NA_integer_ column", {
  df <- test_columnar_all_none_int()
  expect_type(df$count, "integer")
  expect_true(all(is.na(df$count)))
})

test_that("all-None Option<bool> with hint produces NA logical column", {
  df <- test_columnar_all_none_bool()
  expect_type(df$flag, "logical")
  expect_true(all(is.na(df$flag)))
})

test_that("all-None Option<String> with hint produces NA_character_ column", {
  df <- test_columnar_all_none_str()
  expect_type(df$label, "character")
  expect_true(all(is.na(df$label)))
})

test_that("all-None with no hint falls back to list column", {
  df <- test_columnar_all_none_no_hint()
  expect_type(df$score, "list")
})

test_that("leading-None with Some later upgrades type (regression)", {
  df <- test_columnar_leading_none()
  expect_type(df$value, "double")
  expect_equal(df$value, c(NA_real_, NA_real_, 42.0))
})
```

### 7. Update `from_rows` docstring type table

**File**: `miniextendr-api/src/serde/columnar.rs`, `from_rows` doc block (lines 119-128):

Replace the `Option<T>` table row:

```
| `Option<T>` | Same type with NA for `None` |
```

with:

```
| `Option<T>` | Same type with NA for `None`. When ALL rows have `None` for a field, the column type cannot be inferred — use `ColumnarDataFrame::builder().hint("field", ColumnType::Real)` to specify it explicitly. Without a hint the column falls back to a `list` column. |
```

Add a builder example below the existing `from_rows` example.

### 8. Docs site: update `ADAPTER_COOKBOOK.md` or `TYPE_CONVERSIONS.md`

**File**: `docs/ADAPTER_COOKBOOK.md` (or whichever doc currently covers
`ColumnarDataFrame::from_rows` — verified to contain serde content)

Add a "All-None columns" subsection under the columnar data.frame section:

- Explain the limitation (schema discovery is runtime-only)
- Show the builder pattern with `.hint()`
- Note that the fallback (no hint) is a list column, which can be useful for
  heterogeneous data

## Public API surface changes

| Added | Kind | Notes |
|-------|------|-------|
| `ColumnType` | `pub enum` in `serde::columnar`, re-exported from `serde` | Previously crate-private. `#[non_exhaustive]` so new variants (e.g., `Complex`) don't break user match arms. |
| `ColumnarBuilder` | `pub struct` in `serde::columnar`, re-exported from `serde` | Builder returned by `ColumnarDataFrame::builder()`. |
| `ColumnarDataFrame::builder()` | `pub fn` | Returns a `ColumnarBuilder`. |
| `ColumnarBuilder::hint(field, col_type)` | `pub fn` | Chainable; builder-pattern. |
| `ColumnarBuilder::from_rows(&[T])` | `pub fn` | Same signature as `ColumnarDataFrame::from_rows`. |

`ColumnarDataFrame::from_rows` and `vec_to_dataframe` are unchanged.

## Verification checklist

- `just check` passes (no new warnings)
- `just clippy` passes for both `clippy_default` and `clippy_all` feature sets
- `just test` passes (all Rust unit tests)
- `just rcmdinstall && just devtools-test` passes (all R tests)
- `test_columnar_all_none_real()` returns a data.frame with `typeof(df$score) == "double"` and `all(is.na(df$score))`
- `test_columnar_all_none_no_hint()` returns a data.frame with `typeof(df$score) == "list"` (fallback documented, not fixed)
- `test_columnar_leading_none()` returns a data.frame with `df$value == c(NA, NA, 42.0)` and `typeof(df$value) == "double"`
