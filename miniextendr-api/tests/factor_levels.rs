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

/// UnitEnumFactor alone suffices: this type has no RFactor, DataFrameRow,
/// Clone or Copy implementation.
struct OwnedLevel(i32);

impl miniextendr_api::factor::UnitEnumFactor for OwnedLevel {
    const FACTOR_LEVELS: &'static [&'static str] = &["low", "high"];
    fn to_factor_index(self) -> i32 {
        self.0
    }
    fn from_factor_index(idx: i32) -> Option<Self> {
        (1..=2).contains(&idx).then_some(Self(idx))
    }
}

#[test]
fn factor_vec_moves_unit_enum_values_and_retains_empty_levels() {
    use miniextendr_api::factor::FactorVec;
    use miniextendr_api::{IntoR, TryFromSexp};
    r_test_utils::with_r_thread(|| {
        for codes in [vec![2, 1, 2], vec![]] {
            let values = codes.iter().copied().map(OwnedLevel).collect();
            let factor = unsafe { OwnedProtect::new(FactorVec::new(values).into_sexp()) };
            assert!(factor.get().is_factor());
            assert_eq!(factor.get().get_levels().len(), 2);
            let back = FactorVec::<OwnedLevel>::try_from_sexp(factor.get()).unwrap();
            assert_eq!(
                back.into_inner()
                    .into_iter()
                    .map(|v| v.0)
                    .collect::<Vec<_>>(),
                codes
            );
        }
    });
}

#[test]
fn unit_enum_factor_vec_batches_missing_and_invalid_codes() {
    use miniextendr_api::factor::FactorVec;
    use miniextendr_api::{RNativeType, TryFromSexp};
    r_test_utils::with_r_thread(|| {
        let factor = unsafe {
            OwnedProtect::new(build_factor_with_levels(
                &[i32::R_NA, 0, 3],
                &["low", "high"],
            ))
        };
        let error = match FactorVec::<OwnedLevel>::try_from_sexp(factor.get()) {
            Ok(_) => panic!("invalid factor unexpectedly accepted"),
            Err(error) => error.to_string(),
        };
        for reason in [
            "NA is not allowed",
            "code 0 is not a level",
            "code 3 is not a level",
        ] {
            assert!(error.contains(reason), "{error}");
        }
    });
}

/// An enum row flattens a nested frame by scattering its columns over the
/// parent's rows; a factor keeps its levels there, with NA in absent rows.
#[test]
fn scattered_factor_keeps_levels_and_pads_with_na() {
    use miniextendr_api::RNativeType;
    use miniextendr_api::convert::scatter_column;
    r_test_utils::with_r_thread(|| {
        let dense =
            unsafe { OwnedProtect::new(build_factor_with_levels(&[2, 1], &["low", "high"])) };
        let scattered = unsafe { OwnedProtect::new(scatter_column(dense.get(), &[0, 2], 3)) };
        let scattered = scattered.get();
        assert!(scattered.is_factor());
        assert_eq!(scattered.get_levels().len(), 2);
        assert_eq!(unsafe { scattered.as_slice::<i32>() }, [2, i32::R_NA, 1]);
    });
}
