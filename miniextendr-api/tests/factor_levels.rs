//! `build_levels_sexp` makes each level a UTF-8 CHARSXP with `Rf_mkCharLenCE`
//! instead of interning it as an R symbol (#1763).
//!
//! R's symbol table is not public API, so these tests cannot check directly
//! that no symbol was installed. They check the encoding instead: a symbol's
//! PRINTNAME is made in the native encoding, so the old path left non-ASCII
//! levels unmarked, while the CHARSXP path marks them UTF-8.

mod r_test_utils;

use miniextendr_api::factor::{build_factor_with_levels, build_levels_sexp};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::prelude::SexpExt;
use miniextendr_api::sexp_types::CE_UTF8;
use miniextendr_api::sys::Rf_getCharCE;

#[test]
fn non_ascii_levels_are_utf8_charsxps() {
    r_test_utils::with_r_thread(|| {
        let levels = unsafe { OwnedProtect::new(build_levels_sexp(&["café", "naïve", "café"])) };
        let levels = levels.get();
        assert_eq!(levels.string_elt_str(0), Some("café"));
        assert_eq!(levels.string_elt_str(1), Some("naïve"));
        for i in 0..3 {
            let encoding = unsafe { Rf_getCharCE(levels.string_elt(i)) };
            assert_eq!(
                encoding as i32, CE_UTF8 as i32,
                "level {i} is not marked UTF-8"
            );
        }
        // R's global CHARSXP cache shares repeated names.
        assert_eq!(levels.string_elt(0), levels.string_elt(2));
    });
}

#[test]
fn factor_with_non_ascii_levels_round_trips() {
    r_test_utils::with_r_thread(|| {
        let factor = unsafe {
            OwnedProtect::new(build_factor_with_levels(&[2, 1, 2], &["über", "Ölfeld"]))
        };
        let levels = factor.get().get_levels();
        assert_eq!(levels.string_elt_str(0), Some("über"));
        assert_eq!(levels.string_elt_str(1), Some("Ölfeld"));
        assert_eq!(unsafe { factor.get().as_slice::<i32>() }, &[2, 1, 2]);
    });
}

/// `Rf_install` raised an R error on an empty name and on one over 10000
/// bytes; both are valid factor levels.
#[test]
fn empty_and_long_levels_are_accepted() {
    r_test_utils::with_r_thread(|| {
        let long = "a".repeat(10_001);
        let levels = unsafe { OwnedProtect::new(build_levels_sexp(&["", &long])) };
        let levels = levels.get();
        assert_eq!(levels.string_elt_str(0), Some(""));
        assert_eq!(levels.string_elt_str(1), Some(long.as_str()));
    });
}

#[test]
#[should_panic(expected = "level name contains null byte")]
fn null_byte_level_panics() {
    r_test_utils::with_r_thread(|| {
        build_levels_sexp(&["ok", "bad\0name"]);
    });
}
