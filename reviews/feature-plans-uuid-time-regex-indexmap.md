# Implementation plans: uuid, time, regex, indexmap

Date: 2025-12-30  
Scope: miniextendr-api optional features (CRAN‑safe, opt‑in).

## Shared scaffolding (all features)

1) **Cargo features + deps**
   - Add optional deps and feature flags in `miniextendr-api/Cargo.toml`.
   - Keep them **non-default** to avoid CRAN bloat.

2) **Module layout**
   - Create feature modules mirroring existing patterns:
     - `miniextendr-api/src/uuid_impl.rs`
     - `miniextendr-api/src/time_impl.rs`
     - `miniextendr-api/src/regex_impl.rs`
     - `miniextendr-api/src/indexmap_impl.rs`
   - Gate each module in `miniextendr-api/src/lib.rs`:

     ```rust
     #[cfg(feature = "uuid")] pub mod uuid_impl;
     #[cfg(feature = "uuid")] pub use uuid_impl::Uuid;
     ```

3) **Docs**
   - Add a short doc block per feature in `lib.rs` (copy ndarray/either style).
   - Add a small example snippet (feature‑gated doc example is fine).

4) **Tests**
   - Add feature-gated tests under `miniextendr-api/tests/`:
     - `uuid.rs`, `time.rs`, `regex.rs`, `indexmap.rs`
   - Use minimal R types via `TryFromSexp`/`IntoR` and assert round‑trips.

---

## Feature: `uuid`

### Cargo

```
uuid = { version = "1", optional = true, features = ["v4"] }
```

### Conversions

**R → Rust**

- `Uuid` from `character(1)`:
  - accept `NA_character_` only via `Option<Uuid>` (return `None`)
  - `Uuid::parse_str` errors map to `SexpError::InvalidValue`
- `Vec<Uuid>` from `character(n)` (element‑wise parse)

**Rust → R**

- `Uuid` -> `character(1)` using `Uuid::to_string()`
- `Vec<Uuid>` -> `character(n)`

### Notes

- Keep `Option<Uuid>` support explicit (mirrors existing `Option<String>` behavior).

---

## Feature: `time`

### Cargo

```
time = { version = "0.3", optional = true, features = ["formatting", "parsing"] }
```

### Conversions

**R → Rust**

- `OffsetDateTime` from `POSIXct` (numeric seconds, optional `tzone` attr)
- `Date` from `Date` (days since 1970‑01‑01)
- `Option<...>` handles `NA_real_` or `NA_integer_`

**Rust → R**

- `OffsetDateTime` -> `POSIXct` numeric seconds + `tzone` attribute (default "UTC")
- `Date` -> `Date` (days since 1970‑01‑01)

### Notes

- Reuse existing `SEXP` helpers for numeric vectors and attributes.
- Decide policy for fractional seconds (preserve sub‑second as double).

---

## Feature: `regex`

### Cargo

```
regex = { version = "1", optional = true }
```

### Conversions

**R → Rust**

- `Regex` from `character(1)` (compile on demand)
- `Option<Regex>`: `NA_character_` -> `None`

**Rust → R**

- Provide `IntoR` for `Regex` as its pattern string (optional)

### Cache support

- Add helper:

  ```rust
  pub fn compile_regex(pattern: &str) -> Result<ExternalPtr<Regex>, SexpError>
  ```

- Allow `ExternalPtr<Regex>` to be passed into Rust functions for reuse.
- Document that this avoids recompilation in tight loops.

---

## Feature: `indexmap`

### Cargo

```
indexmap = { version = "2", optional = true }
```

### Conversions

**R → Rust**

- `IndexMap<String, T>` from named list.
- If list has **no names**, auto‑name as `"V1"`, `"V2"`, ... (stable order).

**Rust → R**

- `IndexMap<String, T>` -> named list, preserving insertion order.

### Notes

- Use existing `List`/`IntoList` helpers where possible.

---

## R wrapper guidance (optional)

If you want R‑side helpers:

- Expose small wrappers in `rpkg/src/rust/lib.rs`:
  - `compile_regex()` returning an external pointer
  - `uuid_from_str()` for clearer error messages
  - `time_from_posix()`/`time_to_posix()` convenience functions

These are optional; core conversions work without extra R wrappers.
