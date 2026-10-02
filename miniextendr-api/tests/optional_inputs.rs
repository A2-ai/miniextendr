//! `Option<DataFrame>` and `Option<Either<L, R>>` as conversion inputs.
//!
//! `NULL` converts to `None`; any other value converts as the inner type and
//! becomes `Some`. The inner conversion's value, error, metadata and `no_na`
//! hooks carry through unchanged.
//!
//! Before these impls existed, both types failed to compile as parameters
//! (E0275, overflow evaluating `Vec<HashMap<String, _>>: TryFromSexp`): the
//! only candidate was the newtype blanket `Option<T>` impl, whose
//! `Option<T::Inner>` bound sent the solver through the self-recursive
//! `Vec` impls.

mod r_test_utils;

use miniextendr_api::from_r::TryFromSexp;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{DataFrame, SEXP, SexpExt, r_str};

/// `src` evaluated in R's global environment, kept rooted.
fn eval(src: &str) -> OwnedProtect {
    unsafe { OwnedProtect::new(r_str!(src).expect("R source should evaluate")) }
}

/// Both types convert with the inner type's error, so the generated
/// wrapper's error probe sees the same error (and an `RConditionError`'s
/// classes) whatever the `Option` layer.
fn same_error<A, B>()
where
    A: TryFromSexp,
    B: TryFromSexp<Error = A::Error>,
{
}

// region: Option<DataFrame>

#[test]
fn optional_frame_forwards_metadata_and_error_type() {
    same_error::<DataFrame, Option<DataFrame>>();
    assert_eq!(
        <Option<DataFrame> as TryFromSexp>::NATIVE_BORROW,
        <DataFrame as TryFromSexp>::NATIVE_BORROW
    );
    const _: () = assert!(
        <Option<DataFrame> as TryFromSexp>::CHARACTER_ONLY
            == <DataFrame as TryFromSexp>::CHARACTER_ONLY
    );
}

/// `NULL` is `None`, and passes `no_na`. A data frame is the same view of
/// the same object (class, attributes, row names and `NA` cells untouched),
/// and as an `Either` arm or a parameter it refuses no `NA` cell in Rust.
#[test]
fn optional_frame_null_and_data_frame() {
    r_test_utils::with_r_thread(|| {
        let nil = SEXP::nil();
        let none = <Option<DataFrame>>::try_from_sexp(nil).expect("NULL converts");
        assert!(none.is_none());
        assert!(!none.__mx_has_na());
        assert!(!none.__mx_input_has_na(nil));
        let none = unsafe { <Option<DataFrame>>::try_from_sexp_unchecked(nil) }
            .expect("NULL converts unchecked");
        assert!(none.is_none());

        let input = eval(
            "structure(data.frame(id = 1:3, v = c(1, NA, 3), row.names = c('a', 'b', 'c')), \
             class = c('tbl_df', 'tbl', 'data.frame'), note = 'kept')",
        );
        let some = <Option<DataFrame>>::try_from_sexp(input.get()).expect("a data frame converts");
        let frame = some.expect("a data frame is Some");
        assert_eq!(frame.as_sexp(), input.get(), "the same object, not a copy");
        assert_eq!((frame.nrow(), frame.ncol()), (3, 2));
        assert_eq!(frame.names(), ["id", "v"]);
        assert_eq!(
            frame.as_sexp().get_class().xlength(),
            3,
            "the subclass is kept"
        );
        let v: Vec<Option<f64>> = frame.column("v").expect("column v");
        assert_eq!(v, [Some(1.0), None, Some(3.0)], "the NA cell is kept");
        assert!(!some.__mx_has_na());
        assert!(!some.__mx_input_has_na(input.get()), "NA cells pass");

        let unchecked = unsafe { <Option<DataFrame>>::try_from_sexp_unchecked(input.get()) }
            .expect("a data frame converts unchecked");
        assert_eq!(unchecked.map(|f| f.as_sexp()), Some(input.get()));
    });
}

/// A value that is not a data frame fails with exactly the error the bare
/// `DataFrame` conversion gives.
#[test]
fn optional_frame_errors_match_the_bare_conversion() {
    r_test_utils::with_r_thread(|| {
        for src in [
            "1",
            "list(a = 1)",
            "'x'",
            "structure(list(1), class = 'data.frame')",
        ] {
            let input = eval(src);
            let bare = DataFrame::try_from_sexp(input.get())
                .err()
                .unwrap_or_else(|| panic!("{src}: DataFrame refuses it"));
            let optional = <Option<DataFrame>>::try_from_sexp(input.get())
                .err()
                .unwrap_or_else(|| panic!("{src}: Option<DataFrame> refuses it"));
            assert_eq!(format!("{optional:?}"), format!("{bare:?}"), "{src}");
            assert_eq!(optional.to_string(), bare.to_string(), "{src}");
        }
    });
}

// endregion

// region: Option<Either<L, R>>

#[cfg(feature = "either")]
mod either {
    use super::*;
    use miniextendr_api::either_impl::Either;
    use miniextendr_api::{AsNumeric, AsNumericVec};

    #[test]
    fn optional_either_forwards_metadata_and_error_type() {
        same_error::<Either<f64, String>, Option<Either<f64, String>>>();
        same_error::<Either<AsNumericVec, DataFrame>, Option<Either<AsNumericVec, DataFrame>>>();
        assert_eq!(
            <Option<Either<f64, String>> as TryFromSexp>::NATIVE_BORROW,
            <Either<f64, String> as TryFromSexp>::NATIVE_BORROW
        );
        // Character-only exactly when the `Either` is.
        const _: () = assert!(<Option<Either<String, Vec<String>>> as TryFromSexp>::CHARACTER_ONLY);
        const _: () = assert!(!<Option<Either<f64, String>> as TryFromSexp>::CHARACTER_ONLY);
    }

    /// `NULL` is `None`; any other value is the `Either` the bare conversion
    /// makes, tried left first.
    #[test]
    fn optional_either_null_and_arms() {
        r_test_utils::with_r_thread(|| {
            let nil = SEXP::nil();
            let none = <Option<Either<f64, String>>>::try_from_sexp(nil).expect("NULL converts");
            assert_eq!(none, None);
            let none = unsafe { <Option<Either<f64, String>>>::try_from_sexp_unchecked(nil) }
                .expect("NULL converts unchecked");
            assert_eq!(none, None);

            for (src, expected) in [
                ("1.5", Either::Left(1.5)),
                ("'abc'", Either::Right("abc".to_string())),
            ] {
                let input = eval(src);
                let optional = <Option<Either<f64, String>>>::try_from_sexp(input.get())
                    .unwrap_or_else(|e| panic!("{src}: {e}"));
                let bare = <Either<f64, String>>::try_from_sexp(input.get())
                    .unwrap_or_else(|e| panic!("{src}: {e}"));
                assert_eq!(optional, Some(bare), "{src}");
                assert_eq!(optional, Some(expected), "{src}");
            }

            // Text both arms read goes to the left arm, whichever it is.
            let three = eval("'3'");
            let numeric_first =
                <Option<Either<AsNumeric, String>>>::try_from_sexp(three.get()).expect("'3'");
            assert!(matches!(
                numeric_first,
                Some(Either::Left(AsNumeric(Some(3.0))))
            ));
            let text_first =
                <Option<Either<String, AsNumeric>>>::try_from_sexp(three.get()).expect("'3'");
            assert_eq!(text_first.and_then(Either::left).as_deref(), Some("3"));

            // `NULL` is the outer `None` before any arm is tried, also when an
            // arm would read it (`Option<f64>` reads `NULL` as `None`).
            let outer = <Option<Either<Option<f64>, String>>>::try_from_sexp(nil).expect("NULL");
            assert_eq!(outer, None);
        });
    }

    /// `src`'s conversion error as `T`, `Debug` then `Display`; `None` when
    /// it converts.
    fn error_of<T: TryFromSexp>(src: &str) -> Option<(String, String)>
    where
        T::Error: std::fmt::Debug + std::fmt::Display,
    {
        let input = eval(src);
        T::try_from_sexp(input.get())
            .err()
            .map(|e| (format!("{e:?}"), e.to_string()))
    }

    /// A value neither arm reads fails with exactly the bare `Either`'s
    /// error: both arms' reasons, in arm order.
    #[test]
    fn optional_either_errors_match_the_bare_conversion() {
        // Compared outside the R thread, whose panic hook hides the message.
        let errors = r_test_utils::with_r_thread(|| {
            let scalar = ["1L", "c(1, 2)", "TRUE", "list(1)", "c('a', 'b')"].map(|src| {
                (
                    src,
                    error_of::<Either<f64, String>>(src),
                    error_of::<Option<Either<f64, String>>>(src),
                )
            });
            let grid = [
                "list(1, 2)",
                "quote(x)",
                "structure(list(1), class = 'data.frame')",
            ]
            .map(|src| {
                (
                    src,
                    error_of::<Either<AsNumericVec, DataFrame>>(src),
                    error_of::<Option<Either<AsNumericVec, DataFrame>>>(src),
                )
            });
            scalar.into_iter().chain(grid).collect::<Vec<_>>()
        });
        for (src, bare, optional) in errors {
            assert!(bare.is_some(), "{src}: the bare Either refuses it");
            assert_eq!(optional, bare, "{src}");
        }
    }

    /// `no_na` on `Option<Either<AsNumericVec, DataFrame>>` has no R guard:
    /// the C wrapper asks `__mx_input_has_na`. `NULL` passes; a number vector
    /// with `NA` (or text the marker reads as `NA`) is refused by the number
    /// arm; a data frame reaches the `DataFrame` arm with its `NA` cells. The
    /// answers for a given value are the bare `Either`'s.
    #[test]
    fn optional_either_no_na_follows_the_arm() {
        r_test_utils::with_r_thread(|| {
            let nil = SEXP::nil();
            let none = <Option<Either<AsNumericVec, DataFrame>>>::try_from_sexp(nil).expect("NULL");
            assert!(none.is_none());
            assert!(!none.__mx_has_na());
            assert!(!none.__mx_input_has_na(nil));

            for (src, left, has_na, input_has_na) in [
                ("c(1, 2)", true, false, false),
                ("c(1, NA)", true, true, true),
                ("'NA'", true, true, true),
                ("c('1', ' ')", true, true, false),
                ("data.frame(id = 1:2, v = c(1, NA))", false, false, false),
            ] {
                let input = eval(src);
                let optional =
                    <Option<Either<AsNumericVec, DataFrame>>>::try_from_sexp(input.get())
                        .unwrap_or_else(|e| panic!("{src}: {e}"));
                let bare = <Either<AsNumericVec, DataFrame>>::try_from_sexp(input.get())
                    .unwrap_or_else(|e| panic!("{src}: {e}"));
                let value = optional.as_ref().expect("a given value is Some");
                assert_eq!(value.is_left(), left, "{src}: arm");
                assert_eq!(
                    (
                        optional.__mx_has_na(),
                        optional.__mx_input_has_na(input.get())
                    ),
                    (has_na, has_na || input_has_na),
                    "{src}: no_na"
                );
                assert_eq!(
                    (
                        optional.__mx_has_na(),
                        optional.__mx_input_has_na(input.get())
                    ),
                    (bare.__mx_has_na(), bare.__mx_input_has_na(input.get())),
                    "{src}: same as the bare Either"
                );
            }
        });
    }

    /// `Option<DataFrame>` as an `Either` arm forwards to `DataFrame`: its
    /// `NA` cells pass `no_na`, and `NULL` (`None`) holds no `NA`.
    #[test]
    fn optional_frame_arm_keeps_na_cells() {
        r_test_utils::with_r_thread(|| {
            for (src, frame, refused) in [
                ("NULL", false, false),
                ("data.frame(v = c(1, NA))", true, false),
                ("3", false, false),
                ("NA", false, true),
            ] {
                let input = eval(src);
                let value = <Either<AsNumeric, Option<DataFrame>>>::try_from_sexp(input.get())
                    .unwrap_or_else(|e| panic!("{src}: {e}"));
                assert_eq!(
                    value.as_ref().right().is_some_and(Option::is_some),
                    frame,
                    "{src}: arm"
                );
                assert_eq!(value.__mx_input_has_na(input.get()), refused, "{src}");
            }
        });
    }
}

// endregion
