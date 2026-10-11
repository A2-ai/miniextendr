//! Owned DST strings keep scalar text and NA semantics, including lazy vectors.
mod r_test_utils;

use miniextendr_api::altrep_data::AltrepExtract;
use miniextendr_api::altrep_traits::{AltString, AltVec};
use miniextendr_api::expression::RCall;
use miniextendr_api::{Altrep, IntoR, OwnedProtect, SEXP, SEXPTYPE, SexpExt, TryFromSexp};

macro_rules! owned_str_tests {
    ($module:ident, $owner:path) => {
        mod $module {
            use super::*;
            use $owner as Owner;

            #[test]
            fn scalar_and_optional_values_follow_string_semantics() {
                r_test_utils::with_r_thread(|| unsafe {
                    let x = OwnedProtect::new(Owner::<str>::from("æøå").into_sexp());
                    assert_eq!(x.get().type_of(), SEXPTYPE::STRSXP);
                    assert_eq!(x.get().len(), 1);
                    assert_eq!(&*Owner::<str>::try_from_sexp(x.get()).unwrap(), "æøå");
                    let missing = OwnedProtect::new(Option::<Owner<str>>::None.into_sexp());
                    assert_eq!(missing.get().string_elt(0), SEXP::na_string());
                    assert_eq!(&*Owner::<str>::try_from_sexp(missing.get()).unwrap(), "");
                    assert!(
                        Option::<Owner<str>>::try_from_sexp(missing.get())
                            .unwrap()
                            .is_none()
                    );
                    assert!(
                        Option::<Owner<str>>::try_from_sexp(SEXP::nil())
                            .unwrap()
                            .is_none()
                    );
                    let bad = OwnedProtect::new(7_i32.into_sexp());
                    assert!(Owner::<str>::try_from_sexp(bad.get()).is_err());
                });
            }

            #[test]
            fn vector_conversion_owns_strings_and_preserves_optional_na() {
                r_test_utils::with_r_thread(|| unsafe {
                    let values = {
                        let x = OwnedProtect::new(vec![Some("one"), None, Some("")].into_sexp());
                        Vec::<Option<Owner<str>>>::try_from_sexp(x.get()).unwrap()
                    };
                    RCall::new("gc").eval_base().unwrap();
                    let out = OwnedProtect::new(values.into_sexp());
                    assert_eq!(
                        Vec::<Option<String>>::try_from_sexp(out.get()).unwrap(),
                        [Some("one".into()), None, Some("".into())]
                    );
                    let plain = Vec::<Owner<str>>::try_from_sexp(out.get()).unwrap();
                    assert_eq!(
                        plain.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
                        ["one", "", ""]
                    );
                    let out = OwnedProtect::new(plain.into_sexp());
                    assert!(!out.get().is_altrep());
                });
            }

            #[test]
            fn lazy_vector_retains_owners_and_serializes_current_values() {
                r_test_utils::with_r_thread(|| unsafe {
                    let text = Owner::<str>::from("one");
                    let original = text.as_ptr();
                    let x = OwnedProtect::new(Altrep(vec![text, Owner::from("two")]).into_sexp());
                    assert!(x.get().is_altrep());
                    assert!(<Vec<Owner<str>> as AltVec>::dataptr_or_null(x.get()).is_null());
                    assert_eq!(
                        <Vec<Owner<str>> as AltrepExtract>::altrep_extract_ref(x.get())[0].as_ptr(),
                        original
                    );

                    let bytes = OwnedProtect::new(
                        RCall::new("serialize")
                            .arg(x.get())
                            .arg(SEXP::nil())
                            .eval_base()
                            .unwrap(),
                    );
                    let restored = OwnedProtect::new(
                        RCall::new("unserialize")
                            .arg(bytes.get())
                            .eval_base()
                            .unwrap(),
                    );
                    assert!(restored.get().is_altrep());
                    assert_eq!(
                        Vec::<String>::try_from_sexp(restored.get()).unwrap(),
                        ["one", "two"]
                    );

                    <Vec<Owner<str>> as AltString>::set_elt(x.get(), 0, SEXP::na_string());
                    let bytes = OwnedProtect::new(
                        RCall::new("serialize")
                            .arg(x.get())
                            .arg(SEXP::nil())
                            .eval_base()
                            .unwrap(),
                    );
                    let restored = OwnedProtect::new(
                        RCall::new("unserialize")
                            .arg(bytes.get())
                            .eval_base()
                            .unwrap(),
                    );
                    assert_eq!(
                        Vec::<Option<String>>::try_from_sexp(restored.get()).unwrap(),
                        [None, Some("two".into())]
                    );

                    let optional = OwnedProtect::new(
                        Altrep(vec![Some(Owner::<str>::from("one")), None]).into_sexp(),
                    );
                    assert_eq!(
                        Vec::<Option<String>>::try_from_sexp(optional.get()).unwrap(),
                        [Some("one".into()), None]
                    );
                });
            }
        }
    };
}
owned_str_tests!(boxed, std::boxed::Box);
owned_str_tests!(rc, std::rc::Rc);
owned_str_tests!(arc, std::sync::Arc);
