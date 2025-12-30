# Feature shortlist from Blessed.rs (miniextendr scope)

Date: 2025-12-30  
Goal: pick optional, CRAN-safe features that improve Rust<->R interop without bloating defaults.

## Shortlist (recommended)

### 1) `uuid` (high fit)

**Why:** common in data pipelines; maps cleanly to R `character`.  
**R mapping:** `Uuid` ⇄ `character` (scalar + vector).  
**Feature name:** `uuid`.  
**Notes:** No non-API usage. Small dependency footprint.

### 2) `time` (chosen over chrono)

**Why:** R `POSIXct`/`Date` interop is a frequent request.  
**R mapping:** `time::OffsetDateTime` or `chrono::DateTime<Utc>` ⇄ `POSIXct` (numeric + tzone); `Date` ⇄ day counts.  
**Feature name:** `time`.  
**Notes:** Use `time` to keep the dependency surface smaller.

### 3) `regex` (medium/high fit)

**Why:** avoid recompiling in tight loops; allow Rust functions to accept compiled regex.  
**R mapping:** `Regex` from `character(1)`; plus an ExternalPtr cache for reuse.  
**Feature name:** `regex`.  
**Notes:** Add an explicit compile helper (R-side) and still allow on-demand parsing.

### 4) `indexmap` (medium fit)

**Why:** preserve order when converting named lists (R preserves order).  
**R mapping:** `IndexMap<String, T>` ⇄ named `list` (auto-name if missing).  
**Feature name:** `indexmap`.  
**Notes:** Only add if there’s a clear need for ordered maps in conversions.

## Implementation plan (per feature)

### Common steps (all features)

1. Add optional dependency + feature in `miniextendr-api/Cargo.toml`.
2. Add module `*_impl.rs` with `TryFromSexp`/`IntoR` (mirrors `ndarray_impl`, `nalgebra_impl`).
3. Gate module in `lib.rs` with `#[cfg(feature = "...")]`.
4. Add doc snippet in `lib.rs` with `features = ["..."]`.
5. Add tests under `miniextendr-api/tests/` gated by feature.

### uuid plan

- Implement: `Uuid` ⇄ `character` (parse/format).  
- Vector support: `Vec<Uuid>` ⇄ `character`.  
- Error: map parse failure to `SexpError::InvalidValue`.

### time plan (selected)

- Use `time::OffsetDateTime` and `time::Date`.  
- Encode POSIXct: numeric seconds (double) + `tzone` attribute when present.  
- Decode POSIXct: accept numeric + tzone attr; handle NA.  
- Date mapping: days since 1970-01-01 (R Date origin).

### regex plan (selected)

- Implement `Regex` from `character(1)` for on-demand use.  
- Add cache: `ExternalPtr<Regex>` to avoid recompile on repeated calls.  
- Provide `compile_regex()` helper (Rust side) for R to call explicitly.

### indexmap plan (selected)

- Convert named list -> `IndexMap<String, T>` by order.  
- Convert `IndexMap<String, T>` -> named list preserving order.  
- For unnamed list, auto-name keys (e.g., "V1", "V2", ...).

## Decisions (resolved)

- Use `time` (not chrono).
- Support regex on-demand parsing plus an explicit compile/cache helper.
- Auto-name unnamed list entries when converting to `IndexMap`.

If you choose the final set, I can expand the plan into concrete code changes and tests.
