+++
title = "jiff Integration"
weight = 50
description = "Enable with features = [\"jiff\"]. Bundles the IANA timezone database (tzdb-bundle-always) — no system tzdata required. Coexists with the time feature."
+++

Enable with `features = ["jiff"]`. Bundles the IANA timezone database (`tzdb-bundle-always`) — no system tzdata required. Coexists with the `time` feature.

## Type mapping

| Rust type | R type | Notes |
|-----------|--------|-------|
| `jiff::Timestamp` | `POSIXct` (UTC) | Nanosecond precision |
| `jiff::Zoned` | `POSIXct` + `tzone` attr | IANA name round-tripped |
| `jiff::civil::Date` | `Date` | Days since 1970-01-01 |
| `jiff::SignedDuration` | `difftime` (secs) | Signed, nanosecond precision |
| `jiff::Span` | `ExternalPtr` | Via `RSpan` adapter trait |
| `jiff::civil::DateTime` | `ExternalPtr` | Via `RDateTime` adapter trait |
| `jiff::civil::Time` | `ExternalPtr` | Via `RTime` adapter trait |

All scalar types support `Option<T>`, `Vec<T>`, and `Vec<Option<T>>` variants.

## Timezone policy

- **`Zoned` → R**: writes the IANA name from `time_zone().iana_name()` into the `tzone` attribute. Fixed-offset zones without an IANA name fall back to `"UTC"`.
- **R → `Zoned`**: unknown `tzone` → `SexpError::InvalidValue`. No silent UTC fallback — unlike the `time` feature, jiff can represent real IANA zones, so losing them is an error.
- **Empty or missing `tzone`**: treated as UTC.
- **`Vec<Zoned>` → R**: a single `tzone` attribute applies to the whole vector; the first element's timezone is used. Mixed-timezone vectors log a warning (requires `log` feature).

## Fractional seconds

Floor-based split into whole seconds + nanoseconds, matching `time_impl.rs`. Correct for negative timestamps: `-1.2s → -2s + 800_000_000ns`.

## Adapter traits

For types with no base-R scalar analog, wrap in `#[derive(ExternalPtr)]` and implement the relevant trait:

- `RSpan` — component getters (years/months/weeks/days/hours/minutes/seconds/ms/µs/ns), `is_zero`, `is_negative`, `negate`, `abs`
- `RDateTime` — year/month/day/hour/minute/second, `to_date`, `to_time`, `in_tz`
- `RTime` — hour/minute/second/subsec_nanosecond, `on`
- `RTimestamp` — `as_second`, `as_millisecond`, `subsec_nanosecond`, `to_zoned_in`, `strftime`
- `RZoned` — year/month/day/hour/minute/second, `iana_name`, `in_tz`, `start_of_day`, `strftime`
- `RDate` — year/month/day/weekday/day_of_year, `first_of_month`, `last_of_month`, `tomorrow`, `yesterday`, `strftime`
- `RSignedDuration` — `as_seconds_f64`, `as_milliseconds`, `whole_seconds/minutes/hours/days`, `subsec_nanoseconds`, `is_negative`, `is_zero`, `abs`

## ALTREP

`JiffTimestampVec` — lazy `REALSXP` backed by `Arc<Vec<Timestamp>>`. Elements are projected to seconds-since-epoch on access; no upfront conversion. Apply POSIXct class after construction.

## vctrs rcrd constructors

Requires `features = ["jiff", "vctrs"]`. Public helpers in `jiff_impl::vctrs_support`:

- `span_vec_to_rcrd(&[Span]) -> SEXP` — fields: years/months/weeks/days/hours/minutes/seconds/ms/µs/ns (all INTSXP)
- `zoned_vec_to_rcrd(&[Zoned]) -> SEXP` — fields: timestamp (REALSXP, seconds-since-epoch), tz (STRSXP)
- `datetime_vec_to_rcrd(&[DateTime]) -> SEXP` — fields: year/month/day/hour/minute/second/subsec_nanosecond
- `time_vec_to_rcrd(&[Time]) -> SEXP` — fields: hour/minute/second/subsec_nanosecond

All constructors return vctrs rcrd SEXPs with class `c("<type>", "vctrs_rcrd", "vctrs_vctr")`.

## Follow-ups

- #304 — `JiffZonedVec` ALTREP (deferred; mixed-timezone vector semantics need a design decision)
- #305 — expand adapter-trait test coverage (formatting/arithmetic methods, ALTREP laziness counter)
