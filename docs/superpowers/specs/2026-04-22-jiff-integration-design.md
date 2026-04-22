# jiff integration design

**Status:** draft (spec review)
**Date:** 2026-04-22
**Author:** Mossa (+ agent assist)
**Branch:** `feat/jiff-integration`
**Related:** [issues #299–#303](https://github.com/A2-ai/miniextendr/issues?q=is%3Aissue+299..303) (parallel optional integrations)

## Goal

Expose the [`jiff`](https://docs.rs/jiff) datetime crate to R through `miniextendr-api` behind a `jiff` feature flag, with first-class support for:

- scalar + vector conversions for types that have a natural R analog
- `ExternalPtr` wrappers for types that don't
- adapter traits that expose jiff's calendar/rounding/formatting API from R
- vctrs-backed vector types for jiff's record-like types (`Span`, `Zoned`, `DateTime`, `Time`)
- ALTREP-backed lazy vectors for large datetime columns

jiff coexists with the existing `time` feature. No deprecation of `time` in this change. A follow-up issue may revisit `time` once jiff usage is established.

## Non-goals

- Removing or deprecating the `time` optional. If jiff proves superior in real-world use, a separate migration issue handles deprecation.
- serde-integration of jiff types via the `serde_json` path — deferred to a follow-up, gated behind a cross-feature `jiff,serde_json` combination.
- Automatic DST-aware calendar arithmetic helpers exposed as free R functions — callers can use the adapter-trait surface. Sugar wrappers are a follow-up.
- "jiff everywhere" refactors of `time_impl.rs` or `factor.rs`.

## Type mapping

| jiff type | R type | Wrapper | Notes |
|-----------|--------|---------|-------|
| `jiff::Timestamp` | `POSIXct` (UTC) | — | Seconds since epoch, nanosecond precision |
| `jiff::Zoned` | `POSIXct` (+ `tzone` attr set to IANA name) | — | **IANA tz name is round-tripped**, not erased |
| `jiff::civil::Date` | `Date` | — | Days since 1970-01-01 |
| `jiff::civil::DateTime` | — | `ExternalPtr` + vctrs `rcrd` | No base-R scalar analog |
| `jiff::civil::Time` | — | `ExternalPtr` + vctrs `rcrd` | No base-R scalar analog |
| `jiff::SignedDuration` | `difftime` (seconds) | — | Closest fit to `difftime` |
| `jiff::Span` | — | `ExternalPtr` + vctrs `rcrd` | Calendar span, preserves Y/M/D/h/m/s/ms/us/ns separately |

**NA policy** (mirrors `time_impl.rs`):

- `POSIXct`: `NaN` ↔ `None`
- `Date`: `NA_integer_` ↔ `None`
- `difftime`: `NaN` ↔ `None`
- ExternalPtr-backed vector entries (vctrs): per-entry `NA` via the `NULL`-row convention in vctrs `new_rcrd`

**TZ policy:** `Zoned` → `POSIXct` writes the `Zoned::time_zone()` IANA name into the `tzone` attribute verbatim. `POSIXct` with a `tzone` attribute → `Zoned` resolves via `jiff::tz::TimeZone::get(iana)`; unknown tz yields a `SexpError::InvalidValue` (not silent UTC fallback). Empty/missing `tzone` defaults to UTC (matches `time_impl.rs` behavior).

**Fractional seconds policy:** floor-based split into whole seconds + nanoseconds (same as `time_impl.rs`), preserving negative-timestamp correctness.

## Architecture

### Feature + dependency

`miniextendr-api/Cargo.toml`:

```toml
jiff = { version = "0.2", optional = true, default-features = false, features = ["std", "tzdb-bundle-always"] }

[features]
jiff = ["dep:jiff"]
```

`tzdb-bundle-always` is required for correct behavior on Windows (no system tzdb) and for deterministic CI. `std` enables std types. `serde` support is **not** enabled in this change — a future follow-up can gate a `jiff_serde = ["jiff", "serde_json", "jiff/serde"]` super-feature if desired.

Mirrored in `rpkg/Cargo.toml` for the R-side feature passthrough.

### Module layout

```
miniextendr-api/src/optionals/jiff_impl.rs   (new)
miniextendr-api/src/optionals.rs             (module decl + re-exports)
miniextendr-api/src/cached_class.rs          (new: set_posixct_tz(sexp, iana))
```

`jiff_impl.rs` section layout (follows `time_impl.rs`):

- region: `Timestamp ↔ POSIXct`
- region: `Option<Timestamp>` / `Vec<Timestamp>` / `Vec<Option<Timestamp>>`
- region: `Zoned ↔ POSIXct (with tzone)`
- region: `Option<Zoned>` / `Vec<Zoned>` / `Vec<Option<Zoned>>`
- region: `civil::Date ↔ Date` (+ Option + Vec + Vec<Option>)
- region: `SignedDuration ↔ difftime` (+ Option + Vec + Vec<Option>)
- region: `Span` — ExternalPtr + `RSpan` adapter trait
- region: `civil::DateTime` — ExternalPtr + `RDateTime` adapter trait
- region: `civil::Time` — ExternalPtr + `RTime` adapter trait
- region: `RTimestamp` / `RZoned` / `RDate` adapter traits
- region: `RSignedDuration` adapter trait
- region: vctrs `rcrd` constructors for `Span`, `Zoned`, `DateTime`, `Time` (gated on `feature = "vctrs"`)
- region: ALTREP adapters for `Vec<Timestamp>` / `Vec<Zoned>` lazy materialization

### Adapter traits

- `RTimestamp`: `now_utc()`, `as_seconds_f64()`, `as_millis_i64()`, `round(unit)`, `to_zoned(iana) -> Zoned`, `strftime(fmt) -> String`
- `RZoned`: `now(iana)`, `timezone() -> String`, `in_tz(iana) -> Zoned`, `start_of_day() -> Zoned`, `round(unit)`, `strftime(fmt)`
- `RDate`: `today(iana)`, `year() / month() / day()`, `weekday() -> i32`, `day_of_year() -> i32`, `add_days(n)`, `add_months(n)`, `add_years(n)`
- `RSignedDuration`: `as_seconds_f64()`, `as_millis_i64()`, `whole_days()` / `whole_hours()` / …, `is_negative()`, `abs()` — matches `RDuration` surface for drop-in parity with the `time` feature
- `RSpan`: `get_years/months/weeks/days/hours/minutes/seconds/milliseconds/microseconds/nanoseconds`, `is_zero()`, `is_negative()`, `negate()`, `abs()`, `add_to_zoned(z) -> Zoned`
- `RDateTime`, `RTime`: component accessors + formatting parity with `RZoned`/`RDate`

Adapter-trait implementations go on the bare jiff types (blanket impls for users wrapping them in `#[derive(ExternalPtr)]`), not on `TypedExternal` newtypes — matching the `RDuration for time::Duration` precedent.

### vctrs-backed vector types (feature: `jiff` + `vctrs`)

Using `crate::vctrs::new_rcrd`:

- `Vec<Span>` → rcrd with 10 i64 fields (years … nanoseconds)
- `Vec<Zoned>` → rcrd with `timestamp: REALSXP`, `tz: STRSXP`
- `Vec<DateTime>` → rcrd with date + time fields
- `Vec<Time>` → rcrd with hour/minute/second/subsec_nanos fields

Class vector: `c("jiff_<type>", "vctrs_rcrd", "vctrs_vctr")`. Per-entry NA via vctrs convention (one `NA` in any numeric field ⇒ entry is NA).

### ALTREP-backed lazy vectors (feature: `jiff` + `altrep-*` where applicable)

Wrap `Arc<Vec<Timestamp>>` / `Arc<Vec<Zoned>>` behind an ALTREP REALSXP, materializing seconds-since-epoch on demand. Pattern follows `docs/SPARSE_ITERATOR_ALTREP.md`. ALTREP callbacks are `r_unwind` guard mode (no R API in callbacks, Rust-side math only → default `rust_unwind` suffices unless we need `set_posixct_utc`/`set_posixct_tz` on lazy-materialize).

Exposed via a derive on a wrapper struct (e.g. `#[derive(AltrepReal)] #[altrep(len = "len", elt = "elt_secs", class = "JiffTimestampVec")]`) so users can return `JiffTimestampVec(Arc::new(vec![...]))` from `#[miniextendr]` functions.

## Phases

Phases the agent executes in order; each ends with a compile-green checkpoint the reviewer can diff against.

1. **Cargo wiring.** `miniextendr-api/Cargo.toml` `jiff` feature + optional dep with correct sub-features. Mirror in `rpkg/Cargo.toml`. Add `jiff` to the `clippy_all` feature union in the CI workflow(s). `just` recipes adapt if needed.
2. **Core UTC conversions.** `Timestamp ↔ POSIXct (UTC)`, `civil::Date ↔ Date`. `Option<T>`, `Vec<T>`, `Vec<Option<T>>` for each.
3. **Timezone-aware.** `Zoned ↔ POSIXct (+ tzone)` including a new `cached_class::set_posixct_tz(sexp, iana: &str)` helper. Unknown tz → error, not silent fallback.
4. **Durations.** `SignedDuration ↔ difftime` scalars + vectors + options. `RSignedDuration` adapter trait (parity with `RDuration`).
5. **Span.** `ExternalPtr<Span>` wrapper + `RSpan` adapter trait (component accessors, calendar arithmetic, negation, abs).
6. **Civil-only types.** `civil::DateTime`, `civil::Time` as `ExternalPtr` + `RDateTime`, `RTime` adapter traits.
7. **Adapter traits.** `RTimestamp`, `RZoned`, `RDate`. Blanket impls for jiff's bare types. Formatting helpers go through jiff's `strtime` module.
8. **Docs.** `docs/FEATURES.md` new `### jiff` subsection (mirror `### time` structure). `docs/CONVERSION_MATRIX.md` row additions for each mapped type.
9. **Fixtures + tests.** `rpkg/src/rust/jiff_adapter_tests.rs` with R-visible fixtures. `rpkg/tests/testthat/test-jiff.R` covering scalar/vector/option round-trips, tz attribute round-trip for `Zoned`, leap-day Date, Span arithmetic crossing DST, negative `SignedDuration`, invalid tz error.
10. **vctrs support.** `Vec<Span>`, `Vec<Zoned>`, `Vec<DateTime>`, `Vec<Time>` vctrs `rcrd` constructors gated on `#[cfg(all(feature = "jiff", feature = "vctrs"))]`. R-side tests via `vctrs::vec_size`, `vctrs::field`, and printing.
11. **ALTREP support.** Lazy `Vec<Timestamp>` / `Vec<Zoned>` ALTREP wrappers (derive-based where possible). R-side test: allocate 1M-element lazy vector, touch one element, assert no full materialization via `.Internal(inspect(x))` or (preferred) a fixture that records materialization calls.
12. **Verification.** Full `just configure && just rcmdinstall && just devtools-document && just devtools-test`. Both `clippy_default` and `clippy_all` pass `-D warnings`. `just r-cmd-check` on the built tarball is clean. `just lint` clean.

**Checkpoint after each phase:** the agent commits with `[phase N]` prefix in the commit subject so the reviewer can scope `git diff` per phase.

## Review protocol

After the agent reports completion, the reviewer:

1. Re-reads each phase's commit(s) against the phase's acceptance line.
2. Reproduces `clippy_all` and `r-cmd-check` locally (per CLAUDE.md "Reproducing CI clippy before PR").
3. Writes `reviews/2026-04-22-jiff-integration.md` with findings tagged `must-fix` / `nice-to-have` / `follow-up-issue`.
4. Groups findings into an address plan (blocker chain first), dispatches a second agent to execute.
5. Files `gh issue create` for any `follow-up-issue` findings (per feedback_concessions_as_issues).
6. Opens a PR to main with the spec/plan/review files linked in the body.

## Open decisions (locked)

1. **Zoned → POSIXct tzone:** write the actual IANA name. (✅ locked)
2. **Span vectorization:** via vctrs `rcrd`, added as phase 10. (✅ locked)
3. **`time` + `jiff` both in CI `clippy_all`:** yes, orthogonal features. (✅ locked)

## Acceptance criteria (whole-feature)

- `cargo clippy --features jiff -- -D warnings` clean
- `cargo clippy --features rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,vctrs,tinyvec,borsh,connections,nonapi,default-strict,default-coerce,default-r6,default-worker,jiff -- -D warnings` clean
- `just r-cmd-check` on the built tarball with jiff enabled — no NOTES/WARNINGS/ERRORS beyond baseline
- R-side: `identical(attr(as_posixct(zoned_now("Europe/Paris")), "tzone"), "Europe/Paris")`
- R-side: `identical(as.Date(from_jiff_date(jiff_date(2024, 2, 29))), as.Date("2024-02-29"))`
- R-side: `vctrs::vec_size(jiff_span_vec(...)) == length(input)`
- R-side: ALTREP lazy vector of length 1M allocates without materializing (fixture records the call count)

## Follow-ups (not in this PR — will be filed as issues if deferred at review time)

- jiff + `serde_json` sub-feature for serde-compatible Timestamp/Zoned roundtrips
- Deprecation assessment of `time` feature once jiff is production-tested
- Sugar: free R functions for common operations (`now()`, `today()`, `parse_datetime()`) — unambiguous intent, not behind adapter traits
- Strftime/strptime format string validation at compile time via `jiff::fmt::strtime::BrokenDownTime` — currently runtime-validated only
