//! Deserialized ALTREP backing values must outlive R's temporary state.
mod r_test_utils;

use miniextendr_api::altrep_data::AltrepSerialize;
use miniextendr_api::gc_protect::OwnedProtect;
use miniextendr_api::{IntoR, Rcomplex, sys};
use std::borrow::Cow;

#[test]
fn unserialized_native_cows_own_their_storage() {
    r_test_utils::with_r_thread(|| {
        macro_rules! check {
            ($ty:ty, $values:expr) => {{
                let expected: Vec<$ty> = $values;
                let state = unsafe { OwnedProtect::new(expected.clone().into_sexp()) };
                let value =
                    <Cow<'static, [$ty]> as AltrepSerialize>::unserialize(state.get()).unwrap();
                assert!(
                    matches!(value, Cow::Owned(_)),
                    "ALTREP must own decoded storage"
                );
                drop(state);
                unsafe { sys::R_gc() };
                assert_eq!(&*value, expected.as_slice());
            }};
        }
        check!(i32, vec![11, 22]);
        check!(f64, vec![1.5, 2.5]);
        check!(u8, vec![0, 255]);
        check!(Rcomplex, vec![Rcomplex { r: 1.0, i: 2.0 }]);
    });
}

#[test]
fn unserialized_string_cows_own_each_element_and_preserve_na() {
    r_test_utils::with_r_thread(|| {
        let expected = vec!["alpha".to_owned(), "beta".to_owned()];
        let state = unsafe { OwnedProtect::new(expected.clone().into_sexp()) };
        let values = <Vec<Cow<'static, str>> as AltrepSerialize>::unserialize(state.get()).unwrap();
        assert!(values.iter().all(|value| matches!(value, Cow::Owned(_))));
        drop(state);
        unsafe { sys::R_gc() };
        assert_eq!(values, expected);

        let expected = vec![Some("alpha".to_owned()), None, Some(String::new())];
        let state = unsafe { OwnedProtect::new(expected.clone().into_sexp()) };
        let values =
            <Vec<Option<Cow<'static, str>>> as AltrepSerialize>::unserialize(state.get()).unwrap();
        assert!(
            values
                .iter()
                .flatten()
                .all(|value| matches!(value, Cow::Owned(_)))
        );
        drop(state);
        unsafe { sys::R_gc() };
        assert_eq!(
            values
                .into_iter()
                .map(|v| v.map(Cow::into_owned))
                .collect::<Vec<_>>(),
            expected
        );
    });
}
