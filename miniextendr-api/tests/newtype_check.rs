//! `TryFromSexpElement::check_sexp`, given by
//! `#[try_from_sexp(validate = ...)]`: the scalar and every container of a
//! newtype run it on the R value before the inner type reads it (#1815).

mod r_test_utils;

use std::sync::atomic::{AtomicUsize, Ordering};

use miniextendr_api::condition::RError;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{RValue, SEXP, SexpExt, r_str};

/// How many times [`plain_number`] ran.
static CHECKS: AtomicUsize = AtomicUsize::new(0);

/// Refuse a number that carries a unit class, with the class and the unit
/// class as a field.
fn plain_number(x: SEXP) -> Result<(), RError> {
    CHECKS.fetch_add(1, Ordering::Relaxed);
    for class in [c"difftime", c"Date", c"POSIXt"] {
        if x.inherits_class(class) {
            let class = class.to_str().unwrap();
            return Err(RError::new(format!("got a {class}; give a plain number"))
                .class(["unit_refused", "test_error"])
                .data("unit_class", class));
        }
    }
    Ok(())
}

#[derive(miniextendr_api::TryFromSexp, Debug, PartialEq)]
#[try_from_sexp(validate = plain_number)]
struct Elapsed(f64);

/// `src` evaluated and converted to `T` (`unchecked`: by
/// `try_from_sexp_unchecked`), with how many times the check ran.
fn convert<T>(src: &str, unchecked: bool) -> (Result<T, SexpError>, usize)
where
    T: TryFromSexp<Error = SexpError>,
{
    let input = unsafe { OwnedProtect::new(r_str!(src).expect("R source should evaluate")) };
    let before = CHECKS.load(Ordering::Relaxed);
    let value = if unchecked {
        unsafe { T::try_from_sexp_unchecked(input.get()) }
    } else {
        T::try_from_sexp(input.get())
    };
    (value, CHECKS.load(Ordering::Relaxed) - before)
}

/// The refusal `plain_number` gives a value of unit class `class`.
fn assert_refused<T: std::fmt::Debug>(result: Result<T, SexpError>, class: &str) {
    match result {
        Err(SexpError::Condition(e)) => {
            assert_eq!(
                e.message_str(),
                format!("got a {class}; give a plain number")
            );
            assert_eq!(e.classes(), ["unit_refused", "test_error"]);
            assert!(
                matches!(e.fields().as_slice(), [(name, RValue::Character(v))]
                    if name == "unit_class" && v == &[Some(class.to_string())]),
                "{:?}",
                e.fields()
            );
        }
        other => panic!("expected the {class} refusal, got {other:?}"),
    }
}

const MINUTES: &str = "as.difftime(5, units = 'mins')";
const DATES: &str = "as.Date(c('2024-01-01', '2024-06-01'))";
const TIMES: &str = "as.POSIXct(c('2024-01-01', NA), tz = 'UTC')";

/// The scalar runs the check before reading the value.
#[test]
fn scalar_runs_the_check() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Elapsed>("5", unchecked);
            assert_eq!((value.unwrap(), checks), (Elapsed(5.0), 1));
            let (value, checks) = convert::<Elapsed>(MINUTES, unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "difftime");
            // The inner type's own refusal comes after the check, as a
            // `SexpError`.
            let (value, checks) = convert::<Elapsed>("'a'", unchecked);
            assert_eq!(checks, 1);
            assert!(matches!(value, Err(SexpError::Type(_))), "{value:?}");
        }
    });
}

/// `Option<T>` runs the check on a value and never on `NULL`, which stays
/// "not given".
#[test]
fn option_runs_the_check_except_on_null() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Option<Elapsed>>("NULL", unchecked);
            assert_eq!((value.unwrap(), checks), (None, 0));
            let (value, checks) = convert::<Option<Elapsed>>("2.5", unchecked);
            assert_eq!((value.unwrap(), checks), (Some(Elapsed(2.5)), 1));
            let (value, checks) = convert::<Option<Elapsed>>("as.Date('2024-01-01')", unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "Date");
        }
    });
}

/// `Vec<T>` runs the check once, on the whole vector, where R keeps the
/// class; an inner type error (`Vec<f64>`'s `SexpTypeError`) still converts
/// into the container's `SexpError`.
#[test]
fn vec_runs_the_check_once() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Vec<Elapsed>>("c(1, 2, 3)", unchecked);
            assert_eq!(
                (value.unwrap(), checks),
                (vec![Elapsed(1.0), Elapsed(2.0), Elapsed(3.0)], 1)
            );
            let (value, checks) = convert::<Vec<Elapsed>>(DATES, unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "Date");
            let (value, checks) = convert::<Vec<Elapsed>>("c('a', 'b')", unchecked);
            assert_eq!(checks, 1);
            assert!(matches!(value, Err(SexpError::Type(_))), "{value:?}");
        }
    });
}

/// `Vec<Option<T>>` runs the check once, on the whole vector; `NA` elements
/// are still `None`.
#[test]
fn vec_option_runs_the_check_once() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert::<Vec<Option<Elapsed>>>("c(1, NA)", unchecked);
            assert_eq!(
                (value.unwrap(), checks),
                (vec![Some(Elapsed(1.0)), None], 1)
            );
            let (value, checks) = convert::<Vec<Option<Elapsed>>>(TIMES, unchecked);
            assert_eq!(checks, 1);
            assert_refused(value, "POSIXt");
        }
    });
}

/// The argument-error probe takes the refusal's classes and fields and uses
/// its message as the reason after the wrapper's prefix; the crate's
/// `conversion_error_class` follows the refusal's classes, and `e$param` /
/// `e$rust_type` follow its fields.
#[test]
fn argument_error_carries_the_refusal() {
    use miniextendr_api::condition::conversion_err_parts;

    r_test_utils::with_r_thread(|| {
        let (value, _) = convert::<Vec<Elapsed>>(DATES, false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "got a Date; give a plain number");
        assert_eq!(parts.class, ["unit_refused", "test_error"]);
        let data = parts.data.as_ref().expect("the refusal's fields");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].0, "unit_class");

        let parts = conversion_err_parts(
            "'x' must be numeric",
            "x",
            Some("Vec<Elapsed>"),
            &["pkg_argument", "test_error"],
            parts,
            None,
        );
        assert_eq!(
            parts.message,
            "'x' must be numeric: got a Date; give a plain number"
        );
        assert_eq!(parts.class, ["unit_refused", "test_error", "pkg_argument"]);
        let names: Vec<&str> = parts
            .data
            .iter()
            .flatten()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["param", "rust_type", "unit_class"]);
    });
}

/// A check can refuse with any `RConditionError` type, such as a
/// `#[derive(RConditionError)]` enum: it converts into
/// `SexpError::Condition` with its classes and fields.
#[test]
fn derived_condition_refusal_keeps_its_classes() {
    use miniextendr_api::condition::RConditionError;

    #[derive(Debug, RConditionError)]
    #[condition(class = "test_family")]
    enum UnitError {
        #[condition(class = "test_bad_unit", message = "a {unit} is not a level")]
        BadUnit { unit: String },
    }

    fn no_factor(x: SEXP) -> Result<(), UnitError> {
        if x.inherits_class(c"factor") {
            return Err(UnitError::BadUnit {
                unit: "factor".to_string(),
            });
        }
        Ok(())
    }

    #[derive(miniextendr_api::TryFromSexp, Debug)]
    #[try_from_sexp(validate = no_factor)]
    struct Level(i32);

    r_test_utils::with_r_thread(|| {
        let (value, _) = convert::<Vec<Level>>("factor(c('a', 'b'))", false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "a factor is not a level");
        assert_eq!(parts.class, ["test_bad_unit", "test_family"]);
        let names: Vec<&str> = parts
            .data
            .iter()
            .flatten()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["unit"]);
    });
}

/// A check refusal inside an `Either` arm: when the other arm refused the
/// kind of value, the argument error reports the refusal with its classes;
/// when both arms refused the kind, it reports the value's type and no
/// refusal classes.
#[cfg(feature = "either")]
#[test]
fn either_arm_reports_the_refusal() {
    use miniextendr_api::either_impl::Either;
    use miniextendr_api::{AsNumeric, DataFrame};

    #[derive(miniextendr_api::TryFromSexp, Debug)]
    #[try_from_sexp(validate = plain_number)]
    struct Tau(AsNumeric);

    type Interval = Either<Option<Tau>, DataFrame>;

    r_test_utils::with_r_thread(|| {
        let (value, _) = convert::<Interval>(MINUTES, false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "got a difftime; give a plain number");
        assert_eq!(parts.class, ["unit_refused", "test_error"]);

        let (value, _) = convert::<Interval>("list(1)", false);
        let parts = miniextendr_api::__mx_conversion_err_parts!(value.unwrap_err(), true);
        assert_eq!(parts.message, "got list");
        assert!(parts.class.is_empty(), "{:?}", parts.class);

        let (value, _) = convert::<Interval>("NULL", false);
        assert!(matches!(value, Ok(Either::Left(None))));
        let (value, _) = convert::<Interval>("3", false);
        assert!(matches!(
            value,
            Ok(Either::Left(Some(Tau(AsNumeric(Some(3.0))))))
        ));
    });
}

// region: newtypes over List: the check on each element (#1837)

/// Each element check, in the order it ran: `"fit"` for [`fit_object`],
/// `"model"` for [`model_object`].
static ELEMENT_CHECKS: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

/// A model object; a data frame gets advice as its argument message, and a
/// field, `what`.
fn model_object(x: SEXP) -> Result<(), RError> {
    ELEMENT_CHECKS.lock().unwrap().push("model");
    if x.inherits_class(c"mx_model") {
        return Ok(());
    }
    if x.inherits_class(c"data.frame") {
        return Err(RError::new("got a data frame")
            .class(["model_refused", "test_error"])
            .data("what", "data frame")
            .argument_message("use model_from_df() for a data frame"));
    }
    Err(RError::new("got no model object").class("model_refused"))
}

#[derive(miniextendr_api::TryFromSexp, Debug)]
#[try_from_sexp(validate = model_object)]
struct Model(miniextendr_api::List);

/// A fit, which is also a model: a newtype of the newtype.
fn fit_object(x: SEXP) -> Result<(), RError> {
    ELEMENT_CHECKS.lock().unwrap().push("fit");
    if x.inherits_class(c"mx_fit") {
        return Ok(());
    }
    Err(RError::new("got no fit").class("fit_refused"))
}

#[derive(miniextendr_api::TryFromSexp, Debug)]
#[try_from_sexp(validate = fit_object)]
struct Fit(Model);

/// Three models in a plain list, which a check on the outer list would refuse.
const MODELS: &str = "rep(list(structure(list(a = 1), class = 'mx_model')), 3)";
const MODEL: &str = "structure(list(), class = 'mx_model')";
const FIT: &str = "structure(list(), class = c('mx_fit', 'mx_model'))";

/// `src` evaluated and converted to `T` (`unchecked`: by
/// `try_from_sexp_unchecked`), with the element checks that ran.
fn convert_elements<T>(src: &str, unchecked: bool) -> (Result<T, SexpError>, Vec<&'static str>)
where
    T: TryFromSexp<Error = SexpError>,
{
    let input = unsafe { OwnedProtect::new(r_str!(src).expect("R source should evaluate")) };
    ELEMENT_CHECKS.lock().unwrap().clear();
    let value = if unchecked {
        unsafe { T::try_from_sexp_unchecked(input.get()) }
    } else {
        T::try_from_sexp(input.get())
    };
    (value, std::mem::take(&mut *ELEMENT_CHECKS.lock().unwrap()))
}

/// The one error of a list read element by element, with a refusal among
/// its failures: the batched message, the refusals' classes and fields.
fn assert_element_refusal<T: std::fmt::Debug>(
    result: Result<T, SexpError>,
    message: &str,
    classes: &[&str],
    fields: &[&str],
) {
    match result {
        Err(SexpError::Condition(e)) => {
            assert_eq!(e.message_str(), message);
            assert_eq!(e.classes(), classes);
            let names: Vec<&str> = e.fields().iter().map(|(name, _)| name.as_str()).collect();
            assert_eq!(names, fields);
            assert_eq!(e.argument_message_str(), None);
        }
        other => panic!("expected the batched refusal {message:?}, got {other:?}"),
    }
}

/// `Vec<T>` of a newtype over `List` runs the check on each element, never on
/// the outer list, and reports every element that fails.
#[test]
fn vec_of_list_newtype_checks_each_element() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert_elements::<Vec<Model>>(MODELS, unchecked);
            assert_eq!(value.unwrap().len(), 3);
            assert_eq!(checks, ["model"; 3]);

            let (value, checks) = convert_elements::<Vec<Model>>("list()", unchecked);
            assert!(value.unwrap().is_empty());
            assert!(checks.is_empty());

            // One bad element: its position, with the refusal's argument
            // message as its reason.
            let (value, checks) = convert_elements::<Vec<Model>>(
                &format!("list({MODEL}, data.frame(a = 1))"),
                unchecked,
            );
            assert_eq!(checks, ["model"; 2]);
            assert_element_refusal(
                value,
                "use model_from_df() for a data frame (element 2)",
                &["model_refused", "test_error"],
                &["what"],
            );

            // Every bad element, grouped by reason, and every element checked.
            let (value, checks) = convert_elements::<Vec<Model>>(
                &format!("list(1, data.frame(a = 1), {MODEL}, 'x')"),
                unchecked,
            );
            assert_eq!(checks, ["model"; 4]);
            assert_element_refusal(
                value,
                "got no model object (elements 1, 4); \
                 use model_from_df() for a data frame (element 2)",
                &["model_refused", "test_error"],
                &["what"],
            );

            // Not a list: a type error, and no element is checked.
            let (value, checks) = convert_elements::<Vec<Model>>("1:3", unchecked);
            assert!(matches!(value, Err(SexpError::Type(_))), "{value:?}");
            assert!(checks.is_empty());
        }
    });
}

/// `Vec<Option<T>>` of a newtype over `List`: `NULL` elements are `None` and
/// are not checked; every other element is.
#[test]
fn vec_option_of_list_newtype_skips_null() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert_elements::<Vec<Option<Model>>>(
                &format!("list({MODEL}, NULL, {MODEL})"),
                unchecked,
            );
            let present: Vec<bool> = value.unwrap().iter().map(Option::is_some).collect();
            assert_eq!(present, [true, false, true]);
            assert_eq!(checks, ["model"; 2]);

            let (value, checks) =
                convert_elements::<Vec<Option<Model>>>("list(NULL, 2)", unchecked);
            assert_eq!(checks, ["model"]);
            assert_element_refusal(
                value,
                "got no model object (element 2)",
                &["model_refused"],
                &[],
            );
        }
    });
}

/// A newtype of a newtype over `List` checks each element, outer check
/// first, as its scalar does.
#[test]
fn vec_of_nested_list_newtype_checks_outer_then_inner() {
    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, checks) = convert_elements::<Fit>(FIT, unchecked);
            assert!(value.is_ok());
            assert_eq!(checks, ["fit", "model"]);

            let (value, checks) =
                convert_elements::<Vec<Fit>>(&format!("list({FIT}, {FIT})"), unchecked);
            assert_eq!(value.unwrap().len(), 2);
            assert_eq!(checks, ["fit", "model", "fit", "model"]);

            let (value, checks) =
                convert_elements::<Vec<Option<Fit>>>(&format!("list(NULL, {MODEL})"), unchecked);
            assert_eq!(checks, ["fit"]);
            assert_element_refusal(value, "got no fit (element 2)", &["fit_refused"], &[]);
        }
    });
}

/// `Vec<List>` and `Vec<Option<List>>` themselves: every element must be a
/// list (a pairlist is refused too), and every failure is reported, with no
/// refusal classes, as one `InvalidValue`.
#[test]
fn vec_of_list_reads_lists_only() {
    use miniextendr_api::List;

    r_test_utils::with_r_thread(|| {
        for unchecked in [false, true] {
            let (value, _) =
                convert_elements::<Vec<List>>("list(list(1), data.frame(a = 1:2))", unchecked);
            let lens: Vec<isize> = value.unwrap().iter().map(|l| l.len()).collect();
            assert_eq!(lens, [1, 1]);

            let (value, _) =
                convert_elements::<Vec<List>>("list(list(1), 2, pairlist(a = 1), NULL)", unchecked);
            match value {
                Err(SexpError::InvalidValue(message)) => assert_eq!(
                    message,
                    "expected list, got numeric (element 2); expected list, got pairlist, \
                     convert it with as.list() (element 3); expected list, got NULL (element 4)"
                ),
                other => panic!("expected the batched type errors, got {other:?}"),
            }

            let (value, _) =
                convert_elements::<Vec<Option<List>>>("list(NULL, list(), 'a')", unchecked);
            match value {
                Err(SexpError::InvalidValue(message)) => {
                    assert_eq!(message, "expected list, got character (element 3)")
                }
                other => panic!("expected the type error, got {other:?}"),
            }
            let (value, _) = convert_elements::<Vec<Option<List>>>("list(NULL, list())", unchecked);
            let present: Vec<bool> = value.unwrap().iter().map(Option::is_some).collect();
            assert_eq!(present, [false, true]);

            let (value, _) = convert_elements::<Vec<List>>("NULL", unchecked);
            assert!(matches!(value, Err(SexpError::Type(_))), "{value:?}");
        }
    });
}

/// The argument error of a `Vec<T>` of a newtype over `List`: the refusals'
/// classes and fields, and the batched message as the reason after the
/// wrapper's prefix (no element's argument message replaces it).
#[test]
fn vec_of_list_newtype_argument_error() {
    use miniextendr_api::condition::conversion_err_parts;

    r_test_utils::with_r_thread(|| {
        let (value, _) = convert_elements::<Vec<Model>>("list(data.frame(a = 1), 3)", false);
        let error = value.unwrap_err();
        assert_eq!(
            miniextendr_api::__mx_conversion_arg_message!(error, "fits"),
            None
        );
        let parts = miniextendr_api::__mx_conversion_err_parts!(error, false);
        assert_eq!(
            parts.message,
            "use model_from_df() for a data frame (element 1); got no model object (element 2)"
        );
        assert_eq!(parts.class, ["model_refused", "test_error"]);
        let parts = conversion_err_parts(
            "invalid 'fits' argument",
            "fits",
            Some("Vec<Model>"),
            &["pkg_argument"],
            parts,
            None,
        );
        assert_eq!(
            parts.message,
            "invalid 'fits' argument: use model_from_df() for a data frame (element 1); \
             got no model object (element 2)"
        );
        assert_eq!(parts.class, ["model_refused", "test_error", "pkg_argument"]);
        let names: Vec<&str> = parts
            .data
            .iter()
            .flatten()
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(names, ["param", "rust_type", "what"]);
    });
}

// endregion
