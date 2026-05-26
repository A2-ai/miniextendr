# Factor columns deserialise to labels for String fields (#689)

## Goal

`dataframe_to_vec::<T>(sexp)` currently fails — or returns the wrong thing — when
`T` declares a `String` (or `Option<String>`) field for an R factor column.
Factors are stored as `INTSXP` with a `levels` STRSXP attribute, so today the
deserialiser sees an integer column and reports a type mismatch against a
`String` field.

Fix: when a column is a factor (i.e. `INTSXP` carrying a `levels` STRSXP), the
serde visitor for string-type fields receives the label string instead of the
integer code. Serde's type-driven dispatch makes this clean — an `i32` field
keeps its existing integer-code behaviour because the visitor calls
`deserialize_i32`, while a `String` field calls `deserialize_str` /
`deserialize_string` and lands on the new factor branch.

## Background

After the #708 consolidation, all three public entry points
(`dataframe_to_vec`, `with_dataframe_rows`, `dataframe_to_vec_borrowed`) share
a single `CellDeserializer<'sexp, 'n>`. Factor handling lives — and only ever
needs to live — inside `CellDeserializer`'s `Deserializer` impl. The helpers
`deserialize_integer_cell` / `deserialize_real_cell` remain unchanged because
they're only called from the integer/floating arms.

`SexpExt` already provides `is_factor()` (wraps `Rf_isFactor`) and `get_levels()`
(wraps `Rf_getAttrib(_, R_LevelsSymbol)`), so no new FFI surface is required.

## Design

Always-on. The decision is driven by the user's `T` field type via serde's
visitor-method dispatch:

- `String` / `Option<String>` field → `deserialize_str` / `deserialize_string`
  → label lookup path (new).
- `i32` / `Option<i32>` field → `deserialize_i32` → integer-code path
  (existing). Unchanged.
- `char` field → `deserialize_char`. Label-lookup path also applies; if the
  label is not a single char, the existing "string of length N" error fires.
- `deserialize_any` (e.g. `serde_json::Value` style consumers) — we route to
  the label path for factor columns. This matches the principle of least
  surprise: a column with `class = "factor"` and human-readable levels is most
  naturally summarised by its labels.
- `deserialize_option` — factor `NA_INTEGER` already yields `visit_none`
  through the existing `SEXPTYPE::INTSXP` arm; no change needed.

### Per-method changes inside `CellDeserializer`

For each of `deserialize_str`, `deserialize_string`, `deserialize_char`,
`deserialize_any`:

1. Compute `is_factor = self.col.is_factor()`.
2. If `is_factor`, read `INTEGER_ELT(col, row)`:
   - `NA_INTEGER` → `visit_none` (caller used `Option<…>`) or a clear
     `UnexpectedNa` error (caller used non-`Option`).
   - Otherwise, validate the 1-based code against `levels` length and call
     `visitor.visit_borrowed_str(label)`.
   - Out-of-range code → fresh `RSerdeError::Message` carrying column name +
     code + n_levels (deterministic diagnostic).
3. Else, fall through to existing behaviour.

The current `deserialize_str` and `deserialize_string` reject anything that
isn't `STRSXP`, so the factor-detection branch sits in front of that check.
For `deserialize_any`, we extend the existing `SEXPTYPE::INTSXP` arm with a
factor sub-branch (no separate top-level if).

### Implementation notes

- Use `is_factor()` from `SexpExt` (already imported via the `SexpExt` trait).
- Use `get_levels()` from `SexpExt`.
- Use `charsxp_to_str` (already imported from `from_r`).
- Lifetime of the label string is `'de` (= `'sexp`) — same as the existing
  character-column path; the SEXP is rooted by R's argument frame.
- A factor with code `NA_INTEGER` and a non-`Option` String field should produce
  `RSerdeError::UnexpectedNa`, matching the regular `STRSXP`/`INTSXP` NA handling.

## Tests

Integration tests in `miniextendr-api/tests/dataframe_de.rs` (factor-only
module section). Helper `make_factor_dataframe()` builds an R `data.frame`
with one factor column by hand using the `build_factor` / `build_levels_sexp`
helpers from `miniextendr_api::factor`. Tests:

1. **Factor → `String`** returns the label.
2. **Factor → `Option<String>`** returns `Some(label)` for valid codes and
   `None` for `NA_INTEGER`.
3. **Factor → `i32`** returns the 1-based code (regression check — proves the
   change is non-disruptive for users who genuinely want the code).
4. **Empty factor** (0-row data.frame with a factor column) returns
   `Ok(vec![])`.
5. **Out-of-range factor code** synthesised by direct integer writes →
   error mentioning the column name.
6. **Factor → `char`** with single-char level succeeds; multi-char level
   produces the existing "single character" error.

## rpkg fixture

A `#[miniextendr]`-exported `gc_stress_factor_labels()` no-arg wrapper in
`rpkg/src/rust/gc_stress_fixtures.rs` that builds a factor column data.frame
internally and round-trips through `dataframe_to_vec::<Row>` where `Row` has
a `String` factor field. Provides a no-arg testthat target so the fast
gctorture sweep over `rpkg`'s exports exercises this code path end-to-end.

A testthat case in `rpkg/tests/testthat/` (existing serde test file) that
builds a data.frame with a factor column via R, hands it to a small Rust
fixture, and confirms the labels round-trip.

## Doc update

Update the limitations note in `miniextendr-api/src/serde/dataframe_de.rs`
crate-level docs + `dataframe_to_vec` rustdoc + the limitations bullet in
`with_dataframe_rows` / `dataframe_to_vec_borrowed`. The type-mapping table
gains a row for factor columns.

I won't ship a `docs/`-level update in this PR. The serde manual page in
`docs/` already says little about column-type handling and adding a factor
section there is a separate documentation task. Will note as a follow-up.

## Verification

- `cargo test -p miniextendr-api --features serde --test dataframe_de` —
  new + existing tests pass.
- `just configure && just rcmdinstall && just devtools-test` — rpkg fixture
  green.
- `just clippy` (CI's clippy_default subset of `--workspace --all-targets
  --locked -- -D warnings`) clean.
- CI's `clippy_all` curated feature list — reproduce locally.
- gctorture sweep over `rpkg/`'s exports (per CLAUDE.md) catches GC issues
  if any.

## Behaviour change call-out

A user with a factor column declaring `field: String` previously saw a
type-mismatch error. Now they get the label. A user with `field: i32` still
gets the integer code. No silent change for any case that previously
succeeded.

## Out of scope (follow-ups, if any)

- Manual page / docs/ update for the factor behaviour.
- Site/docs sync.
