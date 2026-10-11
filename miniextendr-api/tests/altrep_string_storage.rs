//! String ALTREPs cache each element in data2, and NA there is a value, not a miss.
mod r_test_utils;

use std::cell::Cell;

use miniextendr_api::altrep_data::{
    AltStringData, AltrepExtract, AltrepExtractSubset, AltrepLen, altrep_values_written,
    materialized_altrep_data2,
};
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

fn lazy(values: Vec<Option<String>>) -> OwnedProtect {
    unsafe {
        OwnedProtect::new(
            LazyStrings {
                values,
                calls: Cell::new(0),
                fail_at: Cell::new(None),
            }
            .into_sexp(),
        )
    }
}

#[test]
fn each_element_is_converted_once_and_a_failed_fill_keeps_its_progress() {
    r_test_utils::with_r_thread(|| {
        let x = lazy(vec![
            Some("first".to_owned()),
            None,
            Some("last".to_owned()),
        ]);
        let data = unsafe { <LazyStrings as AltrepExtract>::altrep_extract_ref(x.get()) };

        // NA is cached like any other value.
        for _ in 0..2 {
            assert_eq!(
                <LazyStrings as AltString>::elt(x.get(), 1),
                SEXP::na_string()
            );
        }
        assert_eq!(data.calls.get(), 1);
        assert!(<LazyStrings as AltVec>::dataptr_or_null(x.get()).is_null());

        data.fail_at.set(Some(2));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            <LazyStrings as AltVec>::dataptr(x.get(), false);
        }));
        assert!(failed.is_err());
        assert_eq!(data.calls.get(), 3);
        data.fail_at.set(None);

        // Only the slot the failure left unfilled is converted again.
        let pointer = <LazyStrings as AltVec>::dataptr(x.get(), false);
        assert_eq!(data.calls.get(), 4);
        assert_eq!(<LazyStrings as AltVec>::dataptr(x.get(), false), pointer);
        assert_eq!(
            <LazyStrings as AltString>::elt(x.get(), 1),
            SEXP::na_string()
        );
        assert_eq!(data.calls.get(), 4);
        assert_eq!(
            Vec::<Option<String>>::try_from_sexp(x.get()).unwrap(),
            [Some("first".to_owned()), None, Some("last".to_owned())]
        );
    });
}

#[test]
fn a_returned_element_stays_reachable_from_the_vector() {
    r_test_utils::with_r_thread(|| {
        // A string no other R object holds.
        let expected = format!("only-here-{}", std::process::id());
        let x = lazy(vec![Some(expected.clone())]);
        let elt = <LazyStrings as AltString>::elt(x.get(), 0);
        let cache = unsafe { materialized_altrep_data2(x.get()) }.unwrap();
        assert_eq!(cache.string_elt(0), elt);
        unsafe { miniextendr_api::sys::R_gc() };
        assert_eq!(<LazyStrings as AltString>::elt(x.get(), 0), elt);
        assert_eq!(cache.string_elt_str(0), Some(expected.as_str()));
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

#[test]
fn reads_keep_the_rust_answers_until_a_write() {
    r_test_utils::with_r_thread(|| {
        type Data = Vec<Option<String>>;
        let x = unsafe {
            OwnedProtect::new(
                miniextendr_api::Altrep(vec![Some("a".to_owned()), Some("b".to_owned())])
                    .into_sexp(),
            )
        };

        // Reads fill the cache, but its values are still the Rust data's.
        let _ = <Data as AltString>::elt(x.get(), 0);
        let _ = <Data as AltVec>::dataptr(x.get(), false);
        assert!(unsafe { materialized_altrep_data2(x.get()) }.is_some());
        assert!(!unsafe { altrep_values_written(x.get()) });
        assert_eq!(<Data as AltString>::no_na(x.get()), 1);
        assert!(!<Data as Altrep>::serialized_state(x.get()).is_null_ptr());

        // A writable Dataptr hands R a pointer it may store through.
        let _ = <Data as AltVec>::dataptr(x.get(), true);
        assert!(unsafe { altrep_values_written(x.get()) });
        assert_eq!(<Data as AltString>::no_na(x.get()), 0);
        assert!(<Data as Altrep>::serialized_state(x.get()).is_null_ptr());
    });
}

#[derive(miniextendr_api::AltrepString)]
#[altrep(manual, subset, class = "StringSubsetAudit")]
pub struct SubsetStrings {
    values: Vec<String>,
    subsets: Cell<usize>,
}

impl AltrepLen for SubsetStrings {
    fn len(&self) -> usize {
        self.values.len()
    }
}

impl AltStringData for SubsetStrings {
    fn elt(&self, i: usize) -> Option<&str> {
        Some(&self.values[i])
    }
}

impl AltrepExtractSubset for SubsetStrings {
    fn extract_subset(&self, indices: &[i32]) -> Option<SEXP> {
        self.subsets.set(self.subsets.get() + 1);
        let picked: Vec<String> = indices
            .iter()
            .map(|&i| self.values[usize::try_from(i - 1).unwrap()].clone())
            .collect();
        Some(picked.into_sexp())
    }
}

fn subset_strings(values: &[&str]) -> OwnedProtect {
    unsafe {
        OwnedProtect::new(
            SubsetStrings {
                values: values.iter().map(|&v| v.to_owned()).collect(),
                subsets: Cell::new(0),
            }
            .into_sexp(),
        )
    }
}

fn subset(x: SEXP, indices: Vec<i32>) -> Vec<String> {
    use miniextendr_api::expression::RCall;
    let indices = unsafe { OwnedProtect::new(indices.into_sexp()) };
    let result = unsafe {
        OwnedProtect::new(
            RCall::new("[")
                .arg(x)
                .arg(indices.get())
                .eval_base()
                .unwrap(),
        )
    };
    Vec::<String>::try_from_sexp(result.get()).unwrap()
}

#[test]
fn a_read_only_vector_keeps_its_own_subset() {
    r_test_utils::with_r_thread(|| {
        let x = subset_strings(&["a", "b"]);
        let data = unsafe { <SubsetStrings as AltrepExtract>::altrep_extract_ref(x.get()) };
        // A read fills the cache; the Rust data is still current.
        let _ = <SubsetStrings as AltString>::elt(x.get(), 0);
        assert!(unsafe { materialized_altrep_data2(x.get()) }.is_some());

        assert_eq!(subset(x.get(), vec![2, 1]), ["b", "a"]);
        assert_eq!(data.subsets.get(), 1);
    });
}

#[test]
fn a_subset_after_a_write_reads_the_written_value() {
    r_test_utils::with_r_thread(|| {
        let x = subset_strings(&["a", "b"]);
        let data = unsafe { <SubsetStrings as AltrepExtract>::altrep_extract_ref(x.get()) };
        let replacement = unsafe { OwnedProtect::new(SEXP::charsxp("z")) };
        <SubsetStrings as AltString>::set_elt(x.get(), 0, replacement.get());

        // The class's own subset would read the stale Rust value; R subsets.
        assert_eq!(subset(x.get(), vec![1, 2]), ["z", "b"]);
        assert_eq!(data.subsets.get(), 0);
    });
}
