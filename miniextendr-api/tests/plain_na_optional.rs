//! A plain `NA`, R's untyped missing value, reads as `None` for an optional
//! scalar, as `NULL` does (#2305): a length-1 logical `NA`, its attributes
//! not looked at. Checked and unchecked conversions agree. Every other value
//! keeps the inner type's check, and optional vectors read a plain `NA` as a
//! value.

mod r_test_utils;

use std::any::type_name;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{
    MatchArg, ProtectScope, Rcomplex, SEXP, TryFromRoot, match_arg_null_or,
    match_arg_option_from_sexp, r_str,
};

/// `src` evaluated in R's global environment, kept rooted.
fn eval(src: &str) -> OwnedProtect {
    unsafe { OwnedProtect::new(r_str!(src).expect("R source should evaluate")) }
}

/// Inputs every optional scalar reads as `None`.
const ABSENT: &[&str] = &["NULL", "NA", "c(a = NA)", "structure(NA, class = 'x')"];

/// Logical inputs that are no plain `NA`: every strict optional scalar
/// refuses them.
const NOT_PLAIN_NA: &[&str] = &["c(NA, NA)", "logical(0)"];

/// `Option<U>` reads every [`ABSENT`] input as `None`, checked and
/// unchecked.
fn reads_absent_as_none<U: Debug>()
where
    Option<U>: TryFromSexp,
    <Option<U> as TryFromSexp>::Error: Debug,
{
    for src in ABSENT {
        let x = eval(src);
        let checked = <Option<U>>::try_from_sexp(x.get());
        assert!(
            matches!(checked, Ok(None)),
            "Option<{}> from {src}: {checked:?}",
            type_name::<U>()
        );
        let unchecked = unsafe { <Option<U>>::try_from_sexp_unchecked(x.get()) };
        assert!(
            matches!(unchecked, Ok(None)),
            "Option<{}> from {src}, unchecked: {unchecked:?}",
            type_name::<U>()
        );
    }
}

/// `Option<U>` refuses every input in `srcs`, checked and unchecked.
fn refuses<U: Debug>(srcs: &[&str])
where
    Option<U>: TryFromSexp,
    <Option<U> as TryFromSexp>::Error: Debug,
{
    for src in srcs {
        let x = eval(src);
        let checked = <Option<U>>::try_from_sexp(x.get());
        assert!(
            checked.is_err(),
            "Option<{}> from {src}: {checked:?}",
            type_name::<U>()
        );
        let unchecked = unsafe { <Option<U>>::try_from_sexp_unchecked(x.get()) };
        assert!(
            unchecked.is_err(),
            "Option<{}> from {src}, unchecked: {unchecked:?}",
            type_name::<U>()
        );
    }
}

// region: the optional scalars

#[test]
fn optional_scalars_read_a_plain_na_as_none() {
    r_test_utils::with_r_thread(|| {
        reads_absent_as_none::<i32>();
        reads_absent_as_none::<f64>();
        reads_absent_as_none::<u8>();
        reads_absent_as_none::<Rcomplex>();
        reads_absent_as_none::<String>();
        reads_absent_as_none::<&'static str>();
        reads_absent_as_none::<bool>();
        reads_absent_as_none::<i64>();
        reads_absent_as_none::<u32>();
    });
}

/// A value of another type, or a logical other than a plain `NA`, is still
/// refused; a double is no `i32` without `coerce`.
#[test]
fn optional_scalars_keep_their_checks() {
    r_test_utils::with_r_thread(|| {
        refuses::<i32>(&["c(NA, NA)", "logical(0)", "TRUE", "2", "'a'"]);
        refuses::<f64>(&["c(NA, NA)", "logical(0)", "TRUE", "1L", "'a'"]);
        refuses::<u8>(&["c(NA, NA)", "logical(0)", "TRUE", "1L"]);
        refuses::<Rcomplex>(&["c(NA, NA)", "logical(0)", "TRUE", "1"]);
        refuses::<String>(&["c(NA, NA)", "logical(0)", "TRUE", "1"]);
        refuses::<&'static str>(&["c(NA, NA)", "logical(0)", "TRUE", "1"]);
        refuses::<bool>(&["c(NA, NA)", "logical(0)", "'a'"]);
        refuses::<i64>(NOT_PLAIN_NA);
        refuses::<u32>(NOT_PLAIN_NA);
    });
}

/// A value reads as before, and a typed `NA` is still `None`.
#[test]
fn optional_scalars_read_values_and_typed_na() {
    r_test_utils::with_r_thread(|| {
        let three = eval("3L");
        assert_eq!(<Option<i32>>::try_from_sexp(three.get()).unwrap(), Some(3));
        let na = eval("NA_integer_");
        assert_eq!(<Option<i32>>::try_from_sexp(na.get()).unwrap(), None);
        let half = eval("0.5");
        assert_eq!(<Option<f64>>::try_from_sexp(half.get()).unwrap(), Some(0.5));
        let na = eval("NA_real_");
        assert_eq!(<Option<f64>>::try_from_sexp(na.get()).unwrap(), None);
        let text = eval("'a'");
        assert_eq!(
            <Option<String>>::try_from_sexp(text.get()).unwrap(),
            Some("a".to_string())
        );
        let na = eval("NA_character_");
        assert_eq!(<Option<String>>::try_from_sexp(na.get()).unwrap(), None);
        let byte = eval("as.raw(7)");
        assert_eq!(<Option<u8>>::try_from_sexp(byte.get()).unwrap(), Some(7));
        let truth = eval("TRUE");
        assert_eq!(
            <Option<bool>>::try_from_sexp(truth.get()).unwrap(),
            Some(true)
        );
    });
}

/// The rooted `Option<&str>` the generated wrappers use agrees.
#[test]
fn rooted_optional_str_reads_a_plain_na_as_none() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        for src in ABSENT {
            let root = scope.protect(r_str!(src).expect("R source should evaluate"));
            let value = Option::<&str>::try_from_root(root);
            assert!(matches!(value, Ok(None)), "from {src}: {value:?}");
        }
        for src in ["c(NA, NA)", "TRUE"] {
            let root = scope.protect(r_str!(src).expect("R source should evaluate"));
            assert!(Option::<&str>::try_from_root(root).is_err(), "from {src}");
        }
        let root = scope.protect(r_str!("'a'").expect("R source should evaluate"));
        assert_eq!(Option::<&str>::try_from_root(root).unwrap(), Some("a"));
    });
}

// endregion

// region: optional choices

/// A choice type, as the derive writes one.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Mode(usize);

impl MatchArg for Mode {
    const CHOICES: &'static [&'static str] = &["Fast", "Safe", "Debug"];

    fn from_choice(choice: &str) -> Option<Self> {
        Self::CHOICES.iter().position(|c| *c == choice).map(Mode)
    }

    fn to_choice(self) -> &'static str {
        Self::CHOICES[self.0]
    }
}

/// `Option<Mode>` reads a plain `NA` as `None`; a typed `NA` is still
/// matched, and refused.
#[test]
fn optional_choice_reads_a_plain_na_as_none() {
    r_test_utils::with_r_thread(|| {
        for src in ABSENT {
            let x = eval(src);
            let value = match_arg_option_from_sexp::<Mode>(x.get());
            assert!(matches!(value, Ok(None)), "from {src}: {value:?}");
        }
        let safe = eval("'Safe'");
        assert_eq!(
            match_arg_option_from_sexp::<Mode>(safe.get()).unwrap(),
            Some(Mode(1))
        );
        for src in ["NA_character_", "c(NA, NA)", "'nope'"] {
            let x = eval(src);
            assert!(
                match_arg_option_from_sexp::<Mode>(x.get()).is_err(),
                "from {src}"
            );
        }
    });
}

/// `match_arg_null_or` layers over several_ok vectors and `Either`, whose
/// decoders read a plain `NA` as a value: it gives them the `NA`.
#[test]
fn match_arg_null_or_passes_a_plain_na_on() {
    r_test_utils::with_r_thread(|| {
        let na = eval("NA");
        let seen = match_arg_null_or(na.get(), |sexp| Ok::<_, ()>(sexp == na.get()));
        assert_eq!(seen, Ok(Some(true)));
        let none = match_arg_null_or(SEXP::nil(), |_| Ok::<_, ()>(true));
        assert_eq!(none, Ok(None));
    });
}

// endregion

// region: newtypes and optional vectors

/// How many times [`count_checks`] ran.
static CHECKS: AtomicUsize = AtomicUsize::new(0);

/// Accept every value, counting the calls.
fn count_checks(_: SEXP) -> Result<(), RError> {
    CHECKS.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

#[derive(miniextendr_api::TryFromSexp, Debug, PartialEq)]
#[try_from_sexp(validate = count_checks)]
struct Score(f64);

/// A newtype's `Option` reads a plain `NA` as "not given": its check does
/// not see it, as it does not see `NULL`.
#[test]
fn optional_newtype_skips_its_check_for_a_plain_na() {
    r_test_utils::with_r_thread(|| {
        for src in ABSENT {
            let x = eval(src);
            let before = CHECKS.load(Ordering::Relaxed);
            let value = <Option<Score>>::try_from_sexp(x.get());
            assert!(matches!(value, Ok(None)), "from {src}: {value:?}");
            let unchecked = unsafe { <Option<Score>>::try_from_sexp_unchecked(x.get()) };
            assert!(matches!(unchecked, Ok(None)), "from {src}: {unchecked:?}");
            assert_eq!(CHECKS.load(Ordering::Relaxed), before, "from {src}");
        }
        let x = eval("1.5");
        let before = CHECKS.load(Ordering::Relaxed);
        assert_eq!(
            <Option<Score>>::try_from_sexp(x.get()).unwrap(),
            Some(Score(1.5))
        );
        assert_eq!(CHECKS.load(Ordering::Relaxed), before + 1);
    });
}

/// An optional vector reads a plain `NA` as a value, as before.
#[test]
fn optional_vectors_read_a_plain_na_as_a_value() {
    r_test_utils::with_r_thread(|| {
        let na = eval("NA");
        assert!(<Option<Vec<i32>>>::try_from_sexp(na.get()).is_err());
        assert!(unsafe { <Option<Vec<i32>>>::try_from_sexp_unchecked(na.get()) }.is_err());
        assert_eq!(
            <Option<Vec<Option<bool>>>>::try_from_sexp(na.get()).unwrap(),
            Some(vec![None])
        );
        assert_eq!(
            unsafe { <Option<Vec<Option<bool>>>>::try_from_sexp_unchecked(na.get()) }.unwrap(),
            Some(vec![None])
        );
    });
}

// endregion

// region: optional integrations

#[cfg(feature = "num-complex")]
#[test]
fn optional_complex_reads_a_plain_na_as_none() {
    use miniextendr_api::num_complex_impl::Complex;
    r_test_utils::with_r_thread(|| {
        reads_absent_as_none::<Complex<f64>>();
        refuses::<Complex<f64>>(&["c(NA, NA)", "logical(0)", "TRUE"]);
    });
}

#[cfg(feature = "jiff")]
#[test]
fn optional_jiff_types_read_a_plain_na_as_none() {
    use miniextendr_api::jiff_impl::{Date, SignedDuration, Timestamp, Zoned};
    r_test_utils::with_r_thread(|| {
        reads_absent_as_none::<Timestamp>();
        reads_absent_as_none::<Date>();
        reads_absent_as_none::<SignedDuration>();
        reads_absent_as_none::<Zoned>();
        refuses::<Timestamp>(&["c(NA, NA)", "TRUE"]);
        refuses::<Zoned>(&["c(NA, NA)", "TRUE"]);
    });
}

#[cfg(feature = "time")]
#[test]
fn optional_time_types_read_a_plain_na_as_none() {
    use miniextendr_api::time_impl::{Date, OffsetDateTime};
    r_test_utils::with_r_thread(|| {
        reads_absent_as_none::<OffsetDateTime>();
        reads_absent_as_none::<Date>();
        refuses::<OffsetDateTime>(&["c(NA, NA)", "TRUE"]);
        refuses::<Date>(&["c(NA, NA)", "TRUE"]);
    });
}

// endregion
