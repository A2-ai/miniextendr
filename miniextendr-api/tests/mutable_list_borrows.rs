//! Mutable list conversions reject raw-buffer overlap before producing references.
mod r_test_utils;

use miniextendr_api::altrep_data::{AltIntegerData, AltrepDataptr, AltrepLen};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::ListBuilder;
use miniextendr_api::{IntoR, RLogical, Rcomplex, SEXP, TryFromSexp};
use std::cell::UnsafeCell;
use std::rc::Rc;

fn list(scope: &ProtectScope, elements: &[SEXP]) -> SEXP {
    let builder = unsafe { ListBuilder::new(scope, elements.len()) };
    for (index, &element) in elements.iter().enumerate() {
        unsafe { builder.set(isize::try_from(index).unwrap(), element) };
    }
    builder.into_sexp()
}

fn overlap_error<T: TryFromSexp>(input: SEXP)
where
    T::Error: std::fmt::Display,
{
    let error = T::try_from_sexp(input)
        .err()
        .expect("overlapping references must be refused")
        .to_string();
    assert!(error.contains("overlapping native buffers"), "{error}");
}

#[test]
fn all_native_mutable_list_forms_refuse_repeated_nonempty_buffers() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        macro_rules! check {
            ($t:ty, $value:expr) => {{
                let scalar = scope.protect(vec![$value].into_sexp()).get();
                let input = list(&scope, &[scalar, scalar]);
                overlap_error::<Vec<&mut $t>>(input);
                overlap_error::<Vec<&mut [$t]>>(input);
                let optional = list(&scope, &[scalar, SEXP::nil(), scalar]);
                overlap_error::<Vec<Option<&mut $t>>>(optional);
                overlap_error::<Vec<Option<&mut [$t]>>>(optional);
            }};
        }
        check!(i32, 7_i32);
        check!(f64, 2.5_f64);
        check!(u8, 3_u8);
        check!(RLogical, RLogical::TRUE);
        check!(Rcomplex, Rcomplex { r: 1.0, i: 2.0 });
    });
}

// Deliberately shared native storage, with separate ALTREP identities. This
// models windows over a larger buffer and defeats object-identity checks.
#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, dataptr, class = "MutableListWindow")]
struct Window {
    values: Rc<UnsafeCell<[i32; 3]>>,
    offset: usize,
    len: usize,
}

impl AltrepLen for Window {
    fn altrep_len(&self) -> usize {
        self.len
    }
}
impl AltIntegerData for Window {
    fn elt(&self, index: usize) -> i32 {
        assert!(index < self.len);
        unsafe { *self.values.get().cast::<i32>().add(self.offset + index) }
    }
}
impl AltrepDataptr<i32> for Window {
    fn dataptr(&mut self, _: bool) -> Option<*mut i32> {
        Some(unsafe { self.values.get().cast::<i32>().add(self.offset) })
    }
    fn dataptr_or_null(&self) -> Option<*const i32> {
        Some(unsafe {
            self.values
                .get()
                .cast::<i32>()
                .add(self.offset)
                .cast_const()
        })
    }
}

fn window(
    scope: &ProtectScope,
    values: &Rc<UnsafeCell<[i32; 3]>>,
    offset: usize,
    len: usize,
) -> SEXP {
    assert!(offset + len <= 3);
    unsafe {
        scope
            .protect(
                Window {
                    values: Rc::clone(values),
                    offset,
                    len,
                }
                .into_sexp(),
            )
            .get()
    }
}

#[test]
fn distinct_altrep_windows_cannot_create_overlapping_mutable_references() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let values = Rc::new(UnsafeCell::new([3, 5, 8]));
        let a = window(&scope, &values, 0, 2);
        let b = window(&scope, &values, 1, 2);
        assert_ne!(a, b);
        overlap_error::<Vec<&mut [i32]>>(list(&scope, &[a, b]));
        overlap_error::<Vec<Option<&mut [i32]>>>(list(&scope, &[b, SEXP::nil(), a]));
        let a = window(&scope, &values, 1, 1);
        let b = window(&scope, &values, 1, 1);
        assert_ne!(a, b);
        overlap_error::<Vec<&mut i32>>(list(&scope, &[a, b]));
        overlap_error::<Vec<Option<&mut i32>>>(list(&scope, &[a, SEXP::nil(), b]));
    });
}

#[test]
fn disjoint_windows_empty_slices_and_nulls_remain_accepted() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let values = Rc::new(UnsafeCell::new([3, 5, 8]));
        let a = window(&scope, &values, 0, 1);
        let b = window(&scope, &values, 1, 2);
        {
            let mut slices = Vec::<&mut [i32]>::try_from_sexp(list(&scope, &[a, b])).unwrap();
            slices[0][0] = 13;
            slices[1][0] = 21;
            slices[1][1] = 34;
        }
        assert_eq!(*values.get(), [13, 21, 34]);
        let empty = scope.protect(Vec::<i32>::new().into_sexp()).get();
        let slices =
            Vec::<Option<&mut [i32]>>::try_from_sexp(list(&scope, &[empty, SEXP::nil(), empty]))
                .unwrap();
        assert!(slices[0].as_ref().unwrap().is_empty());
        assert!(slices[1].is_none());
        assert!(slices[2].as_ref().unwrap().is_empty());
    });
}

#[test]
fn list_preflight_reports_every_type_length_and_overlap_failure() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let scalar = scope.protect(7_i32.into_sexp()).get();
        let text = scope.protect("wrong".into_sexp()).get();
        let pair = scope.protect(vec![1_i32, 2].into_sexp()).get();
        let logical = scope.protect(true.into_sexp()).get();
        let input = list(&scope, &[scalar, text, pair, scalar, logical]);
        let error = Vec::<&mut i32>::try_from_sexp(input)
            .unwrap_err()
            .to_string();
        for index in 2..=5 {
            assert!(error.contains(&format!("element {index}")), "{error}");
        }
        assert!(error.contains("overlapping native buffers"), "{error}");
    });
}
