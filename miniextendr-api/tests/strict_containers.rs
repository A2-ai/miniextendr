//! Runtime shapes of the shared strict contiguous-container conversions.
mod r_test_utils;

use miniextendr_api::gc_protect::ProtectScope;
use miniextendr_api::strict::{checked_rows_into_sexp, checked_slice_into_sexp};
use miniextendr_api::{SEXPTYPE, SexpExt, TryFromSexp};

#[test]
fn strict_containers_keep_integer_storage_empty_rows_and_null_rows() {
    r_test_utils::with_r_thread(|| unsafe {
        let scope = ProtectScope::new();
        let values = [1_i64, 2];
        let sexp = scope
            .protect(checked_slice_into_sexp("&[i64]", &values))
            .get();
        assert_eq!(sexp.type_of(), SEXPTYPE::INTSXP);
        assert_eq!(Vec::<i32>::try_from_sexp(sexp).unwrap(), vec![1, 2]);
        let rows = [Some(values.as_slice()), None, Some(&[][..])];
        let sexp = scope
            .protect(checked_rows_into_sexp::<i64, _>(
                "Vec<Option<&[i64]>>",
                rows.into_iter(),
            ))
            .get();
        assert_eq!(sexp.type_of(), SEXPTYPE::VECSXP);
        assert_eq!(sexp.vector_elt(0).type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.vector_elt(1).type_of(), SEXPTYPE::NILSXP);
        assert_eq!(sexp.vector_elt(2).type_of(), SEXPTYPE::INTSXP);
        assert_eq!(sexp.vector_elt(2).len(), 0);
    });
}
