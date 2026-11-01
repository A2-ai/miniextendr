//! Per-string encodings in conversions and string reads.
//!
//! A string marked as bytes (or one that is not valid UTF-8) is refused
//! through the conversion's `Err`, every element of a vector in one error,
//! never a panic. A Latin-1 string is translated by every read:
//! `SexpExt::string_elt_str` and `r_char_str` included, which panic, naming
//! the element, only on a string that is not text.
mod r_test_utils;

use std::borrow::Cow;
use std::panic::{AssertUnwindSafe, catch_unwind};

use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::prelude::{SEXP, SexpExt};
use miniextendr_api::sexp_types::cetype_t;
use miniextendr_api::sys::{Rf_allocVector, Rf_mkCharLenCE};
use miniextendr_api::{OwnedProtect, SEXPTYPE};

const LATIN1_DEV: &[u8] = b"D\xe9V";
const MU_G: &[u8] = "\u{b5}g".as_bytes();
const N_TILDE: &[u8] = "\u{f1}".as_bytes();

/// A protected STRSXP of `(bytes, encoding)` elements; `None` is `NA`.
unsafe fn strsxp(elements: &[Option<(&[u8], cetype_t)>]) -> OwnedProtect {
    unsafe {
        let n = isize::try_from(elements.len()).unwrap();
        let x = OwnedProtect::new(Rf_allocVector(SEXPTYPE::STRSXP, n));
        for (i, element) in (0..n).zip(elements) {
            let charsxp = match element {
                Some((bytes, encoding)) => Rf_mkCharLenCE(
                    bytes.as_ptr().cast(),
                    i32::try_from(bytes.len()).unwrap(),
                    *encoding,
                ),
                None => SEXP::na_string(),
            };
            x.get().set_string_elt(i, charsxp);
        }
        x
    }
}

/// The message of an [`SexpError::InvalidValue`], failing on any other result.
fn invalid_value<T: std::fmt::Debug>(result: Result<T, SexpError>) -> String {
    match result {
        Err(SexpError::InvalidValue(message)) => message,
        other => panic!("expected an InvalidValue error, got {other:?}"),
    }
}

/// The text of the panic `f` raises.
fn panic_message(f: impl FnOnce()) -> String {
    let payload = catch_unwind(AssertUnwindSafe(f)).expect_err("expected a panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .unwrap_or_default()
}

#[test]
fn latin1_is_translated_by_every_read() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = strsxp(&[Some((LATIN1_DEV, cetype_t::CE_LATIN1)), None]);
        let scalar = strsxp(&[Some((LATIN1_DEV, cetype_t::CE_LATIN1))]);
        let x = x.get();
        let scalar = scalar.get();

        assert_eq!(String::try_from_sexp(scalar).unwrap(), "DéV");
        assert_eq!(<&str>::try_from_sexp(scalar).unwrap(), "DéV");
        assert_eq!(
            Option::<String>::try_from_sexp(scalar).unwrap().as_deref(),
            Some("DéV")
        );
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(x).unwrap(),
            vec![Some("DéV".to_owned()), None]
        );
        assert_eq!(x.string_elt_str(0), Some("DéV"));
        assert_eq!(x.string_elt_str(1), None);
        assert_eq!(x.string_elt(0).r_char_str(), Some("DéV"));
    });
}

#[test]
fn a_scalar_marked_as_bytes_is_an_invalid_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = strsxp(&[Some((MU_G, cetype_t::CE_BYTES))]);
        let x = x.get();
        let reason = "marked as bytes, not text";
        assert_eq!(invalid_value(String::try_from_sexp(x)), reason);
        assert_eq!(invalid_value(<&str>::try_from_sexp(x)), reason);
        assert_eq!(invalid_value(<&str>::try_from_sexp_unchecked(x)), reason);
        assert_eq!(invalid_value(Option::<String>::try_from_sexp(x)), reason);
        assert_eq!(invalid_value(Cow::<str>::try_from_sexp(x)), reason);
    });
}

#[test]
fn every_element_marked_as_bytes_is_listed_once() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = strsxp(&[
            Some((b"a", cetype_t::CE_NATIVE)),
            Some((MU_G, cetype_t::CE_BYTES)),
            None,
            Some((N_TILDE, cetype_t::CE_BYTES)),
        ]);
        let x = x.get();
        let message = "marked as bytes, not text (elements 2, 4)";
        assert_eq!(invalid_value(Vec::<String>::try_from_sexp(x)), message);
        assert_eq!(
            invalid_value(Vec::<Option<String>>::try_from_sexp(x)),
            message
        );
        assert_eq!(invalid_value(Vec::<&str>::try_from_sexp(x)), message);
    });
}

#[test]
fn a_string_that_is_not_utf8_is_an_invalid_value() {
    r_test_utils::with_r_thread(|| unsafe {
        let marked = strsxp(&[Some((b"\xff", cetype_t::CE_UTF8))]);
        assert_eq!(
            invalid_value(String::try_from_sexp(marked.get())),
            "not valid UTF-8"
        );
        let native = strsxp(&[
            Some((b"ok", cetype_t::CE_NATIVE)),
            Some((b"a\xff", cetype_t::CE_NATIVE)),
        ]);
        assert_eq!(
            invalid_value(Vec::<String>::try_from_sexp(native.get())),
            "not valid UTF-8 (element 2)"
        );
    });
}

#[test]
fn string_elt_str_panics_naming_an_element_that_is_not_text() {
    r_test_utils::with_r_thread(|| unsafe {
        let x = strsxp(&[
            Some((b"a", cetype_t::CE_NATIVE)),
            Some((MU_G, cetype_t::CE_BYTES)),
        ]);
        let x = x.get();
        assert_eq!(x.string_elt_str(0), Some("a"));
        let message = panic_message(|| {
            let _ = x.string_elt_str(1);
        });
        assert_eq!(
            message,
            "cannot read element 2 of a character vector as Rust text: marked as bytes, not text"
        );
        let message = panic_message(|| {
            let _ = x.string_elt(1).r_char_str();
        });
        assert_eq!(
            message,
            "cannot read an R string as Rust text: marked as bytes, not text"
        );
    });
}
