//! Native (unmarked) strings in any locale (#2276): borrowed in a UTF-8
//! session, translated from the session's encoding otherwise, with one
//! warning per session at the first translation.

mod r_test_utils;

use miniextendr_api::condition::RCondition;
use miniextendr_api::deferred_condition::capture;
use miniextendr_api::encoding::NATIVE_TRANSLATION_CLASS;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::prelude::SexpExt;
use miniextendr_api::{SEXP, r_str};

/// Evaluate R source, protected for the guard's lifetime.
fn r_value(code: &str) -> OwnedProtect {
    let sexp = r_str!(code).expect("R code evaluates");
    unsafe { OwnedProtect::new(sexp) }
}

fn utf8_session() -> bool {
    bool::try_from_sexp(r_value(r#"isTRUE(l10n_info()[["UTF-8"]])"#).get()).unwrap()
}

/// Read a character scalar as `&str`; `true` when it borrows the CHARSXP's
/// own bytes rather than a translation.
fn read_str(x: SEXP) -> (&'static str, bool) {
    let text = <&str>::try_from_sexp(x).unwrap();
    let own = x.string_elt(0).r_char();
    (text, std::ptr::eq(text.as_ptr(), own.cast()))
}

fn is_translation_warning(cond: &RCondition) -> bool {
    matches!(cond, RCondition::Warning { class, .. } if class.iter().any(|c| c == NATIVE_TRANSLATION_CLASS))
}

#[test]
fn native_strings_follow_the_session_locale() {
    r_test_utils::with_r_thread(|| {
        let original = r_value(r#"Sys.getlocale("LC_CTYPE")"#);

        if utf8_session() {
            // Native UTF-8 text is the CHARSXP's own bytes: borrowed, no warning.
            let native = r_value(r#"x <- "fa\u00e7ade"; Encoding(x) <- "unknown"; x"#);
            let ((text, borrowed), conditions) = capture(|| read_str(native.get()));
            assert_eq!((text, borrowed), ("façade", true));
            assert!(conditions.is_empty(), "{conditions:?}");
        }

        let flipped = r_value(r#"suppressWarnings(Sys.setlocale("LC_CTYPE", "C"))"#);
        let flipped = String::try_from_sexp(flipped.get()).unwrap();
        if flipped.is_empty() || utf8_session() {
            eprintln!("skipped: LC_CTYPE=C did not give a non-UTF-8 session");
        } else {
            let native = r_value("rawToChar(as.raw(c(0x66, 0x61, 0xe7, 0x61, 0x64, 0x65)))");
            let (first, conditions) = capture(|| String::try_from_sexp(native.get()).unwrap());
            assert_eq!(first, "fa<e7>ade");
            assert_eq!(conditions.len(), 1, "{conditions:?}");
            assert!(is_translation_warning(&conditions[0]), "{conditions:?}");

            // A translation, not the CHARSXP's bytes; no second warning.
            let ((text, borrowed), conditions) = capture(|| read_str(native.get()));
            assert_eq!((text, borrowed), ("fa<e7>ade", false));
            assert!(conditions.is_empty(), "{conditions:?}");

            // ASCII and UTF-8-marked strings still borrow, silently. `\u` gives
            // a UTF-8-marked string whatever the session's locale.
            for code in [r#""fa\u00e7ade""#, r#""ascii""#] {
                let value = r_value(code);
                let ((_, borrowed), conditions) = capture(|| read_str(value.get()));
                assert!(borrowed, "{code}");
                assert!(conditions.is_empty(), "{code}: {conditions:?}");
            }
        }

        let restore = format!(
            r#"invisible(Sys.setlocale("LC_CTYPE", "{}"))"#,
            String::try_from_sexp(original.get()).unwrap()
        );
        r_value(&restore);
    });
}
