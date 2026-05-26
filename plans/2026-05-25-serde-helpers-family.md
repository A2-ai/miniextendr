# 2026-05-25 — High-level serde dataframe helpers (closes #700, #697, #699)

## Goal

Add three ergonomic helpers around `vec_to_dataframe` / `ColumnarDataFrame` /
`NamedDataFrameListBuilder` for common Rust collection shapes that meet R
data.frames. All three live in `miniextendr-api/src/serde/columnar.rs` and
share a unifying `Shape`-style enum config pattern.

Issues bundled because (per #700's body) "all four are the same gap from
different angles". This PR ships three of them.

## Decisions taken (verbatim "the plan")

- **One unified return enum, named `DataFrameShape`** (not `TaggedShape`,
  not `ColumnarShape`). Used by `result_to_dataframe` *and* by the
  augmented `vec_to_dataframe_split`. Rationale: "shape" matches the new
  config enums on the input side (`ResultShape`, `SplitShape`), so the
  vocabulary lines up.

  ```rust
  pub enum DataFrameShape {
      /// Single data.frame. Used for:
      ///   - vec_to_dataframe_split single-variant short-circuit
      ///   - result_to_dataframe Auto all-Ok
      ///   - result_to_dataframe Collated
      ///   - vec_to_dataframe_split Collated
      Bare(ColumnarDataFrame),

      /// list(results = <df | sentinel>, error = df). result_to_dataframe.
      Split { results: SplitResults, error: ColumnarDataFrame },

      /// list(VariantA = df, VariantB = df, …). vec_to_dataframe_split.
      PerVariantList(Vec<(String, ColumnarDataFrame)>),
  }

  pub enum SplitResults {
      Some(ColumnarDataFrame),
      None(SEXP),  // user-chosen sentinel; SEXP is "Bare(SEXP)" via IntoR
  }

  impl IntoR for DataFrameShape { type Error = Infallible; … }
  ```

  - PR body will note that `vec_to_dataframe_split` was previously returning
    `crate::list::List`; the new shape return widens that to a real enum so
    callers can switch on it in Rust, and it round-trips to the same SEXP
    shape through `IntoR`.

- **`map_to_dataframe<K, V>` returns `ColumnarDataFrame`** directly. No
  shape enum needed (one shape only). Signature:

  ```rust
  pub fn map_to_dataframe<K, V>(
      map: &BTreeMap<K, V>,
      key_column: &str,
  ) -> Result<ColumnarDataFrame, RSerdeError>
  where K: serde::Serialize, V: serde::Serialize;
  ```

  - `BTreeMap` for ordered output.
  - For `HashMap`, ship `hashmap_to_dataframe<K, V>` with `K: Ord` bound
    (collect → sort by serialized key) so output order is deterministic.
    Docstring directs callers wanting raw insertion order to use a
    `BTreeMap` indirection or live with `hashmap_to_dataframe_unsorted` if
    that turns out to be needed (file follow-up issue, not in this PR).
  - Implementation: synthesise an internal `MapEntry { key, value }` row
    type via a custom serializer wrapper `MapRow<'a, K, V>` that flattens
    the (k, v) pair into a single struct whose first field name is
    `key_column` and whose remaining fields are `V`'s serialized fields.
    Routes through the existing `ColumnarDataFrame::from_rows` machinery
    so we get schema discovery and flattening for free.

- **`result_to_dataframe<T, E, S>`** with `ResultShape<S>`:

  ```rust
  pub enum ResultShape<S> {
      Auto { empty_ok_sentinel: S },
      Collated,
      Split { empty_ok_sentinel: S },
  }
  ```

  - `Auto { sentinel }`: all-Ok → `DataFrameShape::Bare(df)`. Any Err →
    `DataFrameShape::Split { results, error }` with `results =
    SplitResults::Some(df)` if at least one Ok, else `SplitResults::None(sentinel)`.
  - `Collated`: union schema (T's fields + E's fields + is_error bool);
    NA-fill for the variant not present. `DataFrameShape::Bare(df)`.
  - `Split { sentinel }`: always returns `DataFrameShape::Split { … }`.

  Implementation:

  - Auto / Split: walk `rows`; collect `Vec<&T>` and `Vec<&E>` via
    partition by reference (no clone). Call `vec_to_dataframe` on each.
    For sentinel path: only allocate the sentinel SEXP if `oks.is_empty()`.
  - Collated: synthesise a `CollatedRow<'a, T, E>` serializer that emits
    `is_error: bool` + every (Ok-variant or Err-variant) field. NA-fill
    comes free from `ColumnarDataFrame::from_rows`'s union-schema
    behaviour: each row only emits its own variant's fields and the
    `is_error` flag; the union schema absorbs both sides; rows missing
    the other variant's keys get NA-padded.
  - **GC discipline**: Auto/Split path builds two intermediate
    `ColumnarDataFrame`s. Push both into a `NamedDataFrameListBuilder`
    intermediately so they're protected for the duration of the helper.
    Then unwrap when we don't actually need the list (we want a typed
    `DataFrameShape::Split`). Actually simpler: build with `OwnedProtect`
    on each, then construct `DataFrameShape::Split` consumed from the
    protections. Match the existing pattern.

- **`vec_to_dataframe_split<T>`** — backwards-incompatible signature.
  Existing single-arg form replaced by:

  ```rust
  pub enum SplitShape {
      /// Today's behaviour: list(VariantA = df, …); single-variant short-circuit.
      PerVariantList,
      /// list(VariantA = df, …), each df has a leading variant-tag column.
      PerVariantListWithTag { column: String },
      /// One data.frame with union schema + leading variant-tag column.
      Collated { column: String },
  }

  pub fn vec_to_dataframe_split<T: Serialize>(
      rows: &[T],
      shape: SplitShape,
  ) -> Result<DataFrameShape, RSerdeError>;
  ```

  Callers updated:
  - `rpkg/src/rust/columnar_flatten_tests.rs` — 4 call sites: pass
    `SplitShape::PerVariantList` and adapt to `DataFrameShape` return.
  - `miniextendr-api/tests/serde_columnar.rs` — 2 call sites.

  CLAUDE.md says "No backwards compatibility: unreleased project" — fine.

  - **PerVariantList** — existing behaviour. Returns
    `DataFrameShape::PerVariantList(Vec<(String, ColumnarDataFrame)>)`
    for multi-variant; `DataFrameShape::Bare(df)` for the single-variant
    short-circuit; `DataFrameShape::Bare(empty_dataframe())` for empty
    input (deliberate change from "unnamed empty list" — see below).
  - **PerVariantListWithTag** — for each per-variant df, prepend the
    variant-name column via a new `prepend_column` method on
    `ColumnarDataFrame` (mirror of `with_column` but inserting at index
    0; will likely need to introduce). All-tag-strings are produced by
    allocating a `STRSXP` of nrow repeats of the variant name.
  - **Collated** — call `from_rows` over a `TaggedVariantRow<'a, T>`
    wrapper that emits `<column>: <variant_name>` plus the variant's
    struct fields flattened. Same union-schema mechanic as
    `result_to_dataframe::Collated`.

  Empty-input behaviour: today returns unnamed empty `list()` because
  variant set is unknowable. Under new shape:
  - `PerVariantList` / `PerVariantListWithTag`: empty
    `DataFrameShape::PerVariantList(vec![])` → empty named list on R side.
    Compatible with `expect_equal(length(result), 0)`. Behavioural diff:
    `is.data.frame()` is FALSE either way.
  - `Collated`: empty `DataFrameShape::Bare(empty_dataframe())` → bare
    0-row data.frame with the user's tag column missing. Or, return an
    error? Pick: error with `"vec_to_dataframe_split(Collated): empty input
    — variant set is unknowable"`. That avoids fabricating a tag column
    out of nowhere.

  Existing test `test_columnar_empty_split` expects an unnamed empty
  list; updating the test under the new return shape:
  `expect_equal(length(result), 0)` should still pass with `PerVariantList(vec![])`
  going through `IntoR`.

## Reused / added primitives

- **`ColumnarDataFrame::prepend_column(name, sexp) -> Self`** — new.
  Like `with_column`, but inserts at index 0 by allocating a new VECSXP
  of `ncol + 1` and copying. Used by `PerVariantListWithTag`.
- **`make_strsxp_repeat(value: &str, n: usize) -> SEXP`** — small internal
  helper that allocates a STRSXP of length `n` filled with one repeated
  CHARSXP. Avoids re-CharSXP-ing per row.

## File list

- **`miniextendr-api/src/serde/columnar.rs`** — add three helper fns + enums
  + `DataFrameShape` + `IntoR for DataFrameShape` + `prepend_column`.
  Migrate `vec_to_dataframe_split` signature.
- **`miniextendr-api/src/serde.rs`** — re-export new public items.
- **`miniextendr-api/tests/serde_columnar.rs`** — update 2 call sites,
  add new R-thread integration tests for the three helpers.
- **`rpkg/src/rust/columnar_flatten_tests.rs`** — update 4 call sites of
  `vec_to_dataframe_split`; add fixtures for the three new helpers /
  shapes.
- **`rpkg/src/rust/gc_stress_fixtures.rs`** — add `gc_stress_*` fixtures
  for each new SEXP-storing path:
  - `gc_stress_map_to_dataframe`
  - `gc_stress_result_to_dataframe_auto` / `collated` / `split`
  - `gc_stress_vec_to_dataframe_split_pervariantwithtag`
  - `gc_stress_vec_to_dataframe_split_collated`
- **`rpkg/tests/testthat/test-columnar-flatten.R`** — update existing
  `vec_to_dataframe_split` tests; add new testthat blocks per fixture.
- **`docs/SERDE_NATIVE.md`** (if exists) — add a short section. If absent,
  skip — don't create new top-level docs files (root CLAUDE.md says).

## Verification (flat priority)

1. `cargo build -p miniextendr-api --features serde` (sandbox-disabled)
2. `cargo test -p miniextendr-api --features serde serde::columnar`
3. `cargo test -p miniextendr-api --features serde --test serde_columnar`
4. `just configure && just rcmdinstall && just force-document`
5. `just devtools-test 2>&1 > /tmp/devtools-test.log` — read log
6. `just clippy` (clippy_default flavour); then the `clippy_all` feature
   list from `.github/workflows/ci.yml`
7. gctorture sweep: load package, then enable gctorture, drive each new
   `gc_stress_*` fixture in a loop ~20 iter, no abort/segfault
8. `cargo test -p miniextendr-macros` — UI snapshot pass-through

## Out of scope (file as follow-up issues if not landed)

- `hashmap_to_dataframe_unsorted` (raw insertion order) — gate on user need
- `variant_name_style` knob (camel/snake/kebab) — serde already provides
  `#[serde(rename_all = …)]` per #699's own body
- Recursive Compound-vs-Compound union widening — unrelated, tracked already
- `typed_dataframe!` macro (#698) — separate sprint

## Plan footnotes

- `DataFrameShape::PerVariantList` carries `Vec<(String, ColumnarDataFrame)>`
  but the IntoR impl will need to convert that to a SEXP through
  `NamedDataFrameListBuilder` so the per-variant SEXPs are protected
  during assembly. Otherwise a GC firing during the names STRSXP
  allocation can reap an unprotected child df.
- `SplitResults::None(SEXP)` carries a raw SEXP; the sentinel is allocated
  by calling `.into_sexp()` on the user's `S: IntoR` before the result
  shape is constructed. The DataFrameShape's own `IntoR` then assembles
  the outer list. **GC**: protect the sentinel + the error df via
  `NamedDataFrameListBuilder` inside the `IntoR` impl so neither gets
  GC'd between the two `set_vector_elt` calls. Plain pairs vec with the
  builder is the cleanest path.
