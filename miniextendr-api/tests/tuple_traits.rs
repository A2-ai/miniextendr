//! Tuple traits share positional list semantics through arity twelve.
mod r_test_utils;

use miniextendr_api::coerce::{Coerce, TryCoerce};
use miniextendr_api::externalptr::TypedExternal;
use miniextendr_api::from_r::{SexpError, TryFromSexp};
use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::list::{IntoList, List, TryFromList};
use miniextendr_api::{IntoR, SEXPTYPE, SexpExt};

type Twelve = (
    i32,
    String,
    bool,
    f64,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
    i32,
);

#[test]
fn twelve_fields_roundtrip_through_both_list_traits() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let input: Twelve = (1, "two".into(), true, 4.5, 5, 6, 7, 8, 9, 10, 11, 12);
        let sexp = scope.protect(input.clone().into_sexp_unchecked()).get();
        assert_eq!(sexp.type_of(), SEXPTYPE::VECSXP);
        assert_eq!(sexp.len(), 12);
        assert_eq!(Twelve::try_from_sexp_unchecked(sexp).unwrap(), input);
        let list = input.clone().into_list();
        let sexp = scope.protect(list.into_sexp()).get();
        let list = List::try_from_sexp(sexp).unwrap();
        assert_eq!(Twelve::try_from_list(list).unwrap(), input);
        assert_eq!(Twelve::TYPE_NAME, "Tuple12");
        assert_eq!(Twelve::TYPE_ID_CSTR, b"std::Tuple12\0");
    });
}

#[test]
fn list_trait_preserves_batched_field_errors() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let sexp = scope.protect(("bad", 2i32, "also bad").into_sexp()).get();
        let list = List::try_from_sexp(sexp).unwrap();
        let error = <(i32, i32, i32)>::try_from_list(list).unwrap_err();
        match error {
            SexpError::InvalidValue(message) => {
                assert_eq!(message, "expected integer, got character (elements 1, 3)");
            }
            other => panic!("expected batched field errors, got {other:?}"),
        }
        let singleton = scope.protect((42i32,).into_list().into_sexp()).get();
        assert_eq!(<(i32,)>::try_from_sexp(singleton).unwrap(), (42,));
        assert_eq!(().into_sexp().type_of(), SEXPTYPE::NILSXP);
    });
}

#[test]
fn singleton_and_twelve_field_coercions_use_the_existing_blanket() {
    assert_eq!(Coerce::<(i32,)>::coerce((7i8,)), (7,));
    assert_eq!(TryCoerce::<(i32,)>::try_coerce((7i8,)).unwrap(), (7,));
    type Wide = (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, i32);
    let result: Wide = (
        1i8, 2i8, 3i8, 4i8, 5i8, 6i8, 7i8, 8i8, 9i8, 10i8, 11i8, 12i8,
    )
        .coerce();
    assert_eq!(result, (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12));
}

#[test]
fn tuple_containers_roundtrip_and_distinguish_null_from_an_empty_list() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        type Row = (i32, String);
        for unchecked in [false, true] {
            let rows = vec![
                Some((7, "seven".to_string())),
                None,
                Some((9, "nine".to_string())),
            ];
            let sexp = scope
                .protect(if unchecked {
                    rows.clone().try_into_sexp_unchecked().unwrap()
                } else {
                    rows.clone().try_into_sexp().unwrap()
                })
                .get();
            assert_eq!(sexp.vector_elt(1).type_of(), SEXPTYPE::NILSXP);
            let actual = if unchecked {
                Vec::<Option<Row>>::try_from_sexp_unchecked(sexp)
            } else {
                Vec::<Option<Row>>::try_from_sexp(sexp)
            }
            .unwrap();
            assert_eq!(actual, rows);
            let boxed: Box<[Row]> = vec![(1, "one".to_string())].into_boxed_slice();
            let sexp = scope.protect(boxed.clone().into_sexp()).get();
            assert_eq!(Box::<[Row]>::try_from_sexp(sexp).unwrap(), boxed);
            let some = scope
                .protect(Some((1, "one".to_string())).into_sexp())
                .get();
            assert_eq!(
                Option::<Row>::try_from_sexp(some).unwrap(),
                Some((1, "one".into()))
            );
            let none = Option::<Row>::None.into_sexp();
            assert_eq!(Option::<Row>::try_from_sexp(none).unwrap(), None);
            let empty = scope.protect(Vec::<Row>::new().into_sexp()).get();
            assert_eq!(empty.type_of(), SEXPTYPE::VECSXP);
            assert!(Vec::<Row>::try_from_sexp(empty).unwrap().is_empty());
            assert!(Option::<Row>::try_from_sexp(empty).is_err());
        }
    });
}

fn tuple_row_class(sexp: miniextendr_api::SEXP) -> Result<(), miniextendr_api::condition::RError> {
    if sexp.inherits_class(c"tuple_row") {
        Ok(())
    } else {
        Err(
            miniextendr_api::condition::RError::new("expected a tuple_row")
                .class("tuple_row_required")
                .data("expected_class", "tuple_row"),
        )
    }
}

#[derive(miniextendr_api::TryFromSexp, Debug, PartialEq)]
#[try_from_sexp(validate = tuple_row_class)]
struct CheckedRow((i32, String));

#[test]
fn tuple_newtype_validators_run_on_rows_and_preserve_classed_errors() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let valid = scope
            .protect(
                miniextendr_api::r_str!(
                    "list(structure(list(1L, 'a'), class = 'tuple_row'), NULL)"
                )
                .unwrap(),
            )
            .get();
        let invalid = scope
            .protect(miniextendr_api::r_str!("list(list(1L, 'a'), NULL, list(3L, 'c'))").unwrap())
            .get();
        for unchecked in [false, true] {
            let read = |sexp| {
                if unchecked {
                    Vec::<Option<CheckedRow>>::try_from_sexp_unchecked(sexp)
                } else {
                    Vec::<Option<CheckedRow>>::try_from_sexp(sexp)
                }
            };
            assert_eq!(
                read(valid).unwrap(),
                vec![Some(CheckedRow((1, "a".into()))), None]
            );
            match read(invalid).unwrap_err() {
                SexpError::Condition(error) => {
                    assert_eq!(error.classes(), ["tuple_row_required"]);
                    assert_eq!(error.message_str(), "expected a tuple_row (elements 1, 3)");
                }
                other => panic!("expected classed row errors, got {other:?}"),
            }
        }
    });
}

#[test]
fn tuple_rows_batch_nested_field_errors() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let sexp = scope
            .protect(vec![("bad", "bad"), ("bad", "bad")].into_sexp())
            .get();
        match Vec::<(i32, i32)>::try_from_sexp(sexp).unwrap_err() {
            SexpError::InvalidValue(message) => {
                assert_eq!(
                    message,
                    "expected integer, got character (elements 1, 2) (elements 1, 2)"
                );
            }
            other => panic!("expected nested row and field errors, got {other:?}"),
        }
    });
}

#[test]
fn tuple_native_metadata_and_na_checks_survive_container_forwarding() {
    use miniextendr_api::from_r::NativeBorrow;
    type Row = (&'static mut i32, String);
    assert_eq!(Option::<Row>::NATIVE_BORROW, Row::NATIVE_BORROW);
    assert_eq!(
        Vec::<Row>::NATIVE_BORROW,
        NativeBorrow::in_list(Row::NATIVE_BORROW)
    );
    assert_eq!(Vec::<Option<Row>>::NATIVE_BORROW, Vec::<Row>::NATIVE_BORROW);
    assert_eq!(Box::<[Row]>::NATIVE_BORROW, Vec::<Row>::NATIVE_BORROW);
    // The hook forwards conversion-introduced missingness; the ordinary
    // input guard handles R's native NA values before conversion.
    use miniextendr_api::convert::AsNumeric;
    assert!((AsNumeric(None), 1i32).__mx_has_na());
    assert!(!(AsNumeric(Some(1.0)), 1i32).__mx_has_na());
    assert!(vec![Some((AsNumeric(None),)), None].__mx_has_na());
    assert!(!Option::<(AsNumeric,)>::None.__mx_has_na());
}

#[test]
fn tuple_fields_preserve_classed_refusals_and_payloads_when_batched() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let input = scope
            .protect(miniextendr_api::r_str!("list(list(1L, 'a'), 2L, list(3L, 'c'))").unwrap())
            .get();
        type Fields = (CheckedRow, i32, CheckedRow);
        for unchecked in [false, true] {
            let result = if unchecked {
                Fields::try_from_sexp_unchecked(input)
            } else {
                Fields::try_from_list(List::try_from_sexp(input).unwrap())
            };
            match result.unwrap_err() {
                SexpError::Condition(error) => {
                    assert_eq!(error.classes(), ["tuple_row_required"]);
                    assert_eq!(error.message_str(), "expected a tuple_row (elements 1, 3)");
                    assert_eq!(error.fields().len(), 1);
                    assert_eq!(error.fields()[0].0, "expected_class");
                    assert_eq!(error.fields()[0].1.as_str(), Some("tuple_row"));
                }
                other => panic!("tuple conversion lost the field condition: {other:?}"),
            }
        }
    });
}

// Length can execute user code. A tuple type mismatch must not consult an
// integer ALTREP's length, even though ordinary integer vectors are harmless.
static TUPLE_INPUT_LENGTH_CALLS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[derive(miniextendr_api::AltrepInteger)]
#[altrep(manual, class = "TupleInputLengthProbe")]
pub struct TupleInputLengthProbe(i32);

impl miniextendr_api::altrep_data::AltrepLen for TupleInputLengthProbe {
    fn altrep_len(&self) -> usize {
        TUPLE_INPUT_LENGTH_CALLS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        1
    }
}

impl miniextendr_api::altrep_data::AltIntegerData for TupleInputLengthProbe {
    fn elt(&self, _: usize) -> i32 {
        self.0
    }
}

#[test]
fn tuple_shape_validation_rejects_wrong_altrep_type_without_calling_length() {
    r_test_utils::with_r_thread(|| unsafe {
        use std::sync::atomic::Ordering;
        let scope = ProtectScope::new();
        let input = scope.protect(TupleInputLengthProbe(42).into_sexp()).get();
        TUPLE_INPUT_LENGTH_CALLS.store(0, Ordering::SeqCst);
        for error in [
            <(i32,)>::try_from_sexp(input).unwrap_err(),
            <(i32,)>::try_from_sexp_unchecked(input).unwrap_err(),
            Option::<(i32,)>::try_from_sexp(input).unwrap_err(),
            Option::<(i32,)>::try_from_sexp_unchecked(input).unwrap_err(),
        ] {
            assert!(matches!(error, SexpError::Type(error)
                if error.expected == SEXPTYPE::VECSXP && error.actual == SEXPTYPE::INTSXP));
        }
        assert_eq!(TUPLE_INPUT_LENGTH_CALLS.load(Ordering::SeqCst), 0);
    });
}
