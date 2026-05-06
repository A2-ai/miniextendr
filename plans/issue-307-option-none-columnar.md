# issue-307 — `ColumnarDataFrame`: all-`None` columns land as logical NA, not list-of-NULL

Tracks https://github.com/A2-ai/miniextendr/issues/307.

## Problem

`ColumnarDataFrame::from_rows` discovers column types by **probing values at runtime**.
When every row carries `None` for an `Option<T>` field, the probe never sees a `Some`,
the column type stays `ColumnType::Generic`, and R gets `list(NULL, NULL, …)` instead of
an atomic vector with `NA`.

Confirmed on `Option<u64>` (dvs2 `AddResult::stored_size`) and `Option<String>` (dvs2
`Config::metadata_folder_name`). Single-row tibbles with one defaulted optional are the
common trigger.

## Approach

The Rust type isn't recoverable inside serde's `serialize_field<T: ?Sized + Serialize>`
on stable — `TypeId::of::<T>()` requires `T: 'static`, the trait bound is fixed, and
specialization is nightly. So we don't try.

R's `NA_logical_` is the universal NA: in any operation with another type it coerces
to that type's NA — `c(NA, 1L)` is integer, `c(NA, "x")` is character,
`bind_rows`/`mutate`/`coalesce` all coerce on contact. An all-NA logical column is
indistinguishable from an all-NA typed column for everything users actually do
downstream.

So at assembly time, when a column ended up `Generic` because every entry was `None`,
emit a `LGLSXP` of length `nrow` filled with `NA_LOGICAL` instead of a list of `NULL`s.
The discriminator is already in the data: `ColumnBuffer::Generic(Vec<Option<SEXP>>)`,
where `push_na` pushes `None` and real values push `Some(sexp)`. `v.iter().all(Option::is_none)`
distinguishes "every row was None" from "rows had values that fell through to the
generic-list path" (`Vec<u8>`, `HashMap<…>`, etc.) — only the first downgrades.

No user-side hint, no derive, no wrapper, no proc-macro changes, no API surface change.

### Schema upgrade (two-phase discovery)

The original single-pass `discover_schema_union` had a critical limitation: when the
first row had `None` for a field, the column type was locked to `Generic` and subsequent
`Some(value)` rows never got to upgrade it — the `if unified_mappings.contains_key(key)
{ continue; }` guard short-circuited on the second row.

This was fixed by replacing the single-pass loop with two-phase discovery:

- **Phase A**: probe every row, accumulate per-key candidates
  (`Candidate::Scalar(ColumnType)` or `Candidate::Compound { fields, sub_map }`).
- **Phase B**: resolve each key using the type lattice:
  - `Compound` beats everything (has concrete shape).
  - `Scalar(non-Generic)` beats `Scalar(Generic)`.
  - `Scalar(Generic)` is the bottom (all-None probes land here).
  - Tie between two non-Generic Scalars: existing-wins (no widening).
  - Tie between two Compounds: existing-wins (recursive union is a follow-up).

This allows `[{x: None}, {x: Some(42u64)}]` to produce a numeric column instead of a
Generic list, and `[{point: None}, {point: Some(Point{x:1.0,y:2.0})}]` to produce
`point_x`/`point_y` columns instead of a single Generic column.

The short-circuit ("if this row contributed no new fields and it's not the first row,
break") was also removed — it was an optimization that was incompatible with the upgrade
semantics.

#### Remaining limitation: truly-all-None nested Option<Struct>

When **every** row has `None` for an `Option<UserStruct>`, no row ever contributes a
`Compound` candidate. Phase B resolves the key to `Scalar(Generic)`, which the
assembly-time all-None downgrade then converts to a single logical-NA column. This is
structurally unfixable without a type-level hint on stable Rust — `TypeId::of::<T>()`
requires `T: 'static` which serde's trait bound doesn't carry. The truly-all-None case
is documented as a known limitation.

#### Remaining limitation: Compound-vs-Compound recursive union

When two rows contribute different `Compound` shapes for the same key (e.g. two enum
variants where the nested struct differs per variant), the first `Compound` wins and
the second is silently discarded. Recursive union of field sets is tracked as a separate
follow-up issue.

## Anchor points (in `miniextendr-api/src/serde/columnar.rs`)

- `assemble_dataframe` (called at line 171) — phase 4, where each `ColumnBuffer` becomes
  a column SEXP. The downgrade lives in `column_to_sexp` (called from `assemble_dataframe`).
- `ColumnBuffer::Generic(Vec<Option<SEXP>>)` — the buffer variant the downgrade inspects.
- `ColumnBuffer::push_na` pushes `None` into the Generic buffer (for pad/missing rows).
- `ColumnBuffer::push_value` for Generic calls `value.serialize(RSerializer)`, which calls
  `RSerializer::serialize_none` → returns `SEXP::nil()`, pushed as `Some(SEXP::nil())`.
  **Key finding**: when `Option<T>` is `None`, `push_value` pushes `Some(SEXP::nil())`,
  NOT `None`. So the all-None check must test for both `None` and `Some(s) if s.is_nil()`.
- `NA_LOGICAL` is `i32::MIN`; constant lives at `miniextendr-api/src/altrep_traits.rs`
  (imported alongside `NA_REAL` at line 15 of `columnar.rs`).

## Plan (flat priority order)

1. **Read `assemble_dataframe`** — not yet inspected at the time this plan was written.
   Confirm the loop shape, the `LGLSXP` allocation pattern (likely
   `Rf_allocVector(SEXPTYPE::LGLSXP, nrow)`), and where the per-column SEXP is set into
   the parent VECSXP. Confirm `OwnedProtect` discipline matches the rest of the file.

2. **Add the all-None downgrade.** In the per-column branch for `ColumnBuffer::Generic`,
   check `v.iter().all(Option::is_none)`. If true, allocate `LGLSXP` of length `nrow`,
   fill with `i32::MIN`, set as the column. If false, fall through to the existing
   list-column assembly. Six lines or so.

3. **Tests in `rpkg/src/rust/columnar_option_none_tests.rs`** (Rust side):
   - `Option<u64>` all-None.
   - `Option<String>` all-None.
   - `Option<bool>` all-None.
   - `Option<UserStruct>` all-None (covers nested-struct flattening with all entries None).
   - `Option<HashMap<…>>` all-None (foreign generic).
   - Mixed `Some`/`None` for each above — schema unchanged.
   - `Vec<u8>` field with values — still list column (no downgrade).
   - `Option<Vec<u8>>` all-None — downgrade fires (no values, no list semantics to preserve).
   - `#[serde(flatten)]` with all-None inner field — typed atomic NA at the flattened position.
   - Enum union: variants A and B both have field `x: Option<u64>`, every row is variant A
     with `x = None` — column lands as logical-NA. Adding any variant-B row with
     `x = Some(42)` flips it to `numeric` via the existing probe.

4. **Tests in `rpkg/tests/testthat/test-columnar-option-none.R`** (R side, mirrors Rust):
   - `is.logical(df$col) && all(is.na(df$col))` for the all-None cases.
   - Coercion smoke test: `dplyr::bind_rows(df_all_none, df_with_values)` produces the
     numeric/character column without warning.
   - `tibble::as_tibble(df)` doesn't reintroduce the list column.

5. **Rustdoc note on `ColumnarDataFrame::from_rows`** (line 119 supported-types table):
   add a row noting that all-`None` `Option<T>` columns land as logical NA and rely on
   R's coercion semantics for downstream typing.

6. **Doc page `docs/COLUMNAR_OPTION_NONE.md`.** Half-screen explainer:
   - The old failure (list-of-NULL).
   - The new behaviour (logical NA via assembly-time downgrade).
   - The R coercion guarantee.
   - When this *isn't* what you want (rare): use `with_column` to inject a typed NA
     vector explicitly — pattern already in the issue body.

7. **CLAUDE.md gotcha line under *Common gotchas*.** One bullet:
   *"`ColumnarDataFrame::from_rows`: columns where every row was `None` land as logical
   NA (not list-of-NULL); R coerces to the surrounding type on first use."*

## What this plan deliberately does NOT do

- **No `TypeId` table.** `TypeId::of::<T>()` requires `T: 'static`; serde's
  `serialize_field<T: ?Sized + Serialize>` doesn't carry it; we can't add it. Dead end
  on stable.
- **No `type_name::<T>()` string match.** Output isn't stability-guaranteed.
- **No user-side hint of any form.** No `#[serde(with = …)]`, no
  `#[miniextendr(column_type = …)]`, no `#[derive(ColumnSchema)]`, no `from_rows_with_schema`.
- **No nightly specialization.**
- **No ALTREP.**
- **No change to `from_rows`'s bound.** Stays `T: Serialize`.

## Risks / open questions

- **`Vec<u8>` (and other genuine list columns) with one all-`None`-emitting field
  alongside.** Each column is independent. The downgrade only fires per-column when
  *every* entry of that column is `None`. A row that has `bytes: vec![]` for the bytes
  field and `stored: None` for the optional field downgrades only the optional column —
  the bytes column stays a list column with `Some(sexp)` entries. Verify with the tests
  in step 3.
- **`Option<()>` / unit-typed options.** `serialize_unit` sets `ColumnType::Generic`
  (line 779) and pushes a generic SEXP via the value path. All-None of `Option<()>` →
  downgrade. All-Some of `Option<()>` → list column. Acceptable; unit options in
  output structs are vanishingly rare.
- **Empty `&[]` input.** Already short-circuits to `empty_dataframe()` at line 130.
  Unaffected.
- **Single-row corner case.** This is the *exact* dvs2 trigger. Tests must include
  `vec![Row { stored: None }]` (length 1), not just length ≥ 2.
- **Existing test `dataframe_collections_test.rs:454`** references "all None" for a
  different field type. Confirm whether that test asserts the list-of-NULL behaviour
  (in which case it needs updating) or asserts something orthogonal.
- **GC discipline in the new branch.** New `Rf_allocVector(LGLSXP, nrow)` allocation
  must be PROTECTed across any subsequent allocation in the same loop iteration if
  the surrounding code does further allocation before storing into the parent VECSXP.
  Match whatever pattern `assemble_dataframe` already uses.
