# jiff Integration

**Goal:** Expose the `jiff` datetime crate to R via a new `jiff` optional feature in `miniextendr-api`.  
**Branch:** `feat/jiff-integration`  
**Spec:** `docs/superpowers/specs/2026-04-22-jiff-integration-design.md`  
**Review:** `reviews/2026-04-22-jiff-integration.md`

---

## What was built

**Cargo wiring**
- `miniextendr-api`: `jiff = { version = "0.2", default-features = false, features = ["std", "tzdb-bundle-always"] }` + feature flag
- `rpkg/Cargo.toml`: `jiff = ["miniextendr-api/jiff"]`
- CI `clippy_all` extended with `jiff`

**Core conversions** (`miniextendr-api/src/optionals/jiff_impl.rs`)

| Rust type | R type | Variants |
|-----------|--------|----------|
| `Timestamp` | `POSIXct` (UTC) | scalar, Option, Vec, Vec\<Option\> |
| `civil::Date` | `Date` | scalar, Option, Vec, Vec\<Option\> |
| `Zoned` | `POSIXct` + `tzone` attr | scalar, Option, Vec, Vec\<Option\> |
| `SignedDuration` | `difftime` (secs) | scalar, Option, Vec, Vec\<Option\> |

Timezone policy: unknown IANA tz → `SexpError::InvalidValue` (not silent UTC). `Vec<Zoned>` uses first element's tz; warns via `log::warn!` on heterogeneous input.

**`cached_class.rs`**: `set_posixct_tz(sexp, iana: &str)` helper added alongside existing `set_posixct_utc`.

**Adapter traits** (all in `jiff_impl.rs`)
- `RSignedDuration` — as_seconds_f64, whole_seconds/minutes/hours/days, subsec_nanoseconds, is_negative, is_zero, abs
- `RSpan` — component getters (years/months/weeks/days/hours/minutes/seconds/ms/µs/ns), is_zero, is_negative, negate, abs
- `RDateTime` — year/month/day/hour/minute/second/subsec_nanosecond, to_date, to_time, in_tz
- `RTime` — hour/minute/second/subsec_nanosecond, on
- `RTimestamp` — as_second, as_millisecond, subsec_nanosecond, to_zoned_in, strftime
- `RZoned` — iana_name, year/month/day/hour/minute/second, in_tz, start_of_day, strftime
- `RDate` — year/month/day, weekday, day_of_year, first/last_of_month, tomorrow, yesterday, strftime

**ALTREP** (gated `jiff`): `JiffTimestampVec` — `Arc<Vec<Timestamp>>` materialized on-access as seconds-since-epoch f64.

**vctrs rcrd constructors** (gated `all(jiff, vctrs)`): `span_vec_to_rcrd`, `zoned_vec_to_rcrd`, `datetime_vec_to_rcrd`, `time_vec_to_rcrd` — all in `mod vctrs_support` with public re-exports.

**rpkg fixtures**: `rpkg/src/rust/jiff_adapter_tests.rs` — 30+ `#[miniextendr]` functions covering round-trips, component extraction, vec/option variants.

**Docs**: `docs/FEATURES.md` and `docs/CONVERSION_MATRIX.md` updated.

---

## Remaining work

- [ ] `Span` ExternalPtr fixture + testthat (`JiffSpan` with `#[derive(ExternalPtr)]`, RSpan accessors)
- [ ] vctrs rcrd fixtures + testthat (`jiff_span_vec_demo`, `jiff_zoned_vec_demo`, etc.; `vctrs::vec_size` + `vctrs::field` assertions)
- [ ] Fix GC protection in vctrs builders: `alloc_int_col` drops its `OwnedProtect` guard before columns are consumed by `List::from_raw_values`; `zoned_vec_to_rcrd` calls `Rf_unprotect(1)` on `tz_col` before the list-build allocation
- [ ] Unknown-IANA tz error test: `expect_error(jiff_zoned_round_trip(bad), "Mars/Olympus")`
- [ ] Leap-day Date round-trip test: `as.Date("2024-02-29")`
- [ ] DateTime/Time fixtures + ≥1 test each (nice-to-have)
- [ ] Heterogeneous-tz `Vec<Zoned>` test asserting first-tz policy (nice-to-have)
- [ ] RDate/RZoned/RTimestamp formatting + arithmetic fixtures — strftime, start_of_day, weekday, first/last_of_month, tomorrow/yesterday (nice-to-have; file issue if not done in-PR)

**Follow-up filed:** issue #304 — `JiffZonedVec` ALTREP (deferred; Zoned ALTREP has heterogeneous-tz complications).
