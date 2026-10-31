//! `Nullable<T>` read from R and through serde (#2270).
//!
//! It reads as `Option<T>` does (`NULL` and `NA` are `None`, the inner
//! conversion's errors and metadata unchanged), converts `None` to `NULL`
//! whatever `T` is, and serializes and deserializes as `Option<T>` does, so
//! a serde struct (an `#[r_data]` field of a saved type) holds one.

mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::into_r::IntoR;
use miniextendr_api::{Nullable, SEXP, SexpExt, r_str};

/// `src` evaluated in R's global environment, kept rooted.
fn eval(src: &str) -> OwnedProtect {
    rooted(r_str!(src).expect("R source should evaluate"))
}

/// `sexp`, kept rooted.
fn rooted(sexp: SEXP) -> OwnedProtect {
    unsafe { OwnedProtect::new(sexp) }
}

/// `Nullable<T>` converts with `Option<T>`'s error, so the setter's and a
/// wrapper's error probe see the same error.
fn same_error<A, B>()
where
    A: TryFromSexp,
    B: TryFromSexp<Error = A::Error>,
{
}

// region: TryFromSexp

#[test]
fn nullable_forwards_the_option_metadata() {
    same_error::<Option<String>, Nullable<String>>();
    same_error::<Option<i32>, Nullable<i32>>();
    const _: () = assert!(<Nullable<String> as TryFromSexp>::CHARACTER_ONLY);
    const _: () = assert!(!<Nullable<i32> as TryFromSexp>::CHARACTER_ONLY);
    assert_eq!(
        <Nullable<Vec<i32>> as TryFromSexp>::NATIVE_BORROW,
        <Option<Vec<i32>> as TryFromSexp>::NATIVE_BORROW
    );
}

/// `NULL` and `NA` read `None`, a value reads `Some`.
#[test]
fn nullable_reads_null_and_na_as_none() {
    r_test_utils::with_r_thread(|| {
        let none = <Nullable<String>>::try_from_sexp(SEXP::nil()).expect("NULL converts");
        assert_eq!(none, Nullable(None));
        let na = eval("NA_character_");
        let none = <Nullable<String>>::try_from_sexp(na.get()).expect("NA converts");
        assert_eq!(none, Nullable(None));
        assert!(none.__mx_input_has_na(na.get()), "an NA input is NA");
        let some = eval("'a'");
        let value = <Nullable<String>>::try_from_sexp(some.get()).expect("a string converts");
        assert_eq!(value, Nullable(Some("a".to_string())));
        let unchecked = unsafe { <Nullable<String>>::try_from_sexp_unchecked(some.get()) }
            .expect("a string converts unchecked");
        assert_eq!(unchecked, value);

        let na = eval("NA_integer_");
        assert_eq!(
            <Nullable<i32>>::try_from_sexp(na.get()).expect("NA converts"),
            Nullable(None)
        );
        let three = eval("3L");
        assert_eq!(
            <Nullable<i32>>::try_from_sexp(three.get()).expect("3L converts"),
            Nullable(Some(3))
        );
        assert_eq!(
            <Nullable<i32>>::try_from_sexp(SEXP::nil()).expect("NULL converts"),
            Nullable(None)
        );
    });
}

/// A value the inner type refuses fails with exactly `Option<T>`'s error.
#[test]
fn nullable_errors_match_the_option_conversion() {
    r_test_utils::with_r_thread(|| {
        for src in ["c('a', 'b')", "1", "list('a')"] {
            let input = eval(src);
            let option = <Option<String>>::try_from_sexp(input.get())
                .err()
                .unwrap_or_else(|| panic!("{src}: Option<String> refuses it"));
            let nullable = <Nullable<String>>::try_from_sexp(input.get())
                .err()
                .unwrap_or_else(|| panic!("{src}: Nullable<String> refuses it"));
            assert_eq!(format!("{nullable:?}"), format!("{option:?}"), "{src}");
        }
    });
}

/// `None` is `NULL` and `Some` converts as the value does, so a value read
/// from R converts back to the same R value.
#[test]
fn nullable_round_trips_through_r() {
    r_test_utils::with_r_thread(|| {
        assert!(Nullable::<String>(None).into_sexp().is_null_or_nil());
        assert!(Nullable::<i32>(None).into_sexp().is_null_or_nil());
        for src in ["'a'", "NULL"] {
            let input = eval(src);
            let value = <Nullable<String>>::try_from_sexp(input.get()).expect("converts");
            let back = rooted(value.into_sexp());
            assert!(back.get().is_identical(input.get()), "{src}");
        }
        // `NA` reads `None`, which converts to `NULL`, not back to `NA`.
        let na = eval("NA_character_");
        let value = <Nullable<String>>::try_from_sexp(na.get()).expect("converts");
        assert!(value.into_sexp().is_null_or_nil());
    });
}
// endregion

// region: serde

#[cfg(feature = "serde")]
mod serde_impls {
    use super::*;
    use miniextendr_api::serde::{__from_r_exact, __to_r_exact, from_r, to_r};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Record {
        id: i32,
        label: Nullable<String>,
        count: Nullable<i32>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct OptionRecord {
        id: i32,
        label: Option<String>,
        count: Option<i32>,
    }

    fn records() -> [(Record, OptionRecord); 2] {
        [
            (
                Record {
                    id: 1,
                    label: Nullable(None),
                    count: Nullable(None),
                },
                OptionRecord {
                    id: 1,
                    label: None,
                    count: None,
                },
            ),
            (
                Record {
                    id: 2,
                    label: Nullable(Some("a".to_string())),
                    count: Nullable(Some(3)),
                },
                OptionRecord {
                    id: 2,
                    label: Some("a".to_string()),
                    count: Some(3),
                },
            ),
        ]
    }

    /// The native serde mapping writes a `Nullable<T>` as it writes the
    /// `Option<T>` (`None` is `NULL`), and reads it back.
    #[test]
    fn nullable_round_trips_through_native_serde() {
        r_test_utils::with_r_thread(|| {
            for (record, option) in records() {
                let sexp = rooted(to_r(&record).expect("serializes"));
                let as_option = rooted(to_r(&option).expect("serializes"));
                assert!(sexp.get().is_identical(as_option.get()), "{record:?}");
                let back: Record = from_r(sexp.get()).expect("deserializes");
                assert_eq!(back, record);
            }
            let none = to_r(&Nullable::<String>(None)).expect("serializes");
            assert!(none.is_null_or_nil());
            let na = eval("list(id = 3L, label = NA_character_, count = NA_integer_)");
            let back: Record = from_r(na.get()).expect("NA deserializes as None");
            assert_eq!(
                back,
                Record {
                    id: 3,
                    label: Nullable(None),
                    count: Nullable(None),
                }
            );
        });
    }

    /// The exact encoding a saved object's value takes (#2186) keeps both
    /// `None` and `Some`.
    #[test]
    fn nullable_round_trips_through_the_exact_encoding() {
        r_test_utils::with_r_thread(|| {
            for (record, _) in records() {
                let sexp = rooted(__to_r_exact(&record).expect("encodes"));
                let back: Record = __from_r_exact(sexp.get()).expect("decodes");
                assert_eq!(back, record);
            }
        });
    }
}
// endregion
