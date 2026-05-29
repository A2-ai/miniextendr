# Design: a unified, intuitive DataFrame interface

Status: **speculative / not yet prioritised** — design proposal for discussion.
Author: design pass (Claude Opus 4.8).
Related: #764, #765 (`try_from_dataframe[_par]` on the `rayon-converts` branch),
#768 (`RDataFrameBuilder`, branch `with-r-dataframe-par`), the stacked
`rayon-flatten-granularity` work, #485 (struct flatten), #513 (`from_rows_par`),
#671 family (`dataframe_to_vec` / `BorrowedRows` / `Protected`).

This document contains **two parts**:

1. **Review** — a complete inventory and critique of every DataFrame-related and
   `*Builder` surface that exists today (on `main` and on the relevant unmerged
   branches).
2. **Proposal** — a single coherent interface centered on an intuitive
   `DataFrame` type plus a `FromDataFrame` / `IntoDataFrame` trait family that
   mirrors the existing `TryFromSexp` / `IntoR` pattern, with a flat migration
   checklist (no backwards-compat shims, per project policy).

The maintainer asked specifically for both, and that the review be part of this
doc. No feature code is changed by this proposal — it is design only.

---

## Part 1 — Review of the current DataFrame + `*Builder` machinery

### 1.1 The two unrelated `DataFrame` concepts

The single most confusing thing about today's surface is that the name
"DataFrame" already refers to **two different, unrelated types**, plus a third
"view" type with a third name:

| Type | File | Direction | Owns |
|---|---|---|---|
| `convert::DataFrame<T: IntoList>` | `miniextendr-api/src/convert.rs:549` | Rust → R | a `Vec<T>` of rows (Rust-side, no SEXP) |
| `serde::ColumnarDataFrame` | `miniextendr-api/src/serde/columnar.rs:100` | both | a built `data.frame` SEXP |
| `dataframe::DataFrameView` | `miniextendr-api/src/dataframe.rs:97` | R → Rust | a `NamedList` over a borrowed SEXP |

`convert::DataFrame<T>` is a row buffer that has *not yet* touched R; calling
`.into_data_frame()` transposes it. `ColumnarDataFrame` is a SEXP that *has*
been built and now offers post-assembly editing (`rename`/`drop`/`select`).
`DataFrameView` is a read wrapper for a `data.frame` arriving from R. Three
types, three names, three mental models, all called some flavour of "data
frame."

### 1.2 User-facing data builders / conversion entry points

For each: entry-point signature, direction, feature gate, error type, cost.

#### `convert::DataFrame<T>` + `IntoDataFrame` trait

- `DataFrame::from_rows(rows: Vec<T>) -> DataFrame<T>` where `T: IntoList`
  (`convert.rs:555`); also `new`/`push`/`FromIterator`/`from_serialize`.
- Trait `IntoDataFrame { fn into_data_frame(self) -> List; }` (`convert.rs:234`),
  plus a `#[doc(hidden)]` `into_named_columns()` used by the enum-flatten codegen.
- Direction: **Rust → R**. Feature gate: none (core). Error type: **none** — it
  `panic!`s on unnamed list elements (`convert.rs:654`).
- `ToDataFrame<T>(pub T)` wrapper + `ToDataFrameExt::to_data_frame()` so a bare
  type implementing `IntoDataFrame` can be returned without making it the
  default `IntoR`.
- Cost: requires `T: IntoList` (usually `#[derive(IntoList)]`), the per-row
  transpose builds and `Rf_protect`s one `List` per row (`convert.rs:638-647`) —
  O(nrow) protects, no parallelism, no column-buffer reuse.

#### `serde::ColumnarDataFrame` + the `vec_to_dataframe` family

- `ColumnarDataFrame::from_rows<T: Serialize>(rows: &[T]) -> Result<_, RSerdeError>`
  (`columnar.rs:131`); thin alias `vec_to_dataframe(rows)` (`columnar.rs:421`).
- Post-assembly builder methods on the *result*: `rename`, `strip_prefix`,
  `drop`, `select`, `prepend_column`, `with_column` (`columnar.rs:193-367`).
- Conversions: `impl IntoR` (returns the SEXP, `columnar.rs:369`),
  `impl TryFromSexp` (validates via `DataFrameView`, `columnar.rs:397`),
  `From<DataFrame<T>>` and `From<DataFrameView>`.
- Direction: **both**. Feature gate: **`serde`**. Error type: `RSerdeError`.
- Cost: requires `T: Serialize`; runs a two-phase schema-discovery +
  column-fill serializer; nested structs flatten with `parent_child` naming;
  all-`None` columns degrade to logical-NA (documented gotcha). Heavier
  machinery than `DataFrame<T>` but no `IntoList` derive needed and supports
  nested flattening / enums.

#### Streaming serde builders

- `iter_to_dataframe<T, I>(iter, nrow_hint) -> Result<ColumnarDataFrame, RSerdeError>`
  (`columnar.rs:561`).
- `serde::DataFrameBuilder<T: Serialize>` (`columnar.rs:753`) — the *third* thing
  named "builder": `new(nrow_hint)`, `with_schema(schema, nrow_hint)`,
  `grow_schema()`, `push(row) -> Result<(), RSerdeError>`, `finish() -> Result<ColumnarDataFrame, _>`.
  Three schema modes (discover-on-first-push / pre-declared / growing).
- `TypeSpec` enum (`columnar.rs:585`) — user-facing column-type hint used only by
  `with_schema`.
- `dispatch_to_dataframes<O, E, I>(iter, nrow_hint, names) -> Result<List, RSerdeError>`
  (`columnar.rs:664`) + `DispatchNames` — streams a `Result`-iterator into a
  named `list(ok=df, err=df)`.
- `vec_to_dataframe_split`, `map_to_dataframe`, `hashmap_to_dataframe`,
  `result_to_dataframe` (`columnar.rs:2900/3434/3465/3579`) — more shaping
  variants.
- Direction: **Rust → R**. Feature gate: **`serde`**. Error: `RSerdeError`.

#### `NamedDataFrameListBuilder`

- `NamedDataFrameListBuilder::new()/with_capacity(n)`, `.push(name, ColumnarDataFrame)`,
  `.build() -> List` (`columnar.rs:445`).
- Assembles a *named list of data.frames* with a single internal `ProtectScope`
  instead of per-result `OwnedProtect`. Direction: Rust → R. Gate: `serde`.
- Cost: only consumes `ColumnarDataFrame` (so it's locked to the serde path).

#### `serde` R → Rust readers (`#671` family)

- `dataframe_to_vec<T: DeserializeOwned>(sexp) -> Result<Vec<T>, RSerdeError>`
  (`dataframe_de.rs:98`) — owned, always materialises.
- `with_dataframe_rows<T, F, R>(sexp, f) -> Result<R, RSerdeError>`
  (`dataframe_de.rs:132`) — closure form.
- `BorrowedRows<'a, T> = Protected<'a, Vec<T>>` (`dataframe_de.rs:164`) +
  `dataframe_to_vec_borrowed(sexp) -> Result<BorrowedRows<'a, T>, RSerdeError>`
  (`dataframe_de.rs:192`) — keeps the source SEXP protected for the lifetime of
  the rows. Direction: **R → Rust**. Gate: `serde`. Error: `RSerdeError`.

#### `DataFrameView`

- `DataFrameView::from_sexp(sexp) -> Result<Self, DataFrameError>`
  (`dataframe.rs:114`), `impl TryFromSexp` (`dataframe.rs:261`), `impl IntoR`
  (returns the backing SEXP unchanged, `dataframe.rs:272`).
- Column access: `column<T>(name)`, `column_index<T>(idx)`, `column_raw(name)`,
  `nrow`/`ncol`/`names`/`contains_column`, `validate(&TypedListSpec)`,
  plus `NamedList::as_data_frame` / `List::as_data_frame` promotion.
- Direction: **R → Rust** (the canonical typed input wrapper). Gate: **none**.
  Error: `DataFrameError`.

#### `AsDataFrame` (coercion trait, distinct from `IntoDataFrame`)

- `as_coerce::AsDataFrame { fn as_data_frame(&self) -> Result<List, AsCoerceError>; }`
  (`as_coerce.rs:192`). Used by `#[miniextendr(as = "data.frame")]` to generate
  an S3 `as.data.frame()` method on an ExternalPtr type. Borrows `&self`.
  Error: `AsCoerceError`. **Note**: this is the trait whose *name* the maintainer
  floated as a candidate for the unified surface, but today it means something
  narrow (S3 coercion on a live R object), not "build a data.frame from Rust
  data."

#### `#[derive(DataFrameRow)]` generated API

For a `struct Row` the derive (`dataframe_derive.rs:754`) emits:

- A companion `struct RowDataFrame { field: Vec<T>, … }` (column-oriented).
- `impl From<Vec<Row>> for RowDataFrame`, `impl IntoDataFrame for RowDataFrame`,
  `impl IntoR` for both `Row` and `Vec<Row>` paths.
- `impl IntoIterator for RowDataFrame -> Row` (+ a generated iterator struct).
- `RowDataFrame::from_rows(Vec<Row>)` (`dataframe_derive.rs:1887`) and, behind
  `#[cfg(feature="rayon")]`, `RowDataFrame::from_rows_par(Vec<Row>)`
  (`dataframe_derive.rs:1758`, parallel scatter-write via `ColumnWriter`).
- `Row::to_dataframe(Vec<Self>) -> RowDataFrame` and
  `Row::from_dataframe(RowDataFrame) -> Vec<Self>` (`dataframe_derive.rs:1903/1874`).
- `markers::DataFrameRow` marker impl (`dataframe_derive.rs:1933`); compile-time
  `IntoList` assertion for plain structs.
- Full enum-payload cardinality matrix, `as_factor`/`as_list` field attrs, nested
  struct flatten (#485), HashMap/BTreeMap parallel `_keys`/`_values` via `unzip`.
- On the **`rayon-converts` branch (#764/#765)** the derive additionally emits
  `Row::try_from_dataframe(sexp) -> Result<Vec<Self>, String>` and
  `try_from_dataframe_par` (`dataframe_derive.rs:2025/2048` on that branch) — the
  R → Rust reverse of `from_rows`. **Error type here is `String`, not any of the
  three structured error enums.** Not on `main` yet.

Cost: the most ergonomic path, but it requires a *companion type* to exist
(`RowDataFrame`), and the verbs are spread across both the row type and the
companion type (`Row::to_dataframe` vs `RowDataFrame::from_rows` vs
`RowDataFrame::from_rows_par`), which is hard to discover.

#### `RDataFrameBuilder` (#768, unmerged, branch `with-r-dataframe-par`)

- `RDataFrameBuilder::new(nrow)` → `.column::<T>(name, |chunk, offset| …)` /
  `.column_str(name, |i| Option<String>)` → `.build() -> SEXP`
  (`rayon_bridge.rs:743` on that branch). Feature gate: **`rayon`**. Error: none
  (asserts on bad nrow/length).
- Direction: **Rust → R**. The heterogeneous-column analogue of `with_r_matrix`:
  each column buffer is R memory filled in parallel; the closure-per-column model
  is *fundamentally different* from every other path (which take typed Rust
  data), so it doesn't currently compose with `ColumnarDataFrame` /
  `DataFrame<T>` at all.
- The stacked **`rayon-flatten-granularity`** branch is intended to add a
  flattened `(column, chunk)` work-granularity variant on top of #768; at the
  time of writing it carries no commits over #768 (work-in-progress), so the
  flatten granularity is a planned refinement of `RDataFrameBuilder::build`'s
  parallel dispatch, not yet a distinct public API.

### 1.3 Internal codegen builders (NOT user-facing data builders)

These live in `miniextendr-macros` and shape generated R/C code. The maintainer
cares mainly about the user-facing data builders above; these are inventoried
only to be explicitly set aside. They should **not** be touched by this redesign.

- `DotCallBuilder` (`r_wrapper_builder.rs:~390`) — emits `.Call(C_…, .call = match.call(), …)`.
- `RArgumentBuilder`, `ListBuilder`, `MethodReturnBuilder` (macro-side R/return codegen).
- `CWrapperContext` (`c_wrapper_builder.rs`) — C-wrapper synthesis.
- `ClassDocBuilder`, `MethodContext`/`r_class_formatter.rs` — doc + class formatting.

There is also a runtime `crate::list::List` builder surface (`from_pairs`,
`from_raw_pairs`) that underlies the data builders but is a general list
primitive, not a data-frame API.

### 1.4 Fragmentation analysis

**How many ways does a user build / read a data.frame today?**

Rust → R (build): `DataFrame<T>::from_rows`, `ToDataFrame`, `impl IntoDataFrame`,
`ColumnarDataFrame::from_rows` / `vec_to_dataframe`, `iter_to_dataframe`,
`serde::DataFrameBuilder`, `dispatch_to_dataframes`, `vec_to_dataframe_split`,
`map_to_dataframe`, `hashmap_to_dataframe`, `result_to_dataframe`,
`NamedDataFrameListBuilder`, `#[derive(DataFrameRow)]` (`from_rows` /
`from_rows_par` / `to_dataframe`), `RDataFrameBuilder` (#768), and the S3
`AsDataFrame` coercion. **~16 entry points.**

R → Rust (read): `DataFrameView` (typed column pulls), `ColumnarDataFrame`
(`TryFromSexp`), `dataframe_to_vec`, `with_dataframe_rows`,
`dataframe_to_vec_borrowed` / `BorrowedRows`, and (on a branch)
`Row::try_from_dataframe[_par]`. **~6 entry points.**

**Inconsistencies / sharp edges:**

1. **Naming.** `DataFrame<T>` (rows, no SEXP) vs `ColumnarDataFrame` (built SEXP)
   vs `DataFrameView` (read) — three names for "data frame." `IntoDataFrame`
   (consuming build) vs `AsDataFrame` (borrowing S3 coercion) are easy to
   confuse; the docs already need a "Comparison with `AsDataFrame`" sidebar
   (`convert.rs:225`). Three different things are called *builder*
   (`DataFrame<T>`, `serde::DataFrameBuilder<T>`, `RDataFrameBuilder`).
2. **Error types.** `DataFrameError`, `RSerdeError`, `AsCoerceError`, and bare
   `String` (the `try_from_dataframe` branch) — four different failure types
   for "data frame conversion went wrong," none convertible into the others.
   `DataFrame<T>::into_data_frame` doesn't even return a `Result` — it `panic!`s.
3. **NA handling differs by path.** `ColumnarDataFrame` documents the all-`None`
   → logical-NA degrade and offers `TypeSpec::Optional` to opt out; `DataFrame<T>`
   relies entirely on each field's `IntoR`; `scatter_column` (enum flatten)
   hand-rolls NA fills per SEXP type. No single NA contract.
4. **Ownership / borrowing.** `DataFrameView` borrows the SEXP;
   `dataframe_to_vec` materialises owned; `BorrowedRows`/`Protected` keeps the
   SEXP rooted for borrowed rows; `ColumnarDataFrame` owns a built SEXP. The
   borrowing story is correct but scattered across three different types.
5. **Feature gating is uneven.** `DataFrame<T>` / `DataFrameView` are core;
   everything columnar/streaming is behind `serde`; the parallel builders are
   behind `rayon`; `from_rows_par` is on the derive behind `rayon`;
   `RDataFrameBuilder` is `rayon`-only and unmerged. A user who wants "parallel
   build of a typed-Rust-data frame" must pick between `from_rows_par` (derive,
   needs companion type) and `RDataFrameBuilder` (closures, needs serde-free
   manual column fill) — two unrelated APIs for the same intent.
6. **The fast path is not transparent.** Choosing sequential vs parallel is a
   *different method name* (`from_rows` vs `from_rows_par`) or a *different type*
   (`RDataFrameBuilder`). There is no single call where "rayon when the feature
   is on" happens automatically.
7. **Verbs are split across types.** To round-trip with the derive you call
   `Row::to_dataframe`, then `RowDataFrame::from_rows_par`, then
   `Row::from_dataframe` — three receivers for one round trip.

---

## Part 2 — Proposal: a unified `DataFrame` + `*DataFrame` trait family

### 2.1 Design goals

- **One name, one mental model.** A single owned `DataFrame` type that *is* a
  built/buildable R `data.frame`, paired with a trait family that reads exactly
  like the existing `IntoR` / `TryFromSexp` pair.
- **Mirror the conversion traits users already know.** miniextendr already has
  `IntoR` (Rust → R) and `TryFromSexp` (R → Rust). The DataFrame surface should
  be the same shape, specialised to the data-frame SEXP.
- **No capability loss.** Row-oriented, column-oriented, serde-driven, streaming,
  the rayon fast paths, and post-assembly editing must all remain reachable.
- **Transparent fast paths.** Sequential vs parallel selected by feature flag and
  a hint, not by a different function name or type.
- **One error type** for data-frame conversion.
- **No backwards compat** (project policy): propose the end state and a flat
  removal/rename checklist; do not shim.

### 2.2 Recommended end state

#### The owned type

```rust
/// An owned, validated R `data.frame`. The single data-frame type.
/// Wraps a built VECSXP with data.frame class + row.names; offers
/// column access (read) and post-assembly editing (rename/drop/select/…).
pub struct DataFrame { /* protected SEXP + cached nrow/ncol + name index */ }
```

This **replaces all three current types**: it absorbs `ColumnarDataFrame`'s
built-SEXP role and post-assembly editing, `DataFrameView`'s typed-column read
API (`column::<T>`, `nrow`, `names`, `validate`), and is the canonical return
type for everything that today returns `ColumnarDataFrame` or
`convert::DataFrame<T>`. `convert::DataFrame<T>` (the row buffer) is **deleted**;
its role is taken over by the `IntoDataFrame for Vec<Row>` impl below — you keep
your `Vec<Row>` as the row buffer and convert when you cross into R.

`DataFrame` implements `IntoR` (return it directly) and `TryFromSexp` (accept it
as a `#[miniextendr]` argument), so it slots into the existing function-codegen
with zero special-casing.

#### The trait family (mirrors `IntoR` / `TryFromSexp`)

```rust
/// Rust data → R data.frame. The data-frame analogue of `IntoR`.
pub trait IntoDataFrame {
    fn into_dataframe(self) -> Result<DataFrame, DataFrameError>;
}

/// R data.frame → Rust data. The data-frame analogue of `TryFromSexp`.
pub trait FromDataFrame: Sized {
    fn from_dataframe(df: &DataFrame) -> Result<Self, DataFrameError>;
}
```

Plus an **ergonomic extension trait** so the verbs read naturally on the data,
not on a companion type:

```rust
pub trait IntoDataFrameExt: IntoDataFrame {
    /// `rows.into_dataframe()` — always available.
    fn into_dataframe(self) -> Result<DataFrame, DataFrameError>;
    /// `rows.into_dataframe_par()` — present only with `feature = "rayon"`,
    /// same result, parallel fill. Transparent fast path.
    #[cfg(feature = "rayon")]
    fn into_dataframe_par(self) -> Result<DataFrame, DataFrameError>;
}
```

Blanket impls:

- `impl<T: IntoList> IntoDataFrame for Vec<T>` — the row-oriented path (replaces
  `DataFrame<T>::from_rows` and the derive's `from_rows`). Returns a `Result`
  instead of panicking.
- `impl<T: Serialize> IntoDataFrame for AsSerialize<Vec<T>>` (or a thin
  `Rows::serde(vec)` newtype) — the serde columnar path (replaces
  `vec_to_dataframe`), gated `serde`.
- `impl<T: DeserializeOwned> FromDataFrame for Vec<T>` via serde (replaces
  `dataframe_to_vec`), gated `serde`.

#### How the existing paths fold in

| Today | End state |
|---|---|
| `DataFrame<T>::from_rows(rows)` | `rows.into_dataframe()?` (blanket `IntoDataFrame for Vec<T>`) |
| `DataFrame<T>::from_rows_par` / derive `from_rows_par` | `rows.into_dataframe_par()?` (transparent, `rayon`-gated) |
| `ToDataFrame<T>` wrapper + `ToDataFrameExt` | deleted — return `DataFrame` or `impl IntoDataFrame` directly |
| `ColumnarDataFrame::from_rows` / `vec_to_dataframe` | `rows.into_dataframe()?` (serde impl) — same `DataFrame` result |
| `ColumnarDataFrame` editing (`rename`/`drop`/`select`/`with_column`) | inherent methods on `DataFrame` |
| `DataFrameView` (read) | `DataFrame` inherent read methods (`column::<T>`, `nrow`, `validate`) |
| `dataframe_to_vec` / `with_dataframe_rows` | `Vec::<T>::from_dataframe(&df)?` / `df.rows::<T>()?` |
| `BorrowedRows` / `dataframe_to_vec_borrowed` | `df.borrow_rows::<T>()` returning `Protected<'_, Vec<T>>` (kept — it's the borrowing primitive) |
| `iter_to_dataframe` / `serde::DataFrameBuilder` | kept as the **streaming** builder, renamed `DataFrameStreamer` and producing a `DataFrame` (see below) |
| `dispatch_to_dataframes` / `*_split` / `map_to_dataframe` / `result_to_dataframe` | kept as **named-shape helpers** producing `List`/`DataFrame`; documented as shaping utilities, not core conversion |
| `NamedDataFrameListBuilder` | renamed `DataFrameListBuilder`, pushes `DataFrame` |
| `RDataFrameBuilder` (#768) | kept as the **closure-fill builder**, `DataFrame::builder(nrow).column(…).build()`, returning `DataFrame` not raw `SEXP` |
| `#[derive(DataFrameRow)]` companion + verbs | derive emits `impl IntoDataFrame`/`FromDataFrame` for the row type; the companion `RowDataFrame` becomes optional/internal |

The result: **one type (`DataFrame`)**, **two core traits
(`IntoDataFrame`/`FromDataFrame`)**, **one error (`DataFrameError`)**, and three
*named, clearly-scoped* assembly tools that all yield a `DataFrame`:

- `DataFrame::builder(nrow)` — closure-per-column parallel fill (ex-`RDataFrameBuilder`).
- `DataFrameStreamer<T>` — incremental row push with schema modes (ex-`serde::DataFrameBuilder`).
- `DataFrameListBuilder` — assemble a named list of `DataFrame`s (ex-`NamedDataFrameListBuilder`).

#### How `#[derive(DataFrameRow)]` plugs in

The derive stops being the *only* ergonomic path and instead becomes the thing
that wires a user struct into the trait family:

- It emits `impl IntoDataFrame for YourRow` and `impl FromDataFrame for Vec<YourRow>`
  (and the `_par` variants under `rayon`) directly, so users call
  `rows.into_dataframe()?` / `Vec::<Row>::from_dataframe(&df)?` — the *same verbs*
  as for any other `IntoDataFrame` type. No `Row::to_dataframe` /
  `RowDataFrame::from_rows` / `Row::from_dataframe` spread across two receivers.
- The companion `RowDataFrame` struct (column vectors) can remain as a
  `#[doc(hidden)]` implementation detail of the parallel scatter-write, or be
  exposed opt-in (`#[dataframe(columns)]`) for users who genuinely want
  column-oriented Rust access. It is no longer the public face.
- The merged-on-#764 `try_from_dataframe[_par] -> Result<Vec, String>` is
  re-expressed as `FromDataFrame for Vec<Row>` returning `DataFrameError`
  (fixes the bare-`String` error inconsistency before it lands on `main`).

This keeps the full enum-payload cardinality matrix, `as_factor`/`as_list`,
nested-struct flatten, and map `_keys`/`_values` expansion exactly as they are —
they are *codegen* details behind `IntoDataFrame::into_dataframe`, not part of
the public verb surface.

### 2.3 Ownership / borrowing, NA, error, feature gating

- **Owned vs borrowed.** `DataFrame` owns a protected built SEXP. For zero-copy
  *reads*, `df.column_slice::<f64>(name) -> Option<&[f64]>` exposes the R buffer
  directly (native types only); `df.borrow_rows::<T>()` returns
  `Protected<'_, Vec<T>>` (today's `BorrowedRows`) for serde row borrows. The
  `Protected` primitive (`gc_protect.rs:957`) is the single mechanism, reused —
  not a separate `DataFrameView` type.
- **NA handling — one contract.** All paths go through `IntoDataFrame`, so the
  all-`None`-column behaviour and the `TypeSpec::Optional` hint live in *one*
  place (the column-buffer assembler). `scatter_column`'s per-SEXP NA fills
  become the shared implementation of "fill NA for column type X," used by both
  the enum-flatten codegen and the streamer.
- **One error type.** `DataFrameError` is extended to carry the cases currently
  split across `RSerdeError` (schema/serialize failures) and the bare `String`.
  `From<RSerdeError> for DataFrameError` bridges the serde internals; the public
  surface only ever surfaces `DataFrameError`. `IntoDataFrame::into_dataframe`
  returns `Result` (no more `panic!` on unnamed columns).
- **Feature gating.** Core traits + `DataFrame` + `Vec<T: IntoList>` path:
  **no feature** (always available). Serde row paths: `serde`. Parallel variants
  (`into_dataframe_par`, `DataFrame::builder`): `rayon`. The *names* are stable
  across feature sets — a missing feature removes the `_par` method, not the
  whole API, so user code degrades to the sequential call by deleting `_par`.

### 2.4 Migration checklist (flat, no phases, no shims)

1. Add `pub struct DataFrame` (owned built SEXP) with read methods absorbed from
   `DataFrameView` and editing methods absorbed from `ColumnarDataFrame`.
   `impl IntoR + TryFromSexp` for it.
2. Define `IntoDataFrame { fn into_dataframe(self) -> Result<DataFrame, DataFrameError> }`
   and `FromDataFrame { fn from_dataframe(df: &DataFrame) -> Result<Self, _> }`;
   add `IntoDataFrameExt::into_dataframe_par` (`rayon`).
3. Blanket `impl IntoDataFrame for Vec<T: IntoList>` (returns `Result`, no panic).
4. Move the serde column assembler under `impl IntoDataFrame for AsSerialize<Vec<T>>`
   and `impl FromDataFrame for Vec<T: DeserializeOwned>`.
5. **Delete** `convert::DataFrame<T>`, `ToDataFrame<T>`, `ToDataFrameExt`, and the
   old `convert::IntoDataFrame` trait (it had a consuming `-> List` signature).
6. **Delete** `dataframe::DataFrameView` and `serde::ColumnarDataFrame`; rename
   the column-buffer internals into `DataFrame`'s impl.
7. Rename `serde::DataFrameBuilder<T>` → `DataFrameStreamer<T>` (produces
   `DataFrame`); `NamedDataFrameListBuilder` → `DataFrameListBuilder` (pushes
   `DataFrame`); land `RDataFrameBuilder` (#768) as `DataFrame::builder(nrow)`
   returning `DataFrame` (not raw `SEXP`).
8. Re-express `dataframe_to_vec` / `with_dataframe_rows` /
   `dataframe_to_vec_borrowed` as `Vec::<T>::from_dataframe`, `df.with_rows`,
   `df.borrow_rows` (the last keeps `Protected`).
9. Fold `RSerdeError` data-frame variants and the bare-`String`
   `try_from_dataframe` error into `DataFrameError`; add `From<RSerdeError>`.
10. Update `#[derive(DataFrameRow)]` to emit `IntoDataFrame`/`FromDataFrame` (and
    `_par`) for the row type; demote the companion `RowDataFrame` to
    `#[doc(hidden)]` / opt-in.
11. Decide the fate of `AsDataFrame` (S3 coercion): either keep it distinct
    (it's borrow-based S3 coercion, a different concept) or rename it to avoid
    collision with `IntoDataFrame` (see open questions).
12. Update `docs/DATAFRAME.md`, `docs/SERDE_R.md`, `docs/RAYON.md`,
    `docs/CONVERSION_MATRIX.md`; rewrite the rpkg fixtures
    (`dataframe_*`, `columnar_*`, `typed_dataframe_*`, `dataframe_rayon_tests.rs`)
    to the new surface; add a `gc_stress_*` fixture for the new `DataFrame::builder`
    SEXP-storage path (per #430 convention); refresh wrappers/NAMESPACE/man and
    the wasm registry.

### 2.5 Trade-offs and alternatives considered

- **Alt A — keep three types, just rename for clarity** (`RowFrame` / `RDataFrame`
  / `DataFrameRef`). Cheaper, but preserves the three-mental-models problem and
  the split error types; rejected.
- **Alt B — make `DataFrame` generic `DataFrame<Layout>`** to encode
  row/column/borrowed in the type. More type-safe but pushes a layout type
  parameter into every `#[miniextendr]` signature (which can't carry generics,
  MXL112) and into docs; rejected as over-engineered.
- **Alt C — `AsDataFrame` as the central trait** (the maintainer's floated name).
  Rejected as the *primary* name because `as_*` already denotes borrowing S3
  coercion across the `as_coerce` module (`AsList`, `AsCharacter`, …); reusing it
  for consuming conversion would muddy a consistent convention. `IntoDataFrame` /
  `FromDataFrame` mirror `Into`/`From` and `IntoR`/`TryFromSexp` exactly, which is
  the strongest intuition.
- **Recommended — Alt (this doc):** one owned `DataFrame`, `IntoDataFrame` /
  `FromDataFrame` mirroring the existing conversion traits, transparent `_par`
  fast paths, one error type. Rationale: it collapses ~22 entry points to a small
  verb set users can guess from `IntoR`/`TryFromSexp`, removes the
  three-names-for-one-thing confusion, unifies NA + error handling, and — because
  there is no backwards-compat constraint — can be done as a clean replacement
  rather than a parallel API.

Prior art (brief, no comparison/benchmarking per maintainer preference): the
`Into`/`From` split is idiomatic Rust; extendr exposes a single `Dataframe`
wrapper type; polars/arrow have one `DataFrame`/`RecordBatch` owned type with
column accessors. The proposal stays closest to miniextendr's own
`IntoR`/`TryFromSexp` precedent rather than importing another framework's model.

### 2.6 Open questions for the maintainer

1. **Trait names:** `IntoDataFrame` / `FromDataFrame` (recommended, mirrors
   `IntoR`/`TryFromSexp`) vs the floated `AsDataFrame` family? The latter
   collides with the existing `as_coerce::AsDataFrame` S3 trait.
2. **Fate of `as_coerce::AsDataFrame`:** keep it as a distinct S3-coercion trait
   (borrow `&self`, used by `#[miniextendr(as = "data.frame")]`), or rename it
   (e.g. `CoerceDataFrame`) so the consuming trait can own the obvious name?
3. **Companion type visibility:** fully hide `RowDataFrame` (`#[doc(hidden)]`),
   or keep it opt-in via `#[dataframe(columns)]` for users who want
   column-oriented Rust access (struct-of-`Vec`s)?
4. **`_par` transparency:** separate `into_dataframe_par()` method (recommended,
   explicit) vs an automatic threshold inside `into_dataframe()` when `rayon` is
   on (more "transparent" but hides a parallelism decision)?
5. **Zero-copy reads:** how far to take `column_slice::<T>()` for native types —
   only `f64`/`i32`/`RLogical`/`u8`/`Rcomplex`, or also expose borrowed string
   iteration?
6. **Sequencing vs in-flight branches:** #764/#765 (`try_from_dataframe`) and
   #768 (`RDataFrameBuilder`) are mid-flight. Land them first and migrate, or
   redirect them onto the new surface before merge (preferred, since #764's
   bare-`String` error would otherwise ship and then need re-doing)?
7. **Streaming + parallel interaction:** should `DataFrameStreamer` gain a
   parallel finish, or is parallelism only meaningful for the
   already-materialised `Vec` and closure-fill paths?
