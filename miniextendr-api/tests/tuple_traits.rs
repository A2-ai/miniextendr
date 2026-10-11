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
