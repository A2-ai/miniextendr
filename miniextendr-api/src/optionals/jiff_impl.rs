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
//!
//! # Vec<Zoned> timezone policy
//!
//! A `Vec<Zoned>` → POSIXct can only carry one `tzone` attribute. When elements have
//! heterogeneous timezones, the first element's timezone is used and a warning is logged
//! (via `log::warn!` when the `log` feature is enabled). Document this limitation to users.

pub use jiff::civil::{Date, DateTime, Time};
pub use jiff::{SignedDuration, Span, Timestamp, Zoned};

use crate::cached_class::{date_class_sexp, set_posixct_tz, set_posixct_utc};
use crate::ffi::{REAL, Rf_allocVector, Rf_protect, Rf_unprotect, SEXP, SEXPTYPE, SexpExt};
use crate::from_r::{SexpError, SexpNaError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

/// Unix epoch as a civil::Date constant.
fn unix_epoch_date() -> Date {
    jiff::civil::date(1970, 1, 1)
}

// region: Timestamp <-> POSIXct (UTC)

impl TryFromSexp for Timestamp {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar POSIXct, got length {}",
                sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::REALSXP,
            }));
        }
        // Floor-based split: correct for negative timestamps.
        let whole = secs.floor() as i64;
        let fract = secs - secs.floor(); // always in [0, 1)
        let nanos = (fract * 1_000_000_000.0) as i32;
        Timestamp::new(whole, nanos)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Timestamp out of range: {e}")))
    }
}

impl IntoR for Timestamp {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            let secs =
                self.as_second() as f64 + (self.subsec_nanosecond() as f64 / 1_000_000_000.0);
            *REAL(vec) = secs;
            set_posixct_utc(vec);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Option<Timestamp>

impl TryFromSexp for Option<Timestamp> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar POSIXct, got length {}",
                sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Ok(None);
        }
        let whole = secs.floor() as i64;
        let fract = secs - secs.floor();
        let nanos = (fract * 1_000_000_000.0) as i32;
        Timestamp::new(whole, nanos)
            .map(Some)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Timestamp out of range: {e}")))
    }
}

impl IntoR for Option<Timestamp> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        match self {
            Some(ts) => ts.into_sexp(),
            None => unsafe {
                let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
                Rf_protect(vec);
                *REAL(vec) = f64::NAN;
                set_posixct_utc(vec);
                Rf_unprotect(1);
                vec
            },
        }
    }
}

// endregion

// region: Vec<Timestamp>

impl TryFromSexp for Vec<Timestamp> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &secs) in src.iter().enumerate() {
            if secs.is_nan() {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Timestamp>",
                    i
                )));
            }
            let whole = secs.floor() as i64;
            let fract = secs - secs.floor();
            let nanos = (fract * 1_000_000_000.0) as i32;
            let ts = Timestamp::new(whole, nanos).map_err(|e| {
                SexpError::InvalidValue(format!("jiff Timestamp out of range at index {i}: {e}"))
            })?;
            result.push(ts);
        }
        Ok(result)
    }
}

impl IntoR for Vec<Timestamp> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, ts) in dst.iter_mut().zip(self) {
                *slot = ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0);
            }
            set_posixct_utc(vec);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Vec<Option<Timestamp>>

impl TryFromSexp for Vec<Option<Timestamp>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &secs) in src.iter().enumerate() {
            if secs.is_nan() {
                result.push(None);
            } else {
                let whole = secs.floor() as i64;
                let fract = secs - secs.floor();
                let nanos = (fract * 1_000_000_000.0) as i32;
                let ts = Timestamp::new(whole, nanos).map_err(|e| {
                    SexpError::InvalidValue(format!(
                        "jiff Timestamp out of range at index {i}: {e}"
                    ))
                })?;
                result.push(Some(ts));
            }
        }
        Ok(result)
    }
}

impl IntoR for Vec<Option<Timestamp>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, opt) in dst.iter_mut().zip(self) {
                *slot = match opt {
                    Some(ts) => {
                        ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
                    }
                    None => f64::NAN,
                };
            }
            set_posixct_utc(vec);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: civil::Date <-> R Date

impl TryFromSexp for Date {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        // R's Date class is stored as a double (days since 1970-01-01).
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar Date, got length {}",
                sexp.len()
            )));
        }
        let days = unsafe { *REAL(sexp) };
        if days.is_nan() {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::REALSXP,
            }));
        }
        let days_i64 = days.trunc() as i64;
        let span = Span::new()
            .try_days(days_i64)
            .map_err(|e| SexpError::InvalidValue(format!("jiff days out of range: {e}")))?;
        unix_epoch_date()
            .checked_add(span)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Date arithmetic: {e}")))
    }
}

impl IntoR for Date {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            // Date::since(epoch) → Span of days
            let span = self.since(unix_epoch_date()).unwrap_or_default();
            *REAL(vec) = span.get_days() as f64;
            vec.set_class(date_class_sexp());
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Option<Date>

impl TryFromSexp for Option<Date> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar Date, got length {}",
                sexp.len()
            )));
        }
        let days = unsafe { *REAL(sexp) };
        if days.is_nan() {
            return Ok(None);
        }
        let days_i64 = days.trunc() as i64;
        let span = Span::new()
            .try_days(days_i64)
            .map_err(|e| SexpError::InvalidValue(format!("jiff days out of range: {e}")))?;
        unix_epoch_date()
            .checked_add(span)
            .map(Some)
            .map_err(|e| SexpError::InvalidValue(format!("jiff Date arithmetic: {e}")))
    }
}

impl IntoR for Option<Date> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        match self {
            Some(d) => d.into_sexp(),
            None => unsafe {
                let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
                Rf_protect(vec);
                *REAL(vec) = f64::NAN;
                vec.set_class(date_class_sexp());
                Rf_unprotect(1);
                vec
            },
        }
    }
}

// endregion

// region: Vec<Date>

impl TryFromSexp for Vec<Date> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &days) in src.iter().enumerate() {
            if days.is_nan() {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Date>",
                    i
                )));
            }
            let days_i64 = days.trunc() as i64;
            let span = Span::new().try_days(days_i64).map_err(|e| {
                SexpError::InvalidValue(format!("jiff days out of range at index {i}: {e}"))
            })?;
            let d = unix_epoch_date().checked_add(span).map_err(|e| {
                SexpError::InvalidValue(format!("jiff Date arithmetic at index {i}: {e}"))
            })?;
            result.push(d);
        }
        Ok(result)
    }
}

impl IntoR for Vec<Date> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, d) in dst.iter_mut().zip(self) {
                let span = d.since(unix_epoch_date()).unwrap_or_default();
                *slot = span.get_days() as f64;
            }
            vec.set_class(date_class_sexp());
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Vec<Option<Date>>

impl TryFromSexp for Vec<Option<Date>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &days) in src.iter().enumerate() {
            if days.is_nan() {
                result.push(None);
            } else {
                let days_i64 = days.trunc() as i64;
                let span = Span::new().try_days(days_i64).map_err(|e| {
                    SexpError::InvalidValue(format!("jiff days out of range at index {i}: {e}"))
                })?;
                let d = unix_epoch_date().checked_add(span).map_err(|e| {
                    SexpError::InvalidValue(format!("jiff Date arithmetic at index {i}: {e}"))
                })?;
                result.push(Some(d));
            }
        }
        Ok(result)
    }
}

impl IntoR for Vec<Option<Date>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, opt) in dst.iter_mut().zip(self) {
                *slot = match opt {
                    Some(d) => {
                        let span = d.since(unix_epoch_date()).unwrap_or_default();
                        span.get_days() as f64
                    }
                    None => f64::NAN,
                };
            }
            vec.set_class(date_class_sexp());
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Zoned <-> POSIXct (tz)

impl TryFromSexp for Zoned {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar POSIXct, got length {}",
                sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::REALSXP,
            }));
        }

        // Read the tzone attribute.
        let tz_name: String = unsafe {
            let tzone_sym = crate::cached_class::tzone_symbol();
            let tzone_attr = sexp.get_attr(tzone_sym);
            if tzone_attr.type_of() == SEXPTYPE::STRSXP && tzone_attr.len() >= 1 {
                let charsxp = tzone_attr.string_elt(0);
                let cstr = std::ffi::CStr::from_ptr(charsxp.r_char());
                cstr.to_string_lossy().into_owned()
            } else {
                String::new()
            }
        };

        let tz = if tz_name.is_empty() || tz_name == "UTC" {
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

impl IntoR for Zoned {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);
            let ts = self.timestamp();
            *REAL(vec) = ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0);
            let iana = self.time_zone().iana_name().unwrap_or("UTC");
            set_posixct_tz(vec, iana);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Option<Zoned>

impl TryFromSexp for Option<Zoned> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar POSIXct, got length {}",
                sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Ok(None);
        }
        Zoned::try_from_sexp(sexp).map(Some)
    }
}

impl IntoR for Option<Zoned> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        match self {
            Some(z) => z.into_sexp(),
            None => unsafe {
                let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
                Rf_protect(vec);
                *REAL(vec) = f64::NAN;
                set_posixct_utc(vec);
                Rf_unprotect(1);
                vec
            },
        }
    }
}

// endregion

// region: Vec<Zoned>

impl TryFromSexp for Vec<Zoned> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }

        // Read the tzone attribute once for the whole vector.
        let tz_name: String = unsafe {
            let tzone_sym = crate::cached_class::tzone_symbol();
            let tzone_attr = sexp.get_attr(tzone_sym);
            if tzone_attr.type_of() == SEXPTYPE::STRSXP && tzone_attr.len() >= 1 {
                let charsxp = tzone_attr.string_elt(0);
                let cstr = std::ffi::CStr::from_ptr(charsxp.r_char());
                cstr.to_string_lossy().into_owned()
            } else {
                String::new()
            }
        };

        let tz = if tz_name.is_empty() || tz_name == "UTC" {
            jiff::tz::TimeZone::UTC
        } else {
            jiff::tz::TimeZone::get(&tz_name)
                .map_err(|e| SexpError::InvalidValue(format!("unknown IANA tz {tz_name:?}: {e}")))?
        };

        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &secs) in src.iter().enumerate() {
            if secs.is_nan() {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Zoned>",
                    i
                )));
            }
            let whole = secs.floor() as i64;
            let fract = secs - secs.floor();
            let nanos = (fract * 1_000_000_000.0) as i32;
            let ts = Timestamp::new(whole, nanos).map_err(|e| {
                SexpError::InvalidValue(format!("jiff Timestamp out of range at index {i}: {e}"))
            })?;
            result.push(ts.to_zoned(tz.clone()));
        }
        Ok(result)
    }
}

impl IntoR for Vec<Zoned> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            // Use the first element's timezone for the vector attribute.
            // Heterogeneous timezones: first tz wins; log a warning if mismatched.
            let first_iana = self
                .first()
                .and_then(|z| z.time_zone().iana_name())
                .unwrap_or("UTC")
                .to_string();

            #[cfg(feature = "log")]
            {
                // Check for heterogeneous timezones.
                if self.len() > 1 {
                    for z in self.iter().skip(1) {
                        let iana = z.time_zone().iana_name().unwrap_or("UTC");
                        if iana != first_iana {
                            log::warn!(
                                "Vec<Zoned>::into_sexp: heterogeneous timezones ({:?} vs {:?}); \
                                 using {:?} for tzone attribute",
                                first_iana,
                                iana,
                                first_iana
                            );
                            break;
                        }
                    }
                }
            }

            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, z) in dst.iter_mut().zip(self) {
                let ts = z.timestamp();
                *slot = ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0);
            }
            set_posixct_tz(vec, &first_iana);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: Vec<Option<Zoned>>

impl TryFromSexp for Vec<Option<Zoned>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }

        let tz_name: String = unsafe {
            let tzone_sym = crate::cached_class::tzone_symbol();
            let tzone_attr = sexp.get_attr(tzone_sym);
            if tzone_attr.type_of() == SEXPTYPE::STRSXP && tzone_attr.len() >= 1 {
                let charsxp = tzone_attr.string_elt(0);
                let cstr = std::ffi::CStr::from_ptr(charsxp.r_char());
                cstr.to_string_lossy().into_owned()
            } else {
                String::new()
            }
        };

        let tz = if tz_name.is_empty() || tz_name == "UTC" {
            jiff::tz::TimeZone::UTC
        } else {
            jiff::tz::TimeZone::get(&tz_name)
                .map_err(|e| SexpError::InvalidValue(format!("unknown IANA tz {tz_name:?}: {e}")))?
        };

        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &secs) in src.iter().enumerate() {
            if secs.is_nan() {
                result.push(None);
            } else {
                let whole = secs.floor() as i64;
                let fract = secs - secs.floor();
                let nanos = (fract * 1_000_000_000.0) as i32;
                let ts = Timestamp::new(whole, nanos).map_err(|e| {
                    SexpError::InvalidValue(format!(
                        "jiff Timestamp out of range at index {i}: {e}"
                    ))
                })?;
                result.push(Some(ts.to_zoned(tz.clone())));
            }
        }
        Ok(result)
    }
}

impl IntoR for Vec<Option<Zoned>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let first_iana = self
                .iter()
                .find_map(|opt| opt.as_ref())
                .and_then(|z| z.time_zone().iana_name())
                .unwrap_or("UTC")
                .to_string();

            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, opt) in dst.iter_mut().zip(self) {
                *slot = match opt {
                    Some(z) => {
                        let ts = z.timestamp();
                        ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
                    }
                    None => f64::NAN,
                };
            }
            set_posixct_tz(vec, &first_iana);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: SignedDuration <-> difftime

fn set_difftime_secs_class(sexp: SEXP) {
    unsafe {
        use crate::ffi::SexpExt as _;
        // Build class = "difftime"
        let class_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        Rf_protect(class_sexp);
        class_sexp.set_string_elt(0, SEXP::charsxp("difftime"));
        sexp.set_class(class_sexp);
        Rf_unprotect(1);

        // Set units = "secs"
        let units_sym = SEXP::symbol("units");
        let units_sexp = Rf_allocVector(SEXPTYPE::STRSXP, 1);
        Rf_protect(units_sexp);
        units_sexp.set_string_elt(0, SEXP::charsxp("secs"));
        sexp.set_attr(units_sym, units_sexp);
        Rf_unprotect(1);
    }
}

impl TryFromSexp for SignedDuration {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar difftime, got length {}",
                sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Err(SexpError::Na(SexpNaError {
                sexp_type: SEXPTYPE::REALSXP,
            }));
        }
        // Accept difftime as seconds regardless of "units" attr for v1.
        let whole = secs.trunc() as i64;
        let frac_nanos = ((secs - secs.trunc()) * 1_000_000_000.0) as i32;
        Ok(SignedDuration::new(whole, frac_nanos))
    }
}

impl IntoR for SignedDuration {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
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

// endregion

// region: Option<SignedDuration> / Vec<SignedDuration> / Vec<Option<SignedDuration>>

impl TryFromSexp for Option<SignedDuration> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual == SEXPTYPE::NILSXP {
            return Ok(None);
        }
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        if sexp.len() != 1 {
            return Err(SexpError::InvalidValue(format!(
                "expected scalar difftime, got length {}",
                sexp.len()
            )));
        }
        let secs = unsafe { *REAL(sexp) };
        if secs.is_nan() {
            return Ok(None);
        }
        let whole = secs.trunc() as i64;
        let frac_nanos = ((secs - secs.trunc()) * 1_000_000_000.0) as i32;
        Ok(Some(SignedDuration::new(whole, frac_nanos)))
    }
}

impl IntoR for Option<SignedDuration> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        match self {
            Some(d) => d.into_sexp(),
            None => unsafe {
                let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
                Rf_protect(vec);
                *REAL(vec) = f64::NAN;
                set_difftime_secs_class(vec);
                Rf_unprotect(1);
                vec
            },
        }
    }
}

impl TryFromSexp for Vec<SignedDuration> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for (i, &secs) in src.iter().enumerate() {
            if secs.is_nan() {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<SignedDuration>",
                    i
                )));
            }
            let whole = secs.trunc() as i64;
            let frac_nanos = ((secs - secs.trunc()) * 1_000_000_000.0) as i32;
            result.push(SignedDuration::new(whole, frac_nanos));
        }
        Ok(result)
    }
}

impl IntoR for Vec<SignedDuration> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, d) in dst.iter_mut().zip(self) {
                *slot = d.as_secs_f64();
            }
            set_difftime_secs_class(vec);
            Rf_unprotect(1);
            vec
        }
    }
}

impl TryFromSexp for Vec<Option<SignedDuration>> {
    type Error = SexpError;

    fn try_from_sexp(sexp: SEXP) -> Result<Self, Self::Error> {
        let actual = sexp.type_of();
        if actual != SEXPTYPE::REALSXP {
            return Err(SexpTypeError {
                expected: SEXPTYPE::REALSXP,
                actual,
            }
            .into());
        }
        let src: &[f64] = unsafe { sexp.as_slice() };
        let mut result = Vec::with_capacity(src.len());
        for &secs in src.iter() {
            if secs.is_nan() {
                result.push(None);
            } else {
                let whole = secs.trunc() as i64;
                let frac_nanos = ((secs - secs.trunc()) * 1_000_000_000.0) as i32;
                result.push(Some(SignedDuration::new(whole, frac_nanos)));
            }
        }
        Ok(result)
    }
}

impl IntoR for Vec<Option<SignedDuration>> {
    type Error = std::convert::Infallible;
    fn try_into_sexp(self) -> Result<SEXP, Self::Error> {
        Ok(self.into_sexp())
    }
    unsafe fn try_into_sexp_unchecked(self) -> Result<SEXP, Self::Error> {
        self.try_into_sexp()
    }
    fn into_sexp(self) -> SEXP {
        unsafe {
            let (vec, dst) = crate::into_r::alloc_r_vector::<f64>(self.len());
            Rf_protect(vec);
            for (slot, opt) in dst.iter_mut().zip(self) {
                *slot = match opt {
                    Some(d) => d.as_secs_f64(),
                    None => f64::NAN,
                };
            }
            set_difftime_secs_class(vec);
            Rf_unprotect(1);
            vec
        }
    }
}

// endregion

// region: RSignedDuration adapter trait

/// Adapter trait for [`jiff::SignedDuration`].
///
/// Provides methods to inspect and manipulate signed durations from R.
pub trait RSignedDuration {
    /// Get the total duration as floating-point seconds.
    fn as_seconds_f64(&self) -> f64;
    /// Get the total duration in milliseconds (truncated to i64).
    fn as_milliseconds(&self) -> i64;
    /// Get the number of whole seconds.
    fn whole_seconds(&self) -> i64;
    /// Get the number of whole minutes.
    fn whole_minutes(&self) -> i64;
    /// Get the number of whole hours.
    fn whole_hours(&self) -> i64;
    /// Get the number of whole days.
    fn whole_days(&self) -> i64;
    /// Get the subsecond nanoseconds component.
    fn subsec_nanoseconds(&self) -> i32;
    /// Check if the duration is negative.
    fn is_negative(&self) -> bool;
    /// Check if the duration is zero.
    fn is_zero(&self) -> bool;
    /// Get the absolute value of this duration.
    fn abs(&self) -> SignedDuration;
}

impl RSignedDuration for SignedDuration {
    fn as_seconds_f64(&self) -> f64 {
        self.as_secs_f64()
    }
    fn as_milliseconds(&self) -> i64 {
        // as_millis() returns i128; clamp to i64 range (saturating, not truncating).
        self.as_millis().clamp(i64::MIN as i128, i64::MAX as i128) as i64
    }
    fn whole_seconds(&self) -> i64 {
        self.as_secs()
    }
    fn whole_minutes(&self) -> i64 {
        self.as_secs() / 60
    }
    fn whole_hours(&self) -> i64 {
        self.as_secs() / 3_600
    }
    fn whole_days(&self) -> i64 {
        self.as_secs() / 86_400
    }
    fn subsec_nanoseconds(&self) -> i32 {
        self.subsec_nanos()
    }
    fn is_negative(&self) -> bool {
        SignedDuration::is_negative(self)
    }
    fn is_zero(&self) -> bool {
        SignedDuration::is_zero(self)
    }
    fn abs(&self) -> SignedDuration {
        SignedDuration::abs(*self)
    }
}

// endregion

// region: RSpan adapter trait

/// Adapter trait for [`jiff::Span`].
///
/// Provides component accessors and arithmetic helpers for `Span` values from R.
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
    // Return types from jiff 0.2:
    //   get_years → i16; get_months/weeks/days/hours → i32
    //   get_minutes/seconds/milliseconds/microseconds/nanoseconds → i64
    fn get_years(&self) -> i64 {
        i64::from(self.get_years())
    }
    fn get_months(&self) -> i64 {
        i64::from(self.get_months())
    }
    fn get_weeks(&self) -> i64 {
        i64::from(self.get_weeks())
    }
    fn get_days(&self) -> i64 {
        i64::from(self.get_days())
    }
    fn get_hours(&self) -> i64 {
        i64::from(self.get_hours())
    }
    fn get_minutes(&self) -> i64 {
        self.get_minutes()
    }
    fn get_seconds(&self) -> i64 {
        self.get_seconds()
    }
    fn get_milliseconds(&self) -> i64 {
        self.get_milliseconds()
    }
    fn get_microseconds(&self) -> i64 {
        self.get_microseconds()
    }
    fn get_nanoseconds(&self) -> i64 {
        self.get_nanoseconds()
    }
    fn is_zero(&self) -> bool {
        Span::is_zero(*self)
    }
    fn is_negative(&self) -> bool {
        Span::is_negative(*self)
    }
    fn negate(&self) -> Span {
        Span::negate(*self)
    }
    fn abs(&self) -> Span {
        Span::abs(*self)
    }
}

// endregion

// region: RDateTime adapter trait

/// Adapter trait for [`jiff::civil::DateTime`].
///
/// Provides component accessors and tz-conversion for civil datetimes from R.
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
    fn year(&self) -> i32 {
        i32::from(DateTime::year(*self))
    }
    fn month(&self) -> i32 {
        i32::from(DateTime::month(*self))
    }
    fn day(&self) -> i32 {
        i32::from(DateTime::day(*self))
    }
    fn hour(&self) -> i32 {
        i32::from(DateTime::hour(*self))
    }
    fn minute(&self) -> i32 {
        i32::from(DateTime::minute(*self))
    }
    fn second(&self) -> i32 {
        i32::from(DateTime::second(*self))
    }
    fn subsec_nanosecond(&self) -> i32 {
        DateTime::subsec_nanosecond(*self)
    }
    fn to_date(&self) -> Date {
        DateTime::date(*self)
    }
    fn to_time(&self) -> Time {
        DateTime::time(*self)
    }
    fn in_tz(&self, iana: &str) -> Result<Zoned, String> {
        DateTime::in_tz(*self, iana).map_err(|e| e.to_string())
    }
}

// endregion

// region: RTime adapter trait

/// Adapter trait for [`jiff::civil::Time`].
///
/// Provides component accessors and date-combination for civil times from R.
pub trait RTime {
    fn hour(&self) -> i32;
    fn minute(&self) -> i32;
    fn second(&self) -> i32;
    fn subsec_nanosecond(&self) -> i32;
    fn on(&self, year: i16, month: i8, day: i8) -> DateTime;
}

impl RTime for Time {
    fn hour(&self) -> i32 {
        i32::from(Time::hour(*self))
    }
    fn minute(&self) -> i32 {
        i32::from(Time::minute(*self))
    }
    fn second(&self) -> i32 {
        i32::from(Time::second(*self))
    }
    fn subsec_nanosecond(&self) -> i32 {
        Time::subsec_nanosecond(*self)
    }
    fn on(&self, year: i16, month: i8, day: i8) -> DateTime {
        Time::on(*self, year, month, day)
    }
}

// endregion

// region: RTimestamp adapter trait

/// Adapter trait for [`jiff::Timestamp`].
///
/// Provides component accessors and tz conversion for timestamps from R.
pub trait RTimestamp {
    fn as_second(&self) -> i64;
    fn as_millisecond(&self) -> i64;
    fn subsec_nanosecond(&self) -> i32;
    fn to_zoned_in(&self, iana: &str) -> Result<Zoned, String>;
    fn strftime(&self, fmt: &str) -> String;
}

impl RTimestamp for Timestamp {
    fn as_second(&self) -> i64 {
        Timestamp::as_second(*self)
    }
    fn as_millisecond(&self) -> i64 {
        Timestamp::as_millisecond(*self)
    }
    fn subsec_nanosecond(&self) -> i32 {
        Timestamp::subsec_nanosecond(*self)
    }
    fn to_zoned_in(&self, iana: &str) -> Result<Zoned, String> {
        Timestamp::in_tz(*self, iana).map_err(|e| e.to_string())
    }
    fn strftime(&self, fmt: &str) -> String {
        Timestamp::strftime(self, fmt).to_string()
    }
}

// endregion

// region: RZoned adapter trait

/// Adapter trait for [`jiff::Zoned`].
///
/// Provides component accessors, tz conversion, and formatting for zoned datetimes from R.
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
    fn iana_name(&self) -> Option<String> {
        self.time_zone().iana_name().map(str::to_string)
    }
    fn year(&self) -> i32 {
        i32::from(Zoned::year(self))
    }
    fn month(&self) -> i32 {
        i32::from(Zoned::month(self))
    }
    fn day(&self) -> i32 {
        i32::from(Zoned::day(self))
    }
    fn hour(&self) -> i32 {
        i32::from(Zoned::hour(self))
    }
    fn minute(&self) -> i32 {
        i32::from(Zoned::minute(self))
    }
    fn second(&self) -> i32 {
        i32::from(Zoned::second(self))
    }
    fn in_tz(&self, iana: &str) -> Result<Zoned, String> {
        // Calling the inherent method directly to avoid confusion with the trait method.
        <Zoned>::in_tz(self, iana).map_err(|e| e.to_string())
    }
    fn start_of_day(&self) -> Result<Zoned, String> {
        <Zoned>::start_of_day(self).map_err(|e| e.to_string())
    }
    fn strftime(&self, fmt: &str) -> String {
        Zoned::strftime(self, fmt).to_string()
    }
}

// endregion

// region: RDate adapter trait

/// Adapter trait for [`jiff::civil::Date`].
///
/// Provides component accessors, calendar helpers, and formatting for civil dates from R.
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
    fn year(&self) -> i32 {
        i32::from(Date::year(*self))
    }
    fn month(&self) -> i32 {
        i32::from(Date::month(*self))
    }
    fn day(&self) -> i32 {
        i32::from(Date::day(*self))
    }
    /// Day of week as 1 = Monday … 7 = Sunday (ISO 8601).
    fn weekday(&self) -> i32 {
        i32::from(Date::weekday(*self).to_monday_one_offset())
    }
    fn day_of_year(&self) -> i32 {
        i32::from(Date::day_of_year(*self))
    }
    fn first_of_month(&self) -> Date {
        Date::first_of_month(*self)
    }
    fn last_of_month(&self) -> Date {
        Date::last_of_month(*self)
    }
    fn tomorrow(&self) -> Result<Date, String> {
        Date::tomorrow(*self).map_err(|e| e.to_string())
    }
    fn yesterday(&self) -> Result<Date, String> {
        Date::yesterday(*self).map_err(|e| e.to_string())
    }
    fn strftime(&self, fmt: &str) -> String {
        Date::strftime(self, fmt).to_string()
    }
}

// endregion

// region: ALTREP lazy vector for Vec<Timestamp>

use std::sync::Arc;

use crate::altrep_data::{AltRealData, AltrepLen};

/// ALTREP-backed lazy vector of `Timestamp`s.
///
/// Materialized on element access as seconds-since-epoch f64. Registered as a
/// `REALSXP` ALTREP vector with class `"JiffTimestampVec"`.
#[derive(miniextendr_macros::AltrepReal)]
#[altrep(class = "JiffTimestampVec", manual)]
pub struct JiffTimestampVec {
    /// Shared ownership of the timestamps.
    pub data: Arc<Vec<Timestamp>>,
}

impl JiffTimestampVec {
    /// Create a new ALTREP-backed timestamp vector.
    pub fn new(data: Vec<Timestamp>) -> Self {
        Self {
            data: Arc::new(data),
        }
    }
}

impl AltrepLen for JiffTimestampVec {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl AltRealData for JiffTimestampVec {
    fn elt(&self, i: usize) -> f64 {
        let ts = &self.data[i];
        ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0)
    }
}

// endregion

// region: vctrs rcrd constructors

#[cfg(feature = "vctrs")]
pub mod vctrs_support {
    use super::*;
    use crate::list::List;
    use crate::vctrs::new_rcrd;

    /// Helper: allocate an INTSXP column of length `n`, fill it, and return an
    /// [`OwnedProtect`] guard that keeps the SEXP protected until the caller
    /// drops it.  The caller **must** hold every guard alive until after
    /// `List::from_raw_values` returns, because `from_raw_values` calls
    /// `Rf_allocVector(VECSXP, …)` which can trigger GC.
    unsafe fn alloc_int_col(
        n: usize,
        extract: impl Fn(usize) -> i32,
    ) -> crate::gc_protect::OwnedProtect {
        let (col, dst) = unsafe { crate::into_r::alloc_r_vector::<i32>(n) };
        // SAFETY: col was just allocated on the R thread; guard keeps it
        // rooted for the caller's entire alloc-and-list-build sequence.
        let guard = unsafe { crate::gc_protect::OwnedProtect::new(col) };
        for (i, slot) in dst.iter_mut().enumerate() {
            *slot = extract(i);
        }
        guard
    }

    /// Convert a slice of `Span`s into a vctrs `jiff_span` rcrd SEXP.
    ///
    /// Fields: `years`, `months`, `weeks`, `days`, `hours`, `minutes`, `seconds`,
    /// `milliseconds`, `microseconds`, `nanoseconds` — all as integer (`INTSXP`).
    pub fn span_vec_to_rcrd(spans: &[Span]) -> SEXP {
        let n = spans.len();
        // get_years → i16; get_months/weeks/days/hours → i32
        // get_minutes/seconds/ms/us/ns → i64 (truncate to i32; in practice always fits)
        //
        // GC-SAFETY: all guards are held alive until after List::from_raw_values
        // returns. from_raw_values calls Rf_allocVector(VECSXP, …) which can
        // trigger GC — so every column must stay protected through that call.
        let g_years = unsafe { alloc_int_col(n, |i| i32::from(spans[i].get_years())) };
        let g_months = unsafe { alloc_int_col(n, |i| spans[i].get_months()) };
        let g_weeks = unsafe { alloc_int_col(n, |i| spans[i].get_weeks()) };
        let g_days = unsafe { alloc_int_col(n, |i| spans[i].get_days()) };
        let g_hours = unsafe { alloc_int_col(n, |i| spans[i].get_hours()) };
        let g_minutes = unsafe { alloc_int_col(n, |i| spans[i].get_minutes() as i32) };
        let g_seconds = unsafe { alloc_int_col(n, |i| spans[i].get_seconds() as i32) };
        let g_milliseconds = unsafe { alloc_int_col(n, |i| spans[i].get_milliseconds() as i32) };
        let g_microseconds = unsafe { alloc_int_col(n, |i| spans[i].get_microseconds() as i32) };
        let g_nanoseconds = unsafe { alloc_int_col(n, |i| spans[i].get_nanoseconds() as i32) };

        let list = List::from_raw_values(vec![
            g_years.get(),
            g_months.get(),
            g_weeks.get(),
            g_days.get(),
            g_hours.get(),
            g_minutes.get(),
            g_seconds.get(),
            g_milliseconds.get(),
            g_microseconds.get(),
            g_nanoseconds.get(),
        ])
        // guards drop here — safe because the list VECSXP now holds references
        // to each column as a child object, keeping them alive via R's GC graph.
        .set_names_str(&[
            "years",
            "months",
            "weeks",
            "days",
            "hours",
            "minutes",
            "seconds",
            "milliseconds",
            "microseconds",
            "nanoseconds",
        ]);

        new_rcrd(list, &["jiff_span"], &[])
            .expect("new_rcrd should not fail for well-formed span fields")
    }

    /// Convert a slice of `Zoned`s into a vctrs `jiff_zoned` rcrd SEXP.
    ///
    /// Fields: `timestamp` (REALSXP, seconds since epoch), `tz` (STRSXP, IANA name).
    pub fn zoned_vec_to_rcrd(zones: &[Zoned]) -> SEXP {
        let n = zones.len();
        let (ts_col, ts_dst) = unsafe { crate::into_r::alloc_r_vector::<f64>(n) };
        // GC-SAFETY: _ts_guard keeps ts_col rooted until after from_raw_values.
        let _ts_guard = unsafe { crate::gc_protect::OwnedProtect::new(ts_col) };

        // STRSXP column — allocate and protect via OwnedProtect so it outlives
        // the List::from_raw_values call (which allocates a VECSXP and can GC).
        let tz_col = unsafe { Rf_allocVector(SEXPTYPE::STRSXP, n as crate::ffi::R_xlen_t) };
        // GC-SAFETY: _tz_guard keeps tz_col rooted until after from_raw_values.
        let _tz_guard = unsafe { crate::gc_protect::OwnedProtect::new(tz_col) };

        for (i, z) in zones.iter().enumerate() {
            let ts = z.timestamp();
            ts_dst[i] = ts.as_second() as f64 + (ts.subsec_nanosecond() as f64 / 1_000_000_000.0);
            let iana = z.time_zone().iana_name().unwrap_or("UTC");
            tz_col.set_string_elt(i as crate::ffi::R_xlen_t, SEXP::charsxp(iana));
        }
        // Guards (_ts_guard, _tz_guard) drop after from_raw_values returns —
        // at that point the VECSXP owns both columns via SET_VECTOR_ELT, keeping
        // them alive through R's GC object graph.
        let list = List::from_raw_values(vec![ts_col, tz_col]).set_names_str(&["timestamp", "tz"]);

        new_rcrd(list, &["jiff_zoned"], &[])
            .expect("new_rcrd should not fail for well-formed zoned fields")
    }

    /// Convert a slice of `DateTime`s into a vctrs `jiff_datetime` rcrd SEXP.
    ///
    /// Fields: `year`, `month`, `day`, `hour`, `minute`, `second`, `nanosecond`.
    pub fn datetime_vec_to_rcrd(dts: &[DateTime]) -> SEXP {
        let n = dts.len();
        // GC-SAFETY: guards kept alive until after from_raw_values (see span_vec_to_rcrd).
        let g_year = unsafe { alloc_int_col(n, |i| i32::from(dts[i].year())) };
        let g_month = unsafe { alloc_int_col(n, |i| i32::from(dts[i].month())) };
        let g_day = unsafe { alloc_int_col(n, |i| i32::from(dts[i].day())) };
        let g_hour = unsafe { alloc_int_col(n, |i| i32::from(dts[i].hour())) };
        let g_minute = unsafe { alloc_int_col(n, |i| i32::from(dts[i].minute())) };
        let g_second = unsafe { alloc_int_col(n, |i| i32::from(dts[i].second())) };
        let g_nanosecond = unsafe { alloc_int_col(n, |i| dts[i].subsec_nanosecond()) };

        let list = List::from_raw_values(vec![
            g_year.get(),
            g_month.get(),
            g_day.get(),
            g_hour.get(),
            g_minute.get(),
            g_second.get(),
            g_nanosecond.get(),
        ])
        .set_names_str(&[
            "year",
            "month",
            "day",
            "hour",
            "minute",
            "second",
            "nanosecond",
        ]);

        new_rcrd(list, &["jiff_datetime"], &[])
            .expect("new_rcrd should not fail for well-formed datetime fields")
    }

    /// Convert a slice of `Time`s into a vctrs `jiff_time` rcrd SEXP.
    ///
    /// Fields: `hour`, `minute`, `second`, `nanosecond`.
    pub fn time_vec_to_rcrd(times: &[Time]) -> SEXP {
        let n = times.len();
        // GC-SAFETY: guards kept alive until after from_raw_values (see span_vec_to_rcrd).
        let g_hour = unsafe { alloc_int_col(n, |i| i32::from(times[i].hour())) };
        let g_minute = unsafe { alloc_int_col(n, |i| i32::from(times[i].minute())) };
        let g_second = unsafe { alloc_int_col(n, |i| i32::from(times[i].second())) };
        let g_nanosecond = unsafe { alloc_int_col(n, |i| times[i].subsec_nanosecond()) };

        let list = List::from_raw_values(vec![
            g_hour.get(),
            g_minute.get(),
            g_second.get(),
            g_nanosecond.get(),
        ])
        .set_names_str(&["hour", "minute", "second", "nanosecond"]);

        new_rcrd(list, &["jiff_time"], &[])
            .expect("new_rcrd should not fail for well-formed time fields")
    }
}

/// Convert a slice of `Span`s into a vctrs `jiff_span` rcrd SEXP.
#[cfg(feature = "vctrs")]
pub fn span_vec_to_rcrd(spans: &[Span]) -> SEXP {
    vctrs_support::span_vec_to_rcrd(spans)
}

/// Convert a slice of `Zoned`s into a vctrs `jiff_zoned` rcrd SEXP.
#[cfg(feature = "vctrs")]
pub fn zoned_vec_to_rcrd(zones: &[Zoned]) -> SEXP {
    vctrs_support::zoned_vec_to_rcrd(zones)
}

/// Convert a slice of `DateTime`s into a vctrs `jiff_datetime` rcrd SEXP.
#[cfg(feature = "vctrs")]
pub fn datetime_vec_to_rcrd(dts: &[DateTime]) -> SEXP {
    vctrs_support::datetime_vec_to_rcrd(dts)
}

/// Convert a slice of `Time`s into a vctrs `jiff_time` rcrd SEXP.
#[cfg(feature = "vctrs")]
pub fn time_vec_to_rcrd(times: &[Time]) -> SEXP {
    vctrs_support::time_vec_to_rcrd(times)
}

// endregion
