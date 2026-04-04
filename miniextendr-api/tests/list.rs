//! Integration tests for List wrapper and IntoList/TryFromList derives.

mod r_test_utils;

use miniextendr_api::ffi::{
    Rboolean, Rf_isNewList, SexpExt,
};
use miniextendr_api::from_r::{SexpLengthError, TryFromSexp};
use miniextendr_api::into_r::IntoR;
use miniextendr_api::list::{IntoList as _, List, TryFromList};
use std::ffi::CStr;

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
            names.string_elt_str(i as miniextendr_api::ffi::R_xlen_t)
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
        unsafe {
            assert!(list.as_sexp().is_list());
            assert_ne!(Rf_isNewList(list.as_sexp()), Rboolean::FALSE);
            assert_eq!(list.as_sexp().xlength(), 2);
        }
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
