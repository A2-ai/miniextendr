# typed_dataframe! macro — implementation plan (#698)

**Goal**: Add `typed_dataframe!` macro analogous to existing `typed_list!`, for compile-time-validated R `data.frame` inputs.

## Read first

1. `miniextendr-macros/src/typed_list.rs` — the existing macro to mirror.
2. `miniextendr-macros/src/lib.rs` — `typed_list!` `#[proc_macro]` export point (around 2427).
3. `miniextendr-macros/src/miniextendr_fn.rs` — `dots = typed_list!(...)` attribute sugar.
4. `miniextendr-api/src/dataframe.rs` — runtime `DataFrameView::from_sexp` / `column<T>()`.
5. CLAUDE.md and `miniextendr-dots` skill.

## API surface (from #698)

```rust
typed_dataframe! {
    /// The shape we accept for the Theoph PK dataset.
    pub TheophDf {
        subject: i32,
        weight: f64,
        dose: f64,
        time: f64,
        conc: f64,
        // Optional columns:
        // subject_label: Option<String>,
    }
}

#[miniextendr]
pub fn theoph_observations(df: TheophDf) -> i32 {
    // df.subject() -> &[i32], df.weight() -> &[f64], etc.
    df.nrow() as i32
}
```

## Storage shape

Owned struct (no explicit `'_` lifetime). The macro emits:

```rust
pub struct TheophDf {
    sexp: SEXP,
    nrow: usize,
    subject_col: SEXP,
    weight_col: SEXP,
    // Optional column → Option<SEXP>:
    // subject_label_col: Option<SEXP>,
}

impl TheophDf {
    pub fn subject(&self) -> &[i32] {
        // safe slice over self.subject_col with self.nrow length
        // data.frame invariant: column length == nrow
        unsafe { self.subject_col.as_slice::<i32>() }
    }
    pub fn weight(&self) -> &[f64] { unsafe { self.weight_col.as_slice::<f64>() } }
    pub fn subject_label(&self) -> Option<&[i32]> {
        self.subject_label_col.map(|col| unsafe { col.as_slice::<i32>() })
    }
    pub fn nrow(&self) -> usize { self.nrow }
    pub fn ncol(&self) -> usize { /* count of declared columns */ }
    pub fn as_sexp(&self) -> SEXP { self.sexp }
}
```

`as_slice<T: RNativeType>` already exists on `SexpExt` (public). Its lifetime is `'static` but actually tied to SEXP protection. Since the `#[miniextendr]` call wrapper protects the SEXP across the call, the slice is valid throughout the function body.

Returning `&[T]` from `&self` accessors gives a slice tied to `&self` — that's the right contract.

## Lifetime/MXL112 considerations

Field accessors take `&self` so the returned `&[T]` is bound to `self`. No explicit lifetime params on the type, so this is fine through `#[miniextendr]` (MXL112 only rejects explicit lifetime params, not lifetime-elided returns).

## Column → Rust type → SEXP type

| Rust type | SEXPTYPE | RNativeType impl |
|---|---|---|
| `i32` | INTSXP | i32 |
| `f64` | REALSXP | f64 |
| `u8` | RAWSXP | u8 |
| `bool` | LGLSXP | wraps via RLogical (NOT RNativeType — i32 underneath but bool needs Option<bool>); first pass: support via `Vec<bool>` accessor that converts. Better: support `RLogical` slice and add a follow-up issue for `bool` ergonomics.
| `Rcomplex` | CPLXSXP | Rcomplex |
| `String` | STRSXP | not a slice — needs special accessor returning `StrVec` or iterator. Out of scope for v1 — file follow-up.

**Scope v1**: support `i32`, `f64`, `u8`, `Rcomplex`. Document that `bool` / `String` are not yet supported as field types; file a follow-up issue for them.

## Macro shape

The macro takes a *struct-shaped* declaration (different from `typed_list!`'s expression form), more akin to a `#[derive]` target. Syntax:

```rust
typed_dataframe! {
    #[allow_extra]              // optional — default is true (allow extra columns)
    /// doc comment
    pub TheophDf {
        subject: i32,
        weight: f64,
        flag: Option<i32>,      // optional column
    }
}
```

The `#[allow_extra]`/`@exact` mode toggle: keep it simple — default allow-extra. To make exact mode, accept `@exact;` prefix like `typed_list!`. (Will document as follow-up if not in v1.)

For v1: simplest possible — required columns only via `name: type`, optional via `name: Option<type>`. Strict mode via leading `@exact;` (consistent with typed_list).

## Generated code outline

For each `name: type`:
- Required: field `name_col: SEXP`; method `pub fn name(&self) -> &[type]`.
- Optional (`Option<type>`): field `name_col: Option<SEXP>`; method `pub fn name(&self) -> Option<&[type]>`.

`TryFromSexp` impl walks every declared column, batches all errors:

```rust
impl TryFromSexp for TheophDf {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // 1. is data.frame? (uses SexpExt::is_data_frame)
        if !sexp.is_data_frame() {
            return Err(SexpError::InvalidValue(
                "expected a data.frame".to_string()
            ));
        }

        // 2. Build a NamedList for O(1) column lookup
        let view = DataFrameView::from_sexp(sexp)
            .map_err(|e| SexpError::InvalidValue(e.to_string()))?;
        let nrow = view.nrow();

        // 3. Walk each declared column, batching errors
        let mut errs: Vec<String> = Vec::new();
        let subject_col = match view.column_raw("subject") {
            None => { errs.push("missing column `subject`".into()); SEXP::nil() }
            Some(col) => {
                if col.type_of() != SEXPTYPE::INTSXP {
                    errs.push(format!("column `subject`: expected integer, got {:?}", col.type_of()));
                }
                col
            }
        };
        // ... per column ...
        let subject_label_col = match view.column_raw("subject_label") {
            None => None,
            Some(col) => {
                if col.type_of() != SEXPTYPE::INTSXP {
                    errs.push(format!("column `subject_label`: expected integer, got {:?}", col.type_of()));
                    None
                } else { Some(col) }
            }
        };

        if !errs.is_empty() {
            return Err(SexpError::InvalidValue(format!(
                "TheophDf: {}", errs.join("; ")
            )));
        }

        Ok(TheophDf {
            sexp,
            nrow,
            subject_col,
            // ...
            subject_label_col,
        })
    }
}
```

## File layout

1. **Parser**: `miniextendr-macros/src/typed_dataframe.rs` — `TypedDataframeInput`, `TypedDataframeField`, `expand_typed_dataframe`.
2. **Module declaration**: add `mod typed_dataframe;` to `miniextendr-macros/src/lib.rs` (near `mod typed_list;`).
3. **Proc-macro entry**: `#[proc_macro] pub fn typed_dataframe(...)` in `lib.rs` (next to `typed_list`).
4. **Tests**:
   - UI tests under `miniextendr-macros/tests/ui/typed_dataframe_*.rs` for invalid shapes.
   - rpkg fixture: `rpkg/src/rust/typed_dataframe_tests.rs` — declare `TheophDf`, `#[miniextendr] fn theoph_nrow(df: TheophDf) -> i32`.
   - testthat: `rpkg/tests/testthat/test-typed-dataframe.R`.
   - gctorture fixture: `gc_stress_typed_dataframe()` in `rpkg/src/rust/gc_stress_fixtures.rs` that synthesises a data.frame internally and drives `TryFromSexp`.

## Error UX

Collect-all errors per CLAUDE.md "Collect all errors in vectorized ops". Single `SexpError::InvalidValue` with semicolon-joined per-column messages.

## Out of scope (follow-up issues)

- `bool` and `String` column types — needs special accessors (Vec<Option<bool>> / StrVec).
- `dots = typed_dataframe!(...)` attribute sugar (paralleling `dots = typed_list!(...)`) — file issue.
- Compile-time row-count constraints (`#[typed_dataframe(min_rows = 1)]`) — file issue.
- `Iter` adapter for row-major access — file issue. Not in v1 because the natural shape is column-major slices.
- `@exact` mode (reject extra columns) — file issue if not implemented in v1.

## Verification

- `cargo test -p miniextendr-macros` (UI snapshots + parser tests).
- `just configure && just rcmdinstall && just force-document` for rpkg fixture.
- `just devtools-test`.
- gctorture sweep.
- `just clippy` and reproduce CI `clippy_all`.

## PR

Title: `feat(macros): typed_dataframe! for compile-time-validated data.frame inputs (#698)`
Closes #698. Reference any follow-up issues filed.
