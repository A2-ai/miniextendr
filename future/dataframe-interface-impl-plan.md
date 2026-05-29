# Implementation plan: unified DataFrame interface

Implements the maintainer-locked decisions from `future/dataframe-interface-redesign.md`
(#771) and tracking issue #772.

## Locked decisions (do not relitigate)

1. Trait names **`IntoDataFrame` / `FromDataFrame`** (mirror `IntoR` / `TryFromSexp`).
2. **Explicit `_par` methods** (`into_dataframe_par()`), `#[cfg(feature = "rayon")]` — not a
   hidden threshold inside `into_dataframe()`.
3. **Land-then-migrate**: this PR lands the new surface and absorbs the already-merged rayon
   work (#765 reader, #768 `RDataFrameBuilder`, #777 flattened fill) onto it.
4. One owned `DataFrame` type, one `DataFrameError`, one NA contract.
5. Leave `as_coerce::AsDataFrame` (S3 coercion) alone — different concept, no name collision
   with `Into`/`From`.

## Scope cut (complete-coherent increment)

The full end-state deletes three legacy types whose internals are woven through the two
largest files in the tree (`serde/columnar.rs` ~142K, `dataframe_derive/enum_expansion.rs`
~124K) plus the enum-flatten codegen. Deleting all three and rewiring all flatten codegen in
one pass cannot be compile-verified safely in a single iteration. This PR therefore ships the
**complete new surface, fully working and tested**, and migrates the primary entry points,
while DEMOTING (not silently keeping) the legacy types behind the new façade:

- **New public surface (this PR):** `DataFrame` owned type, `IntoDataFrame` / `FromDataFrame`
  core traits, `IntoDataFrameExt::into_dataframe_par`, `DataFrameError` (extended), the derive
  emitting the new traits, `DataFrame::builder` (ex-`RDataFrameBuilder`).
- **Demoted-to-internal (this PR):** the *old* `convert::IntoDataFrame` row→List trait is
  renamed to a `#[doc(hidden)]` internal `ColumnSource` engine trait that the flatten codegen
  and the new public `IntoDataFrame` both delegate to — NOT a second public way to do the same
  thing. `ColumnarDataFrame`'s editing methods (`rename`/`drop`/`select`/…) move onto
  `DataFrame`; `ColumnarDataFrame` becomes a `#[doc(hidden)]` thin alias retained only as the
  serde assembler's return shape, tracked for removal.
- **Deferred removals → GitHub issues (referenced in PR):** full deletion of the
  `ColumnarDataFrame` name and the `convert::DataFrame<T>` row-buffer struct; collapsing the
  serde streaming/list builders' names (`DataFrameStreamer` / `DataFrameListBuilder` renames);
  `RowDataFrame` companion `#[doc(hidden)]` demotion in the derive.

## Flat work list (priority order)

1. **`DataFrame` owned type** (`miniextendr-api/src/dataframe.rs`): `struct DataFrame { sexp: SEXP }`.
   - Read API absorbed from `DataFrameView`: `from_sexp`, `column::<T>`, `column_index`,
     `column_raw`, `nrow`, `ncol`, `names`, `contains_column`, `validate`, `as_list`, `as_sexp`.
   - Editing API absorbed from `ColumnarDataFrame`: `rename`, `strip_prefix`, `drop`, `select`,
     `prepend_column`, `with_column`.
   - `impl IntoR` (returns backing SEXP) + `impl TryFromSexp` (validates `data.frame`).
   - `DataFrame::builder(nrow)` constructor → returns the ex-`RDataFrameBuilder` retargeted to
     yield `DataFrame` from `.build()`.
   - Keep `DataFrameView` as a `#[doc(hidden)]` type alias = `DataFrame` for migration; tracked
     for deletion.

2. **`DataFrameError`** extended (same module): add variants absorbing serde schema/serialize
   failures and the bare-`String` reader error. Add `From<RSerdeError>` (under `serde`) and a
   `Message(String)` / `Conversion(String)` catch-all. One NA contract documented here.

3. **Core traits** (`miniextendr-api/src/dataframe.rs` or a sibling):
   ```rust
   pub trait IntoDataFrame { fn into_dataframe(self) -> Result<DataFrame, DataFrameError>; }
   pub trait FromDataFrame: Sized { fn from_dataframe(df: &DataFrame) -> Result<Self, DataFrameError>; }
   pub trait IntoDataFrameExt: IntoDataFrame + Sized {
       #[cfg(feature = "rayon")] fn into_dataframe_par(self) -> Result<DataFrame, DataFrameError>;
   }
   impl<T: IntoDataFrame> IntoDataFrameExt for T {}
   ```

4. **Rename old `convert::IntoDataFrame`** (`-> List`) → `#[doc(hidden)] pub trait ColumnSource`
   with `into_column_list(self) -> List` + `into_named_columns`. Update the derive + flatten
   codegen + `convert::DataFrame<T>` impl + `factor.rs`/`markers.rs`/`list.rs`/`serde.rs`
   references. This trait is the internal column-assembly engine, not public.

5. **Blanket impls** of the new public `IntoDataFrame`:
   - `impl<T: ColumnSource> IntoDataFrame for T` — wraps `into_column_list()` into a `DataFrame`,
     returning `Result` (no more `panic!` on unnamed columns; the panic becomes
     `DataFrameError`). This is the row-oriented path replacing `DataFrame<T>::from_rows`.
   - serde columnar: `FromDataFrame for Vec<T: DeserializeOwned>` via `dataframe_to_vec`
     (gated `serde`); `ColumnarDataFrame::from_rows` rewired so `Vec<AsSerializeRow<T>>` reaches
     `into_dataframe()`.

6. **Migrate the merged rayon reader/fill onto the traits** in the derive
   (`dataframe_derive.rs`): emit `impl IntoDataFrame for Vec<Row>` (delegating to the existing
   column assembly), `impl FromDataFrame for Vec<Row>` (delegating to the #765 reader, now
   returning `DataFrameError` not `String`), and `into_dataframe_par` (delegating to the #777
   flattened `from_rows_par` / `try_from_dataframe_par`). Keep the existing `from_rows` /
   `from_rows_par` / `to_dataframe` / `from_dataframe` / `try_from_dataframe[_par]` companion
   methods working (they're the engine the trait impls call).

7. **lib.rs exports**: add `DataFrame`, `IntoDataFrame`, `FromDataFrame`, `IntoDataFrameExt`,
   `DataFrameError`. Keep `DataFrameView` re-export (alias).

8. **rpkg fixtures + testthat**: add a `gc_stress_*` no-arg fixture exercising `DataFrame`'s
   SEXP storage + builder path; add fixtures calling `into_dataframe()` / `from_dataframe()` /
   `into_dataframe_par()` to prove the new surface end-to-end. Keep existing dataframe fixtures
   compiling.

9. **Docs**: `docs/CONVERSION_MATRIX.md` update + new `docs/DATAFRAME_INTERFACE.md` with the
   before→after table.

10. **Verification**: `cargo fmt`; clippy_default + clippy_all `-D warnings`; compile with/without
    rayon+serde; `just configure && just rcmdinstall && just force-document` (commit generated
    artifacts in lockstep); `just devtools-test`; gctorture pass; UI snapshots if errors changed.

11. **Issues**: file `gh issue` per deferred legacy-type removal (ColumnarDataFrame name,
    convert::DataFrame<T>, RowDataFrame demotion, streamer/list-builder renames), reference in PR.
