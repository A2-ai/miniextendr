# jiff Integration Implementation Plan

> **For agentic workers:** This plan assumes you are the sole implementation agent executing all phases end-to-end on branch `feat/jiff-integration` (worktree off `origin/main`). Do not branch further. Commit after every phase with `[phase N] <summary>`. Keep the branch green: each phase ends with compile-clean `cargo check -p miniextendr-api --features jiff`.

**Goal:** Expose the `jiff` datetime crate to R via a new `jiff` optional feature in `miniextendr-api`, mirroring the `time_impl.rs` pattern with added vctrs-backed vector types and ALTREP-backed lazy vectors.

**Architecture:** New `miniextendr-api/src/optionals/jiff_impl.rs` providing `TryFromSexp`/`IntoR` for `Timestamp`/`civil::Date`/`SignedDuration`/`Zoned`, `ExternalPtr`-wrapped `Span`/`civil::DateTime`/`civil::Time`, adapter traits, vctrs `rcrd` constructors gated on `#[cfg(all(feature = "jiff", feature = "vctrs"))]`, and ALTREP derive-based lazy vectors. jiff coexists with the existing `time` feature — no deprecation.

**Tech stack:** jiff 0.2 (default-features off, `std` + `tzdb-bundle-always`), existing `miniextendr-api` FFI + ExternalPtr + vctrs + altrep infrastructure, rpkg test fixtures, testthat.

**Spec:** `docs/superpowers/specs/2026-04-22-jiff-integration-design.md`

---

## Ground rules

- **Cross-check jiff's 0.2 API** via `cargo doc --open -p jiff` or `https://docs.rs/jiff/0.2` before every phase — the code snippets in this plan are close-to-correct but not a substitute for real API confirmation. If a method name drifted (e.g. `as_second` vs `to_second`), fix it and move on — do not stall.
- **Sandbox:** every compiling command (`just check`, `cargo build`, `just rcmdinstall`, `just r-cmd-check`) must run with `dangerouslyDisableSandbox: true`.
- **Output capture:** long-running commands redirect to `/tmp/<name>.log` and use the Read tool on the log — never `tail`/`head` the log (CLAUDE.md).
- **After Rust changes that affect R wrapper output** (any `#[miniextendr]` add/remove, any trait impl adapter added to rpkg fixtures): run `just configure && just rcmdinstall && just devtools-document` and commit the regenerated `rpkg/R/miniextendr-wrappers.R` + `NAMESPACE` + `man/*.Rd` together with the Rust change.
- **Flag concessions as issues:** anything deferred gets `gh issue create` referenced in the final PR body. No silent scope cuts.
- **Warnings:** fix every warning you see. No "known issues" left for later.
- **Never `mod.rs`:** use `foo.rs` + `foo/` directory (CLAUDE.md).

---

## Phase 1: Cargo wiring + CI feature union

**Goal:** `cargo check -p miniextendr-api --features jiff` compiles an empty-but-present `jiff_impl` module. Nothing R-facing yet.

**Files:**
- Modify: `miniextendr-api/Cargo.toml`
- Modify: `miniextendr-api/src/optionals.rs`
- Create: `miniextendr-api/src/optionals/jiff_impl.rs`
- Modify: `rpkg/Cargo.toml` (mirror feature passthrough)
- Modify: `.github/workflows/*.yml` (add `jiff` to the `clippy_all` feature union)

### Steps

- [ ] **1.1 Add jiff dep + feature to `miniextendr-api/Cargo.toml`.**

Locate the `[dependencies]` block with `time = { ... }` and add immediately below:

```toml
jiff = { version = "0.2", optional = true, default-features = false, features = ["std", "tzdb-bundle-always"] }
```

Locate `[features]` and add under the string/date grouping (alphabetical, next to `time = ["dep:time"]`):

```toml
## Enable jiff integration for datetime types with first-class IANA timezone support.
## Provides conversions for Timestamp/Zoned/civil::Date/SignedDuration, ExternalPtr
## wrappers for Span/DateTime/Time, and optional vctrs + ALTREP layers.
jiff = ["dep:jiff"]
```

- [ ] **1.2 Create an empty jiff_impl module with the header doc-comment.**

Write `miniextendr-api/src/optionals/jiff_impl.rs`:

```rust
//! Integration with the `jiff` crate.
//!
//! Provides conversions between R date/time types and `jiff` types.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `POSIXct` (UTC) | `jiff::Timestamp` | Seconds since epoch, ns precision |
//! | `POSIXct` (+ `tzone`) | `jiff::Zoned` | IANA timezone round-tripped |
//! | `Date` | `jiff::civil::Date` | Days since 1970-01-01 |
//! | `difftime` | `jiff::SignedDuration` | Seconds (f64) |
//! | — | `jiff::Span` | ExternalPtr + `RSpan` adapter trait |
//! | — | `jiff::civil::DateTime` | ExternalPtr + `RDateTime` adapter trait |
//! | — | `jiff::civil::Time` | ExternalPtr + `RTime` adapter trait |
//!
//! Enable with `features = ["jiff"]`. Coexists with the `time` feature.
//!
//! # Timezone policy
//!
//! - `Zoned` → POSIXct writes the IANA name from `Zoned::time_zone()` into the `tzone` attr.
//! - POSIXct with unknown `tzone` yields an error (NOT silent UTC fallback, unlike the
//!   `time` feature — `jiff` can represent real IANA zones so we refuse to lose them).
//! - POSIXct with no `tzone` or empty `tzone` is treated as UTC.
//!
//! # Fractional seconds
//!
//! Floor-based split into whole seconds + nanoseconds — matches `time_impl.rs`. Correct
//! for negative timestamps (-1.2s → -2s + 800_000_000ns).

pub use jiff::{SignedDuration, Span, Timestamp, Zoned};
pub use jiff::civil::{Date, DateTime, Time};
```

- [ ] **1.3 Wire `jiff_impl` into `optionals.rs`.**

Open `miniextendr-api/src/optionals.rs`, find the `// region: Date/Time` block (contains the `time` feature), and add immediately after the `time` re-export block:

```rust
/// Date and time conversions via the `jiff` crate — first-class IANA timezone support.
///
/// Coexists with the `time` feature. Enable with `features = ["jiff"]`.
#[cfg(feature = "jiff")]
pub mod jiff_impl;
#[cfg(feature = "jiff")]
pub use jiff_impl::{Date as JiffDate, DateTime as JiffDateTime, SignedDuration, Span, Time as JiffTime, Timestamp, Zoned};
```

Also add a row to the feature-table docstring at the top of `optionals.rs`:

```
//! | `jiff` | `jiff_impl` | Date/time conversions with IANA tz (parallel to `time`) |
```

Place it directly under the `time` row.

- [ ] **1.4 Mirror the feature in `rpkg/Cargo.toml`.**

Find the `[features]` table in `rpkg/Cargo.toml` where `time = ["miniextendr-api/time"]` lives, add below:

```toml
jiff = ["miniextendr-api/jiff"]
```

If there's a "default" feature list or a bundle feature (e.g. `bundle = [...]`), **do not** add jiff to it — users opt in explicitly. Check by searching the file for `"time"` — match the pattern only where `time` appears in consumer bundles and leave jiff out of the same bundles for this phase (follow-up decision).

- [ ] **1.5 Add jiff to the CI `clippy_all` feature union.**

```bash
rg -n 'clippy_all|rayon,rand' .github/workflows/ 2>/dev/null
```

For each matching workflow, append `,jiff` at the end of the feature list exactly once (the list is the long alphabetical-ish comma-joined string per CLAUDE.md). Keep the leading `--features` flag and quoting intact.

- [ ] **1.6 Compile checkpoint.**

```bash
dangerouslyDisableSandbox just check
```

Expected: green. If red, the jiff features list is likely wrong — `tzdb-bundle-always` is the most-commonly-typoed.

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase1.log
```

Read the log. Expect no warnings.

- [ ] **1.7 Commit.**

```bash
git add miniextendr-api/Cargo.toml miniextendr-api/src/optionals.rs miniextendr-api/src/optionals/jiff_impl.rs rpkg/Cargo.toml .github/workflows/
git commit -m "[phase 1] jiff: cargo wiring + empty optional module

Adds the feature flag, optional dep, and empty jiff_impl module so
downstream phases can fill in trait impls incrementally. CI clippy_all
now includes jiff alongside time.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 2: Core UTC conversions — `Timestamp` + `civil::Date`

**Goal:** `jiff::Timestamp ↔ POSIXct(UTC)` and `jiff::civil::Date ↔ Date`, plus `Option<T>`/`Vec<T>`/`Vec<Option<T>>` variants. No tz handling yet (phase 3).

**Files:** modify `miniextendr-api/src/optionals/jiff_impl.rs`.

**Reference:** `miniextendr-api/src/optionals/time_impl.rs` lines 64–607 — copy the shape precisely, swapping `OffsetDateTime` → `Timestamp` and `time::Date` → `jiff::civil::Date`.

### Steps

- [ ] **2.1 Add use-imports to `jiff_impl.rs` (after the `pub use` block).**

```rust
use crate::cached_class::set_posixct_utc;
use crate::ffi::{INTEGER, REAL, Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpNaError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;
```

- [ ] **2.2 Implement `Timestamp ↔ POSIXct(UTC)`.**

Add under a `// region: Timestamp <-> POSIXct (UTC)` section. Core math is identical to `time_impl.rs` but uses jiff's API:

```rust
impl TryFromSexp for Timestamp {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::REALSXP, actual }.into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar POSIXct, got length {}", sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Err(SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::REALSXP }));
        }
        let whole = secs.floor() as i64;
        let fract = secs - secs.floor();
        let nanos = (fract * 1_000_000_000.0) as i32;
        Timestamp::new(whole, nanos)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Timestamp out of range: {e}")))
    }
}

impl IntoR for Timestamp {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> { Ok(self.into_sexp()) }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> { self.try_into_sexp() }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            let secs = self.as_second() as f64 + (self.subsec_nanosecond() as f64 / 1_000_000_000.0);
            *REAL(vec) = secs;
            set_posixct_utc(vec);
            Rf_unprotect(1);
            vec
        }
    }
}
```

**Note:** `Timestamp::new(secs, nanos)` may be spelled `Timestamp::new(secs: i64, nanos: i32) -> Result<Timestamp, Error>` in 0.2. Cross-check. `as_second()` is confirmed; `subsec_nanosecond()` exists (singular). If the crate renamed either, substitute.

- [ ] **2.3 Implement `Option<Timestamp>`.**

Copy the shape from `time_impl.rs:139–203` (`Option<OffsetDateTime>`), substituting:
- `NILSXP` input → `Ok(None)`
- `NaN` input → `Ok(None)`
- On `IntoR for Option<Timestamp>`: `None` → write `f64::NAN` + `set_posixct_utc`; `Some(ts)` → delegate to `ts.into_sexp()`.

Full code follows the time_impl.rs precedent line-for-line.

- [ ] **2.4 Implement `Vec<Timestamp>` and `Vec<Option<Timestamp>>`.**

Mirror `time_impl.rs:207–345`. Key detail: `set_posixct_utc` is called **once** on the output vector, after the loop. NA handling: `NaN` per element → `None`.

- [ ] **2.5 Implement `civil::Date ↔ R Date`.**

R's Date is an integer (days since 1970-01-01, or a double for fractional; we target integer for round-trip). Core conversion:

```rust
use jiff::civil::{date, Date};

fn unix_epoch_date() -> Date { date(1970, 1, 1) }

impl TryFromSexp for Date {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        // Accept both INTSXP and REALSXP — R's Date is sometimes stored as double.
        let actual = sexp.type_of();
        let days: f64 = match actual {
            SEXPTYPE::INTSXP => {
                if sexp.len() != 1 { return Err(SexpError::InvalidValue(format!("expected scalar Date, got length {}", sexp.len()))); }
                let v = unsafe { *INTEGER(sexp) };
                if v == i32::MIN { return Err(SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::INTSXP })); }
                v as f64
            }
            SEXPTYPE::REALSXP => {
                if sexp.len() != 1 { return Err(SexpError::InvalidValue(format!("expected scalar Date, got length {}", sexp.len()))); }
                let v = unsafe { *REAL(sexp) };
                if v.is_nan() { return Err(SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::REALSXP })); }
                v
            }
            _ => return Err(SexpTypeError { expected: SEXPTYPE::INTSXP, actual }.into()),
        };
        // Truncate days to i64; add as a Span to the unix epoch.
        let days_i = days.trunc() as i64;
        unix_epoch_date()
            .checked_add(jiff::Span::new().try_days(days_i).map_err(|e| SexpError::InvalidValue(format!("jiff days out of range: {e}")))?)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Date arithmetic: {e}")))
    }
}

impl IntoR for Date {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> { Ok(self.into_sexp()) }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> { self.try_into_sexp() }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            // Compute days since unix epoch via Span::get_days after (self - epoch).
            let span = self.since(unix_epoch_date()).unwrap_or_default();
            *REAL(vec) = span.get_days() as f64;
            crate::cached_class::set_class(vec, "Date"); // helper below
            Rf_unprotect(1);
            vec
        }
    }
}
```

**If `cached_class::set_class` does not exist**, use the established pattern in `time_impl.rs:385–410` (they call into `cached_class::set_date_class` or similar). Search: `rg -n 'set_.*class' miniextendr-api/src/cached_class.rs` and use the existing helper. Do **not** invent new helpers beyond what phase 3 explicitly adds.

- [ ] **2.6 `Option<Date>` / `Vec<Date>` / `Vec<Option<Date>>`.**

Mirror `time_impl.rs:412–607`. `NA_integer_` (i.e. `i32::MIN`) and `NaN` are both NA triggers.

- [ ] **2.7 Compile + clippy checkpoint.**

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase2.log
```

Read the log. Green required before committing.

- [ ] **2.8 Commit.**

```bash
git add miniextendr-api/src/optionals/jiff_impl.rs
git commit -m "[phase 2] jiff: Timestamp and civil::Date conversions

Core UTC scalar + Option + Vec + Vec<Option> conversions between
jiff types and R POSIXct/Date. Mirrors the time_impl.rs structure.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 3: Timezone-aware — `Zoned ↔ POSIXct (+ tzone)`

**Goal:** Round-trip the IANA tz name through POSIXct's `tzone` attribute. Add a `set_posixct_tz` helper in `cached_class.rs` if not already present.

**Files:**
- Modify: `miniextendr-api/src/cached_class.rs` (new helper)
- Modify: `miniextendr-api/src/optionals/jiff_impl.rs`

### Steps

- [ ] **3.1 Inspect `cached_class.rs` for an existing tz-writing helper.**

```bash
rg -n 'tzone|set_posixct' miniextendr-api/src/cached_class.rs
```

If a `set_posixct_tz(sexp, iana: &str)` or `set_class_with_attrs(sexp, classes: &[&str], attrs: &[(&str, SEXP)])` exists, use it. Otherwise add the helper.

- [ ] **3.2 Add `set_posixct_tz` helper (if needed).**

Below the existing `set_posixct_utc` in `cached_class.rs`:

```rust
/// Set the class and `tzone` attribute for a POSIXct vector.
///
/// Writes class `c("POSIXct", "POSIXt")` and the IANA `tzone` attribute. Used by
/// the jiff integration to round-trip `Zoned` timezone identity.
pub fn set_posixct_tz(sexp: SEXP, iana: &str) {
    unsafe {
        set_posixct_utc(sexp); // writes class first
        // Then override the tzone attribute.
        let tzone_sym = crate::ffi::Rf_install(c"tzone".as_ptr());
        let tzone_str = crate::ffi::Rf_mkString(
            std::ffi::CString::new(iana).unwrap_or_else(|_| std::ffi::CString::new("UTC").unwrap()).as_ptr()
        );
        crate::ffi::Rf_protect(tzone_str);
        crate::ffi::Rf_setAttrib(sexp, tzone_sym, tzone_str);
        crate::ffi::Rf_unprotect(1);
    }
}
```

**Verify** `Rf_install`, `Rf_mkString`, `Rf_setAttrib` are the correct FFI names in this crate — grep for usage. Also verify the `c"..."` C-string literal syntax is accepted by the crate's MSRV. If not, use `std::ffi::CString::new(...).unwrap().as_ptr()` for both strings.

- [ ] **3.3 Implement `Zoned` → POSIXct (`IntoR`).**

Append to `jiff_impl.rs` under `// region: Zoned <-> POSIXct (tz)`:

```rust
impl IntoR for Zoned {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> { Ok(self.into_sexp()) }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> { self.try_into_sexp() }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            let ts = self.timestamp();
            *REAL(vec) = ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0);
            crate::cached_class::set_posixct_tz(vec, self.time_zone().iana_name().unwrap_or("UTC"));
            Rf_unprotect(1);
            vec
        }
    }
}
```

**API cross-check:** `Zoned::timestamp() -> Timestamp`, `Zoned::time_zone() -> &TimeZone`, `TimeZone::iana_name() -> Option<&str>`. Some fixed-offset zones don't have an IANA name — the `unwrap_or("UTC")` is deliberate (fallback for fixed offsets; issue if tests hit it).

- [ ] **3.4 Implement `Zoned` ← POSIXct (`TryFromSexp`).**

```rust
impl TryFromSexp for Zoned {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::REALSXP, actual }.into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!("expected scalar POSIXct, got length {}", sexp.len())));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Err(SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::REALSXP }));
        }

        // Read tzone attr.
        let tz_name: String = unsafe {
            let tzone_sym = crate::ffi::Rf_install(c"tzone".as_ptr());
            let tzone_attr = crate::ffi::Rf_getAttrib(sexp, tzone_sym);
            if tzone_attr.type_of() == SEXPTYPE::STRSXP && tzone_attr.len() >= 1 {
                let charsxp = crate::ffi::STRING_ELT(tzone_attr, 0);
                let cstr = std::ffi::CStr::from_ptr(crate::ffi::R_CHAR(charsxp));
                cstr.to_string_lossy().into_owned()
            } else {
                String::new()
            }
        };

        let tz = if tz_name.is_empty() {
            jiff::tz::TimeZone::UTC
        } else {
            jiff::tz::TimeZone::get(&tz_name)
                .map_err(|e| SexpError::InvalidValue(format!("unknown IANA tz {tz_name:?}: {e}")))?
        };

        let whole = secs.floor() as i64;
        let fract = secs - secs.floor();
        let nanos = (fract * 1_000_000_000.0) as i32;
        let ts = Timestamp::new(whole, nanos)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Timestamp out of range: {e}")))?;
        Ok(ts.to_zoned(tz))
    }
}
```

**API cross-check:** `Rf_getAttrib`, `STRING_ELT`, `R_CHAR`, `TimeZone::UTC`, `TimeZone::get`. Names may differ — `jiff::tz::TimeZone::get` may return `Result<TimeZone, Error>` directly. Confirm.

- [ ] **3.5 Option/Vec variants.**

`Option<Zoned>`, `Vec<Zoned>`, `Vec<Option<Zoned>>` — mirror phase 2's `Timestamp` patterns. For `Vec<Zoned>`: the output POSIXct vector can only carry one `tzone` attribute — if the elements have heterogeneous tzs, write the tz of the **first** element and emit a debug log (via `log::warn!` only if `feature = "log"` is on — gate with `#[cfg(feature = "log")]`). Document this limitation in the module doc-comment.

- [ ] **3.6 Compile + clippy checkpoint.**

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase3.log
```

- [ ] **3.7 Commit.**

```bash
git add miniextendr-api/src/cached_class.rs miniextendr-api/src/optionals/jiff_impl.rs
git commit -m "[phase 3] jiff: Zoned <-> POSIXct with IANA tz round-trip

Adds set_posixct_tz helper and Zoned conversions. Unknown IANA tz on
input yields InvalidValue error (not silent UTC fallback). Vec<Zoned>
picks the first element's tz when heterogeneous (documented + log::warn).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 4: Durations — `SignedDuration ↔ difftime`

**Goal:** Scalar, option, and vector conversions for `SignedDuration` against R `difftime` (seconds as numeric + `units = "secs"` + class `difftime`). Add `RSignedDuration` adapter trait mirroring `RDuration`.

**Files:** modify `miniextendr-api/src/optionals/jiff_impl.rs`.

**Reference:** `time_impl.rs:609–723` for the `RDuration` shape.

### Steps

- [ ] **4.1 Add `// region: SignedDuration <-> difftime` section.**

```rust
fn set_difftime_secs_class(sexp: SEXP) {
    unsafe {
        let classes = ["difftime"];
        // Use existing cached_class helpers if present; otherwise allocate STRSXP + Rf_setAttrib.
        crate::cached_class::set_class(sexp, &classes);
        let units_sym = crate::ffi::Rf_install(c"units".as_ptr());
        let units_val = crate::ffi::Rf_mkString(c"secs".as_ptr());
        crate::ffi::Rf_protect(units_val);
        crate::ffi::Rf_setAttrib(sexp, units_sym, units_val);
        crate::ffi::Rf_unprotect(1);
    }
}
```

**Verify** `cached_class::set_class` accepts a slice of `&str`. If instead it takes a single class, iterate. Adapt to the real signature.

- [ ] **4.2 Implement `TryFromSexp` / `IntoR` for `SignedDuration`.**

```rust
impl TryFromSexp for SignedDuration {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError { expected: SEXPTYPE::REALSXP, actual }.into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!("expected scalar difftime, got length {}", sexp.len())));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Err(SexpError::Na(SexpNaError { sexp_type: SEXPTYPE::REALSXP }));
        }
        // Accept difftime regardless of "units" attr for v1 — normalize upstream if needed.
        // (If tests fail because R sent a "mins" difftime, add unit-aware scaling here.)
        let whole = secs.trunc() as i64;
        let frac_nanos = ((secs - secs.trunc()) * 1_000_000_000.0) as i32;
        Ok(SignedDuration::new(whole, frac_nanos))
    }
}

impl IntoR for SignedDuration {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> { Ok(self.into_sexp()) }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> { self.try_into_sexp() }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            *REAL(vec) = self.as_secs_f64();
            set_difftime_secs_class(vec);
            Rf_unprotect(1);
            vec
        }
    }
}
```

**API cross-check:** `SignedDuration::new(secs: i64, nanos: i32) -> Self`, `as_secs_f64() -> f64`. If `new` panics or returns `Result`, adjust.

- [ ] **4.3 Option + Vec + Vec<Option> variants.**

Mirror phase 2 patterns.

- [ ] **4.4 `RSignedDuration` adapter trait.**

```rust
pub trait RSignedDuration {
    fn as_seconds_f64(&self) -> f64;
    fn as_milliseconds(&self) -> i64;
    fn whole_seconds(&self) -> i64;
    fn whole_minutes(&self) -> i64;
    fn whole_hours(&self) -> i64;
    fn whole_days(&self) -> i64;
    fn subsec_nanoseconds(&self) -> i32;
    fn is_negative(&self) -> bool;
    fn is_zero(&self) -> bool;
    fn abs(&self) -> SignedDuration;
}

impl RSignedDuration for SignedDuration {
    fn as_seconds_f64(&self) -> f64 { SignedDuration::as_secs_f64(*self) }
    fn as_milliseconds(&self) -> i64 { self.as_millis() as i64 }
    fn whole_seconds(&self) -> i64 { self.as_secs() }
    fn whole_minutes(&self) -> i64 { self.as_secs() / 60 }
    fn whole_hours(&self) -> i64 { self.as_secs() / 3600 }
    fn whole_days(&self) -> i64 { self.as_secs() / 86_400 }
    fn subsec_nanoseconds(&self) -> i32 { self.subsec_nanos() }
    fn is_negative(&self) -> bool { self.is_negative() }
    fn is_zero(&self) -> bool { self.is_zero() }
    fn abs(&self) -> SignedDuration { SignedDuration::abs(*self) }
}
```

**API cross-check:** `SignedDuration::as_millis() -> i128`, `as_secs() -> i64`, `subsec_nanos() -> i32`, `is_negative()`, `is_zero()`, `abs()`. Several of these may have slightly different names — confirm.

- [ ] **4.5 Compile + clippy.**

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase4.log
```

- [ ] **4.6 Commit.**

```bash
git add miniextendr-api/src/optionals/jiff_impl.rs
git commit -m "[phase 4] jiff: SignedDuration <-> difftime + RSignedDuration trait

Parity with RDuration from time_impl.rs. Unit-less difftime (secs)
for v1 — follow-up issue if unit-aware input handling is needed.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 5: `Span` via ExternalPtr + `RSpan` adapter trait

**Goal:** Wrap `jiff::Span` in an `ExternalPtr`, expose calendar component accessors + arithmetic via an `RSpan` trait.

**Files:** modify `miniextendr-api/src/optionals/jiff_impl.rs`.

**Reference:** ExternalPtr patterns: `rpkg/src/rust/uuid_adapter_tests.rs` for a minimal adapter trait; `docs/EXTERNALPTR.md` for the wrapper contract.

### Steps

- [ ] **5.1 Define `RSpan` trait with component accessors.**

```rust
pub trait RSpan {
    fn get_years(&self) -> i64;
    fn get_months(&self) -> i64;
    fn get_weeks(&self) -> i64;
    fn get_days(&self) -> i64;
    fn get_hours(&self) -> i64;
    fn get_minutes(&self) -> i64;
    fn get_seconds(&self) -> i64;
    fn get_milliseconds(&self) -> i64;
    fn get_microseconds(&self) -> i64;
    fn get_nanoseconds(&self) -> i64;
    fn is_zero(&self) -> bool;
    fn is_negative(&self) -> bool;
    fn negate(&self) -> Span;
    fn abs(&self) -> Span;
}

impl RSpan for Span {
    fn get_years(&self) -> i64 { Span::get_years(*self) as i64 }
    fn get_months(&self) -> i64 { Span::get_months(*self) as i64 }
    fn get_weeks(&self) -> i64 { Span::get_weeks(*self) as i64 }
    fn get_days(&self) -> i64 { Span::get_days(*self) as i64 }
    fn get_hours(&self) -> i64 { Span::get_hours(*self) as i64 }
    fn get_minutes(&self) -> i64 { Span::get_minutes(*self) as i64 }
    fn get_seconds(&self) -> i64 { Span::get_seconds(*self) as i64 }
    fn get_milliseconds(&self) -> i64 { Span::get_milliseconds(*self) as i64 }
    fn get_microseconds(&self) -> i64 { Span::get_microseconds(*self) as i64 }
    fn get_nanoseconds(&self) -> i64 { Span::get_nanoseconds(*self) as i64 }
    fn is_zero(&self) -> bool { Span::is_zero(self) }
    fn is_negative(&self) -> bool { Span::is_negative(self) }
    fn negate(&self) -> Span { Span::negate(*self) }
    fn abs(&self) -> Span { Span::abs(*self) }
}
```

**API cross-check:** jiff's accessors may return `i16` for years/months, `i32` for days, `i64` for sub-second units — the `as i64` cast normalizes. If a method is named `years()` (no `get_`), update.

- [ ] **5.2 Compile + clippy.**

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase5.log
```

- [ ] **5.3 Commit.**

```bash
git add miniextendr-api/src/optionals/jiff_impl.rs
git commit -m "[phase 5] jiff: Span ExternalPtr + RSpan adapter trait

Component accessors, is_zero/is_negative, negate, abs. Wrapping in
#[derive(ExternalPtr)] is exercised in phase 9 fixtures.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 6: Civil-only types — `DateTime`, `Time` via ExternalPtr + adapter traits

**Goal:** Adapter traits for `jiff::civil::DateTime` and `jiff::civil::Time`. No scalar R conversion (they have no base-R analog).

**Files:** modify `miniextendr-api/src/optionals/jiff_impl.rs`.

### Steps

- [ ] **6.1 `RDateTime` trait.**

```rust
pub trait RDateTime {
    fn year(&self) -> i32;
    fn month(&self) -> i32;
    fn day(&self) -> i32;
    fn hour(&self) -> i32;
    fn minute(&self) -> i32;
    fn second(&self) -> i32;
    fn subsec_nanosecond(&self) -> i32;
    fn to_date(&self) -> Date;
    fn to_time(&self) -> Time;
    fn in_tz(&self, iana: &str) -> Result<Zoned, String>;
}

impl RDateTime for DateTime {
    fn year(&self) -> i32 { DateTime::year(*self) as i32 }
    fn month(&self) -> i32 { DateTime::month(*self) as i32 }
    fn day(&self) -> i32 { DateTime::day(*self) as i32 }
    fn hour(&self) -> i32 { DateTime::hour(*self) as i32 }
    fn minute(&self) -> i32 { DateTime::minute(*self) as i32 }
    fn second(&self) -> i32 { DateTime::second(*self) as i32 }
    fn subsec_nanosecond(&self) -> i32 { DateTime::subsec_nanosecond(*self) as i32 }
    fn to_date(&self) -> Date { DateTime::date(*self) }
    fn to_time(&self) -> Time { DateTime::time(*self) }
    fn in_tz(&self, iana: &str) -> Result<Zoned, String> {
        self.in_tz(iana).map_err(|e| e.to_string())
    }
}
```

- [ ] **6.2 `RTime` trait.**

```rust
pub trait RTime {
    fn hour(&self) -> i32;
    fn minute(&self) -> i32;
    fn second(&self) -> i32;
    fn subsec_nanosecond(&self) -> i32;
    fn on(&self, year: i16, month: i8, day: i8) -> Result<DateTime, String>;
}

impl RTime for Time {
    fn hour(&self) -> i32 { Time::hour(*self) as i32 }
    fn minute(&self) -> i32 { Time::minute(*self) as i32 }
    fn second(&self) -> i32 { Time::second(*self) as i32 }
    fn subsec_nanosecond(&self) -> i32 { Time::subsec_nanosecond(*self) as i32 }
    fn on(&self, year: i16, month: i8, day: i8) -> Result<DateTime, String> {
        Date::new(year, month, day)
            .map(|d| d.at(self.hour(), self.minute(), self.second(), self.subsec_nanosecond() as i32))
            .map_err(|e| e.to_string())
    }
}
```

**API cross-check:** `Date::new(year: i16, month: i8, day: i8) -> Result<Date, Error>`. Arg types may differ.

- [ ] **6.3 Compile + clippy + commit.**

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase6.log
```

```bash
git add miniextendr-api/src/optionals/jiff_impl.rs
git commit -m "[phase 6] jiff: RDateTime and RTime adapter traits

Component accessors + in_tz/on conversions. No scalar SEXP mapping —
users wrap these in #[derive(ExternalPtr)].

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 7: `RTimestamp`, `RZoned`, `RDate` adapter traits

**Goal:** Consistent adapter surface for the scalar-convertible types, parallel to `RDateTimeFormat` from `time_impl.rs:725+`.

**Files:** modify `miniextendr-api/src/optionals/jiff_impl.rs`.

### Steps

- [ ] **7.1 `RTimestamp`.**

```rust
pub trait RTimestamp {
    fn as_second(&self) -> i64;
    fn as_millisecond(&self) -> i64;
    fn subsec_nanosecond(&self) -> i32;
    fn to_zoned_in(&self, iana: &str) -> Result<Zoned, String>;
    fn strftime(&self, fmt: &str) -> String;
}

impl RTimestamp for Timestamp {
    fn as_second(&self) -> i64 { Timestamp::as_second(*self) }
    fn as_millisecond(&self) -> i64 { Timestamp::as_millisecond(*self) }
    fn subsec_nanosecond(&self) -> i32 { Timestamp::subsec_nanosecond(*self) }
    fn to_zoned_in(&self, iana: &str) -> Result<Zoned, String> {
        self.in_tz(iana).map_err(|e| e.to_string())
    }
    fn strftime(&self, fmt: &str) -> String {
        // Timestamps format via UTC; for local formatting, route through Zoned.
        self.strftime(fmt).to_string()
    }
}
```

- [ ] **7.2 `RZoned`.**

```rust
pub trait RZoned {
    fn iana_name(&self) -> Option<String>;
    fn year(&self) -> i32;
    fn month(&self) -> i32;
    fn day(&self) -> i32;
    fn hour(&self) -> i32;
    fn minute(&self) -> i32;
    fn second(&self) -> i32;
    fn in_tz(&self, iana: &str) -> Result<Zoned, String>;
    fn start_of_day(&self) -> Result<Zoned, String>;
    fn strftime(&self, fmt: &str) -> String;
}

impl RZoned for Zoned {
    fn iana_name(&self) -> Option<String> { self.time_zone().iana_name().map(str::to_string) }
    fn year(&self) -> i32 { Zoned::year(self) as i32 }
    fn month(&self) -> i32 { Zoned::month(self) as i32 }
    fn day(&self) -> i32 { Zoned::day(self) as i32 }
    fn hour(&self) -> i32 { Zoned::hour(self) as i32 }
    fn minute(&self) -> i32 { Zoned::minute(self) as i32 }
    fn second(&self) -> i32 { Zoned::second(self) as i32 }
    fn in_tz(&self, iana: &str) -> Result<Zoned, String> { Zoned::in_tz(self, iana).map_err(|e| e.to_string()) }
    fn start_of_day(&self) -> Result<Zoned, String> { self.start_of_day().map_err(|e| e.to_string()) }
    fn strftime(&self, fmt: &str) -> String { Zoned::strftime(self, fmt).to_string() }
}
```

- [ ] **7.3 `RDate`.**

```rust
pub trait RDate {
    fn year(&self) -> i32;
    fn month(&self) -> i32;
    fn day(&self) -> i32;
    fn weekday(&self) -> i32;
    fn day_of_year(&self) -> i32;
    fn first_of_month(&self) -> Date;
    fn last_of_month(&self) -> Date;
    fn tomorrow(&self) -> Result<Date, String>;
    fn yesterday(&self) -> Result<Date, String>;
    fn strftime(&self, fmt: &str) -> String;
}

impl RDate for Date {
    fn year(&self) -> i32 { Date::year(*self) as i32 }
    fn month(&self) -> i32 { Date::month(*self) as i32 }
    fn day(&self) -> i32 { Date::day(*self) as i32 }
    fn weekday(&self) -> i32 { Date::weekday(*self).to_monday_one_offset() as i32 }
    fn day_of_year(&self) -> i32 { Date::day_of_year(*self) as i32 }
    fn first_of_month(&self) -> Date { Date::first_of_month(*self) }
    fn last_of_month(&self) -> Date { Date::last_of_month(*self) }
    fn tomorrow(&self) -> Result<Date, String> { self.tomorrow().map_err(|e| e.to_string()) }
    fn yesterday(&self) -> Result<Date, String> { self.yesterday().map_err(|e| e.to_string()) }
    fn strftime(&self, fmt: &str) -> String { Date::strftime(self, fmt).to_string() }
}
```

**API cross-check:** `Weekday::to_monday_one_offset()` is 1-based Mon..Sun. If not present, use `to_sunday_zero_offset() + 1` or similar. `first_of_month`/`last_of_month` confirmed. `tomorrow`/`yesterday` may return `Result`.

- [ ] **7.4 Compile + commit.**

```bash
dangerouslyDisableSandbox cargo clippy -p miniextendr-api --features jiff -- -D warnings 2>&1 > /tmp/clippy-jiff-phase7.log
git add miniextendr-api/src/optionals/jiff_impl.rs
git commit -m "[phase 7] jiff: RTimestamp, RZoned, RDate adapter traits

Component accessors, formatting, and tz/date arithmetic helpers.
Completes the scalar adapter surface.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 8: Documentation

**Files:**
- Modify: `docs/FEATURES.md`
- Modify: `docs/CONVERSION_MATRIX.md`

### Steps

- [ ] **8.1 `docs/FEATURES.md` — feature table row.**

Add a row under the existing `| time |` row, matching that row's format:

```
| `jiff` | `OffsetDateTime`-parallel `Timestamp`/`Zoned`/`civil::Date`/`SignedDuration` with IANA tz | jiff (with `std`, `tzdb-bundle-always`) |
```

- [ ] **8.2 `docs/FEATURES.md` — `### jiff` subsection.**

Add immediately after the existing `### time` subsection. Mirror that subsection's structure:

```markdown
### `jiff`

Date and time conversions via the `jiff` crate with **first-class IANA timezone support**.

| Rust | R | Notes |
|------|---|-------|
| `jiff::Timestamp` | `POSIXct` (UTC) | Nanosecond precision |
| `jiff::Zoned` | `POSIXct` (+ `tzone` attr) | IANA tz round-tripped |
| `jiff::civil::Date` | `Date` | Days since 1970-01-01 |
| `jiff::SignedDuration` | `difftime` (secs) | |
| `jiff::Span` | `ExternalPtr<Span>` | Calendar span, via `RSpan` trait |
| `jiff::civil::DateTime` | `ExternalPtr<DateTime>` | Via `RDateTime` trait |
| `jiff::civil::Time` | `ExternalPtr<Time>` | Via `RTime` trait |

```r
# Unknown IANA tz → R error (not silent UTC)
bad <- structure(0, class = c("POSIXct", "POSIXt"), tzone = "Mars/Olympus")
from_jiff_zoned(bad) # Error: unknown IANA tz "Mars/Olympus"
```

Use `features = ["jiff"]`. Coexists with the `time` feature.
```

- [ ] **8.3 `docs/CONVERSION_MATRIX.md` — add jiff rows.**

Append rows to the existing R-type-to-Rust-type tables, one per conversion above. Follow the file's column order and escape conventions.

- [ ] **8.4 Commit.**

```bash
git add docs/
git commit -m "[phase 8] docs: jiff feature in FEATURES.md + CONVERSION_MATRIX.md

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 9: rpkg fixtures + testthat tests

**Goal:** R-visible fixtures exercising every conversion, adapter trait, and error path added in phases 2–7. ALTREP + vctrs fixtures are added in phases 10–11.

**Files:**
- Create: `rpkg/src/rust/jiff_adapter_tests.rs`
- Modify: `rpkg/src/rust/lib.rs` (register the module)
- Create: `rpkg/tests/testthat/test-jiff.R`
- Modify: `rpkg/Cargo.toml` — the `jiff` passthrough added in phase 1.4 needs to be **actually used** here.

### Steps

- [ ] **9.1 Create `rpkg/src/rust/jiff_adapter_tests.rs`.**

Mirror the style of `rpkg/src/rust/uuid_adapter_tests.rs`. Fixtures:

```rust
#![cfg(feature = "jiff")]

use miniextendr::{miniextendr, ExternalPtr};
use miniextendr::optionals::{
    Timestamp, Zoned, JiffDate, SignedDuration, Span, JiffDateTime, JiffTime,
    RSpan, RDate, RZoned, RTimestamp, RSignedDuration, RDateTime, RTime,
};
use jiff::civil::date;
use jiff::ToSpan;

#[miniextendr]
fn jiff_timestamp_round_trip(x: Timestamp) -> Timestamp { x }

#[miniextendr]
fn jiff_timestamp_now() -> Timestamp { Timestamp::now() }

#[miniextendr]
fn jiff_zoned_in(iana: String) -> Result<Zoned, String> {
    Timestamp::now().in_tz(&iana).map_err(|e| e.to_string())
}

#[miniextendr]
fn jiff_zoned_round_trip(x: Zoned) -> Zoned { x }

#[miniextendr]
fn jiff_zoned_iana(x: Zoned) -> String {
    x.time_zone().iana_name().unwrap_or("").to_string()
}

#[miniextendr]
fn jiff_date_round_trip(x: JiffDate) -> JiffDate { x }

#[miniextendr]
fn jiff_date_components(x: JiffDate) -> Vec<i32> {
    vec![x.year() as i32, x.month() as i32, x.day() as i32]
}

#[miniextendr]
fn jiff_duration_round_trip(x: SignedDuration) -> SignedDuration { x }

#[miniextendr]
fn jiff_duration_abs(x: SignedDuration) -> SignedDuration { x.abs() }

// Span via ExternalPtr
#[derive(ExternalPtr)]
pub struct JiffSpan(pub Span);

#[miniextendr]
impl JiffSpan {
    pub fn new(years: i64, months: i64, days: i64) -> Self {
        JiffSpan(years.years().months(months).days(days))
    }
    pub fn years(&self) -> i64 { self.0.get_years() as i64 }
    pub fn months(&self) -> i64 { self.0.get_months() as i64 }
    pub fn days(&self) -> i64 { self.0.get_days() as i64 }
    pub fn is_zero(&self) -> bool { self.0.is_zero() }
    pub fn is_negative(&self) -> bool { self.0.is_negative() }
}

// Vec round-trips
#[miniextendr]
fn jiff_timestamp_vec_round_trip(x: Vec<Timestamp>) -> Vec<Timestamp> { x }

#[miniextendr]
fn jiff_timestamp_opt_vec_round_trip(x: Vec<Option<Timestamp>>) -> Vec<Option<Timestamp>> { x }

#[miniextendr]
fn jiff_date_vec_round_trip(x: Vec<JiffDate>) -> Vec<JiffDate> { x }
```

- [ ] **9.2 Register the module in `rpkg/src/rust/lib.rs`.**

Find the existing `#[cfg(feature = "time")] pub mod time_adapter_tests;` (or equivalent) and add directly below:

```rust
#[cfg(feature = "jiff")]
pub mod jiff_adapter_tests;
```

- [ ] **9.3 Ensure `rpkg/Cargo.toml` jiff feature activates this module.**

Verify the phase 1.4 passthrough is intact:

```toml
jiff = ["miniextendr-api/jiff"]
```

Add jiff to the rpkg build's default feature list **only if** the project's convention enables other optional features by default in dev builds. Check `just rcmdinstall` behavior: `just --show rcmdinstall | grep features`. If `time` is enabled by default, enable `jiff` too. If not, leave jiff opt-in.

- [ ] **9.4 Install the package + regenerate wrappers.**

```bash
dangerouslyDisableSandbox bash -c 'just configure && just rcmdinstall 2>&1 > /tmp/rcmdinstall-jiff.log'
```

Read `/tmp/rcmdinstall-jiff.log` via the Read tool. Install must succeed.

```bash
dangerouslyDisableSandbox just devtools-document 2>&1 > /tmp/devtools-doc-jiff.log
```

Commit the regenerated R/ files + man/ + NAMESPACE immediately — the pre-commit hook blocks mismatched wrapper/NAMESPACE commits.

- [ ] **9.5 Create `rpkg/tests/testthat/test-jiff.R`.**

```r
# Skip entire file if jiff feature not compiled
skip_if_not_jiff <- function() {
  if (!exists("jiff_timestamp_now", mode = "function")) {
    skip("jiff feature not enabled in this build")
  }
}

test_that("jiff Timestamp round-trips via POSIXct UTC", {
  skip_if_not_jiff()
  t <- as.POSIXct("2024-07-15 12:34:56", tz = "UTC")
  out <- jiff_timestamp_round_trip(t)
  expect_equal(as.numeric(out), as.numeric(t))
  expect_equal(attr(out, "tzone"), "UTC")
})

test_that("jiff Zoned preserves IANA tzone attr", {
  skip_if_not_jiff()
  z <- jiff_zoned_in("Europe/Paris")
  expect_s3_class(z, "POSIXct")
  expect_equal(attr(z, "tzone"), "Europe/Paris")

  # round-trip
  z2 <- jiff_zoned_round_trip(z)
  expect_equal(attr(z2, "tzone"), "Europe/Paris")
  expect_equal(jiff_zoned_iana(z), "Europe/Paris")
})

test_that("unknown IANA tz errors out", {
  skip_if_not_jiff()
  bad <- structure(0, class = c("POSIXct", "POSIXt"), tzone = "Mars/Olympus")
  expect_error(jiff_zoned_round_trip(bad), "Mars/Olympus")
})

test_that("civil::Date round-trips via R Date", {
  skip_if_not_jiff()
  d <- as.Date("2024-02-29")  # leap day
  out <- jiff_date_round_trip(d)
  expect_equal(out, d)
  expect_equal(jiff_date_components(d), c(2024L, 2L, 29L))
})

test_that("SignedDuration round-trips via difftime", {
  skip_if_not_jiff()
  dd <- as.difftime(3661, units = "secs")
  out <- jiff_duration_round_trip(dd)
  expect_equal(as.numeric(out), 3661)
  expect_s3_class(out, "difftime")
})

test_that("NA POSIXct round-trips as NA", {
  skip_if_not_jiff()
  x <- c(as.POSIXct("2024-01-01", tz = "UTC"), NA)
  out <- jiff_timestamp_opt_vec_round_trip(x)
  expect_true(is.na(out[2]))
  expect_equal(as.numeric(out[1]), as.numeric(x[1]))
})

test_that("Span accessors work", {
  skip_if_not_jiff()
  s <- JiffSpan$new(years = 1, months = 2, days = 15)
  expect_equal(s$years(), 1L)
  expect_equal(s$months(), 2L)
  expect_equal(s$days(), 15L)
  expect_false(s$is_zero())
  expect_false(s$is_negative())
})

test_that("vector round-trips work", {
  skip_if_not_jiff()
  ts <- seq.POSIXt(as.POSIXct("2024-01-01", tz = "UTC"), by = "1 hour", length.out = 5)
  out <- jiff_timestamp_vec_round_trip(ts)
  expect_equal(as.numeric(out), as.numeric(ts))

  dates <- seq.Date(as.Date("2024-02-28"), by = "1 day", length.out = 4)
  out_d <- jiff_date_vec_round_trip(dates)
  expect_equal(out_d, dates)
})
```

- [ ] **9.6 Run tests.**

```bash
dangerouslyDisableSandbox just devtools-test 2>&1 > /tmp/devtools-test-jiff.log
```

Read the log. Fix any test failures in-place before committing.

- [ ] **9.7 Commit.**

```bash
git add rpkg/
git commit -m "[phase 9] jiff: rpkg fixtures + testthat tests for scalar/vector/option paths

Covers Timestamp/Zoned/Date/SignedDuration/Span round-trips, NA
handling, IANA tz preservation, unknown-tz error path, leap day.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 10: vctrs `rcrd` constructors for `Span`, `Zoned`, `DateTime`, `Time`

**Goal:** Proper vctrs-backed vector types for jiff's record-like types, so R users get `vctrs::vec_size`, `vctrs::field`, `[[`, and coercion behavior for free.

**Files:**
- Modify: `miniextendr-api/src/optionals/jiff_impl.rs` (new `// region: vctrs support` gated on `all(feature = "jiff", feature = "vctrs")`)
- Create: `rpkg/src/rust/jiff_vctrs_tests.rs` (fixtures exposed to R)
- Modify: `rpkg/src/rust/lib.rs` (module registration under same cfg)
- Modify: `rpkg/tests/testthat/test-jiff.R` (new testthat block)

### Steps

- [ ] **10.1 Add `// region: vctrs support` to `jiff_impl.rs`.**

Gate the entire region:

```rust
#[cfg(feature = "vctrs")]
mod vctrs {
    use super::*;
    use crate::vctrs::new_rcrd;
    use crate::ffi::{INTEGER, Rf_allocVector, Rf_protect, Rf_unprotect, R_NaInt, SEXP, SEXPTYPE};

    fn vec_span_to_rcrd(spans: &[Span]) -> SEXP {
        // 10 i64 fields → allocate one INTSXP column per accessor (i32-truncated; follow-up
        // issue if span components exceed i32 in practice).
        let n = spans.len() as crate::ffi::R_xlen_t;
        let mk_col = |extract: &dyn Fn(&Span) -> i64| -> SEXP {
            unsafe {
                let col = Rf_allocVector(SEXPTYPE::INTSXP, n);
                Rf_protect(col);
                let dst = INTEGER(col);
                for (i, s) in spans.iter().enumerate() {
                    *dst.add(i) = extract(s) as i32;
                }
                Rf_unprotect(1);
                col
            }
        };
        let fields = [
            ("years", mk_col(&|s| s.get_years() as i64)),
            ("months", mk_col(&|s| s.get_months() as i64)),
            ("weeks", mk_col(&|s| s.get_weeks() as i64)),
            ("days", mk_col(&|s| s.get_days() as i64)),
            ("hours", mk_col(&|s| s.get_hours() as i64)),
            ("minutes", mk_col(&|s| s.get_minutes() as i64)),
            ("seconds", mk_col(&|s| s.get_seconds() as i64)),
            ("milliseconds", mk_col(&|s| s.get_milliseconds() as i64)),
            ("microseconds", mk_col(&|s| s.get_microseconds() as i64)),
            ("nanoseconds", mk_col(&|s| s.get_nanoseconds() as i64)),
        ];
        new_rcrd(&fields, &["jiff_span", "vctrs_rcrd", "vctrs_vctr"], &[], None)
            .expect("new_rcrd should not fail for well-formed fields")
    }
    // ... vec_zoned_to_rcrd, vec_datetime_to_rcrd, vec_time_to_rcrd similarly ...
}
```

**API cross-check:** The exact signature of `crate::vctrs::new_rcrd` — confirm. It returns `Result<SEXP, VctrsBuildError>` per the module inspection. The `&[(&str, SEXP)]` field-pairs form may differ; inspect `miniextendr-api/src/vctrs.rs` and adapt.

Implement `vec_zoned_to_rcrd`, `vec_datetime_to_rcrd`, `vec_time_to_rcrd` following the same shape. For `Zoned`: two fields, `timestamp: REALSXP` (seconds-since-epoch f64) and `tz: STRSXP` (IANA name per element).

- [ ] **10.2 Expose via rpkg fixtures.**

Create `rpkg/src/rust/jiff_vctrs_tests.rs`:

```rust
#![cfg(all(feature = "jiff", feature = "vctrs"))]

use miniextendr::miniextendr;
use miniextendr::optionals::{Span};
use jiff::ToSpan;

// Return a raw SEXP that the vctrs helper builds — exposed via miniextendr's
// SEXP return path.
#[miniextendr]
fn jiff_span_vec_demo() -> miniextendr::ffi::SEXP {
    let spans = vec![
        1.year().months(2),
        3.months().days(15),
        Span::new(),
    ];
    // Public wrapper around the crate-private vec_span_to_rcrd:
    miniextendr::optionals::jiff_impl::vctrs_demo::span_vec_to_rcrd(&spans)
}
```

**Surface decision:** the `vctrs` submodule stays private; expose public helper functions at the `jiff_impl` top level:

```rust
/// Public helper: convert `&[Span]` into a vctrs `jiff_span` rcrd SEXP.
#[cfg(feature = "vctrs")]
pub fn span_vec_to_rcrd(spans: &[Span]) -> SEXP {
    vctrs::vec_span_to_rcrd(spans)
}

/// Likewise: `zoned_vec_to_rcrd`, `datetime_vec_to_rcrd`, `time_vec_to_rcrd` —
/// same shape, wrapping the private `vec_<type>_to_rcrd` functions in `mod vctrs`.
```

Then the rpkg fixture uses the top-level helper directly:

```rust
fn jiff_span_vec_demo() -> miniextendr::ffi::SEXP {
    miniextendr::optionals::jiff_impl::span_vec_to_rcrd(&[
        1.year().months(2),
        3.months().days(15),
        Span::new(),
    ])
}
```

- [ ] **10.3 Register module in `rpkg/src/rust/lib.rs`.**

```rust
#[cfg(all(feature = "jiff", feature = "vctrs"))]
pub mod jiff_vctrs_tests;
```

- [ ] **10.4 testthat coverage.**

Append to `rpkg/tests/testthat/test-jiff.R`:

```r
test_that("jiff Span vctrs rcrd works", {
  skip_if_not_jiff()
  if (!requireNamespace("vctrs", quietly = TRUE)) skip("vctrs not installed")
  if (!exists("jiff_span_vec_demo", mode = "function")) skip("vctrs feature off")

  v <- jiff_span_vec_demo()
  expect_s3_class(v, "jiff_span")
  expect_s3_class(v, "vctrs_rcrd")
  expect_equal(vctrs::vec_size(v), 3L)
  expect_equal(vctrs::field(v, "years"), c(1L, 0L, 0L))
  expect_equal(vctrs::field(v, "months"), c(2L, 3L, 0L))
  expect_equal(vctrs::field(v, "days"), c(0L, 15L, 0L))
})
```

- [ ] **10.5 Build + test.**

```bash
dangerouslyDisableSandbox bash -c 'just configure && just rcmdinstall && just devtools-document && just devtools-test 2>&1 > /tmp/devtools-test-jiff-vctrs.log'
```

Read log. Fix in place.

- [ ] **10.6 Commit.**

```bash
git add miniextendr-api/src/optionals/jiff_impl.rs rpkg/src/rust/jiff_vctrs_tests.rs rpkg/src/rust/lib.rs rpkg/tests/testthat/test-jiff.R rpkg/R rpkg/NAMESPACE rpkg/man
git commit -m "[phase 10] jiff: vctrs rcrd constructors for Span/Zoned/DateTime/Time

Public span_vec_to_rcrd/zoned_vec_to_rcrd/datetime_vec_to_rcrd/time_vec_to_rcrd
helpers gated on all(feature = jiff, feature = vctrs). R-side tests via
vctrs::vec_size + vctrs::field.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 11: ALTREP lazy vectors for `Vec<Timestamp>`

**Goal:** Wrap `Arc<Vec<Timestamp>>` behind an ALTREP REALSXP that lazily computes seconds-since-epoch on element access, avoiding the O(n) upfront conversion for large vectors.

**Scope decision for this phase:** Timestamp only. Zoned ALTREP is deferred to a follow-up (more complex due to tz attribute interaction with ALTREP's materialization semantics). File a `gh issue create` for Zoned ALTREP before committing this phase.

**Files:**
- Modify: `miniextendr-api/src/optionals/jiff_impl.rs` (new `// region: ALTREP` section)
- Create: `rpkg/src/rust/jiff_altrep_tests.rs`
- Modify: `rpkg/src/rust/lib.rs`
- Modify: `rpkg/tests/testthat/test-jiff.R`

**Reference:** `docs/SPARSE_ITERATOR_ALTREP.md`, `docs/ALTREP_EXAMPLES.md`, existing derives in `miniextendr-api/src/altrep*.rs`.

### Steps

- [ ] **11.1 Inspect existing ALTREP derives for the right macro shape.**

```bash
rg -n '#\[derive\(AltrepReal|#\[altrep' miniextendr-api/src/ rpkg/src/rust/ | head -20
```

Identify whether the project supports a derive-level expression for `elt` (function returning `f64`) given a per-struct closure, or requires the manual `#[altrep(manual)]` path with hand-written `AltrepLen` + `AltRealData`.

- [ ] **11.2 Add `JiffTimestampVec` ALTREP wrapper.**

Assuming derive support (adapt to `#[altrep(manual)]` if derive doesn't fit):

```rust
use std::sync::Arc;

/// ALTREP-backed lazy vector of `Timestamp`s. Materialized-on-access as seconds-since-epoch
/// f64. Class: POSIXct (UTC) once materialized.
#[derive(miniextendr::AltrepReal)]
#[altrep(len = "len", elt = "elt_secs", class = "JiffTimestampVec")]
pub struct JiffTimestampVec {
    pub data: Arc<Vec<Timestamp>>,
}

impl JiffTimestampVec {
    pub fn new(data: Vec<Timestamp>) -> Self { Self { data: Arc::new(data) } }
    pub fn len(&self) -> usize { self.data.len() }
    pub fn elt_secs(&self, i: usize) -> f64 {
        let ts = &self.data[i];
        ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
    }
}
```

**API cross-check:** confirm `AltrepReal` derive + attribute form against `miniextendr-api/src/altrep.rs` / `docs/ALTREP_QUICKREF.md`. If the derive takes method name strings that must match trait methods (e.g. `AltrepRealElt::elt` signature), ensure the inherent method conforms.

- [ ] **11.3 Expose via rpkg.**

`rpkg/src/rust/jiff_altrep_tests.rs`:

```rust
#![cfg(feature = "jiff")]

use miniextendr::miniextendr;
use miniextendr::optionals::{JiffTimestampVec, Timestamp};

#[miniextendr]
fn jiff_altrep_large(n: i32) -> JiffTimestampVec {
    let base = Timestamp::from_second(1_704_067_200).expect("valid base");
    let mut v = Vec::with_capacity(n as usize);
    for i in 0..n as i64 {
        v.push(
            base.checked_add(jiff::SignedDuration::new(i * 3600, 0)).unwrap(),
        );
    }
    JiffTimestampVec::new(v)
}
```

Register in `rpkg/src/rust/lib.rs`:

```rust
#[cfg(feature = "jiff")]
pub mod jiff_altrep_tests;
```

- [ ] **11.4 testthat coverage.**

Append to `rpkg/tests/testthat/test-jiff.R`:

```r
test_that("jiff Timestamp ALTREP vector materializes correctly", {
  skip_if_not_jiff()
  if (!exists("jiff_altrep_large", mode = "function")) skip("altrep fixture missing")
  v <- jiff_altrep_large(1000L)
  expect_equal(length(v), 1000L)
  # First element = base timestamp (2024-01-01T00:00:00Z)
  expect_equal(as.numeric(v[1]), 1704067200)
  # 500th element = base + 499h
  expect_equal(as.numeric(v[500]), 1704067200 + 499 * 3600)
})
```

- [ ] **11.5 File follow-up issue for Zoned ALTREP.**

```bash
gh issue create --title "feat(jiff): ALTREP wrapper for Vec<Zoned>" --label enhancement --body "$(cat <<'EOF'
Follow-up from the jiff integration PR. Phase 11 landed Timestamp-only ALTREP because Zoned ALTREP interacts non-trivially with the tzone attribute (class assignment during materialization, heterogeneous-tz vector semantics).

Scope: extend `JiffTimestampVec` with `JiffZonedVec` keyed on a shared tz OR carrying per-element tz names. Decide semantics in a short design note.

Reference: the main jiff integration PR for context.
EOF
)"
```

Record the returned issue URL — reference it in the final PR body.

- [ ] **11.6 Build + test + commit.**

```bash
dangerouslyDisableSandbox bash -c 'just configure && just rcmdinstall && just devtools-document && just devtools-test 2>&1 > /tmp/devtools-test-jiff-altrep.log'
```

Read log. Green required.

```bash
git add miniextendr-api/src/optionals/jiff_impl.rs rpkg/src/rust/jiff_altrep_tests.rs rpkg/src/rust/lib.rs rpkg/tests/testthat/test-jiff.R rpkg/R rpkg/NAMESPACE rpkg/man
git commit -m "[phase 11] jiff: ALTREP lazy vector for Vec<Timestamp>

JiffTimestampVec lazily materializes seconds-since-epoch on element
access. Zoned ALTREP deferred — follow-up issue linked in PR body.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Phase 12: Full verification

**Goal:** All of CLAUDE.md's "Reproducing CI clippy before PR" + build + test requirements green before the agent reports done.

### Steps

- [ ] **12.1 Full `just check` / `just clippy` / `just test`.**

```bash
dangerouslyDisableSandbox just check   2>&1 > /tmp/just-check.log
dangerouslyDisableSandbox just clippy  2>&1 > /tmp/just-clippy.log
dangerouslyDisableSandbox just test    2>&1 > /tmp/just-test.log
```

Read each log. Zero warnings, zero failures.

- [ ] **12.2 Reproduce CI clippy_default (`-D warnings`).**

```bash
dangerouslyDisableSandbox cargo clippy --workspace --all-targets --locked -- -D warnings 2>&1 > /tmp/clippy-default.log
```

- [ ] **12.3 Reproduce CI clippy_all.**

```bash
dangerouslyDisableSandbox cargo clippy --workspace --all-targets --locked \
  --features rayon,rand,rand_distr,either,ndarray,nalgebra,serde,serde_json,num-bigint,rust_decimal,ordered-float,uuid,regex,indexmap,time,num-traits,bytes,num-complex,url,sha2,bitflags,bitvec,aho-corasick,toml,tabled,raw_conversions,vctrs,tinyvec,borsh,connections,nonapi,default-strict,default-coerce,default-r6,default-worker,jiff \
  -- -D warnings \
  2>&1 > /tmp/clippy-all.log
```

- [ ] **12.4 `miniextendr-lint`.**

```bash
dangerouslyDisableSandbox just lint 2>&1 > /tmp/lint.log
```

- [ ] **12.5 R CMD check on built tarball.**

```bash
dangerouslyDisableSandbox bash -c 'just configure && just vendor && just r-cmd-build && just r-cmd-check 2>&1 > /tmp/rcmdcheck.log'
```

Read the log. Investigate any NOTE/WARNING/ERROR. Must be clean beyond baseline (compare against a fresh `git stash` + `just r-cmd-check` on the diff's base if needed).

- [ ] **12.6 No commit for this phase unless fixes were needed.**

If any of the steps required source changes, commit them as `[phase 12] verification fixes`. Otherwise skip the commit and proceed to plan-complete.

- [ ] **12.7 Report done.**

Write a short summary to the controlling process:

- branch name
- which phases had notable deviations from the plan (e.g. jiff API method-name drift)
- any concessions made + the `gh issue create` URLs already filed
- log file locations for the review pass

---

## Acceptance checklist (whole-feature)

- [ ] `cargo clippy -p miniextendr-api --features jiff -- -D warnings` clean
- [ ] Full `clippy_default` union clean
- [ ] Full `clippy_all` union (including `jiff`) clean
- [ ] `just lint` clean
- [ ] `just devtools-test` green
- [ ] `just r-cmd-check` clean (no new NOTES/WARNINGS/ERRORS)
- [ ] `rpkg/R/miniextendr-wrappers.R`, `rpkg/NAMESPACE`, `rpkg/man/*.Rd` committed in sync with Rust changes (pre-commit hook passes)
- [ ] `docs/FEATURES.md` + `docs/CONVERSION_MATRIX.md` updated
- [ ] 12 commits on `feat/jiff-integration`, one per phase (except phase 12 which is optional), each prefixed `[phase N]`
- [ ] All deferred items filed as `gh issue`s, linked in final PR body
