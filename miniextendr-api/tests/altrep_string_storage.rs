//! String materialization must distinguish real NA from absent storage.
mod r_test_utils;

use std::cell::Cell;

use miniextendr_api::altrep_data::{AltStringData, AltrepExtract, AltrepLen};
use miniextendr_api::altrep_traits::{AltString, AltVec, Altrep};
use miniextendr_api::{IntoR, OwnedProtect, SEXP, SexpExt, TryFromSexp};

#[derive(miniextendr_api::AltrepString)]
#[altrep(manual, class = "StringStorageAudit")]
pub struct LazyStrings {
    values: Vec<Option<String>>,
    calls: Cell<usize>,
    fail_at: Cell<Option<usize>>,
}

impl AltrepLen for LazyStrings {
    fn len(&self) -> usize {
        self.values.len()
    }
}

impl AltStringData for LazyStrings {
    fn elt(&self, i: usize) -> Option<&str> {
        self.calls.set(self.calls.get() + 1);
        assert_ne!(self.fail_at.get(), Some(i), "fixture conversion failure");
        self.values[i].as_deref()
    }
}

#[test]
fn element_reads_stay_lazy_and_materialization_is_published_only_when_complete() {
    r_test_utils::with_r_thread(|| {
        let x = unsafe {
            OwnedProtect::new(
                LazyStrings {
                    values: vec![Some("first".to_owned()), None, Some("last".to_owned())],
                    calls: Cell::new(0),
                    fail_at: Cell::new(None),
                }
                .into_sexp(),
            )
        };
        assert_eq!(
            <LazyStrings as AltString>::elt(x.get(), 1),
            SEXP::na_string()
        );
        assert!(<LazyStrings as AltVec>::dataptr_or_null(x.get()).is_null());
        let data = unsafe { <LazyStrings as AltrepExtract>::altrep_extract_ref(x.get()) };
        assert_eq!(data.calls.get(), 1);

        data.fail_at.set(Some(2));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            <LazyStrings as AltVec>::dataptr(x.get(), false);
        }));
        assert!(failed.is_err());
        assert!(<LazyStrings as AltVec>::dataptr_or_null(x.get()).is_null());
        data.fail_at.set(None);

        let pointer = <LazyStrings as AltVec>::dataptr(x.get(), false);
        assert_eq!(
            <LazyStrings as AltVec>::dataptr_or_null(x.get()),
            pointer.cast_const()
        );
        let calls = data.calls.get();
        assert_eq!(
            <LazyStrings as AltString>::elt(x.get(), 1),
            SEXP::na_string()
        );
        assert_eq!(<LazyStrings as AltVec>::dataptr(x.get(), false), pointer);
        assert_eq!(data.calls.get(), calls);
    });
}

#[test]
fn string_replacement_preserves_na_and_serializes_current_values() {
    r_test_utils::with_r_thread(|| {
        type Data = Vec<Option<String>>;
        let x = unsafe {
            OwnedProtect::new(
                miniextendr_api::Altrep(vec![
                    Some("before".to_owned()),
                    None,
                    Some("unchanged".to_owned()),
                ])
                .into_sexp(),
            )
        };
        // R's default ALTSTRING setter errors; the bridge must register its setter.
        const {
            assert!(<Data as AltString>::HAS_SET_ELT);
        }
        <Data as AltString>::set_elt(x.get(), 0, SEXP::na_string());
        assert_eq!(<Data as AltString>::elt(x.get(), 0), SEXP::na_string());

        let replacement = unsafe { OwnedProtect::new(SEXP::charsxp("after")) };
        <Data as AltString>::set_elt(x.get(), 1, replacement.get());
        let expected = vec![None, Some("after".to_owned()), Some("unchanged".to_owned())];
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(x.get()).unwrap(),
            expected
        );
        assert_eq!(<Data as AltString>::no_na(x.get()), 0);
        assert!(<Data as Altrep>::serialized_state(x.get()).is_null_ptr());

        use miniextendr_api::expression::RCall;
        let bytes = unsafe {
            OwnedProtect::new(
                RCall::new("serialize")
                    .arg(x.get())
                    .arg(SEXP::nil())
                    .eval_base()
                    .unwrap(),
            )
        };
        let restored = unsafe {
            OwnedProtect::new(
                RCall::new("unserialize")
                    .arg(bytes.get())
                    .eval_base()
                    .unwrap(),
            )
        };
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(restored.get()).unwrap(),
            expected
        );
    });
}
