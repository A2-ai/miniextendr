//! ALTREP lists can return fresh, uncached values from every Elt callback.
mod r_test_utils;

use miniextendr_api::altrep_data::{AltListData, AltrepLen};
use miniextendr_api::expression::r_eval_str_global;
use miniextendr_api::{IntoR, ProtectScope, SEXP, SexpExt, TryFromRoot};

#[derive(miniextendr_api::AltrepList)]
#[altrep(manual, class = "TransientRootedList")]
pub struct TransientList;

impl AltrepLen for TransientList {
    fn len(&self) -> usize {
        4
    }
}

impl AltListData for TransientList {
    fn elt(&self, index: usize) -> SEXP {
        unsafe {
            miniextendr_api::sys::R_gc();
        }
        let i = i32::try_from(index).unwrap();
        vec![i, i + 10].into_sexp()
    }
}

#[test]
fn native_list_children_survive_later_uncached_elt_calls_and_gc() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(TransientList.into_sexp());
        let values: Vec<&[i32]> = Vec::try_from_root(root).unwrap();
        miniextendr_api::sys::R_gc();
        // Reuse any freed node of this size, so a collected element shows.
        for _ in 0..10_000 {
            let _ = vec![-1_i32, -1].into_sexp();
        }
        assert_eq!(values, [&[0, 10][..], &[1, 11], &[2, 12], &[3, 13]]);
    });
}

#[test]
fn nested_optional_borrows_retain_shape_and_batch_invalid_elements() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(r_eval_str_global("list(list(7L, NULL), list(9L))").unwrap());
        let values: Vec<Vec<Option<&i32>>> = Vec::try_from_root(root).unwrap();
        miniextendr_api::sys::R_gc();
        assert_eq!(values, [vec![Some(&7), None], vec![Some(&9)]]);
        let missing: Option<Vec<&i32>> = Option::try_from_root(scope.rooted(SEXP::nil())).unwrap();
        assert!(missing.is_none());

        let invalid = scope.protect(r_eval_str_global("list(1.5, 2L, 3.5)").unwrap());
        let error = Vec::<&i32>::try_from_root(invalid).unwrap_err().to_string();
        assert!(error.contains("1, 3"), "{error}");
    });
}

#[test]
fn character_reference_vectors_preserve_na_and_empty_strings() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let root = scope.protect(vec![Some("retained"), None, Some("")].into_sexp());
        let values: Vec<&str> = Vec::try_from_root(root).unwrap();
        let optional: Vec<Option<&str>> = Vec::try_from_root(root).unwrap();
        miniextendr_api::sys::R_gc();
        assert_eq!(values, ["retained", "", ""]);
        assert_eq!(optional, [Some("retained"), None, Some("")]);
    });
}

#[test]
fn large_borrowed_lists_do_not_use_one_protect_stack_entry_per_element() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let source = scope.alloc_vecsxp(70_000);
        let scalar = scope.scalar_integer(7);
        for index in 0..70_000 {
            source
                .get()
                .set_vector_elt(isize::try_from(index).unwrap(), scalar.get());
        }
        let values: Vec<&i32> = Vec::try_from_root(source).unwrap();
        miniextendr_api::sys::R_gc();
        assert_eq!(values.len(), 70_000);
        assert!(values.iter().all(|value| **value == 7));
    });
}
