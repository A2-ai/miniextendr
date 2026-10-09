//! Integration tests for List wrapper and IntoList/TryFromList derives.

mod r_test_utils;

use miniextendr_api::from_r::{SexpLengthError, TryFromSexp};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::list::{IntoList as _, List, TryFromList};
use miniextendr_api::prelude::SexpExt;

#[derive(Debug, PartialEq)]
struct Foo {
    a: i32,
    b: String,
}

impl miniextendr_api::list::IntoList for Foo {
    fn into_list(self) -> List {
        List::from_raw_pairs(vec![("a", self.a.into_sexp()), ("b", self.b.into_sexp())])
    }
}

impl TryFromList for Foo {
    type Error = (String, miniextendr_api::from_r::SexpError);

    fn try_from_list(list: List) -> Result<Self, Self::Error> {
        let expected = 2;
        let actual = list.len() as usize;
        if actual < expected {
            return Err((
                "__len__".to_string(),
                SexpLengthError { expected, actual }.into(),
            ));
        }

        let a = TryFromSexp::try_from_sexp(list.get(0).ok_or_else(|| {
            (
                "__len__".to_string(),
                SexpLengthError { expected, actual }.into(),
            )
        })?)
        .map_err(|e| ("a".to_string(), e))?;

        let b = TryFromSexp::try_from_sexp(list.get(1).ok_or_else(|| {
            (
                "__len__".to_string(),
                SexpLengthError { expected, actual }.into(),
            )
        })?)
        .map_err(|e| ("b".to_string(), e))?;

        Ok(Foo { a, b })
    }
}

fn names_as_vec(list: List) -> Vec<String> {
    let names = list.as_sexp().get_names();
    if names.is_nil() {
        return vec![];
    }
    let len = names.len();
    (0..len)
        .map(|i| {
            names
                .string_elt_str(i as miniextendr_api::R_xlen_t)
                .unwrap_or("")
                .to_string()
        })
        .collect()
}

#[test]
fn derive_into_list_and_back() {
    r_test_utils::with_r_thread(|| {
        let foo = Foo {
            a: 7,
            b: "hi".to_string(),
        };

        let list = foo.into_list();
        assert!(list.as_sexp().is_list());
        assert_eq!(list.as_sexp().xlength(), 2);
        assert_eq!(names_as_vec(list), vec!["a", "b"]);

        let roundtrip = Foo::try_from_list(list).unwrap();
        assert_eq!(
            roundtrip,
            Foo {
                a: 7,
                b: "hi".into()
            }
        );
    });
}

#[test]
fn try_from_list_reports_length() {
    r_test_utils::with_r_thread(|| {
        let short = List::from_pairs(vec![("a", 1i32)]);
        let err = Foo::try_from_list(short).unwrap_err();
        assert_eq!(err.0, "__len__");
    });
}

#[test]
fn try_from_list_reports_field_name_on_type_error() {
    r_test_utils::with_r_thread(|| {
        // Make `a` the wrong type (string instead of int)
        let bad = List::from_pairs(vec![("a", "oops"), ("b", "ok")]);
        let err = Foo::try_from_list(bad).unwrap_err();
        assert_eq!(err.0, "a");
    });
}

#[test]
fn from_raw_pairs_empty_is_length_zero_vecsxp_with_names() {
    r_test_utils::with_r_thread(|| {
        let list = List::from_raw_pairs_empty();
        assert!(list.as_sexp().is_list(), "should be VECSXP");
        assert_eq!(list.as_sexp().xlength(), 0, "should have length 0");
        let names = list.as_sexp().get_names();
        assert!(names.is_character(), "names attribute should be STRSXP");
        assert_eq!(names.xlength(), 0, "names should have length 0");
    });
}

use miniextendr_api::ExternalPtr;

#[derive(ExternalPtr, miniextendr_api::IntoList)]
struct Dual(i32);

#[test]
fn into_r_prefers_externalptr_over_list() {
    r_test_utils::with_r_thread(|| {
        let dual = Dual(10);
        let sexp = dual.into_sexp();
        assert!(sexp.is_external_ptr());
    });
}

#[derive(miniextendr_api::IntoList, miniextendr_api::PreferList)]
struct ListFirst {
    a: i32,
}

#[test]
fn prefer_list_changes_intor() {
    r_test_utils::with_r_thread(|| {
        let lf = ListFirst { a: 5 };
        let sexp = lf.into_sexp();
        assert!(sexp.is_list());
    });
}

/// `get_named` / `get_index` are generic over any `TryFromSexp` error type, so
/// a nested list is fetched as `List` directly (its error is
/// `SexpTypeError`, not `SexpError`). Regression test for the bound relaxed
/// in #865; surfaced again while building a nested-config walker downstream.
#[test]
fn get_named_fetches_nested_list() {
    r_test_utils::with_r_thread(|| {
        let inner = List::from_pairs(vec![("b", 1i32)]);
        let outer = List::from_raw_pairs(vec![("inner", inner.as_sexp()), ("n", 2i32.into_sexp())]);

        let fetched: List = outer
            .get_named("inner")
            .expect("nested list element is fetched as List");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched.get_named::<i32>("b"), Some(1));

        // Same relaxation on the positional accessor.
        assert_eq!(outer.get_index::<List>(0).map(|l| l.len()), Some(1));

        // A non-list element fails the conversion and yields None rather than
        // a type error; a missing name yields None too.
        assert!(outer.get_named::<List>("n").is_none());
        assert!(outer.get_named::<List>("missing").is_none());
    });
}

// region: Repeated names (#1754)

use miniextendr_api::from_r::SexpError;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::list::NamedList;

/// Evaluate R source that yields a list, protected for the test's duration.
fn r_list(code: &str) -> (OwnedProtect, List) {
    let sexp = miniextendr_api::r_str!(code).expect("R code evaluates");
    let guard = unsafe { OwnedProtect::new(sexp) };
    let list = List::try_from_sexp(guard.get()).expect("a list converts to List");
    (guard, list)
}

/// A field-reading struct for the `#[derive(TryFromList)]` checks below.
#[derive(Debug, PartialEq, miniextendr_api::TryFromList)]
struct Picked {
    x: f64,
    y: Option<f64>,
    #[into_list(ignore)]
    skipped: i32,
}

#[test]
fn list_accepts_repeated_names() {
    r_test_utils::with_r_thread(|| {
        let (_g, list) = r_list("list(a = 1, a = 2)");
        assert_eq!(list.len(), 2);

        // A non-list is still a type error.
        let sexp = miniextendr_api::r_str!("1:3").unwrap();
        let err = List::try_from_sexp(sexp).unwrap_err();
        assert_eq!(err.expected, miniextendr_api::SEXPTYPE::VECSXP);
        assert_eq!(err.actual, miniextendr_api::SEXPTYPE::INTSXP);
    });
}

/// A pairlist is refused, not coerced (#1866): the coerced copy would be an
/// object nothing roots. `Option<List>` and `NamedList` read through `List`
/// and refuse it the same way.
#[test]
fn list_refuses_pairlist() {
    use miniextendr_api::SEXPTYPE::{LISTSXP, VECSXP};

    r_test_utils::with_r_thread(|| {
        for code in ["pairlist(a = 1, a = 2)", "formals(function(a, b) NULL)"] {
            let sexp = miniextendr_api::r_str!(code).expect("R code evaluates");
            let guard = unsafe { OwnedProtect::new(sexp) };

            let err = List::try_from_sexp(guard.get()).unwrap_err();
            assert_eq!((err.expected, err.actual), (VECSXP, LISTSXP), "{code}");

            for err in [
                <Option<List>>::try_from_sexp(guard.get()).err(),
                NamedList::try_from_sexp(guard.get()).err(),
            ] {
                match err {
                    Some(SexpError::Type(e)) => {
                        assert_eq!((e.expected, e.actual), (VECSXP, LISTSXP), "{code}")
                    }
                    other => panic!("{code}: expected a type error, got {other:?}"),
                }
            }
        }
    });
}

#[test]
fn get_named_returns_first_of_repeated_name() {
    r_test_utils::with_r_thread(|| {
        let (_g, list) = r_list("list(a = 1, b = 2, a = 3)");
        assert_eq!(list.get_named::<f64>("a"), Some(1.0));
        assert_eq!(
            list.get_named_sexp("a").and_then(|s| s.as_real()),
            Some(1.0)
        );
        assert_eq!(list.get_named::<f64>("b"), Some(2.0));
        assert_eq!(list.get_named::<f64>("c"), None);
    });
}

#[test]
fn first_duplicate_name_cases() {
    r_test_utils::with_r_thread(|| {
        let dup = |code: &str| {
            let (_g, list) = r_list(code);
            list.first_duplicate_name()
        };
        assert_eq!(dup("list(a = 1, b = 2)"), None);
        assert_eq!(dup("list(a = 1, a = 2)"), Some("a".to_string()));
        // The first name to repeat, not the first name that has a repeat.
        assert_eq!(
            dup("list(a = 1, b = 2, b = 3, a = 4)"),
            Some("b".to_string())
        );
        // Unnamed and empty lists have no names to repeat.
        assert_eq!(dup("list(1, 2)"), None);
        assert_eq!(dup("list()"), None);
        // NA and empty names are skipped, however many there are.
        assert_eq!(
            dup(r#"setNames(list(1, 2, 3, 4, 5), c(NA, NA, "", "", "a"))"#),
            None
        );
        assert_eq!(
            dup(r#"setNames(list(1, 2, 3, 4), c(NA, "", "a", "a"))"#),
            Some("a".to_string())
        );
    });
}

#[test]
fn named_list_first_occurrence_wins() {
    r_test_utils::with_r_thread(|| {
        let (_g, list) = r_list(r#"setNames(list(1, 2, 3, 4), c("a", "b", "a", ""))"#);
        let named = NamedList::new(list).expect("list has names");
        assert_eq!(named.get::<f64>("a"), Some(1.0));
        assert_eq!(named.get_raw("a").and_then(|s| s.as_real()), Some(1.0));
        assert_eq!(named.named_len(), 2);
        assert_eq!(named.len(), 4);
    });
}

#[test]
fn derive_try_from_list_refuses_repeated_field_name() {
    r_test_utils::with_r_thread(|| {
        let (_g, list) = r_list("list(x = 1, x = 2)");
        match Picked::try_from_list(list) {
            Err(SexpError::DuplicateName(name)) => assert_eq!(name, "x"),
            other => panic!("expected DuplicateName(\"x\"), got {other:?}"),
        }

        // A field name repeated later, with other names in between.
        let (_g, list) = r_list("list(y = 0, x = 1, z = 2, y = 3)");
        match Picked::try_from_list(list) {
            Err(SexpError::DuplicateName(name)) => assert_eq!(name, "y"),
            other => panic!("expected DuplicateName(\"y\"), got {other:?}"),
        }
    });
}

#[test]
fn derive_try_from_list_ignores_other_repeated_names() {
    r_test_utils::with_r_thread(|| {
        let expected = Picked {
            x: 1.0,
            y: None,
            skipped: 0,
        };

        // A repeated name that is not a field. (`y` is an `Option` field: it
        // must be present, and `NULL` reads as `None`.)
        let (_g, list) = r_list("list(x = 1, y = NULL, extra = 2, extra = 3)");
        assert_eq!(Picked::try_from_list(list).unwrap(), expected);

        // A repeated name that matches an ignored field is not read either.
        let (_g, list) = r_list("list(skipped = 5, x = 1, y = NULL, skipped = 6)");
        assert_eq!(Picked::try_from_list(list).unwrap(), expected);

        // Fields are found wherever they are, with NA names in between.
        let (_g, list) = r_list(r#"setNames(list(9, 2, 1), c(NA, "y", "x"))"#);
        assert_eq!(
            Picked::try_from_list(list).unwrap(),
            Picked {
                x: 1.0,
                y: Some(2.0),
                skipped: 0,
            }
        );
    });
}

/// A field named `list`: the generated `try_from_list` must not name its
/// parameter `list`, or the field binding would shadow it for `y`.
#[derive(Debug, PartialEq, miniextendr_api::TryFromList)]
struct WithListField {
    list: Vec<f64>,
    y: f64,
}

#[test]
fn derive_try_from_list_field_named_list() {
    r_test_utils::with_r_thread(|| {
        let (_g, list) = r_list("list(list = c(1, 2), y = 3)");
        assert_eq!(
            WithListField::try_from_list(list).unwrap(),
            WithListField {
                list: vec![1.0, 2.0],
                y: 3.0,
            }
        );
    });
}

#[test]
fn derive_try_from_list_missing_and_wrong_type() {
    r_test_utils::with_r_thread(|| {
        // An unnamed list has no field names.
        let (_g, list) = r_list("list(1, 2)");
        match Picked::try_from_list(list) {
            Err(SexpError::MissingField(name)) => assert_eq!(name, "x"),
            other => panic!("expected MissingField(\"x\"), got {other:?}"),
        }

        // A present field of the wrong type reports its conversion error.
        let (_g, list) = r_list(r#"list(x = "one")"#);
        let err = Picked::try_from_list(list).unwrap_err();
        assert!(
            !matches!(
                err,
                SexpError::MissingField(_) | SexpError::DuplicateName(_)
            ),
            "expected the field's conversion error, got {err:?}"
        );
    });
}

// endregion
