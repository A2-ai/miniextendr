//! Integration with the `time` crate.
//!
//! Provides conversions between R date/time types and `time` crate types.
//!
//! | R Type | Rust Type | Notes |
//! |--------|-----------|-------|
//! | `POSIXct` | `OffsetDateTime` | Seconds since epoch + timezone |
//! | `Date` | `time::Date` | Days since 1970-01-01 |
//!
//! # Features
//!
//! Enable this module with the `time` feature:
//!
//! ```toml
//! [dependencies]
//! miniextendr-api = { version = "0.1", features = ["time"] }
//! ```
//!
//! # Fractional Seconds Policy
//!
//! R's POSIXct stores fractional seconds as floating-point. When converting to Rust:
//! - Fractional seconds are **truncated** (not rounded) to nanoseconds
//! - This matches typical timestamp handling expectations
//!
//! # Timezone Handling
//!
//! - POSIXct with `tzone = "UTC"` maps to UTC offset
//! - POSIXct without tzone or with empty tzone defaults to UTC
//! - Other timezones are converted assuming UTC (R doesn't store actual offset)
//!
//! **Note:** R's timezone handling is complex. For reliable timezone support,
//! consider storing timestamps as UTC and converting in R.
//!
//! # Example
//!
//! ```ignore
//! use time::OffsetDateTime;
//!
//! #[miniextendr]
//! fn now() -> OffsetDateTime {
//!     OffsetDateTime::now_utc()
//! }
//!
//! #[miniextendr]
//! fn days_since(date: time::Date) -> i32 {
//!     let today = OffsetDateTime::now_utc().date();
//!     (today - date).whole_days() as i32
//! }
//! ```

pub use time::{Date, OffsetDateTime};

use crate::ffi::{
    CE_UTF8, REAL, Rf_allocVector, Rf_install, Rf_mkCharLenCE, Rf_mkString, Rf_protect,
    Rf_setAttrib, Rf_unprotect, SET_STRING_ELT, SEXP, SEXPTYPE, SexpExt,
};
use crate::from_r::{SexpError, SexpNaError, SexpTypeError, TryFromSexp};
use crate::into_r::IntoR;

/// Unix epoch as an OffsetDateTime constant.
const UNIX_EPOCH: OffsetDateTime = time::macros::datetime!(1970-01-01 0:00 UTC);

/// Unix epoch as a Date constant.
const UNIX_EPOCH_DATE: Date = time::macros::date!(1970 - 01 - 01);

// =============================================================================
// OffsetDateTime <-> POSIXct
// =============================================================================

impl TryFromSexp for OffsetDateTime {
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

        // Split into whole seconds and fractional nanoseconds
        let whole_secs = secs.trunc() as i64;
        let nanos = ((secs.fract().abs()) * 1_000_000_000.0) as i32;

        // Calculate from epoch
        let duration = time::Duration::new(whole_secs, nanos);
        UNIX_EPOCH
            .checked_add(duration)
            .ok_or_else(|| SexpError::InvalidValue("timestamp out of range".to_string()))
    }
}

impl IntoR for OffsetDateTime {
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);

            // Calculate seconds since epoch
            let duration = self - UNIX_EPOCH;
            let secs = duration.whole_seconds() as f64
                + (duration.subsec_nanoseconds() as f64 / 1_000_000_000.0);
            *REAL(vec) = secs;

            // Set class = c("POSIXct", "POSIXt")
            let class_vec = Rf_allocVector(SEXPTYPE::STRSXP, 2);
            Rf_protect(class_vec);
            let posixct = Rf_mkCharLenCE("POSIXct\0".as_ptr().cast(), 7, CE_UTF8);
            let posixt = Rf_mkCharLenCE("POSIXt\0".as_ptr().cast(), 6, CE_UTF8);
            SET_STRING_ELT(class_vec, 0, posixct);
            SET_STRING_ELT(class_vec, 1, posixt);
            let class_sym = Rf_install("class\0".as_ptr().cast());
            Rf_setAttrib(vec, class_sym, class_vec);

            // Set tzone = "UTC" (we always output as UTC)
            let tzone = Rf_mkString("UTC\0".as_ptr().cast());
            let tzone_sym = Rf_install("tzone\0".as_ptr().cast());
            Rf_setAttrib(vec, tzone_sym, tzone);

            Rf_unprotect(2);
            vec
        }
    }
}

// =============================================================================
// Option<OffsetDateTime>
// =============================================================================

impl TryFromSexp for Option<OffsetDateTime> {
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
            return Ok(None);
        }

        let whole_secs = secs.trunc() as i64;
        let nanos = ((secs.fract().abs()) * 1_000_000_000.0) as i32;
        let duration = time::Duration::new(whole_secs, nanos);

        UNIX_EPOCH
            .checked_add(duration)
            .map(Some)
            .ok_or_else(|| SexpError::InvalidValue("timestamp out of range".to_string()))
    }
}

impl IntoR for Option<OffsetDateTime> {
    fn into_sexp(self) -> SEXP {
        match self {
            Some(dt) => dt.into_sexp(),
            None => unsafe {
                let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
                Rf_protect(vec);
                *REAL(vec) = f64::NAN;

                // Set class = c("POSIXct", "POSIXt")
                let class_vec = Rf_allocVector(SEXPTYPE::STRSXP, 2);
                Rf_protect(class_vec);
                let posixct = Rf_mkCharLenCE("POSIXct\0".as_ptr().cast(), 7, CE_UTF8);
                let posixt = Rf_mkCharLenCE("POSIXt\0".as_ptr().cast(), 6, CE_UTF8);
                SET_STRING_ELT(class_vec, 0, posixct);
                SET_STRING_ELT(class_vec, 1, posixt);
                let class_sym = Rf_install("class\0".as_ptr().cast());
                Rf_setAttrib(vec, class_sym, class_vec);

                let tzone = Rf_mkString("UTC\0".as_ptr().cast());
                let tzone_sym = Rf_install("tzone\0".as_ptr().cast());
                Rf_setAttrib(vec, tzone_sym, tzone);

                Rf_unprotect(2);
                vec
            },
        }
    }
}

// =============================================================================
// Vec<OffsetDateTime>
// =============================================================================

impl TryFromSexp for Vec<OffsetDateTime> {
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

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);
        let ptr = unsafe { REAL(sexp) };

        for i in 0..len {
            let secs = unsafe { *ptr.add(i) };
            if secs.is_nan() {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<OffsetDateTime>",
                    i
                )));
            }

            let whole_secs = secs.trunc() as i64;
            let nanos = ((secs.fract().abs()) * 1_000_000_000.0) as i32;
            let duration = time::Duration::new(whole_secs, nanos);

            let dt = UNIX_EPOCH.checked_add(duration).ok_or_else(|| {
                SexpError::InvalidValue(format!("timestamp out of range at index {}", i))
            })?;
            result.push(dt);
        }

        Ok(result)
    }
}

impl IntoR for Vec<OffsetDateTime> {
    fn into_sexp(self) -> SEXP {
        unsafe {
            let n = self.len();
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, n as crate::ffi::R_xlen_t);
            Rf_protect(vec);

            let ptr = REAL(vec);
            for (i, dt) in self.into_iter().enumerate() {
                let duration = dt - UNIX_EPOCH;
                let secs = duration.whole_seconds() as f64
                    + (duration.subsec_nanoseconds() as f64 / 1_000_000_000.0);
                *ptr.add(i) = secs;
            }

            // Set class = c("POSIXct", "POSIXt")
            let class_vec = Rf_allocVector(SEXPTYPE::STRSXP, 2);
            Rf_protect(class_vec);
            let posixct = Rf_mkCharLenCE("POSIXct\0".as_ptr().cast(), 7, CE_UTF8);
            let posixt = Rf_mkCharLenCE("POSIXt\0".as_ptr().cast(), 6, CE_UTF8);
            SET_STRING_ELT(class_vec, 0, posixct);
            SET_STRING_ELT(class_vec, 1, posixt);
            let class_sym = Rf_install("class\0".as_ptr().cast());
            Rf_setAttrib(vec, class_sym, class_vec);

            let tzone = Rf_mkString("UTC\0".as_ptr().cast());
            let tzone_sym = Rf_install("tzone\0".as_ptr().cast());
            Rf_setAttrib(vec, tzone_sym, tzone);

            Rf_unprotect(2);
            vec
        }
    }
}

// =============================================================================
// Vec<Option<OffsetDateTime>>
// =============================================================================

impl TryFromSexp for Vec<Option<OffsetDateTime>> {
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

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);
        let ptr = unsafe { REAL(sexp) };

        for i in 0..len {
            let secs = unsafe { *ptr.add(i) };
            if secs.is_nan() {
                result.push(None);
            } else {
                let whole_secs = secs.trunc() as i64;
                let nanos = ((secs.fract().abs()) * 1_000_000_000.0) as i32;
                let duration = time::Duration::new(whole_secs, nanos);

                let dt = UNIX_EPOCH.checked_add(duration).ok_or_else(|| {
                    SexpError::InvalidValue(format!("timestamp out of range at index {}", i))
                })?;
                result.push(Some(dt));
            }
        }

        Ok(result)
    }
}

impl IntoR for Vec<Option<OffsetDateTime>> {
    fn into_sexp(self) -> SEXP {
        unsafe {
            let n = self.len();
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, n as crate::ffi::R_xlen_t);
            Rf_protect(vec);

            let ptr = REAL(vec);
            for (i, opt) in self.into_iter().enumerate() {
                match opt {
                    Some(dt) => {
                        let duration = dt - UNIX_EPOCH;
                        let secs = duration.whole_seconds() as f64
                            + (duration.subsec_nanoseconds() as f64 / 1_000_000_000.0);
                        *ptr.add(i) = secs;
                    }
                    None => {
                        *ptr.add(i) = f64::NAN;
                    }
                }
            }

            // Set class = c("POSIXct", "POSIXt")
            let class_vec = Rf_allocVector(SEXPTYPE::STRSXP, 2);
            Rf_protect(class_vec);
            let posixct = Rf_mkCharLenCE("POSIXct\0".as_ptr().cast(), 7, CE_UTF8);
            let posixt = Rf_mkCharLenCE("POSIXt\0".as_ptr().cast(), 6, CE_UTF8);
            SET_STRING_ELT(class_vec, 0, posixct);
            SET_STRING_ELT(class_vec, 1, posixt);
            let class_sym = Rf_install("class\0".as_ptr().cast());
            Rf_setAttrib(vec, class_sym, class_vec);

            let tzone = Rf_mkString("UTC\0".as_ptr().cast());
            let tzone_sym = Rf_install("tzone\0".as_ptr().cast());
            Rf_setAttrib(vec, tzone_sym, tzone);

            Rf_unprotect(2);
            vec
        }
    }
}

// =============================================================================
// Date <-> R Date
// =============================================================================

impl TryFromSexp for Date {
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
        let duration = time::Duration::days(days_i64);

        UNIX_EPOCH_DATE
            .checked_add(duration)
            .ok_or_else(|| SexpError::InvalidValue("date out of range".to_string()))
    }
}

impl IntoR for Date {
    fn into_sexp(self) -> SEXP {
        unsafe {
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
            Rf_protect(vec);

            // Calculate days since epoch
            let days = (self - UNIX_EPOCH_DATE).whole_days() as f64;
            *REAL(vec) = days;

            // Set class = "Date"
            let class_sym = Rf_install("class\0".as_ptr().cast());
            let date_class = Rf_mkString("Date\0".as_ptr().cast());
            Rf_setAttrib(vec, class_sym, date_class);

            Rf_unprotect(1);
            vec
        }
    }
}

// =============================================================================
// Option<Date>
// =============================================================================

impl TryFromSexp for Option<Date> {
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
                "expected scalar Date, got length {}",
                sexp.len()
            )));
        }

        let days = unsafe { *REAL(sexp) };
        if days.is_nan() {
            return Ok(None);
        }

        let days_i64 = days.trunc() as i64;
        let duration = time::Duration::days(days_i64);

        UNIX_EPOCH_DATE
            .checked_add(duration)
            .map(Some)
            .ok_or_else(|| SexpError::InvalidValue("date out of range".to_string()))
    }
}

impl IntoR for Option<Date> {
    fn into_sexp(self) -> SEXP {
        match self {
            Some(d) => d.into_sexp(),
            None => unsafe {
                let vec = Rf_allocVector(SEXPTYPE::REALSXP, 1);
                Rf_protect(vec);
                *REAL(vec) = f64::NAN;

                let class_sym = Rf_install("class\0".as_ptr().cast());
                let date_class = Rf_mkString("Date\0".as_ptr().cast());
                Rf_setAttrib(vec, class_sym, date_class);

                Rf_unprotect(1);
                vec
            },
        }
    }
}

// =============================================================================
// Vec<Date>
// =============================================================================

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

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);
        let ptr = unsafe { REAL(sexp) };

        for i in 0..len {
            let days = unsafe { *ptr.add(i) };
            if days.is_nan() {
                return Err(SexpError::InvalidValue(format!(
                    "NA at index {} not allowed for Vec<Date>",
                    i
                )));
            }

            let days_i64 = days.trunc() as i64;
            let duration = time::Duration::days(days_i64);

            let d = UNIX_EPOCH_DATE.checked_add(duration).ok_or_else(|| {
                SexpError::InvalidValue(format!("date out of range at index {}", i))
            })?;
            result.push(d);
        }

        Ok(result)
    }
}

impl IntoR for Vec<Date> {
    fn into_sexp(self) -> SEXP {
        unsafe {
            let n = self.len();
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, n as crate::ffi::R_xlen_t);
            Rf_protect(vec);

            let ptr = REAL(vec);
            for (i, d) in self.into_iter().enumerate() {
                let days = (d - UNIX_EPOCH_DATE).whole_days() as f64;
                *ptr.add(i) = days;
            }

            let class_sym = Rf_install("class\0".as_ptr().cast());
            let date_class = Rf_mkString("Date\0".as_ptr().cast());
            Rf_setAttrib(vec, class_sym, date_class);

            Rf_unprotect(1);
            vec
        }
    }
}

// =============================================================================
// Vec<Option<Date>>
// =============================================================================

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

        let len = sexp.len();
        let mut result = Vec::with_capacity(len);
        let ptr = unsafe { REAL(sexp) };

        for i in 0..len {
            let days = unsafe { *ptr.add(i) };
            if days.is_nan() {
                result.push(None);
            } else {
                let days_i64 = days.trunc() as i64;
                let duration = time::Duration::days(days_i64);

                let d = UNIX_EPOCH_DATE.checked_add(duration).ok_or_else(|| {
                    SexpError::InvalidValue(format!("date out of range at index {}", i))
                })?;
                result.push(Some(d));
            }
        }

        Ok(result)
    }
}

impl IntoR for Vec<Option<Date>> {
    fn into_sexp(self) -> SEXP {
        unsafe {
            let n = self.len();
            let vec = Rf_allocVector(SEXPTYPE::REALSXP, n as crate::ffi::R_xlen_t);
            Rf_protect(vec);

            let ptr = REAL(vec);
            for (i, opt) in self.into_iter().enumerate() {
                match opt {
                    Some(d) => {
                        let days = (d - UNIX_EPOCH_DATE).whole_days() as f64;
                        *ptr.add(i) = days;
                    }
                    None => {
                        *ptr.add(i) = f64::NAN;
                    }
                }
            }

            let class_sym = Rf_install("class\0".as_ptr().cast());
            let date_class = Rf_mkString("Date\0".as_ptr().cast());
            Rf_setAttrib(vec, class_sym, date_class);

            Rf_unprotect(1);
            vec
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_constants_correct() {
        assert_eq!(UNIX_EPOCH.year(), 1970);
        assert_eq!(UNIX_EPOCH.month(), time::Month::January);
        assert_eq!(UNIX_EPOCH.day(), 1);

        assert_eq!(UNIX_EPOCH_DATE.year(), 1970);
        assert_eq!(UNIX_EPOCH_DATE.month(), time::Month::January);
        assert_eq!(UNIX_EPOCH_DATE.day(), 1);
    }

    #[test]
    fn date_arithmetic() {
        let d = UNIX_EPOCH_DATE;
        let d2 = d.checked_add(time::Duration::days(365)).unwrap();
        assert_eq!(d2.year(), 1971);
    }

    #[test]
    fn datetime_arithmetic() {
        let dt = UNIX_EPOCH;
        let dt2 = dt.checked_add(time::Duration::hours(24)).unwrap();
        assert_eq!(dt2.day(), 2);
    }
}
